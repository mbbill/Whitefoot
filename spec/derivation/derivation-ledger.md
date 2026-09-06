# Derivation Ledger — kernel spec (living artifact)

The original full audit covered `kernel-spec-v0.3.md` and `docs/constitution.md`
on 2026-07-07. Versioned amendments below carry that audit through
the active specification at `spec/kernel-spec.md`, whose version and digest
are the chain tail in `governance/APPROVALS.md`; each superseded version is
archived at `spec/kernel-spec-vN.md`, and the v0.40 through v0.49 amendments
at the end of this file bind their changed derivations.
Requirement (META-6): every rule is provably
derived, directly or indirectly, from the constitution — or flagged. Statuses:
**derived** (existence and form), **derived_existence_only** (the rule must
exist; this form is minimality-selected and awaits its experiment),
**underived** (no chain; may not ratify).

Rows and amendments through v0.39 remain in this ledger as historical
derivation evidence. The unversioned table preserves those derivation chains;
the v0.40 through v0.49 amendments below and the active
specification define the changed rows. The table is not independent live
source guidance. In
particular, its historical `claim`, `traps`, CLM, PRV,
`deny_claims`, strict-partition, runtime-latch, and proof-replay vocabulary does
not describe the active compiler. v0.40 removes SCOPE-4,
DIAG-3, TRAP-1, CLM-1 through CLM-3, and PRV-1 through PRV-3; the retained rows
below explain only why those released rules once existed.

**v0.49 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.49 adds and retires no rule and moves no row's status. It
amends [PRF-1] in place, so every derivation status carries over from v0.48.)
**v0.48 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.48 adds and retires no rule and moves no row's status. It
amends [GRAM-4] and [PRF-1] in place, so every derivation status carries over
from v0.47.)
**v0.47 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.47 adds and retires no rule and moves no row's status. It
amends [INV-1] in place, so every derivation status carries over from v0.46.)
**v0.46 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.46 adds and retires no rule and moves no row's status. It
amends [FN-8], [ENT-3] and [ENT-6] in place, so every derivation status
carries over from v0.45 and the v0.46 amendment below binds the changed
sentences.)
**v0.45 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.45 adds and retires no rule and moves no row's status. It
amends [ENT-3] and [ENT-6] in place, so every derivation status carries over
from v0.44 and the v0.45 amendment below binds the changed sentences.)
**v0.44 statistics: 81 derived · 55 existence-only · 0 underived**
(136 rules: v0.44 adds derived MSR-3 and CALL-6 and existence-only MSR-5 and
CALL-4, retires no rule, and moves no existing row's status, so every other
derivation status carries over from v0.43).
**v0.43 statistics: 79 derived · 53 existence-only · 0 underived**
(132 rules: v0.43 adds and retires no rule and moves no row's status — it
amends OWN-3, OWN-11, FORM-8, and ENT-6 in place — so every derivation status
carries over from v0.42).
**v0.42 statistics: 79 derived · 53 existence-only · 0 underived**
(132 rules: v0.42 adds existence-only FORM-8, retires no rule, and respells
region positions only, so every other derivation status carries over from
v0.41).
**v0.41 statistics: 79 derived · 52 existence-only · 0 underived**
(131 rules: v0.41 respells 21 rules and adds or retires none, so every
derivation status carries over from v0.40).
**v0.40 statistics: 79 derived · 52 existence-only · 0 underived**
(131 rules: remove the nine historical rules named above, add existence-only
INV-1 and PRF-1, and retain every other rule's derivation status).
**v0.39 statistics: 86 derived · 52 existence-only · 0 underived**
(138 rules).

## Re-grounding priority queue (weakest chains)

- FORM-4 (no comments; doc fields) — existence rests on spec regularity alone; the doc-field replacement is audit-flagged R1-unaccounted human residue with no carded grounding and no recorded R5 exception; the registered doc-vs-inline-comment scaffolding experiment (audit obligation e) never ran. Nearest entry in the whole ledger to underived.
- GRAM-7 (statement-only match) — recorded selection ground is the literal R3 disqualifier ('cheapest to specify' / 'preserve one arm shape'), while the critique record's own W1 signal ('the most common pattern an AI writer needs') favors the rejected value-arm candidate; time-urgent because FORM-1 + EX-1 bake the helper idiom into normative bytes.
- TYPE-6 (no-shadowing) — verbatim 'cheapest for a canonical-form language' provenance; uniquely, no gate entry flagged it or scheduled validation before the 2026-07-05 audit; no card weighs fresh-name burden against stale-binding confusion for W1 writers; an R2-reversal watch site.
- OP-2 negation half — the div/rem half is a semantic theorem (no modular semantics for divisor zero), but the ineg.wrap exclusion is unjustified by the rule's own rationale: two's-complement wrapping negation is sound modular arithmetic (Rust ships wrapping_neg; LLVM 'sub 0, x' wraps). Minimality-selected inside an otherwise theorem-backed rule; N003 also sits outside both independent edge-case verification rounds. [ADDRESSED v0.3 — see ledger entry]
- META-4 (stated-once) — status derived, but the thinnest of the four regularity invariants: its recorded selection ground is D2 spec-compactness, i.e. W2-only post-D2a; carried by the D2a retention clause plus CI-machine-checkability; the ledger even records an apparent META-4 self-violation (FORM-1 and META-1 both state the one-spelling fact with no cross-reference). [ADDRESSED v0.3 — see ledger entry]
- OWN-9 (noalias consequences) — status derived but effectively single-card-class: the P0 payoff is in-kind only; no noalias on/off benchmark exists anywhere in the corpus and the Rust noalias enable/disable/miscompile history is still uncarded backlog, so the magnitude of the rule's entire P0 framing is unmeasured.
- FN-6 (polymorphic-recursion criterion) — the syntactic same-type-parameters criterion is strictly stronger than termination requires (rejects finite permutation cycles), yet it is absent from the R3-provisional register entirely — the thinnest audit coverage of any provisional-class form; rejection rate on AI-written recursive-generic code unmeasured. [ADDRESSED v0.3 — see ledger entry]
- FN-7 no-globals clause — rides an uncarded P0/F001 plausibility argument (ambient global aliasing would erode the noalias fact base); no card, debate round, or gate entry decides no-global-state specifically, and reversing it later is a breaking META-5 delta under FORM-1. [ADDRESSED v0.3 — see ledger entry]
- FORM-6 (unit token) — the disjoint-production discharge is analytic, but the underlying token choice has zero provenance of any kind (no card, experiment, debate, or gate entry) and deviates from FORM-3's TYPEID casing convention; a register gap with FORM-1 time-urgency in miniature. [ADDRESSED v0.3 — see ledger entry]
- SYS-2/SYS-7 (`IoError` class membership) — the thirty portable classes entered the specification asserted rather than selected: the dossier states the set with no census, no comparison against another portable class set, and no corpus study, and the thirty-one-issue edge-case review challenged the payload representation (issue 16) and the outcome-type disposition (issue 3) without ever challenging the membership. Time-urgent under ERR-2: adding or removing a class later is a site-enumerated edit at every exhaustive match over `IoError`, and FORM-1 makes a respelling a breaking canonical-form change.
- SYS-7 (two-field target detail) — `code: u32` plus `origin: u8` is reasoned from the allocation-free requirement (no SYS-5 release row, no `allocates` on any operation row) but unmeasured against any real target's error domain; the owner accepted it as a judgment item without override, which records acceptance rather than selection, and the priced richer-payload alternative was never built.
- SYS-1 (declaration home) — v0.33 removes the kind-conditioned visibility fork and reserves the complete system domain in every program. The original Route C selection debt therefore no longer describes the rule in force, but the replacement still carries a prospective reservation-cost debt: the earlier census found none of the then-current system spellings in the active corpus, not what the permanent global reservation will cost future writers. Completion: repeat the collision census once the ordinary corpus is large enough to make that cost observable.
- FN-7 (entry punctuation and label mechanism) — v0.33 fixes the only program kind to `command`, so the former IDENT-plus-closed-table kind mechanism and its debt disappear. The `input_label` `as` form and the `command.*` label spellings remain proposal- or regularity-selected: the dossier deliberately left binder punctuation to the specification proposal, and no alternative was compared. Like GRAM-5's place forms, this remains time-urgent because FORM-1 makes any evidence-driven switch a breaking canonical-form change in every program that consumes a command input.
- SYS-4 (kind taxonomy) — the three-kind classification is argued with nothing in v0.18 checking it, its earlier distinguishing criterion was found wrong and deleted during review rather than validated, and the rule inherits CAP-1's unratified two-predicate form. The debt is prospective by construction: no program's acceptance depends on it, and it fixes what the concurrency layer may assume, so it must be re-grounded before that layer binds it.
- QUAL-3 (required emitted shape) — the rule's entire P0 framing has zero measured instances: no target slice exists, the §9.1 cost gates are pre-registered but unrun, and the rule's own 'inlined, or any remaining call is shown to be immaterial' term carries no threshold by design. Same debt class as OWN-9 — chain sound, magnitude unpriced — with the difference that here the magnitude IS the rule.
- HOST-3 (open backing-lifetime premise) — the one-type theorem is sound and its premise is enforced at QUAL-2, but the general backing-lifetime rule for producers outside the command-lifetime premise is an explicitly open decision between two named candidates (region-bearing lease; owned-backing resource type). Until it is settled, HOST-3 covers exactly one producer, and the parallel-search witness names that open decision as an independent gating dependency.

## Ledger

| Rule | Feature | Status | Derivation chain | Notes |
|---|---|---|---|---|
| META-6 | derivation-ledger requirement; orphaned chains auto-flag | ✅ derived | Owner mandate 2026-07-07 ('for each feature we must prove it is derived from the constitution') + R1 (a rule with no chain serves nothing) + W3 (arbitrary rules are unauditable) -> META-6; self-applied here. | The native `whitefoot-spec` gate checks that every active rule ID has a row in this ledger; this row is the rule auditing itself. |
| ERR-3 | `propagate` Result forwarding with auto-derived context | ✅ derived | R4 (shift-left recoverable errors; manual re-match invites silent context loss) + W1 (one mechanical pattern) + W3 (propagation cannot drop the error) -> v0.3.1 rule text records the chain; checked-program context records per DIAG-2. v0.13 aligns the operand with OWN-13's already-derived consuming-context model: a direct bare affine own-rooted Result place is consumed exactly once, while explicit `move` remains valid. | Same-E-only in v0 (TYPE-4 no conversions); error-type mapping deferred with delta. The v0.13 amendment changes only how the operand's existing affine consumption is written, not the Ok/Err control flow, cleanup, borrow restrictions, effects, or runtime behavior. |
| ERR-4 | Per-operation typed-outcome versus static-proof classification | ✅ derived | R4 places expected environment or input failure in a typed `Result`, while T1/T2 require every undefined operation, layout, address, parallel-independence, and bounded-completion domain to be proved before execution. META-2 fixes that classification in the operation table and owning rule rather than at each call site. | Unavailable external resources and TCB failures remain the separate SCOPE-3 boundary. No call-site override, writer abort, or hidden runtime proof fallback exists. |
| SCOPE-1 | Writer-facing kernel; gated family unreachable by writers | ✅ derived | W3 (no writer-emittable unsafe or trust) -> round-3 unsafe-hatch synthesis 2026-07-02 (no free-form writer-emittable unsafe anywhere; exactly one gated fact-boundary construct family with obligation ledger) + D0a (gated channel is toolchain-side, AI-authored human-approved) -> kernel programs contain no gated constructs. T1/T2 premises require writer-unreachability of unsafe. R0 delta named in constitution: Rust's unsafe is writer-accessible everywhere. Stub visibility/pricing per round-4 kernel-priced-first law (GATE/LEDGER stubs counted in v0.1 delta). | Strongest W3 rule in the section. Gate-efficacy experiments (D0a's declared revisit trigger) are unscheduled per the audit, but they affect the human-approval process for gated content, not the writer-unreachability this rule states. |
| SCOPE-2 | Acceptance: canonical parse and every specification judgment succeeds | ✅ derived | T1/T2 require every supported partial operation to be proved before it can execute; W3 forbids a writer-stated conclusion, debug mode, optimizer assumption, or runtime fallback from substituting for that proof. R4 moves an unavailable proof to a rule-citing compile-time rejection, while P0 benefits because successful proof erases all proof syntax and emits no guard. | This is one source-checking state with no writer-emittable third state. A compiler inconsistency is repaired in compiler code and tests rather than hedged by another verifier. |
| SCOPE-3 | No-UB envelope conditional on TCB and foreign-code behavior | ✅ derived | T2 verbatim: no-UB conditional on declared TCB (round-3 Layer 4), itself derived from T1's premises plus R4 (silent corruption is the forbidden failure mode). D1a owner ruling confirmed the Rust-class conditional envelope, gated on checker-core feasibility — that blocking gate PASSED 2026-07-02 (prototype, 19/19 tests after v0.1). Clause (b) FFI conditionality per round-3 mandatory conservative FFI declaration frames + D4 rewrite-first FFI-narrow scoping. Conditional (not unconditional) form was an explicit owner arbitration, recorded 2026-07-02. | v0.40 temporarily leaves heap exhaustion, stack exhaustion, OS quota, and runtime-start availability outside the source outcome model. This does not defer layout, address, target-domain, parallel-independence, or bounded-completion proof. The resource scope cut changes neither the Constitution nor the no-UB direction. |
| SCOPE-4 | Contract violations trap with machine-readable report; abort, no unwinding | ✅ derived | R4 ladder (runtime trap ranks above forbidden silent corruption) + W3 ('failures trap with reports, never silently' — constitutional text) -> round-2 decided law: no exceptions/unwinding, Result + trap=abort (error-handling verdict: replace_with_alternative). Machine-readable report per R4 rule-citing diagnostics + R1/W1 feedback loop (T1 derivation: an unattended writer cannot debug latent runtime failure). No-unwind consistent with D4's unwind-abort-at-boundary FFI ruling. | The report-content half is under-delivered: DIAG-3 report field schemas remain DEFERRED, which the audit flags as R4-load-bearing rather than deferrable; the free-text trap message is a flagged R1 human-residue channel. Neither undermines this rule's own chain. |
| FORM-1 | One spelling, one byte form; reject non-canonical; never auto-format | 🟡 existence-only | Existence derived: R3 (one way to say anything) + W3 (canonical bytes leave nowhere to hide edits) + round-2 syntax verdict — drift detection, node-path diagnostics, content-addressed caching on the carded D001/D004 compile-time channel, F003/N006 frontend-soundness-chokepoint argument against any second surface. Conservative-extension clause from round-4 decided law. Form NOT derived: reject-never-canonicalize vs accept-and-canonicalize was never tested; specific byte form untested — R3-provisional register item (constitution audit 2026-07-05). | Unrun: drift-hypothesis test, caching-realization measurement, reject-vs-canonicalize A/B with repair-round-trip counts. Audit R1 tension: every byte-level deviation costs a full generate-reject-regenerate loop. Time-urgent: FORM-1 itself makes any later evidence-driven form switch a breaking canonical-form change. |
| FORM-2 | Exhaustive byte-level formatting: indent, spacing, line discipline | 🟡 existence-only | Existence derived: FORM-1's one-byte-form regime plus W3 canonical bytes require a TOTAL byte-format definition (an unspecified formatting dimension would reopen multiple spellings); round-2 syntax obligations mandate exactly one legal byte-level formatting. Form NOT derived: the specific conventions (two-space indent, blank-line separation, inter-token spacing table, one statement per line) were never tested for model formatting-error rates — R3-provisional register item. | Audit additionally flags indentation/spacing as R1 human-residue: they encode information already carried by braces/separators; the training-distribution-alignment defense exists but is uncarded. R1 justification pass owed (card the grounding or record an explicit R5 exception). W2 (token-count) arguments for or against are non-gating post-D2a. |
| FORM-3 | Lexical classes: IDENT, TYPEID, REGIONID, LABEL, OPNAME | 🟡 existence-only | Existence derived: GRAM-1 deterministic single-parse grammar + META-2 context-freedom require disjoint, casing/sigil-partitioned token classes (token kind decidable without context). OPNAME's mode-suffix shape (iadd.wrap as one token) is the evidence-backed part: round-2 floor 'one numeric mode per arithmetic node' + N001/N002 (N001 CONFIRMED 2026-07-06, N002 partially verified). Specific sigil choices (apostrophe REGIONID borrowed from Rust, @LABEL, casing split) NOT derived: convention-borrowed without the divergence census LEX-1 itself mandates. | Register gap: FORM-3 is not in the R3-provisional register, but its surface-shape choices are the same untested class as the registered GRAM-5 place forms. The v0.2 gate entry records the full-lexicon Rust-divergence audit as backlog — that audit is the completing step for the borrowed REGIONID spelling. Census entry opened for the apostrophe REGIONID: Rust-lifetime prior PARTIALLY matches (names a borrow region) but diverges on inference expectation (Rust lifetimes are inferred/elided; ours are always explicit lexical blocks) — mitigation: spec states explicitness; rename question held for the full D3 lexicon audit. |
| FORM-4 | No comments; documentation only in doc fields | 🟡 existence-only | Existence derived: the FORM-1/W3 canonical-byte regime must legislate comments — free comment bytes are unbounded alternate spellings of the same semantic construct and a hash-invisible edit channel. The no-comments FORM rests on spec regularity alone (audit finding): R5 removes only the human-readability argument for comments; it does not positively select their removal for AI writers. R3-provisional register item; R2 watch. | Registered experiment (audit obligation e): doc-only vs inline comment scaffolding as models' local reasoning anchors, across W1 tiers — never run. The doc-field replacement itself is audit-flagged R1-unaccounted human residue (prose in a language humans are declared not to read); it needs a carded grounding (writer re-read context, R5 ledger-audit support) or an explicit recorded R5 exception. Nearest item in the ledger to underived. |
| FORM-5 | Exhaustive literals: decimal-only suffixed numerics; no boolean literals | 🟡 existence-only | Existence derived: FORM-1/R3 require an exhaustive single-spelling literal grammar; mandatory type suffix aligns with the round-2 floor (zero implicit value-changing conversions) and the D2-corrected explicit-facts ruling; one-canonical-escape-per-character is one-spelling applied to strings. Form NOT derived: decimal-only (no hex/binary for bitmask code) never evaluated against AI codegen needs, and no-boolean-literals belongs to the never-debated match-only-conditional complex (GRAM-6/PRE-1/FORM-5) — both R3-provisional register items. | Registered experiments: decimal-only vs hex/binary literals (audit obligation f); conditional-form A/B including Bool-as-prelude-enum (obligation b). The STRING-only-in-doc/check restriction usefully CONTAINS the audit-flagged human-residue free-text channel but was not itself evidence-selected. |
| FORM-6 | unit token resolved by disjoint grammar productions | ✅ derived | Given the choice of a single shared token, GRAM-1 determinism + META-2 (no context-dependent spellings) FORCE exactly this rule: the disjoint-production statement is the analytic discharge of an apparent META-2 violation. The underlying token choice is NOT derived: lowercase 'unit' in type position deviates from the FORM-3 TYPEID convention, and a two-spelling alternative (TYPEID 'Unit' + value 'unit') was never evaluated; no card, experiment, debate, or gate entry exists. | Register gap — flag for R3-register extension. Low stakes, but FORM-1 makes a later respelling a breaking canonical-form change, so the same time-urgency argument the audit applies to GRAM-7 applies here in miniature. FIXED v0.3: provenance recorded — lowercase follows the TYPE-1 primitive convention; one-spelling per R3. |
| LEX-1 | Naming policy: invariant names, no backend vocabulary; uniq ruling | 🟡 existence-only | No-backend-IR-vocabulary half fully derived from R6 (artifacts must not marry one backend; 'noalias' names a lowering consequence, not a source invariant — F001 is the fact, not the name). Invariant-naming + divergence-census half instantiates W1 (borrowed names with divergent semantics mislead weak writers carrying training priors — an effect round-2 syntax records as admissible but unquantified) and R4 (diagnostics cite names that state checked invariants). The uniq-not-mut ruling is owner ruling D3 (v0.2 gate, 2026-07-03) resting on an analytic divergence argument, not an A/B. | The R6 half needs no experiment and is the audit's only completed backend-coupling check. Completing the rest: a naming A/B across W1 tiers (uniq vs mut error rates) and the backlogged full-lexicon Rust-divergence audit. Two-axis mode vocabulary DEFERRED with recorded delta per META-5. |
| GRAM-1 | Deterministic unambiguous grammar; 1:1 productions; no desugaring | ✅ derived | W3 (canonical bytes leave nowhere to hide edits) + R4/DIAG-1 (diagnostics cite exactly what the writer wrote; no desugared ghost forms) -> round-4 decided regularity invariants (one spelling per construct, no sugar/inference in the writable surface), retained under W1 grounding by the D2a clause now in the constitution's own W2 text; one-parse determinism is a precondition of FORM-1's single legal byte form and DIAG-2's acceptance-decidable-from-artifact. | Caveat: the claim that regularity invariants survive under W1 grounding is an owner assertion (D2a), not yet experimentally validated — all three round-4 topics (spec-budget, familiarity, compiler-as-teacher) still sit at research_needed. The two-token-lookahead bound is an unexamined implementation detail; harmless because any resolution preserving one-parse serves the chain. |
| GRAM-2 | Item grammar: fn, struct, enum, contract, conform declarations | 🟡 existence-only | Existence: PROG-1 closed world + round-2 decided law require a closed declaration inventory. Generics production: R2 (round-2 generics keep_essential, checker-collapse precedent; Go pre-1.18 natural experiment) + D001 CONFIRMED (monomorphization) -> P0. struct/enum: round-2 error-handling and dispatch decided law (ERR-1, D002). But contract_decl/conform_decl transcribe FN-3, shipped normative while the round-2 interfaces verdict sits at needs_evidence (audit procedural breach); the doc production transcribes FORM-4, an audit-flagged human-residue rule with no recorded R1 grounding. v0.18 adds the `program_kind` and `input_label` productions and modifies `fn_decl` and `param`; both new productions transcribe FN-7's entry forms and therefore inherit FN-7's status rather than adding an independent one, exactly as `contract_decl`/`conform_decl` transcribe FN-3. PROG-1's closed world still forces a closed declaration inventory, and the two productions add no new item kind. | Mixed rule: the generics/struct/enum/fn productions are fully derived; contract_decl/conform_decl inherit FN-3's blocking interfaces back-fill experiment (Go interface{}-boxing as reference failure mode); doc awaits the R1 justification pass or an explicit R5 exception in user-directives.md. Byte-level choices (semicolons, field order) are FORM-2-class, provisional. |
| GRAM-3 | Type and mode grammar; modes always written, region-explicit | ✅ derived | T1 via explicit modes/regions: D1a checker-simplification lever (explicit regions/borrows over inference, owner-sanctioned) -> checker-core prototype PASSED 2026-07-02 (D1a gate) -> P1 feasibility; P0 via 'region-explicit borrows' as a named R0 delta of record and F001 (ownership/exclusivity = the optimizer's noalias fact base); `&uniq` spelling fixed by LEX-1 owner ruling D3 with divergence-census rationale. | Leaf productions inherit open items ledgered elsewhere: array/slice inventory (see TYPE-2; arrays-loops gate prototype_needed) and const := literals only (see CONST-1). The two-axis mode vocabulary (exclusivity x write-permission) is a recorded LEX-1 DEFERRED delta; the full-lexicon Rust-divergence census is still backlog per the v0.2 gate entry. |
| GRAM-4 | Statement inventory: let, set, loop, break, region, check, match | 🟡 existence-only | Existence and most members derived: region_stmt is the OWN-3 lexical-region D1a lever -> T1; check_stmt is the stated-and-checked channel from the round-3 proven-else-checked decision -> W3/T2; let with mandatory mode/type feeds TYPE-5's boundary-explicit floor. But loop_stmt/break_stmt bare form is the R3-register ANCHOR case (the register's 'GRAM-5/6' citation maps to these v0.2 productions): chosen for grammar minimality, counted/structured/iterator alternatives never evaluated for W1 error rates or provability (backlog r3_loop_form). | Provisional parts: the loop form (register anchor; arrays-loops gate still prototype_needed, and the audit flags an active R1-performance tension — bare loop+break surfaces zero A005 facts the leading candidate supplies by construction); the match arm shape (GRAM-7 register item); and check's free-text STRING message (audit human-residue flag — DIAG-3 already mandates rule ID + node path; needs an R1 grounding or recorded R5 exception). The loop-form evaluation folded into the arrays-loops prototype would complete the loop half. |
| GRAM-5 | Expression/place grammar: explicit move, borrows; prefix deref/index places | 🟡 existence-only | Semantic skeleton derived: explicit `move` makes OWN-1 affine consumption syntactically visible -> T1 + W3 (no hidden ownership transfer) + W1 (weak writer sees consumption); borrow_expr with mandatory REGIONID is the D1a explicit-borrows lever -> T1 -> F001 noalias facts -> P0; index-as-sole-place feeds OP-4 bounds -> T2 via round-3 proven-else-checked. But the deref(p)/index<T>(p,i) PREFIX spellings are on the R3-provisional register: one-spelling minimality choices with no AI-codegen evaluation on record (the audit's thinnest, single-audit item). | What the checker consumes (explicit move/borrow/index nodes) is derived; the surface spellings are untested — the AI-codegen validation harness (audit new obligation, blocks all register items) would complete this. Additional unflagged gap: `construct` is positional (fields in declared order) with no named-field form, never evaluated for W1 field-order transposition errors; not on the register. |
| GRAM-6 | No operators, no if/while/for; match conditionals; loop+break iteration | 🟡 existence-only | Existence of ONE canonical operator/control surface: R3 one-way + FORM-1 + W3 canonical bytes. The specific form is minimality-selected on all three axes (audit, three-auditor consensus): no-if first appears in kernel-spec-v0 with no debate, card, or gate entry behind it, and D002 grounds exhaustive-match dispatch, not scalar boolean conditionals; the prefix-only arithmetic surface has all AI-writer behavioral claims EMPIRICAL-UNTESTED per round2-syntax (1.35-1.48x token proxy unpriced, precedence-misbinding vs transcription-error rates unmeasured, training-distribution penalty unquantified); loop+break froze the weakest-fact iteration form while the round-1 arrays-loops gate (prototype_needed) has a leading candidate supplying A005 consumer facts by construction — an open R1-performance/P0 tension. | Three register items converge on this rule (loop form, no-if/match-only, prefix surface). Blocking registered experiments: syntax A/B with real tokenizers and whole programs, conditional-form A/B (match-on-Bool vs dedicated two-arm branch) across W1 tiers, loop-form evaluation measuring error rates AND trip-count/independence provability. Time-urgent: FORM-1 makes any later evidence-driven switch a breaking canonical-form change. Round-4 familiarity re-scoped to evidence input only, so familiarity cannot rescue or condemn this form without measurement. |
| GRAM-7 | match in two productions (statement + let-init); value-match via give; helper-fn idiom deleted | 🟡 existence-only | Existence: R3 demands exactly one match form (statement XOR expression), so a rule fixing the category must exist. This form: the spec-critique round-1 record posed exactly two candidates and picked statement-only to 'preserve one arm shape' — cheapest-to-specify, the selection ground R3 explicitly disallows — while the same record calls conditional initialization 'the most common pattern an AI writer needs' (audit, verified against the raw critique record). | Weakest form-selection in section 3: the recorded ground is the literal R3 disqualifier and the record's own W1 signal points toward the rejected value-arm candidate. Completion: the registered conditional-form A/B including helper-fn vs value-arm idiom cost. Time-urgent — FORM-1 plus EX-1's normative bytes bake the helper idiom in, making an evidence-driven switch to expression-match a breaking canonical-form change; the audit lists GRAM-7 among the most plausible R2-reversal sites. |
| TYPE-1 | Primitives: fixed-width ints, IEEE floats, unit; Bool excluded | ✅ derived | P0: machine-code performance forces machine-width types; round-2 decided law 'one numeric mode per arithmetic node' presupposes this fixed-width typed inventory (round-1 numeric-semantics debate; N001 CONFIRMED explicit UB/poison optimization contract, N002 nsw->poison corroborated). The R0 delta of record lives in OP-1's per-node modes, not the inventory — parity with Rust's primitive set is acceptable here because the set is hardware-forced, not a design choice. | The Bool-as-prelude-enum parenthetical inherits the provisional conditional-form register item (no boolean literals; GRAM-6/PRE-1/FORM-5). Inventory cuts are undebated: no i128, no pointer-width usize/isize (len returns u64) — no card or gate entry either way; a W1/P0 evaluation of index-width ergonomics would close it. N002's wraps-modulo verbatim quote remains PARTIALLY_VERIFIED pending a sectioned LangRef fetch (card hygiene, not form risk). |
| TYPE-2 | Composites: struct, enum, array, slice, box, arena | 🟡 existence-only | struct/enum: round-2 decided law — Result+trap error handling (ERR-1) and exhaustive match as core dispatch, grounded in D002 CONFIRMED (vtable dispatch costs) -> P0/W3. box/arena: round-2 memory-automation replace_with_alternative decided law ('no GC/pervasive RC; checked ownership + explicit storage contracts') + D1a lexical-region calculus -> T1 -> F001 -> P0. array<T,N>/slice<'r,T>: existence from P0 + T1/T2 (bounds-checkable region-carrying views, OP-4), but these specific minimal forms were fixed while the round-1 arrays-loops gate sits at prototype_needed with a leading candidate carrying shape/trip-count/independence facts by construction (A001/A005/A008/A009-class). v0.18 adds the opaque system types as a distinct class. Their properties are forced downward rather than chosen: no writer-visible field, variant, literal, size, alignment, or representation follows from SYS-2's compiler-owned inventory; a bare TYPEID with no `targs` is a complete written type under GRAM-3 and region-free under STOR-5, so it may be stored; and OWN-1 affinity follows from each type's single owner. The composite-constructor inventory is unchanged and no new region-bearing form is introduced. | Four of six constructors fully derived; the array/slice halves need the arrays-loops prototype to complete — richer fact-carrying array types (MemRef strides, Futhark size-in-type, Chapel domains) are carded but were never run against the minimal form. Not on the R3 register by name, but the audit's loop-form R1-performance tension is the identical pattern one production over; flagged here honestly rather than laundered under the register's silence. |
| TYPE-3 | All types/modes/effects nameable without compiler execution | ✅ derived | W1: a weak writer must be able to write any needed type from the in-context spec alone — compiler-execution-dependent or unnameable types break that categorically; coherence precondition of the decided boundary-explicitness floor (round-2 explicit-instantiation law + FN-1 signatures + D2-corrected uniform annotation): if every let and call site must state types, every type must be finitely writable; W3/FORM-1: one canonical name per type is the one-spelling rule lifted to the type level. | A coherence requirement, so no experiment is needed for the rule itself. It holds only because FN-5 removes unnameable types (closure types are the standard counterexample); if the provisional FN-5 env-struct replacement is revised after its registered A/B against restricted closures, TYPE-3 must be re-verified against the replacement's type grammar. |
| TYPE-4 | No implicit conversions; explicit cvt returns Result when narrowing | ✅ derived | Round-2 decided law verbatim ('zero implicit value-changing conversions', a floor since 2026-07-02) -> W3 (a silent value change is exactly the corruption/cheat channel the writer must not have) + R4 ladder (narrowing is a check-time Result value, never silent truncation; total widening stays pure) + T2 no-UB envelope (N001 CONFIRMED poison contract; N002 nsw->poison corroborated by the verified-UB manual). R0 delta nameable: Rust's `as` silently truncates; cvt makes narrowing a checked Result. | The strongest-derived rule in section 4: existence and specific form (total-vs-Result split) both trace to decided law plus the R4 ladder. Residual debt is card hygiene only: N002 remains PARTIALLY_VERIFIED, and the audit lists N001/N002 verification as a section-4/7 ratification precondition — N001 was discharged 2026-07-06; N002's verbatim quote is pending. |
| TYPE-5 | No cross-statement inference; full annotations everywhere; explicit instantiation | 🟡 existence-only | Boundary half (signatures, call-site type/region/const arguments): decided floor — round-2 monomorphization-with-explicit-instantiation law + D2-corrected owner ruling (program verbosity free; explicit facts beat generation tokens) -> P0 via the 'more optimizer-visible facts than rustc emits' R0 delta of record. Interior half (every let fully annotated): the round-2 type-inference verdict still sits at needs_evidence, yet the mandate shipped normative — audit procedural breach; the deciding redundancy-independence experiment (correlated-annotation-noise vs checked-redundancy error correction, O(k^2) token/cascade costs) never ran. | On the R3 register; the redundancy-independence experiment is a BLOCKING ratification precondition (audit adoption item 2). Post-D2a the spec-budget objection to inference is moot, but that removes an opposing argument rather than supplying W1 selection evidence — the derivation still needs the experiment. The merged audits split on blocking vs non-blocking-confirmatory priority; an owner ruling is owed. The boundary half would remain derived even if the interior half is reversed. |
| TYPE-6 | Declaration-before-use; no shadowing; disjoint namespaces | 🟡 existence-only | Existence: GRAM-1 determinism + W3 demand a name-binding rule (ambiguous or context-dependent resolution would break canonical one-parse and checker soundness), and declaration-before-use suits the no-inference D1a checker direction. The no-shadowing/no-redeclaration form: recorded origin is verbatim 'cheapest for a canonical-form language' (spec-critique-round1-raw.json) — the literal R3 disqualifier; not among the D1a checker-simplification levers; no card or experiment weighs fresh-name burden against stale-binding confusion for W1 writers. v0.18 adds admitted system declarations as a third source in the lexical-IDENT, nominal-type, and constructor rows (SYS-1). The rule's own judgments are unchanged: whole-unit uniqueness, grammar-role domain selection, and collision-rather-than-shadowing all apply to system entries exactly as to PRE-1 entries, and a source declaration colliding with an inventory entry rejects the unit with neither declaration resolving. | Weakest provenance on the register: uniquely, no gate entry flagged it or scheduled validation before the 2026-07-05 audit. Registered completion paths (audit obligation c): A/B shadowing-on/off across W1 model tiers, or re-ground no-shadowing as a D1a checker lever with evidence recorded in notes/checker-feasibility-findings.jsonl. Also an R2-reversal watch site alongside GRAM-7 and no-if. The namespace-disjointness, declaration-before-use, and label-scoping clauses are the derived residue. The v0.18 third source neither increases nor discharges the no-shadowing derivation debt — the fresh-name-burden experiment remains unrun, and it is now unrun over a larger reserved surface in every compilation unit. |
| CONST-1 | closed non-arithmetic const-expr sublanguage; const N usable/forwardable (D8 closed) | ✅ derived | Existence: round-2 decided law — 'no in-language metaprogramming (closed constant-expression sublanguage only)' — demands a constant-expression rule, and array<T,N> (TYPE-2/GRAM-3) forces at least integer literals. THIS form (literals only, sublanguage DEFERRED with recorded delta): an admitted stopgap the audit lists as a live constitutional tension — the metaprogramming ban is law while its replacement is deferred, inviting exactly R2's own cited natural-experiment failure mode (Go pre-1.18 go:generate: unchecked external templating filling a language gap). | Ratification precondition per the audit's extended list: deliver the closed constant-expression sublanguage (+1 section META-5 delta) or produce evidence that weak writers need no generation-time abstraction beyond monomorphized generics; the audit additionally mandates a monitor for external-templating emergence in any writer corpus. The R2 exposure grows the longer v0 persists, so this is a time-sensitive debt, not passive backlog. |
| OWN-1 | Single ownership; copy/affine classes; explicit and contextual consumption; whole-binding death | 🟡 existence-only | T1 (both derivations: P1 via R4/W1/W3, and P0 via F001 noalias fact base) -> round-2 memory-automation decided law (checked ownership replaces GC/RC; evidence asymmetry F001/F006/C004 vs F007/J004) -> affine+move form per D1a owner-sanctioned levers (K001 Austral use-once linearity, K003 Featherweight Rust copy/move semantics) and spec-critique v0.1 blocking fix (copy/affine classification + explicit `move` spelling). Explicit move serves W3/R4 in ordinary expression contexts; the closed OWN-13 match and v0.13 ERR-3 propagation contexts state their consumption structurally and therefore consume a direct bare affine own-rooted place without hiding whether consumption occurs. | Core (single owner, affine consumption, copy/affine split) is fully derived and prototype-tested (D1a gate 19/19). Outside the two closed contextual-consuming forms, bare affine use still rejects at check time. Demoted for the strictness knobs: whole-binding death on partial moves and no-reinitialization are completeness cuts STRICTER than the reconciliation target (K003: FR handles partial moves), with sound-program rejection rate unmeasured — same debt class as register items OWN-3/8/11. Completing steps: FR mapping (section-5 ratification precondition) plus rejection-rate/repair-loop measurement on the R3 validation harness. |
| OWN-2 | Three borrow modes (own, shared, uniq), always written | ✅ derived | T1 via F001 exclusivity — shared-XOR-unique is the verified Rust/FR natural experiment (F001, C004, K003) -> OWN-9 noalias facts -> P0. Region-explicit borrows are a named R0 delta of record in the constitution. Always-written modes = D1a lever 'explicit regions/borrows over inference' (checker-feasibility findings, owner ruling 2026-07-02) + D2 corrected ruling (program verbosity acceptable; explicit facts beat generation tokens). `uniq` naming = owner ruling D3/LEX-1 with divergence-census rationale (exclusivity is the invariant, mutation only its permission). | Interior application (modes on every let) shares TYPE-5's R3-provisional debt — the round-2 type-inference verdict sits at needs_evidence and the redundancy-independence experiment is unrun; the boundary half is a decided floor. Two-axis mode vocabulary (frozen/shared-write) DEFERRED with recorded delta under LEX-1. Any spec-uniformity argument here is W2-flavored and weak post-D2a; the load-bearing grounds (D1a checker feasibility + F001) are independent and stand. |
| OWN-3 | Lexical regions with outlives-or-equals; incomparable caller regions fail closed | 🟡 existence-only | Existence: T1 requires a region model to bound borrow liveness; region-explicit borrows are a named P0/R0 delta over Rust. Form: lexical-only regions and unique names are D1a owner-sanctioned checker levers ('lexical borrows before NLL-class flow sensitivity'; K001 Austral scope-delineated borrows, K002 Polonius declarative-rules evidence); fail-closed incomparability is the W3/OWN-8 posture (never guess an ordering). | Explicit R3-provisional register item ('checker completeness levers OWN-3/8/11 — rejection-rate unmeasured'). Selection ground is the D1a implementation-effort ceiling (owner-sanctioned), not evidence the completeness cut is cheap for AI writers; sound-program rejection rate and restructuring churn unmeasured. K004 Oxide is the recorded NLL upgrade path if lexical proves too lossy; FR (K003) reconciliation is the ratification precondition. |
| OWN-4 | Borrows live to region end; flow only outlives-ward | ✅ derived | T1: dangling references unrepresentable -> given OWN-3's lexical regions, a borrow may be stored/passed/returned only into regions it outlives — the unique sound rule for that skeleton. Edge-case review validated the form: the v0 direction was INVERTED (unsound), found independently by two critics (spec-critique round 1, v0.1 blocking fix) and now prototype-tested (D1a gate, 19/19). | Inherits OWN-3's provisional lexical-liveness lever (named-region liveness, not last-use). The inversion history is precisely why FR (K003) reconciliation remains the completing step: the checker-prototype report records that rule-text soundness at scale, not code volume, is the residual risk. |
| OWN-5 | Resolved-place exclusivity; no moves through borrows; finite direct-slice origins | ✅ derived | T1 via OWN-5 exclusivity (the constitution's T1 clause cites F001 by name) -> OWN-9 noalias facts -> P0/R0. Independently P1: W3 — the writer cannot fabricate aliased mutation; R4 — violations reject at check time, never surface at runtime. Form: shared-XOR-uniq with reads-under-shared is the verified natural-experiment semantics (F001; C004 data-race half); restatement over resolved places and the move-through-borrow ban are spec-critique blocking fixes closing demonstrated holes. v0.17 extends only direct slice values: the closed producer set yields finite static origin sets that overapproximate one runtime root, and every access is checked against every member. | The best-derived rule in the section — it is T1's own anchor. Borrow holders remain singleton-rooted under the FR reconciliation; direct slices use the separate finite-origin proof in the v0.17 amendment and reconciliation note. The P0 payoff remains in-kind, not in-magnitude (no noalias on/off benchmark exists; Rust noalias enable/disable/miscompile history still sits in the missing-research backlog). v0.20: the creation carve-out, the suspension parenthetical, and the never-both-usable pair name the arm-scoped child (OWN-13) and the returned reborrow (OWN-14) beside the statement-scoped child; the exclusivity invariant and its judgment are unchanged. |
| OWN-6 | Borrow holders; place resolution; statement-scoped temporaries | ✅ derived | T1: without holder resolution, places rooted at holder bindings alias &uniq content invisibly — a demonstrated v0 soundness hole (checker-prototype v0.1 report: holder-rooted aliasing; resolve() added as a blocking fix). Form implemented and machine-tested (19/19); OWN-5/OWN-7 judgments are defined over resolved places, making the exclusivity theorem actually enforceable. | Counterexample-driven provenance. The call-scoped temporary clause (un-let borrows live to end of enclosing statement) is an untested detail choice; FR reconciliation (K003 covers reborrowing) should confirm the resolution rewrite generalizes. v0.20: one added cross-reference hands every written non-argument reborrow occurrence to OWN-14 and the derived match-payload binder to OWN-13; the argument-position child rule is unchanged. |
| OWN-7 | Conservative overlap: prefix, literal-index disjointness, pairwise slice origins | 🟡 existence-only | Existence: T1 — OWN-5 is vacuous without an overlap judgment. Form: the prefix rule gives exact struct-field disjointness; index disjointness only when both indices are unequal literals is conservatism selected under the D1a soundness-over-completeness lever. v0.17 replaces the former blanket slice-overlap approximation with the forced set lifting: two fully substituted slices overlap when any pair of possible resolved-place origins overlaps; immutable const needs no write-conflict claim. | Rejection cost on AI-written array and multi-origin slice code remains unmeasured. Formal origins never prove actual arguments disjoint, and no runtime branch narrows the static set. This still interacts with the arrays-loops gate and A005-style dependence facts; a future runtime-disjointness check would require its own approved fact path. |
| OWN-8 | Reject-when-unsure; diagnostics cite rule and restructuring | 🟡 existence-only | Existence forced: W3 + T1 — accepting unproven programs either trusts the writer (W3-forbidden) or voids the theorem, so the checker must fail closed; owner ruling D1a adopts the lever near-verbatim ('checker may reject sound-but-hard patterns and demand restructuring'). The diagnostic clause chains to R4 (rule-citing check-time rejection) and DIAG-1. R0 note: the posture is Rust-like, but no writer-accessible unsafe escape (W3) is the named delta. | Explicit R3-provisional register item. The posture's EXISTENCE is fully forced; what is untested is its W1 price — sound-program rejection rate and repair-loop convergence (R2 exposure: a checker cut that inflates rejections harms AI codegen, the audit's D1a-R2 tension). Rejection-rate measurement accompanies the section-5 formal-calculus reconciliation as a ratification precondition. |
| OWN-9 | Non-normative optimizer noalias consequences of borrows | ✅ derived | P0 directly: F001 (exclusivity -> unaliased access paths) + F002 (non-interference around writes is the payoff class) + F003/F004 (LLVM noalias encodings consume such facts) — this rule is the R0 delta of record ('more optimizer-visible facts than rustc emits') rendered as spec text. R1 satisfied via P0 service; non-normative, so it costs no acceptance semantics; LEX-1 keeps backend vocabulary (noalias) out of the writer surface. | Payoff is in-kind, not in-magnitude: no noalias on/off benchmark exists anywhere in the corpus (round-1 aliasing evidence gap; Rust noalias incident history still in the missing-research backlog). If magnitude measures small, this rule's P0 framing weakens, but the underlying rules' T1 grounding is unaffected. v0.20: the non-normative suspended-pair note extends to the arm-scoped child and the returned reborrow; the guarantee (one usable mutable path per place) is unchanged. |
| OWN-10 | Borrowed storage must outlive the borrow's region | ✅ derived | T1 with a concrete counterexample: the v0 spec+prototype ACCEPTED returning a borrow of an own parameter into a caller-supplied region — a dangling reference (checker-prototype v0.1 report; the former positive test became the negative test). Rule added as a v0.1 blocking fix; three-case form (own-rooted / borrow-rooted / arena-rooted) is the closure of that hole across all place roots; implemented and tested (19/19). | Strongest provenance short of formal proof: counterexample-driven, then machine-checked. The arena-rooted case depends on STOR-4's region-carrying types (whose family evidence is thinner — see STOR-4); FR reconciliation remains the ratification gate for the general formulation. |
| OWN-11 | Loops: no outer-region borrows or outer moves inside | 🟡 existence-only | Existence: T1 across back-edges — a flow-insensitive checker must prevent per-iteration re-moves (use-after-move) and borrows that outlive an iteration. Form: the bluntest sound cut (ban outer-region borrows and outer moves entirely), adopted as a D1a effort lever via the v0.1 blocking fix; audit register item OWN-3/8/11. | Rejection rate on realistic loop code unmeasured and NOT covered by the checker prototype (v0.1 remaining-scope gap). Interacts with the R3-provisional loop form (GRAM-5/6, the audit's anchor case) and the open arrays-loops gate (A005 trip-count/reduction facts): the registered loop-form experiment should evaluate OWN-11's restructuring cost jointly. FR back-edge treatment is part of the reconciliation. |
| OWN-12 | Call boundary: region substitution, borrow arguments, effect-row check | 🟡 existence-only | Existence: T1 interprocedurally — without a call rule, two overlapping &uniq arguments and callee writes behind caller borrows are representable unsoundness (spec-critique round-1 blocking finding; the OWN-CALL cluster fix converged across critics). Form: signature-carried facts (FN-1 discipline) judged under OWN-5 at instantiated regions — the critique's own fix text, near-verbatim. | Existence is forced; this exact instantiation discipline was never machine-checked — OWN-12 is in the checker prototype's remaining-scope gaps, and the effect-row clause leans on Section 9, itself gated on effect-exemplar carding before ratification. FR (K003) call-boundary mapping plus prototype extension would complete the chain. |
| OWN-13 | Match moves owned scrutinee; binder modes derived from scrutinee | 🟡 existence-only | Existence: T1 — match/ownership interaction was wholly unspecified in v0 (flagged by all four critics; binding payload by value through a shared borrow would move out of borrowed content). Form: the critique record poses two candidates — written binder modes (per OWN-2's no-default stance, critic C) vs uniformly derived modes (critic D) — and the spec chose derived-not-written on one-uniform-rule/stated-once grounds, the minimality basis R3 disqualifies as a selection ground. | Creates the kernel's only unwritten mode, in tension with the OWN-2/TYPE-5 always-written regime; whether weak writers correctly predict binder modes is a W1 question with no experiment run. Not prototype-covered (v0.1 remaining-scope gap). Post-D2a the stated-once ground is W2-flavored and weak. Completing steps: match-binder written-vs-derived A/B on the R3 harness, plus FR-style model checking of the match rules. v0.20: the borrow-mode payload binder becomes an arm-scoped child reborrow of the scrutinee's root binding — resolution and sibling judgment reuse OWN-6's machinery, a `uniq` root is suspended with non-resumption inside its region falling out of OWN-4's existing named-region liveness (no new clock), and shared roots stay plain overlapping shared borrows. Closes the v0.19 OWN-13/OWN-5 contradiction (a derived `&uniq` binder every use of which OWN-5's exception list confiscated), witnessed by `own13-pos-uniq-match-payloads` after its approved binder rename and reproduced against the conforming compiler. Arm-end resumption is the recorded relaxation candidate, blocked on the PROVISIONAL arm-result join settling when an escaped binder borrow ends. |
| OWN-14 | Non-argument reborrow disposition; returned reborrow | 🟡 existence-only | Existence: the v0.19 gap is recorded evidence — OWN-6 defined reborrow forms only in call-argument position, three conformance cases exercise the return-position form, and the 0024 investigation stopped rather than invent a rule; the live no-reborrow decision node cards result reborrows as the standing evidence-first relief valve (2026-07-08 fact). Form: admission is by subsumption over already-derived law — a returned reborrow of holder h is caller-indistinguishable from the already-legal `return h;` (OWN-4) with an equal-or-stricter region obligation (OWN-10 borrow-rooted case) and an equal-or-narrower place, and return position is the sole non-argument position where no program point observes parent and child both usable, so OWN-5's exclusivity invariant is preserved by the same suspension already used for statement-scoped children. | v0.20 addition. Existence-only because the return-position-only scope and mode preservation are completeness cuts in the OWN-3/8/11 class: the deferred forms (bound, give-position, stored, downgrade, match-binder parents, opaque call-result roots) await their own owner-gated reviews, and the FR reconciliation of this edge (returned reborrow as FR reborrow plus return) is owed with the section-5 ratification pass. |
| STOR-1 | Storage class by type: box heap, arena region, else frame | ✅ derived | Existence: round-2 memory-automation decided law — explicit, checked storage contracts replace GC/pervasive RC (evidence asymmetry: F001/F006/C004 checked-ownership facts vs F007/J004 conditional, never-guaranteed escape analysis) -> T1 + P0 (guaranteed frame residency is the R0 delta vs managed runtimes). Form: v0's stack-default-with-unless clause was unanimously blocking in the spec critique (META-2/META-3; round-4 zero-context-dependence, retained under W1 per D2a); storage-as-function-of-type follows from region-carrying arena types (the STOR-4 soundness fix) plus META-4 non-redundancy. foreign_shared reservation + demotion-as-floor-violation is round-3 decided law verbatim (foreign-retainable memory pre-declared at allocation site; compiler-inferred demotion REJECTED), rescoped by D4. | Type-carried vs per-binding storage annotation was never W1-tested (the checked-redundancy question shared with TYPE-5's unrun experiment), but the annotation would duplicate information the type already spells, so the candidate axis is thin. Vs Rust the storage classes themselves are parity; the R0 delta lives in the contract vocabulary and STOR-3's artifact-surfaced deallocation, not here. |
| STOR-2 | box_new/arena_new creation calls; deref content access | ✅ derived | P0: the `own box<T>` return is verified card F004 made a typed invariant — LLVM's noalias return models a fresh allocation-like result, the strongest allocation contract. T1: arena_new's region-carrying return type is the STOR-4 soundness fix (escape locally decidable per META-2). Allocation-as-ordinary-table-call feeds EFF-2's syntactic allocates rows -> W3 (effect rows checkable both ways, no hidden allocation). | Surface spellings inherit R3-provisional register debts ledgered elsewhere (OP-1 prefix-call surface; GRAM-5 deref place form). allocates(heap/arena) row semantics lean on Section 9, gated on effect-exemplar carding. F004's payoff magnitude is unmeasured, as with all noalias facts. |
| STOR-3 | Compiler-derived drops and releases surfaced on every region-exit edge | 🟡 existence-only | Round-2 memory automation requires compiler-derived cleanup to remain explicit in the checked program. T1/R4 require every normal exit to dispose each owner exactly once; W3 forbids hidden writer finalizers; P0 permits compiler scheduling and elimination only after semantics are explicit. v0.37 keeps release as per-type semantic data but replaces the old memory-row plus family-authority pair with one ordinary state-effect row and one suspension/milestone target contract. The table-local `owner` path projects through existing move identity under EFF-2. | The compiler-derived-versus-writer-written spelling experiment and reverse-declaration-order experiment remain open. Memory reclamation rows stay empty. System releases may carry `writes(owner)`, and fresh local release frames out without parent ancestry. |
| STOR-4 | Arena values confined within their region's block | 🟡 existence-only | Existence: T1 forced by counterexample — v0's arena escape was proven unsound by two blocking critics (alloc_arena returned bare `own T` carrying no trace of 'r; moving past region end was well-typed use-after-free, and the claimed OWN-4 enforcement could not fire). Form: the critique's own fix — region-carrying arena<'r,T> makes confinement locally decidable (META-2), with content borrows routed through OWN-10. | Untested on two axes: arena content borrows/slices sit in the checker prototype's remaining-scope gaps, and the region/arena mechanism family still has ZERO exemplar cards — Cyclone/MLKit (incl. leak pathologies)/Koka/DPJ remain in the missing-research backlog, a condition the round-2 verdict itself set ('mandating it today would violate the standing rule'). K003 Featherweight Rust has no arena construct, so FR reconciliation does not discharge this rule; it needs its own family carding plus the round-2 requirement-(d) region-quality/timeliness lints. |
| STOR-5 | Storage is borrow-free and region-free in the enumerated content carriers | ✅ derived | T1 — the v0.7 statement-scoped child-reborrow non-escape guarantee requires that a borrow cannot be stashed in storage. v0.17 closes the analogous returned-slice hole: a region-bearing leaf hidden in a struct field, enum payload, array/buffer element, or box/arena content would require per-leaf provenance and cleanup state that the direct FN-1 signature cannot express. W3 forbids approximating that hidden state, and R4/OWN-8 select explicit rejection until the separate stored-value design exists. The recursive post-substitution check closes generic and `box_new`/`arena_new` laundering through one ordinary type judgment. | Borrow storage remains grammatically absent. Region-bearing content is now a real source rejection: function and nominal generic arguments are owned by FN-2, while enumerated stored-content positions are owned by STOR-5. A direct nested-slice type is not itself one of those positions and remains language-valid but compiler-unsupported; ordinary `slice_of` cannot inhabit it through an array or buffer because that source element position is prohibited. No runtime representation or check changes. |
| STOR-6 | Selected-target layout and exact target-domain allocation/address arithmetic | ✅ derived | T2 requires every materialized value and address calculation to remain defined on the selected target; R4 requires failure before allocation or address formation; R6 forbids replacing the target constraint with a backend-specific source-language size cap. DIAG-2 and the facts-off invariant require one checked target-stage obligation which combines the selected target's concrete layout and address-index domain with already-proved source ceilings. Every such obligation must be discharged before emission; a runtime guard, hidden fallback, or target-specific narrowing of the source `u64` domain would move an unproved address into execution and is therefore unavailable. | A static target-layout or target-domain failure is a target compilation failure: it neither rejects the source under DIAG-1 nor creates a source-language runtime path. External allocation availability remains the separate SCOPE-3 resource boundary; it does not weaken the mandatory layout, stride, allocation-ceiling, or address proof. |
| OP-1 | All computation as mode-named prefix calls from operation table | 🟡 existence-only | P0/R0 delta of record 'per-node numeric modes' + round-2 decided law (one numeric mode per arithmetic node; zero implicit conversions; nothing overloaded) + N001 (definedness lattice is an optimization contract; CONFIRMED 2026-07-06) + N002 (wrap-vs-nsw contract split; PARTIALLY_VERIFIED) + round-1 numeric-semantics leading candidate (truthful-lowering explicit-mode architecture, the only position robust to the soundness ruling) -> a mode-explicit, non-overloaded operation inventory must exist. The prefix-call-only writer surface is R3-provisional: constitution-audit register lists OP-1/GRAM-6 explicitly; per round2-syntax all AI-writer behavioral claims are EMPIRICAL-UNTESTED. The v0.8 `eeq`/`ene` addition is evidence-selected rather than inferred from existing compiler code: the investigation-baseline compiler census found 253 non-integer equality sites across 93 functions, 18 files, and 22 tag-only types; the intervening v0.7 slice leaves 251 sites across 92 functions and the same 18 files/22 types, and the live census must be rerun immediately before landing. The accepted v0.7 structural fallback requires 6,952 variant-pair arms across the 21 non-`Bool` types, with 11 `Bool` sites using existing Boolean operations, while the smaller 21-mapper/262-arm/484-call form needs an external injectivity guard because v0.7 does not reject duplicate mapper result codes. Same-variant identity over one exact nominal tag-only enum is a finite, pure, total operation, so the operation-table row closes that W3 gap without adding conversion, ordering, payload observation, or a safety-relevant fact channel. v0.18 adds an admitted system operation as a third callee source for an IDENT callee (SYS-1); absence from the selected operation-family, function, or system-operation inventory remains one OP-1 rejection. `ReservedLowerNames` is unchanged and every system operation spelling is verified disjoint from it as SYS-2 specification data. | The mode architecture is the best-derived half of the kernel; the surface form is not. Completing experiments: the pre-registered syntax A/B (real tokenizers, whole programs, operand-order vs precedence-misbinding error rates, repair iterations) across W1 model tiers. Residual card debts: checked-mode lowering is uncarded (round-1: only wrap and nsw lowerings are card-backed), and N002's wraps-modulo verbatim quote is still pending a sectioned LangRef fetch. The `eeq`/`ene` spelling has no independent weak-writer A/B; its selection is constrained by the already-derived OP-7 domain-prefix rule and by the rejection of overloaded `ieq`/`ine`. Payload equality and structural equality remain unselected. |
| OP-2 | No wrap modes for division, remainder, negation overflow | ✅ derived | Div/rem half: T2 — no modular-arithmetic semantics exists for divisor zero, so a .wrap form would be UB (T2 violation) or an untruthful lowering (W3/F003); N003 (LLVM div-by-zero is immediate UB) prices the hazard; R4 leaves .trap/.checked as the ladder. META-3 (no exception clauses) dictates the table-data form, per the v0.1 delta 'OP carve-outs became table data'. Negation half: does NOT follow from the stated divisor-zero argument. | Edge-case finding: two's-complement wrapping negation IS sound, well-defined modular arithmetic (Rust ships wrapping_neg; LLVM 'sub 0, x' wraps), so the absence of ineg.wrap is minimality-selected, not T2-forced, and OP-2's own rationale is silent on it. Either add ineg.wrap or restate OP-2 to justify the negation exclusion honestly. N003 is workflow-confirmed but absent from both independent edge-case verification rounds; the div/rem argument is a semantic theorem independent of the card, so that half stands regardless. FIXED v0.3: ineg.wrap added (two's-complement wrapping negation is sound modular arithmetic); rationale scoped to div/rem where it is a genuine semantic theorem. |
| OP-3 | Strict-only IEEE float modes; fast-math open | 🟡 existence-only | Existence of per-node named FP modes: round-2 one-mode-per-node decided law + P0/R0 delta of record (no debug/release semantic divergence, per-node modes). Strict-only content: T2/W3 conservative floor — round-1 numeric-semantics found the FP corpus a near-total void ('three advocates converging on strict-default from priors must not be laundered into corroboration'), so R3 forbids adopting any evidence-free relaxation; strict is the one FP choice that cannot be semantically wrong, only slow. Marking approximation modes OPEN rather than deciding them is itself R3-compliant. | Strict is admissible-by-elimination, not evidence-selected: 'strict IEEE' vs 'LLVM default unflagged FP' is an uncarded distinction (constrained intrinsics/strictfp have zero corpus coverage), so even .strict's implementability-as-specified is unpriced. Completing work: the commissioned FP evidence phase from round-1 follow-ups (per-flag fast-math semantics, flag dropping across inlining, contraction, rounding/exceptions, reproducibility) plus a fast-math performance-delta measurement before any relaxation plank. |
| OP-4 | Every subscript proves its exact element bound before lowering | ✅ derived | T1 makes an out-of-bounds access memory unsafety, and T2 forbids emitting the access without a domain proof. ENT-6 therefore submits the exact `i < len(base)` goal to the fixed deterministic proof context; success authorizes the operation and failure rejects at the source subscript. P0 is served because no runtime guard remains on the accepted path. | Even catastrophic proof-writing or compile-cost evidence cannot reopen an unchecked fallback; it can only motivate a stronger fixed proof domain, explicit source proof, or program restructuring. |
| OP-5 | Source conditions and contract predicates have exact type `own Bool` | 🟡 existence-only | Canonical control flow and contract checking need one total predicate type whose branch edges can establish their exact positive and negative facts. W3 requires authority to come from the executed edge or a separately checked proof rather than from spelling a Boolean conclusion. | The exact `Bool` surface remains existence-only with the surrounding conditional grammar. This rule adds no runtime assertion statement, report, or effect. |
| FN-1 | Signatures fully state modes, types, formal-path state effects, regions, and direct-slice result ceilings | ✅ derived | W1 locality requires callers to check a call from its signature without opening the callee. P0/R0 require exact effect rows, while T1 requires ownership authority and effect behavior to remain distinguishable. v0.37 therefore keeps regions in modes and types for loan liveness, names affected state by parameter ordinal plus static field path, derives a result-state routing summary from existing move/aggregate/return flow, and derives suspension and completion milestones as target summary. v0.17's direct `own slice` ceiling remains unchanged. | Neither derived summary supplies alias or concurrency authority. Result routing only prevents a pass-through owner from losing the identity ordinary ownership already established; it adds no source feature or runtime identity. Contract members use the same formal-path row; only bound `fn_decl` bodies receive EFF-2 and return validation. Dynamic element effects and returned-descriptor provenance remain separate deferred additions. |
| FN-2 | Monomorphization-only, explicit, concrete-rechecked, region-free generic arguments | ✅ derived | R2 precedent (round-2 generics checker-collapse; Go pre-1.18 natural experiment) demands generics exist; round-2 verdict keep_essential conditioned on D001 -> monomorphized code performs as concrete code -> P0/R0; explicit arguments follow TYPE-5 and every instance is rechecked under W3/T1. v0.17 adds the conservative region-free boundary: a region-bearing function or nominal type argument can hide slice or arena leaves whose per-leaf provenance and cleanup are absent, so W3 forbids treating the compiler's missing state as acceptance and R4/OWN-8 select a rule-citing rejection. | This is an explicit v0.16-valid-but-unsupported to v0.17 source-rejection change, not an implementation failure recast as invalid source. Direct written slice and arena parameters/results remain available where their owning rules admit them. General per-leaf generic metadata is a deferred amendment. The earlier drift and check-loop-latency debts remain. |
| FN-3 | Complete static source contracts and conformances; no callable behavior | 🟡 existence-only | R2 preserves the need for an eventual bounded-abstraction mechanism and a law home, but the specific behavior surface remains unselected. W3 requires every admitted conformance to be one complete checked unit. The candidate normalizes effect rows to read paths, write paths, and allocations. A path uses the root parameter ordinal and static field ordinals, so parameter spellings do not affect equality; region alpha-renaming applies only to modes, types, and arena allocations. No path or allocation may be omitted or added. | This supersedes v0.18's six-part normalization over read/write regions plus payload-free `external` and `blocks`. Static closure still does not ratify contract/conform as a behavior-abstraction surface. Source-contract generic bounds and member calls remain absent. |
| FN-4 | Mandatory checked-law admission; checked facts have only specification-fixed consumers | ✅ derived | W3 requires every writer-stated law to be discharged rather than trusted, and R4 requires an unavailable or refuted discharge to fail closed. P0 preserves the measured checked-law opportunity without creating a second proof authority: the source semantic checker establishes the closed whole-conformance/member/body/table relation once and retains the resulting checked law fact in the same in-memory checked program. A fixed transformation may consume that fact only after its own ordinary IR/target applicability test; it does not independently rederive the source relation. The diagnostic derivation explains the originating decision and never authorizes optimization. `pure` is one required premise and never proves totality or an equation by itself. | The closed LAWNAME and saturating-add table is a deliberately bounded admission calculus, not a complete algebra system. More laws, operations, or transformation consumers require a specification addition. No such transformation is active yet, so accepted laws currently change neither checks nor facts-off lowering; this is an implementation gap, not a reason to serialize the fact or add a second verification pass. |
| FN-5 | No function values, dynamic dispatch, or contract-member calls | 🟡 existence-only | Ban half derived: P0 via D002 (verified — dyn-Trait dispatch adds lookup cost and prevents inlining) + D007 (verified — devirtualization/indirect-inlining value) -> round-2 decided law 'no implicit dynamic dispatch (exhaustive match is the core dispatch)'; closures verdict replace_with_alternative. v0.16 states the present boundary honestly: a contract member is not a callee, source-contract bounds are rejected, and validated conformance metadata cannot select an ordinary call. | The replacement half remains unselected. Env-struct behavior parameterization, an explicit member-call spelling, bound substitution, and the direct-call proof are deferred until the registered env-struct-versus-restricted-closure experiment and a real program justify them. v0.16 neither supplies nor claims the old documented env-struct mechanism. |
| FN-6 | Recursion allowed; polymorphic recursion rejected syntactically | 🟡 existence-only | Round-2 decided law monomorphization-only (D001, P0) forces rejecting polymorphic recursion — an unbounded instantiation set makes compile non-terminating — and R4 places the rejection at check time. Existence is therefore mechanically derived. The specific criterion (every call in a cycle instantiates at exactly the caller's own type parameters) is a conservative decidable form, strictly stronger than termination requires (it rejects finite permutation cycles). | Same class as the OWN-8 reject-when-unsure levers but NOT in the R3-provisional register — thinnest audit coverage of the FN section; flagged here. Sound-program rejection rate for AI-written mutually recursive generic code is unmeasured; a W1-tier codegen experiment on recursive-generic patterns would complete the derivation or force a relaxation. v0.3: deliberate over-strength now RECORDED in the rule (OWN-8 reject-and-restructure posture); rejection-rate experiment registered. Form still awaits measurement. |
| FN-7 | Single main entry, capped effects, no globals/'static | 🟡 existence-only | PROG-1's closed world demands a unique entry. The command entry's explicit standard-input parameters follow from PROG-3 and the no-ambient-state rule: every system state value begins as a named ordinary parameter rather than an ambient function or retained process object. Ordinal identity distinguishes `command.stdout` from `command.stderr` even though both use `Output`; FORM-1/GRAM-8 select increasing table order. v0.37 adds `command.files: own FileFactory` so file-open authority is likewise a parameter rather than ambient process state. | v0.37 changes the effect cap from region/capability subjects to paths rooted in labelled inputs and appends the file-factory input. No card decides global-state rejection specifically; adding statics or `'static` remains a breaking META-5 delta. The input-label spelling remains proposal-selected. |
| FN-8 | Erased `requires` goals proved at every ordinary call | 🟡 existence-only | The base64 ceiling and later real clients show that one entry relation can discharge many partial operations. W3 forbids trusting the clause, so each caller proves the fully substituted goal before transfer; only then does the body receive it as a static fact. R4 makes a missing proof a call-site rejection, and P0 follows because no callee prologue or fallback branch is emitted. | The contract expression calculus remains deliberately finite and pure/total. Broader relations require more fixed proof rules or explicit source proof, not an executable contract form. |
| FN-9 | Verified normal-return relations publish facts to callers | 🟡 existence-only | Real transfer and append clients require one caller-visible relation that the callee proves at every selected normal exit. W3/T1 make that proof mandatory and reject vacuity; ordinary substitution, support kills, and finite SCC publication carry the result without opening the callee or trusting the written clause. ENT-2 through ENT-5 and DIAG-2 provide one deterministic proof system rather than a solver, runtime fallback, recognizer, or second proof authority. | The `ensures` spelling, selected-result routes, and currently admitted carrier forms remain existence-only. Wider result expressions or delivery forms require their own finite semantics and evidence. |
| INV-1 | Source-written loop-header and program-point invariant checked by fixed finite induction | 🟡 existence-only | T2 and W3 require every partial operation to be justified by machine proof rather than a writer assertion; R4 requires failure before lowering when the proof is missing. A loop recurrence cannot in general be summarized by acyclic branch propagation alone, so the writer supplies the induction proposition while the compiler checks the base case and every arbitrary reachable backedge. P0 is served because a proved invariant removes runtime guards and the proof syntax is erased. Restricting the first surface to affine integer `ile`, fixed checked arithmetic, a direct counted-body prefix, and exact exhaustion gives one deterministic algorithm with a source-size work bound and no SMT, timeout, invariant search, premise search, or runtime fallback. | The need for explicit induction and the prove-or-reject boundary are derived; the exact `invariant` spelling, direct-prefix placement, and first affine grammar are the minimal first implementation selected for the measured `weigh` path and remain existence-only until broader real programs compare their writing cost. Optional loop labels do not affect structural ownership of the invariant. |
| PRF-1 | Source-written finite affine proof checked premise-by-premise and erased | 🟡 existence-only | T2 and W3 require a missing theorem to enter acceptance only after machine proof, while compilation must remain deterministic, terminating, and directly work-bounded. The source therefore chooses the target, premise relations, and intermediate steps; the compiler snapshots the pre-proof context, proves every written `use` independently in that same snapshot, and checks one specification-fixed coefficient-one affine sum in source order. This makes checker work proportional to written proof length and eliminates SMT, timeout, premise selection, coefficient search, case search, and runtime fallback. P0 is served because the accepted conclusion can discharge partial operations and `par` obligations before the proof statement is erased. | The need for an explicit finite written proof and a fixed checker is derived. The exact `invariant`/`use` spelling, coefficient-one first calculus, and affine-only initial surface are minimality-selected for the current real programs and remain existence-only until wider source compares proof length, diagnostic quality, and missing expressivity. |
| EFF-1 | Canonical reads/writes/allocates rows over formal-rooted static state paths | 🟡 existence-only | Exact rows remain required by P0/R0, W3, and the round-3 writer floor. A region says how long a loan lives, while a parameter path says which state is touched. Two parameters may share one lifetime and one owned state resource has no borrow lifetime, so REGIONID cannot be effect identity. The selected `IDENT(.IDENT)*` path names a formal root and static struct fields; one row covers memory and system state. `external`, `blocks`, `world`, runtime-assertion effects, and a capability category are absent. | Dynamic subscripts, ranges, and enum payload paths require separate overlap rules and remain deferred. `pure` is exactly the empty reads/writes/allocates row, not termination. |
| EFF-2 | Exact exhibits relation over places; calls and releases use ordinary substitution | ✅ derived | Undeclared behavior violates W3 and overdeclared behavior violates the exact-effect P0/R0 contract. Direct accesses resolve to formal-rooted places; calls substitute each formal effect path through its actual place; holder and slice projection reuse OWN-5/OWN-6; moves preserve the ownership identity they already carry. Fresh local state frames out of the enclosing signature. `reserve(factory)` therefore contributes its own `writes(factory)`, while `create(permit)` and later writes to a fresh local result do not invent child-to-factory ancestry. Release uses the same owner identity and conservative FN-1 exit graph. | No new source move rule, resource root, parent graph, or effect domain is selected. The compiler-derived result-state summary only preserves existing ownership across calls. Framing out is not a dead-code license: the checked inner call retains its local instantiated effect and deletion still needs an ordinary closed-state, escape, result, release, and no-observer proof. A path describes behavior and never shrinks a loan. Existing box, buffer, arena, and const cleanup rows remain empty; system resource release rows are per-type state effects. |
| EFF-3 | pure licenses dedup/reorder; elimination needs termination proof | ✅ derived | P0 via F002-class redundancy payoffs requires deduplication and reordering of genuinely pure equal calls. T2 and W3 still forbid eliminating an unused call without a termination proof. v0.37 defines `pure` as the absence of every state read, state write, allocation, and trap; system I/O receives no special category or license. A parameter write remains observable even when the call result is unused. | The derivation is unchanged, but the old references to payload-free `external` and `blocks` are superseded. Kernel purity is computed by EFF-2 rather than trusted. The cost of retaining unused terminating pure calls remains unmeasured. |
| EFF-4 | Accepted source has no writer-reachable abort effect or hidden proof fallback | ✅ derived | T1/T2 and W3 require every supported partial operation to be proved before execution, while ERR-1 and ERR-3 keep expected failures as typed values. Removing the writer assertion therefore removes its abort effect, unwinding question, cleanup exception, report arbitration, and parallel failure-order problem. P0 benefits because proof erasure adds no exceptional control edge. | TCB failure and unavailable external resources remain outside source semantics under SCOPE-3. They do not authorize a writer abort construct or weaken cleanup on ordinary typed exits. |
| ERR-1 | Errors are Result/Option values; no exceptions or unwinding | ✅ derived | Round-2 error-handling verdict replace_with_alternative -> decided language law 'no exceptions/unwinding (Result + trap=abort)'. P0: F008 CONFIRMED cost card, refuted (0-3) zero-cost-EH exculpations, unwind edges taxing exactly the F002-class payoffs the aliasing corpus exists to unlock; D001 CONFIRMED — monomorphized Result lowers to concrete branch-based values, matching the F009/SIL visible-semantics lesson. W1/R4: exception safety is an implicit, cross-cutting, compiler-unchecked, rare-path invariant — the worst class for a weak writer — while Result drift is caught at every type-checked call site (shift-left). | The verdict itself rules the untested residue non-flipping: 'no untested claim, resolved either way, would put catchable exceptions or drop-during-unwind back in the core'. Honest gap: the verdict accepted the alternative only WITH named requirements, and those are largely undischarged — ERR-3's verified propagation, auto-derived context, and Result-vs-trap classification are DEFERRED, and the anti-sentinel lints exist nowhere. ERR-1 currently ships without the obligations that made it acceptable; see ERR-3. |
| ERR-2 | Exhaustive match, no wildcard arms, variant-addition edit lists | ✅ derived | W3 verbatim: 'exhaustiveness cannot be silenced' — a wildcard arm is precisely the silencing device, and the 2026-07-05 constitution amendment names no-wildcards among the scattered anti-cheat rules unified under W3. Round-2 decided law: exhaustive match is the core dispatch (D002 CONFIRMED 2026-07-02). R4 + R1: variant addition surfaces as check-time, site-enumerated edit lists instead of runtime surprises, feeding the repair loop with rule-citing diagnostics. | W3 is a P1 floor, so the unmeasured W1 cost (edit churn on variant addition across a large generated codebase, flagged in the round-2 AI-writer analysis) can price the toolchain contract but cannot flip the rule; the edit-list generator is that contract's unbuilt half. Statement-only match (GRAM-7) is R3-provisional, but that concerns arm SHAPE, not exhaustiveness — ERR-2 survives any GRAM-7 experimental outcome. |
| PROG-1 | One closed compilation unit; no ABI, loading, reflection | ✅ derived | Round-2 modules verdict replace_with_alternative -> decided language law 'closed-world whole-program compilation'; P0 via D007 (verified — LTO/IPA cross-module visibility enables devirtualization and indirect inlining) and D002 (verified — dispatch opacity blocks inlining); no dynamic loading/reflection additionally closes a W3 injection channel and keeps the T1/T2 envelope decidable over the whole program (SCOPE-2/3). v0.18 names the system declaration domain admitted to a unit as a third source of language names beside source declarations and the prelude, and states that compiler-owned system operations are implemented by an approved target entry rather than by foreign code, so they are not the external boundary (GATE-2). The closed-world derivation is unchanged: the domain adds no include, import, module, separate compilation, incremental semantic cache, internal ABI, dynamic loading, reflection, or source-path lookup, and its members are compiler-owned data of the specification rather than a second compilation input. The gated FFI wall remains the only external boundary, so the D007/D002 whole-program visibility argument is untouched. | Audit-recorded R1 tension: whole-program recompile latency at target scale (including FN-2 instantiation re-checking) is unmeasured; the check-loop latency budget is a registered blocking obligation. It could force a caching/incrementality strategy, but not a semantics change — the rule's derivation stands. |
| DIAG-1 | Rejections cite one rule ID, node path, mechanical fix | ✅ derived | R4's own text names this rung: 'check-time rejection with rule-citing diagnostics'; W1 repair loop grounding via round-4 decided law (spec-primary pedagogy with repair loop as reinforcement); determinism/byte-stability from W3 reproducibility (nowhere to hide, stable feedback for the writer). Feasibility exercised by the D1a checker-core prototype gate (PASSED 2026-07-02; negatives asserting exact rule IDs per DIAG-1). v0.18 inserts one declaration-inventory rank at position 5 (a TYPE-6 collision with an admitted system declaration) and adds one origin kind `(System, system_declaration_ordinal)`; v0.17 ranks 5 and 6 become ranks 6 and 7 with unchanged premises and ranks 1-4 are unchanged. Both are forced by SYS-1 rather than chosen: a user/system collision is neither a PRE-1 collision nor representable in the previous two-member origin sum, and this rule's determinism requires exactly one rank and one totally ordered origin for every possible collision. Target-qualification failure joins the non-source-rejection list beside target-layout failure on STOR-6's existing ground — it cites no language rule and carries no expected-terminal set. SYS-3's admission predicate is decided at the stage this rule fixes and creates no candidate, event, or rejection of its own. | The 'mechanical fix or restructuring' clause's efficacy for W1 repair-loop convergence is unmeasured — round-4 compiler-as-teacher is still research_needed and the AI-codegen validation harness (first-parse success, repair iterations) is the completing instrument. Core form is nonetheless directly constitutional. v0.20: the same-node case of simultaneously established post-resolution rejections now cites the first-defined established rule, reusing this rule's existing first-appearance rank (W3 determinism: the writer's repair target no longer varies by implementation at one offending use); cross-node ordering keeps the single-executable determinism law unchanged. |
| DIAG-2 | Private checked-program lowering authority with explicit operations and checked facts | ✅ derived | W3 requires semantic authority to be unambiguous, while T1 requires cleanup, ownership, effects, and every proof-required operation to remain explicit until the ordinary semantic checker has discharged its exact goal. One private in-memory checked program is the sole lowering input; it records operations, target obligations, finite origins, contracts, source proof facts, and their deterministic diagnostic derivations. A later stage consumes a fact only through a specification-fixed applicability rule. | The checked program is ordinary data in the same whole-unit compilation, not a runtime tag, cache artifact, independent certificate, serialized receipt, or second acceptance authority. Internal inconsistency is a compiler bug rather than an occasion to add another verifier. |
| DIAG-3 | One exact runtime trap record | ✅ derived | W3 requires failures to report rather than disappear; R4 requires a rule-citing runtime trap; SCOPE-4 fixes abort without unwinding. The exact v0.17 record contains the rule ID, fixed message, function name, and source node path in a fixed JSON field order, the minimum deterministic information needed to locate and classify a failing checked site. | v0.17 retains the reduced trap record selected in v0.11 unchanged. Returned-slice provenance adds no runtime report, trap, or required check. Target-domain TCB/resource failure remains outside this record. |
| CAP-1 | No capability category; ownership and effects are the complete concurrency vocabulary | 🟡 existence-only | T1 and C004 require data-race freedom; OWN-2/OWN-5 already provide the shared-versus-exclusive law that delivers it. Introducing a second system-resource permission lattice would duplicate that proof and create disagreement states. v0.37 therefore uses ordinary `own`, `&`, `&uniq`, place overlap, and effects for both memory and I/O. | This supersedes the unratified `Shareable`/`Sendable` stub and the later family-fragment candidate. A future thread construct may derive mobility from ownership and representation, but no such construct exists in this version. General race conditions remain outside C004's scope. |
| PAR-1 | Permitted execution overlap of two sibling calls; overlap is unobservable | 🟡 existence-only | T1 obtains safety from OWN-5 loans and OWN-7 place overlap; R1/P0 reuse EFF-2 actual-path substitution, FN-1 control flow, ordinary data dependencies, and already-proved operation domains. A suspending target summary changes lowering only and retains each call loan until its `loan-released(path)` fact; it neither grants nor denies overlap. Distinct Output or file values may overlap, while two calls borrowing one state resource `&uniq` cannot. | No grammar or source marker is added. All admitted calls already passed the same static proof checker; there is no runtime-assertion eligibility gate or latch. Outside PAR-2's fixed single-binder affine refinement, general dynamic-range and element-disjointness proofs remain deferred. |
| PAR-2 | Permitted overlap of counted-loop iterations, including a single-binder affine own-root write-only map, with accumulator recombination over an enumerated exactly-associative operation set | 🟡 existence-only | The enumerated associative operations remain the basis for recombination. The affine refinement consumes the proved OP-4 outcome and exact value image retained by the same source semantic check. FN-1 gives distinct binder values, while the nonzero coefficient, direct `set`, direct own root, one identical map per root, and mapped-root read/loan exclusions refine each iteration's footprint to the disjoint range `[a*i+b, a*i+b+1)`. Ownership, effects, and selected-target layout/address proof remain mandatory. PAR-2 neither repeats the bounds proof, recognizes parser shape, nor searches for a range, coefficient, injectivity argument, or alternate premise. | Adds no grammar or declaration. Multiple maps of one root, multi-term or dynamic ranges, borrowed-root maps, read/write transforms, and multiple accumulators remain deferred. Bounded queue/completion proof is a separate mandatory runtime protocol obligation, not a source availability assumption. |
| PAR-3 | Permitted staged overlap of a loop body cut at its first `may-suspend` submission, with per-iteration replication under a byte-coverage condition | ✅ derived | The facts-off law requires overlap to preserve meaning, and this rule derives it from machinery that already exists rather than from a new authority: every footprint and every loan is formed exactly as PAR-1 forms one, and the four dispositions are read off OWN-5 exclusivity and OWN-7 place overlap. Its unit is the iteration the FN-1 statement graph already gives, not an index subrange, which is why it admits `loop_stmt` without recovering a trip count or an induction variable and why P0's demand for I/O depth is met without a writer-visible depth, window, or batch surface. Two schedule restrictions carry the rest: prologues run in index order and never overlap, which is what admits SYS-10's short unique factory loan with no replication, and the remainder's writes to storage rooted outside the body, together with its reads of such storage the body writes, occur in iteration order, which is what admits an ordinary source-order accumulator write with no associativity, identity, or combination tree and what makes a cursor the remainder reads and then advances safe to grant. SYS-10's own sentence that a permit reserves no host resource forces the resource clause: descriptors the program's own opens consume are not the execution resources SCOPE-3 excuses, so an overlap holding more of them at once retires and retries at the source-order footprint rather than turning an Ok into an Err. | Adds no grammar, keyword, operation, type, outcome, or writer-visible scheduling marker. Replication is admissible only for a copy element type under a byte-coverage condition; the coverage analysis that discharges it for a hoisted place is deferred, and until it exists only storage the body itself constructs is replicated. A body whose first I/O error returns gets no staged permission at all, because the exit condition refuses it; closing that needs a closed-state proof extended to a call whose result is a fresh resource and is recorded, not derived. Composition with PAR-2's index split is unsound and excluded: two lanes running one body would hold two simultaneously live exclusive loans on one enclosing factory. |
| GATE-1 | Contract/signature/law/storage edits are one gated, audited operation | ✅ derived | W3 literal text ('contracts cannot be weakened to make a failing body pass') made unrepresentable rather than detected, per R4 -> round-3 decided law (no free-form writer-emittable unsafe anywhere; exactly one gated construct family; foreign-shared pre-declared at allocation site grounds the storage-contract clause) -> D0a owner ruling (gated content AI-authored, human-approved) with R5 auditability of the trusted base as the recorded deliberate exception. | D0a's own revisit condition (gate-efficacy experiments) has no owning gate entry yet — a registered audit obligation. The single-operation/single-audit-trail packaging follows round-3's 'exactly one' discipline rather than a tested alternative, but W3 fully determines the rule's substance, so this stays derived. |
| LEDGER-1 | One boundary-construct family with per-fact obligation ledger | ✅ derived | W3 (no writer-emittable unsafe or trust) + R4 (manifest-free members unrepresentable, not audited after the fact) -> round-3 synthesis decided law verbatim (exactly one gated fact-boundary construct family sharing a per-fact soundness-obligation ledger; formal spine amendment: toolchain-gated ledger entries are the sole trusted-assertion class) + D0a (AI-authored, human-approved); T2's conditionality on a declared TCB (SCOPE-3) requires exactly this explicit boundary. | The family/ledger form is the decided part. Member-level detail is still open: round-3 ffi-attenuation remains research_needed, and D4 (2026-07-06, rewrite-first FFI-narrow) rescoped FFI members to C-ABI out-calls, foreign_shared buffers, unwind-abort, single-threaded-entry. Per D4's gate entry the wall composite is unaffected, so the rule's derivation stands. |
| PRE-1 | Prelude fixed: Bool, Option, Result, three error enums | 🟡 existence-only | Result/Option half derived: round-2 error-handling verdict replace_with_alternative -> decided law 'no exceptions/unwinding (Result + trap=abort)' under R4; error enums (Overflow, DivideByZero, NarrowError) follow the OP-1 table whose grounding cards are N001 (verified) and N002 (partially verified). Bool-as-prelude-enum exists only to serve the no-if/match-only conditional, which the audit found has no debate, card, or gate entry behind it — R3-provisional register (GRAM-6/PRE-1). Normative-counted status from the v0.1 blocking fix + round-4 no-shadow-spec rule. | The conditional-form A/B (match-on-Bool True()/False() arms vs a dedicated two-arm branch) is a registered blocking experiment; if it flips, Bool's prelude role and FORM-5's no-boolean-literals rule change together — breaking under FORM-1, hence audit-marked time-urgent. The counted-prelude framing was W2-motivated; post-D2a its retained ground is W3/one-spelling, which holds. |
| EX-1 | Byte-exact normative worked example program | 🟡 existence-only | Existence derived: W1 via round-4 decided law (spec-primary pedagogy; teaching pack is a generated, tested build artifact) + W3 canonical bytes (FORM-1/2 need a normative byte-exact exemplar to pin the single legal formatting). Form not evidence-selected: the example normatively bakes in the GRAM-7 helper-function conditional-initialization idiom, which the spec-critique record picked as cheapest-to-specify — the exact ground R3 disallows; R3-provisional register (statement-only match), flagged by all three audits. | Audit marks this time-urgent: FORM-1 makes any later evidence-driven switch (e.g. GRAM-7 to expression-match with value arms) a breaking canonical-form change, and EX-1 hard-codes the current idiom into normative bytes. Completing experiments: the registered conditional-initialization idiom A/B (helper-fn vs value-arm) across W1 model tiers; EX-1 must be re-cut if GRAM-7 flips. |
| META-1 | One spelling per construct; productions map 1:1 to nodes | ✅ derived | R3 near-verbatim ('one way to say anything') + round-2 syntax verdict: every desugaring/elision rule is unverified frontend transformation surface at the F003 soundness chokepoint, N006's Alive2 discipline prices each rewrite at proof grade, so 1:1 production-to-core-node with no desugaring; W3 canonical bytes require the unique spelling. Round-4 regularity invariant, explicitly retained under W1 grounding per D2a (constitutional text). | META-1 demands that ONE survivor spelling exists; WHICH spelling survives is separately provisional for the 13 register items — the audit's price-vs-select rule (spec-delta prices, evidence selects) governs those. Pre-D2a W2 spec-budget motivation is now secondary; the W1/W3 grounding is the recorded one. |
| META-2 | No context-dependent rules; defaulting rules do not exist | ✅ derived | Round-4 decided regularity law ('zero context-dependent rules') retained under W1 grounding per D2a; no-defaulting half independently re-derived from R4 — constitution-audit adopted item 5 records that any defaulting rule (e.g. instantiating round-3 'wrap fallback') would violate R4 + META-2 simultaneously — and from the D2-corrected owner ruling foreclosing elision/inference (uniform verbose-everywhere annotation stands). | No experiment isolates context-dependence harm to weak writers; the chain terminates in constitutional text (D2a retention) plus the recorded R4 analytics, so none is needed — but a W1 measurement would harden the asserted grounding. |
| META-3 | No exception clauses; total rules or table data | ✅ derived | Round-4 decided regularity law ('empty exception lists') retained under W1 grounding per D2a; enacted in the v0.1 revision (exception clauses reduced to 0: STOR-2 default clause deleted, OP carve-outs became table data — META-5 declaration 2026-07-02); 'exceptions ±' is a priced META-5 delta dimension, making the invariant CI-enforceable. | Same asserted-under-W1 caveat as META-2: the weak-writer case for total-rules-over-exceptions is constitutional assertion plus round-4 law, not measurement. W2 spec-budget origin is weak post-D2a; the rule survives on the D2a retention clause. |
| META-4 | Each normative fact stated once; cross-references elsewhere | ✅ derived | Round-4 decided regularity law ('each fact stated once') retained under W1 grounding per D2a (constitutional text); analytically required by the round-4 single machine-checked CI artifact and by META-5 delta accounting — unique definition sites make rule deltas countable and internal contradictions structurally impossible (a duplicated fact that drifts would create context-dependent meaning, violating META-2). | Thinnest of the four regularity invariants: its recorded selection ground is D2 spec-compactness, i.e. W2 — weak post-D2a; the D2a retention and CI-machine-checkability arguments carry it. A weak-model spec-comprehension experiment (stated-once vs checked-redundant spec) would put evidence under the asserted W1 grounding. CI observation: FORM-1 and META-1 both state the one-spelling fact with no cross-reference — an apparent META-4 self-violation worth fixing. Re-grounded post-D2a under W1/W3: duplicate normative statements create reconciliation burden for weak writers and a spec-drift channel (two statements can diverge silently); CI-machine-checkable. No longer W2-only. |
| META-5 | Every spec change declares its delta and selection ground | ✅ derived | Round-4 decided law verbatim ('every design proposal must declare its spec delta') + constitution-audit adopted item 1: every delta also declares its selection ground (evidence-selected vs minimality-selected; spec-delta prices, evidence selects; minimality-selected forms enter the provisional register). This makes META-5 the enforcement mechanism for R3 + D2a reframing. | Work-branch candidates put both declarations in the status header without approval. If exact specification content merges, rule 4 of `WORKFLOW.md` records that content in `governance/APPROVALS.md`. |
| FORM-7 | numeric-literal range/leading-zero/non-finite reject; inline sign | ✅ derived | T2 (no undefined value) + R4 (check-reject over silent corruption) + W3 (an out-of-range literal is the silent-corruption channel a cheat-proof language forbids) + FORM-1 (integer leading-zero reject). Inline sign discharges iK::MIN (op-table reconciliation). | Range/non-finite reject R4-forced (evidence-selected); float canonical decimal form DEFERRED to the FORM-1 reject-vs-canonicalize gate. |
| TYPE-7 | deref typing; no implicit read-through-borrow | ✅ derived | TYPE-4 (no implicit conversions) + META-2 (no context-dependent meaning) + OWN-1 (copy-on-use for primitives) + OWN-5 (reads through shared borrow permitted). Fills a real typing gap: deref had no typing rule; the auto read-through-borrow alternative violates TYPE-4 + META-2. | evidence-selected. |
| SET-1 | Target-first assignment to a writable copy-typed place, with post-RHS revalidation and one store | ✅ derived | Real loops, buffers and stateful programs require mutation (project P0 experiment path), while T1/T2 forbid an uninitialized ownership hole. OWN-1 + STOR-1/3 make copy-only replacement the complete safe first form: the old copy needs no disposition and the owner remains initialized. R4 + OP-4 require target checks to run before unrelated RHS effects, and OWN-5 requires the commit to be judged against the post-RHS loan state. DIAG-2 carries the checked target and single store so lowering cannot re-evaluate or weaken them. | Affine take/replace is SET-2 (v0.31): failure is EFF-4 no-commit abort, overlap is OWN-5's unchanged loan judgment, and old-value disposition is the mandatory fresh binding. Target-first rather than RHS-first is correctness-selected because an invalid target must not execute unrelated effects. |
| SET-2 | Atomic affine-place replacement binding the previous owner under the new `let` | ✅ derived | Growable collections require an affine backing to leave and enter a struct field (STOR-1's recorded collection blocker; the batch-0070 growable-vector, byte-string, and affine-element consumers), while T1/T2 forbid a writer-observable uninitialized hole and STOR-3 forbids implicit destruction. Requiring the replacement in the same operation discharges the hole by construction, and the mandatory old-value binder is forced by the no-implicit-destruction constraint: the previous owner must leave through an ordinary binding whose scope-exit release STOR-3 already derives. SET-1 supplies the complete target, order, writability, and revalidation judgments; OWN-5 exclusivity makes the through-`&uniq` exchange unobservable; ENT-5's kill (a) removes every fact the exchanged value carried, length facts included, so obligation discharge never trusts superseded storage. Typed holes and closed-scope holes were rejected as per-place flow state (owner ruling D1a's checker-scale levers), swap-only as binding revival against OWN-1; see `research/investigations/take-replace/DESIGN.md`. | evidence-selected (candidate v0.31). The `len(b)`->`len(p)` transport is deliberately absent: a monotone ENT-1 successor when a corpus program needs post-install subscripts through the container. |
| OP-6 | cvt exact-or-Result partition (29 value-preserving total pairs; no rounding) | ✅ derived | TYPE-4 (no implicit value-changing conversion) + W3 (no silent value change, the named R0 delta over Rust as) + R4 (non-exact to checked Result, never silent) + T2 (float-to-int via fptosi.sat + guards, never raw fptosi UB). 29-pair total set arithmetic-verified. | evidence-selected. |
| OP-7 | op-name convention: `i`/`f`/`b`/`e` domain prefix, mode-suffix axis, nominal/signedness-checked lowering | ✅ derived | W1 (op names predictable from spec alone) + R3/FORM-1 (one spelling per op) + META-2 (context-free op-vs-fn resolution) + P0 (per-node modes, the named R0 delta). Formalizes the existing ilt=slt/ult discipline. v0.8 applies the same truthful-domain rule to enum comparison: `e` names tag-only enum equality, while `i` remains integer-only. `Bool` uses `b` for logic and `e` for equality, selected by the explicit operation name rather than contextual overload resolution. | evidence-adjacent. The new `e` prefix is regularity-selected from the derived naming rule and backed by the enum-equality source/code-shape census, but has not received a separate weak-writer naming trial. |
| OP-8 | edge/table-data semantics + confirmed LLVM lowerings for the added ops; every totality edge closed | ✅ derived | T2 (no added row is poison — each edge total-or-trapped) + W3 (no silent corruption; deterministic fmin=llvm.minimum) + R4 (trap rungs for shift/abs overflow) + P0 (each op is a hardware/IEEE instruction; the R0 parity ground). Lowerings verified on clang 21; imul.sat widen-clamp avoids LLVM #51019; minnum rejected for signed-zero nondeterminism. v0.8 adds a closed totality argument for `eeq`/`ene`: valid same-nominal-type tag discriminants compare with one equality/inequality operation at the selected width; there is no poison edge, payload access, signedness choice, conversion, trap, or invalidator. | evidence-selected (hardware/IEEE-forced). The focused three-variant democ probe observed one raw-IR `icmp eq i32`; clang `-O2` recovered one caller comparison from the mapper fallback but retained its branching helper definition. This is code-shape evidence only, with no timing or byte-count claim. Production lowering still requires the second edge-case review in the packet. |
| CONST-2 | const items: immutable program-lifetime rodata; primitive or array<T,N> of const-eligible T | ✅ derived | serves constant tables (masks/trig/CRC) + T1 (cvalue totally defines the value) + reconciled with FN-7 (immutable rodata triggers none of FN-7's three mutable-state hazards) + P0 (frozen rodata is a whole-program read-only fact base; no static mut). Borrow-any-region via the OWN-10 const clause avoids re-admitting a 'static region. | evidence-selected; struct/enum-typed consts deferred. |
| OP-9 | buffer allocation size is statically qualified by `buffer_fits<T>(n)` | ✅ derived | T2 forbids silent under-allocation from `n * sizeof(T)` overflow. R4 shift-left and W3's single explicit trust boundary select a proof-required total Boolean domain query over a hidden runtime trap; exact target layout remains checked during qualification. The hazard is specific to count-scaled allocation and absent from box/arena construction. | v0.33 supersedes the former runtime-size trap with `AllocationFit`; a retained claim over the same canonical goal is the only runtime backstop. |
| GIVE-1 | explicit value terminator for let-init match; give-completeness | 🟡 existence-only | R4 (missing delivery is a loud reject, not silent unit) + W3 (delivery cannot be silenced; no tail-expr hiding place) + W1 (one more terminator of the return/break class; no new arm shape) + META-2 (context-dependent legality not meaning, via the break precedent) + D1a (structural last-statement recursion, below the ownership checker). | minimality/regularity-selected -> R3-PROVISIONAL, needs_experiment. |
| GRAM-8 | named-in-declared-order construction; positional removed | 🟡 existence-only | R4 (positional same-typed-field transposition is silent corruption, unrepresentable-as-error; named lifts it to check-reject) + FORM-1/META-1 (one spelling; declared order is one byte sequence) + META-2 (name-iff-two-same-typed rejected) + TYPE-5 (field name a redundant checked fact). D3 census: Swift memberwise, C++20 designated-init, Zig no-positional all PASS. | direction R4-forced; W1 transposition-rate magnitude experiment-gated; needs_experiment. |
| GRAM-9 | flat three-address / A-normal form; call/construct only at expr position | 🟡 existence-only | FORM-1/META-1 (nesting-vs-let-split were two spellings; atoms-in-argument-position makes non-nesting a grammar guarantee) + W1 (collapses the nested-paren/argument-boundary surface; per-line node-path diagnostics) + TYPE-5 (every intermediate carries explicit mode+type). P0 NEUTRAL (mem2reg/SROA make a named local and the subexpression the identical SSA value; not a runtime delta). Precedent: LLVM IR / Rust MIR / ANF. | FORM-1/regularity force it; weak-writer net-sign vs nesting experiment-gated; needs_experiment. |
| GRAM-10 | named match binders (field: freshBinder) in declared order | 🟡 existence-only | R4 (read-side symmetry: positional binders of a two-field variant silently transpose; order+name checking lifts it to reject) + regularity (write-named/read-positional asymmetry is the irregularity the enemy names) + TYPE-6 dodge (fresh binder distinct from the field name, so two arms binding same-named fields never collide). OWN-13 unchanged. | R4+regularity force it; magnitude gated with GRAM-8's experiment; needs_experiment. |
| GRAM-11 | named-in-declared-order arguments for user-fn calls; positional operands for table ops | 🟡 existence-only | R4 (a fn call with two same-typed params has the same silent-transposition hazard GRAM-8 closes for construction; named-in-declared-order lifts it to check-reject) + FORM-1/META-1 (declared order is one byte sequence; names are checked-redundant facts, never a reordering license) + FN-1 (param names from the signature). Table-op calls stay positional (operands order-intrinsic); op-vs-fn is the existing name-lookup partition. v0.18 extends the named-argument discipline to a callee resolving to an admitted system operation, with parameter names fixed by SYS-2 instead of FN-1. The extension is downstream of this rule rather than a concession to it: the existing partition — named arguments for functions, positional order-intrinsic operands for table operations — excluded the operation-table declaration home by form before preference entered the comparison. | direction R4-forced; W1 magnitude experiment-gated with GRAM-8; needs_experiment. The new v0.18 callee class inherits the same unrun GRAM-8 transposition-rate experiment; nothing about system operations makes that magnitude easier or harder to measure. |
| PROG-3 | Program start: declared standard inputs supplied once, start failure before entry, exact `ExitStatus` mapping | ✅ derived | FN-7's already-recorded no-ambient-authority argument (W3: hidden inter-function channels invisible in signatures; FN-1 signatures-as-trust-unit) -> a program's complete standard-input access is exactly what its entry declares, so start supplies precisely those and nothing else. R4 places an unsuppliable input at the earliest rung available: start fails before any source statement executes, before any owner exists, so there is nothing to trap over and no recoverable outcome to invent. ERR-4 classification keeps that target/environment failure out of the source vocabulary; SCOPE-4/EFF-4 keep a trap out of the status channel (no return edge, no release, no `ExitStatus`). The no-observable-start-aggregate clause is the recorded rejection of the affine `Process` object (dossier §5; mcts move 2026-08-05: one retained affine holder falsely serializes files, output, networking, clocks, and workers, and sharing it would need a central lock or hidden aliasing). Cleanup-then-status ordering is review decision 20, which kept the true claim (release runs on the return edge, then the target maps the value) and dropped the false implication that cleanup precedes the program's own choice of status. | Three termination channels are now disjoint by construction: start failure (pre-entry, no status), normal return (exactly one `own ExitStatus`), and trap (no status, TRAP-1). The `own unit` entry form is left with no fixed process status, recorded as such rather than given a silent default. |
| EFF-5 | System interaction is ordinary owned state with no ambient or interior mutation | ✅ derived | T1 requires every mutation to have one exclusive or owned access path. W3 requires an outside observation or publication to be visible in the signature, and P0 requires independent state values to remain parallel. v0.37 therefore makes system objects ordinary state: an advancing or mutating operation takes `&uniq` or `own` and declares `writes(parameter)`, while a stable selector may use `&` plus `reads(parameter)`. File open makes the observation occurrence explicit by consuming and writing a one-shot permit; `DirectoryRead` and path bytes remain shared selector inputs. Resource creation changes an explicit factory, allocator, or permit parameter, and the target retains its ordinary loans until the milestone. | This supersedes both the blanket sequential-external rule and the capability root/family-fragment candidate. Fresh results carry ordinary binding identity and no parent ancestry. Host hard links, descriptor duplication, redirection, and other-process aliases do not alter Whitefoot place identity. Target-private rings and queues remain TCB protocol state rather than shared Whitefoot storage. |
| SYS-1 | One globally admitted compiler-owned system declaration domain | 🟡 existence-only | SYS names need a compiler-owned declaration home because source, prelude, and the operation table cannot supply them. TYPE-6, DIAG-1, and PROG-1 force whole-unit collision and deterministic ordinal identity. v0.33 superseded conditional admission: the complete system domain is present in every compilation unit, removing a visibility fork that depended on entry form. | The declaration-home surface remains an owner-selected form with future spelling-reservation cost unmeasured. Global admission does not make any value ambient; FN-7 and EFF-5 still require system state as ordinary parameters. |
| SYS-2 | Closed inventory: 18 nominal types, 40 constructors, 16 operations | 🟡 existence-only | PROG-1 and SYS-1 require finite compiler-owned data. Opacity, region freedom, affinity, nongeneric calls, and no source lookalike follow existing type and resolver rules. v0.37 gives every operation one formal-path state row: stable immutable inputs use `reads(parameter)`, changed buffers and state resources use `writes(parameter)` through `&uniq` or `own`, and lifetimes remain solely in modes and types. `FileFactory`, `FilePermit`, and total `reserve_file` expose file-open occurrence state through the same rules. The target contract contains suspension and milestones only. | Inventory membership remains witness-driven rather than comparatively selected. Counts are 18 nominal types, 40 constructors, 63 fields, 16 operations, 22 region parameters, 44 value parameters, and 203 declaration records. The 28-class `IoError` set remains uncarded. |
| SYS-3 | Every compilation unit admits the complete system inventory | ✅ derived | v0.33 removed conditional visibility. One global inventory avoids a circular or entry-kind-dependent resolver branch, while SYS-1 collisions and DIAG-1 ordering remain deterministic. | Admission changes name visibility only. It does not supply a system value, ambient mutable state, effect, call, or entry parameter. |
| SYS-4 | No system-only state-kind or capability classification | ✅ derived | OWN-2 and EFF-1 already state every operation's complete permission and behavior. Repeating the decision as a nominal `immutable`/`resource` kind would add a second classifier, create exceptions such as positioned `ReadFile`, and carry no independent consumer. v0.37 therefore lets each system signature state `&` versus `&uniq`/`own` and exact `reads`/`writes` directly. | Opaque types still have compiler-owned construction, representation, target, and release contracts under SYS-2/SYS-5. Those contracts do not grant source access or form a second type hierarchy. A later split must return ordinary independent owners and justify that operation directly. |
| SYS-5 | One completion policy per system resource type; release-complete; exact release rows | ✅ derived | STOR-3 and T1 require one release action per type. v0.37 expresses a closing release as `writes(owner)` in the same state-effect system and uses only `never-suspends` or `may-suspend` plus per-path loan-release and terminal target milestones. Moving an incoming owner preserves its formal identity; releasing a fresh local owner frames out. Never retrying ambiguous close remains a correctness theorem, and release remains distinct from flush, sync, atomic commit, or durability. | Close and writeback diagnostics may still be lost exactly where SYS-12 states. `Output` detach and immutable-value consumes have empty rows. Whole-process abort performs no release. |
| SYS-6 | Per-operation outcome types; PRE-1 `Result` for two outcomes; one prefixed enum beyond that | ✅ derived | ERR-2 refutes one shared outcome union because exhaustive callers would write dead arms and every added variant would edit the corpus. `Result<T,E>` covers two outcomes; operation-prefixed enums cover larger sets under TYPE-6's flat constructor domain. TYPE-4 keeps error types distinct. | In v0.37 `propagate` may chain `open_read`, `write_once`, `open_directory`, `open_directory_source`, and `open_file` through `IoError`. `PathError.PathInvalid` and `IoError.InvalidPath` remain distinct. The command entry returns `ExitStatus`, so it matches every failure explicitly. |
| SYS-7 | `IoError` closed portable class set with fixed-size inline target detail | 🟡 existence-only | ERR-4 fixes classification by table, ERR-2 requires a closed portable discriminator, and R6 forbids exposing native errno as the portable control domain. `Other` closes unmapped failures and `Unsupported` prevents silent weakening. Fixed-size `code: u32` and `origin: u8` allocate nothing and have no release action. | The exact 28-class membership and field widths remain unmeasured and owner-accepted rather than evidence-selected. A heap-backed detail would require an allocation effect and its own release contract. |
| SYS-8 | One-attempt transfers over a caller-owned buffer; range validation before every other action | ✅ derived | R4 requires both half-open range obligations before target transfer, leaving resource and buffers unchanged on proof failure. W3 forbids a hidden retry after progress. A no-progress host interruption or readiness refusal remains adapter progress and publishes no writer outcome; `Interrupted` and `WouldBlock` are absent. Caller-owned initialized buffers avoid an input copy, and exact per-outcome disposition keeps target counts inside the TCB contract. | Empty-range and zero-progress conventions remain stated rather than measured. `read_at` is positioned, has no byte cursor, and observes its `ReadFile` through a shared loan; destination mutation remains exclusive. Repetition and retry policy remain ordinary source loops. |
| SYS-9 | `Args` and `HostString`: immutable entry value, zero-copy leases, two content routes | ✅ derived | Mostly instantiation of rules that carry their own rows: HOST-2 fixes the two routes, HOST-3 fixes the one-type lease rule, SYS-8 fixes the transfer semantics. The independent contribution is P0: `arg_get` returns an inline lease with no allocation and no byte copy because the performance review required exactly that change (a per-argument allocation was the reviewed defect), and several leases may refer to the same immutable bytes without any aliasing consequence because the backing is immutable. `args_count` and `host_bytes_len` are total; the text route is fallible because conversion to text is fallible, which is TYPE-4's no-implicit-value-change rule at the string boundary. | The specification itself fixes the lossless route only for one-byte native code units and delegates a wider family to target qualification. The exact `x86_64-pc-windows-msvc` row now fixes that meaning without narrowing or transcoding: each UTF-16 code unit contributes its two in-memory little-endian bytes, `host_bytes_len` returns twice the code-unit count, and `host_copy_bytes` transfers those exact bytes. Native run [33305475906](https://github.com/mbbill/Whitefoot/actions/runs/33305475906) passes U+4241 through `host_string_bytes.wf` and requires the raw `41 42` bytes. Other wider-code-unit families remain unqualified. |
| SYS-10 | Explicit file-open factory and one-shot permit; shared directory selector; fresh returned owners | ✅ derived | A lookup occurrence can vary, so it needs explicit advancing state, but the directory handle itself is only a stable selector and should not serialize independent opens. EFF-5/OWN-5 therefore select total `reserve_file(&uniq FileFactory) -> own FilePermit`; each open consumes and writes one permit while reading `&DirectoryRead` and its path/name. The permit burns on success and failure, and every successful resource result is a fresh ordinary owner with no ancestry. Because reservation is proof-only and consults no host, the closed PRV table marks both its fresh permit result and its factory write internal. | Reservation promises no host quota, so `ResourceExhausted` remains an open result. Two short factory loans may mint two permits; their opens may overlap through one shared directory. Permit erasure adds no native open ABI argument. Directory release remains release-complete under the existing close-diagnostic ground. |
| SYS-11 | `ReadFile` random-access state; shared positioned observations; release-complete | ✅ derived | An explicit offset removes the byte cursor and the operation changes no caller-visible `ReadFile` state. `read_at` therefore takes `&ReadFile`, reads the file and destination capacity, writes only the destination, and retains both loans to completion. Separate opens return distinct ordinary owners; host aliases do not alter place identity. Release-complete keeps the existing close-diagnostic ground. | Reads on one value may overlap when their destination loans and other dependencies permit. A later consuming close may expose its diagnostic but must consume on every outcome. |
| SYS-12 | `Output` state resource with unique writes and stated release limitation | ✅ derived | T1/OWN-5 require `write_once` to take `&uniq Output`; `reads(output, source), writes(output)` records its observation and transition, and the loan itself preserves per-value source order without shared ordered mutation. Standard output and error are distinct ordinary owners, so calls may overlap; host redirection does not merge their places. Release only detaches and reports nothing. | Close-only or writeback failure can still be lost, so a redirected command may return success after late failure. A stronger durable output needs its own finish operation and type contract. |
| SYS-13 | `ExitStatus` opaque immutable value with one total constructor | ✅ derived | TYPE-4 and TYPE-5 require an explicit constructor for the otherwise unwritable opaque result. Every `u8` is valid, so `exit_status` is total, pure, allocation-free, and performs no host call or state effect. | The target maps the returned code exactly onto process status. Startup failure and trap remain outside that mapping; release is a logical consume. |
| SYS-14 | `DirectorySource` enumeration state: permit-backed creation, unique cursor advance, portable records, no path value formed | 🟡 existence-only | Batch-0071 proved traversal inexpressible without enumeration. `open_directory_source` consumes and writes one `FilePermit`, reads a shared `DirectoryRead`, and returns a fresh Source; `directory_next` uniquely writes the Source and destination. OWN-5 therefore supplies single-cursor order without shared ordered reservations. HOST-3/PATH-1 still force enumerated names to remain caller-buffer components rather than `HostString` or `RelativePath` values. | Record layout, three-operation split, and in-place rewrite remain minimality-selected. Distinct Sources may overlap; one Source cannot. Linux mapping remains fail-closed when absent. |
| HOST-1 | Host strings are lossless target-indexed code-unit sequences, not text; exactly two families | ✅ derived | W3 directly: a normalizing, case-folding, substituting, or truncating string boundary is precisely the silent value change a cheat-proof language forbids, and TYPE-4's zero-implicit-value-changing-conversion floor applies at the host boundary exactly as it applies inside the language. R6 forbids marrying the source semantics to one target's string domain, which is why code-unit width and family are properties of the selected target (STOR-6) that no source construct observes. The closure is the load-bearing half and it is enforced upward, not downward: a target outside both families qualifies only under a specification amendment giving it its own lossless family, and without that amendment it fails qualification for exactly those semantic IDs — no implementation narrows the semantics to what its own string domain can carry. Review decision 13 applied that rule to the obvious case, scoping the WASI claim: WASI does not currently qualify for the lossless host-string and path operations, which is a qualification failure for those operations rather than a licence to narrow them. | The WASI evidence shaped the selection without becoming the contract: Unicode-only paths are a recorded anti-lesson (mcts, 2026-08-05), alongside pollable composition failure and the missing caller-buffer route. The two-family enumeration is the closure of the actual target landscape rather than an arbitrary cut; a third family costs one META-5 amendment, not a semantic renegotiation. |
| HOST-2 | Host-string/UTF-8 conversion is explicit and fallible; exactly two routes | ✅ derived | TYPE-4 verbatim — no rule admits a host string where text is required or text where a host string is required, and a host string reaches source content only through an operation naming the route it takes. W3 supplies the anti-cheat clauses that make the text route honest: it never emits a replacement code point, drops a code unit, produces a truncated encoding, or copies part of an encoding, because each of those is a silent value change dressed as success. R4 puts the failure on the recoverable-value rung (an explicit invalid-text outcome) rather than a trap or a lossy success. The lossless route's existence is evidence-driven rather than designed: review decision 1 restored it because a UTF-8-only operation set silently restricted process arguments to Unicode against the dossier's own completeness requirement and made the mandated non-text-argument test inexecutable. | Escaped, quoted, and lossy display are a DEFERRED presentation family with their own delta, deliberately not a third mode of either route — the boundary that keeps 'display' from becoming a silent lossy conversion. Exact names, signatures, preconditions, and outcome types are SYS-2 inventory data with transfer semantics in SYS-8. |
| HOST-3 | Exactly one host-string type; inline command-lifetime lease; a different backing is a different type | ✅ derived | STOR-1 makes storage class and release action functions of type, so a producer without command-lifetime backing must introduce a distinct owned-backing string resource with its own release action and type contract. TYPE-4 forbids implicit retyping. QUAL-2 guarantees that command argument backing outlives every inline lease; the lease is neither a borrow nor region-bearing. | Producers outside the command-lifetime premise still require a later region-bearing or owned-backing design. Lease identity remains checked-program metadata only. v0.37 changes “family contract” to the same ordinary type/state contract used everywhere else. |
| PATH-1 | Relative paths admitted by construction from one host string; no source text assembly | ✅ derived | W3 and R4 make path construction explicit and fallible. HOST-1 requires exact code-unit preservation without normalization, and NUL or a target-root prefix is rejected because the target API or DirectoryRead-relative contract cannot represent it honestly. R6 keeps the prefix set as target data. HOST-3/STOR-1 let success retype the same inline lease without allocation or copy. | Component types, absolute paths, and decomposition, join, and display remain deferred. v0.37 names the supplied base as DirectoryRead state rather than a capability; path semantics do not change. |
| PATH-2 | Directory-relative resolution is process-equivalent and makes no confinement claim | ✅ derived | W3 requires the contract to admit that resolution through `.`, `..`, links, reparse points, and mount transitions may escape the directory named by a `DirectoryRead` value. A future confined directory is a distinct type with a stronger target contract. R4/QUAL-1 forbid prefix-concatenation emulation when the target lacks a directory-relative facility. | Absolute paths, cross-root operations, and target-root prefixes need distinct inputs and operations. v0.37 changes ownership and effect exposure, not resolution semantics. |
| QUAL-1 | One target-independent semantic ID per operation; static qualification table; stop, never narrow | ✅ derived | R6 directly — artifacts must not marry one backend or ISA, so the operation's identity is a target-independent semantic ID owned by this specification, and the checked program carries only that ID. The no-lookalike clause is project law made normative: an operation's identity comes from resolution in the system declaration domain, and no source function name or spelling, logical path, project, corpus, test, or signature lookalike ever selects, adds, or removes one — the dossier lists 'a source-recognized primitive lookalike' as a rejection gate. PROG-1 forbids the registry, negotiation protocol, dynamic loading, and plugin interface that a dynamic version of this rule would need, leaving compiler-internal data. STOR-6 supplies the precedent for the failure mode: a qualification failure is a target failure that stops compilation, is not a source-language rejection, and cites no language rule. META-5 governs identity change: an approved implementation may be replaced only within one semantic identity, and a change to any element the record binds is a different semantic ID under a new specification version and a compatibility review, never a target-code update. | 'Qualification never narrows a semantic ID to what a target can supply' is the load-bearing sentence; every other clause exists to make it enforceable. The table maps `(specification version, semantic ID, target, program kind)` to exactly one approved implementation version and one private ABI symbol — a fixed compiler-internal enum and table, with no WIT parser, semver registry, dynamic loader, or plugin protocol. |
| QUAL-2 | Target guarantees; the four stated ones; failure and startup refusal both before entry | ✅ derived | QUAL-1 needs an exact admission condition, and R4 selects the earliest rung: a target that cannot supply a required guarantee fails qualification and compilation stops, rather than admitting the operation under a weaker guarantee. The four guarantees are stated here for one recorded reason — each is a property of the target with nothing in a program to check, so qualification is the only sound enforcement point. Command-lifetime argument backing is what HOST-3's lease rests on, discharged either as stable native argument backing or as one complete snapshot taken before any Whitefoot code runs; a qualified target that cannot establish it for one invocation refuses startup before entry rather than entering with backing that fails the guarantee. The lossless code-unit family is HOST-1's closure applied to the host-string and path IDs. Directory-relative resolution is PATH-2's no-emulation boundary: the target supplies its own directory-relative facility or those IDs fail qualification, never a prefix-concatenation substitute. Directory enumeration is SYS-14's one-host-call facility; absence fails those IDs, while a present facility without an approved ABI mapping remains `MissingMapping`. PROG-3 places every qualification failure and startup refusal before entry, so neither is a source-returned status, a recoverable outcome, or a trap. | This is the rule that makes HOST-3, PATH-2, and SYS-14 honest: each premise no source rule can check is checked once, at build or startup, and never assumed. |
| QUAL-3 | Static selection; required emitted shape; bootstrap-owned one-time normalization | ✅ derived | P0/R0 require no per-call dispatch table, operation-ID switch, target tag, handle table, heap allocation, transferred-data copy, global system lock, or per-call signal operation on the native hot path. An `inline-terminal` outcome performs the required checks, at most one direct host attempt, one outcome check, and a cold failure mapper; it remains part of the one completion contract rather than a blocking source mode. The bootstrap installs broken-pipe normalization once before entry so the failure reaches `write_once` as a recoverable result. | Evidence remains emitted-code and symbol inspection rather than a language judgment. Target-specific measurements are still required, and the cost magnitude remains unpriced. |
| TRAP-1 | Trap under held system resources: whole-process abort, unchanged | ✅ derived | Preservation rather than extension, which is why it derives cleanly: SCOPE-4 and EFF-4 are retained exactly — the runtime attempts the mandatory DIAG-3 record, then aborts the whole process without unwinding and without language cleanup, and PROG-3 produces no status. That no release, close, flush, detach, or completion action runs after a contract violation is not a new rule but EFF-2's existing statement that release actions run only on normal edges; SYS-5's table therefore contributes nothing on a trap. SCOPE-3 places operating-system process teardown of memory and descriptors inside the declared TCB rather than presenting it as a language cleanup guarantee — the distinction W3 requires between what the language promises and what the host happens to do. The not-rolled-back clause is honesty about external effects: bytes already written remain written, an object already created remains created. R5-style boundary: a host requiring an instance to fail without ending its process runs that instance in a separate process. | The payoff is structural: because a trap ends the owning process, no instance resource table, per-instance reaper, or pending-operation transfer is required, and none appears on a synchronous transfer path (QUAL-3). Host-surviving in-process trap containment is a DEFERRED amendment whose own cost is recorded in the dossier (instance resource table, pending-operation transfer to a reaper, delayed reclamation until quiescence). Provenance caveat: TRAP-1 is an addition beyond the dossier's own delta inventory, ratified at exact approval (2026-08-05) rather than reviewed as a numbered rule in the thirty-one-issue pass; its chain rests on the retained SCOPE-4/EFF-4 law, not on independent review provenance. |
| GATE-2 | The system domain is not the gated boundary family; the separation is exact in both directions | ✅ derived | LEDGER-1's literal text forces it: there is exactly one boundary-construct family (unsafe regions, FFI extern frames, trusted primitive imports) sharing one per-fact soundness-obligation ledger, so housing compiler-owned system operations there would merge them with general FFI — the ground on which the dossier rejects Route B outright. A system operation holds no ledger entry and is not writer-authored, writer-approved, or gate-edited (GATE-1), so SCOPE-1's conclusion follows mechanically: a program calling system operations contains no gated construct and remains a kernel program, and SCOPE-3's foreign-code condition is not engaged because an approved target entry is not foreign code. The converse half is equally forced by the same 'exactly one family' discipline: general FFI, arbitrary imported or exported foreign calls, raw host-ABI calls, and writer-declared external signatures remain reserved to LEDGER-1 and are unreachable through the system domain. META-5 keeps the boundary from eroding: adding a system operation is a specification amendment, never a gate approval, a ledger entry, or a target-implementation act. | Same provenance caveat as TRAP-1: an addition beyond the dossier's own delta inventory, ratified at exact approval. Its content is an analytic separation over LEDGER-1/GATE-1/SCOPE-1, so the chain does not depend on review provenance. The dossier records that the SCOPE-1 and SCOPE-3 objections often raised against Route B do not hold cleanly and were not relied on; this row follows that discipline and rests on LEDGER-1 alone. |
| PRV-1 | Finite two-class explicit-dataflow provenance over retained value, storage, result, write, and call components | ✅ derived | R4 puts an explicit domain/value path above a runtime assertion trap when correct external input may falsify an obligation, while W3 forbids the writer from asserting or annotating a provenance class to escape that policy. R1 therefore requires the smallest mechanically derived classifier that serves this distinction, and R3 selects its form from the recorded evidence: PROBE-W1's subject-position correction rejects predicate-wide taint; PROBE-TAINT and task 0046 select the two classes, internal `len`, internal program-bounded transfer counts, explicit data edges only, root-plus-explicit-offset place reads, and direct payload projections needed for canonical 3/3 without sibling-error contamination. Reusing ENT-6's already-retained positive dependency transfer avoids a second fact or goal language. Command inputs and the environment-origin [SYS-2] cells are the only unconditional external seeds; SYS-9 and the transfer contracts derive the internal success-count cells. Boolean disjunction and finite datum-set union over the closed component/call domain are monotone, so the first least fixed point exists, terminates, is unique, and satisfies ENT-1/W3 implementation agreement. | The explicit-flow boundary deliberately excludes control choice, write-address choice, path-sensitive storage, recursive payload paths, and implicit flow; those are measured scope exclusions rather than unstated exceptions or a claim of noninterference. SYS-2 remains existence-only overall because its operation inventory and `IoError` membership retain their prior form debt; the new component classifications themselves are theorem- or evidence-derived. |
| PRV-2 | Derived caller-visible provenance column, finite protected-demand composition, and deterministic call-argument rejection | ✅ derived | FN-1's derived caller-locality boundary requires a caller-visible summary rather than body inspection, and W3 requires that summary to be compiler-derived rather than writer-authored. Task 0041 showed that a parameter-position set loses the protected leaf needed for a truthful diagnostic; task 0046's independent rewalk selected the exact `(parameter datum, protected leaf)` relation and reproduced canonical 3/3 plus the 14-call/24-argument projection. v0.26's derived ENT-6 requirement occurrence, complete/U/B outcomes, and subject-only bridge supply the exact O3 closure without a recognizer, mention-all-parameters rule, whole-goal support, or second proof language. With PRV-1 pairs frozen, the remaining transfers only union finite direct-demand, bridge, target, and event sets over PROG-1/FN-2's closed finite instances, so the second least fixed point is unique, terminating, recursion-safe, and traversal-order independent. FN-8 full-state acceptance remains first. DIAG-1/DIAG-2 plus R4 require one actionable event at each existing argument atom while retaining every target; post-convergence NodePath/declaration ordering makes the witness deterministic without entering either lattice. | Multiple leaves or routes at one `(call, argument)` enlarge its retained target set rather than duplicating the rejection. Witness ordering is diagnostic closure over existing canonical orders and changes neither acceptance nor provenance. |
| PRV-3 | Local constrained-subject gate over complete, unasserted, and S4-blinded states | ✅ derived | W3's no-trust rule and R4's shift-left ladder require an externally controlled protected subject not to become legal solely because a writer-authored `check` or `claim` will trap at runtime; the honest repair is a real branch/value outcome or removal of the external value from constrained-subject position. PROBE-W1 rejected predicate-wide taint and selected the obligation's constrained subject, while PROBE-TAINT and task 0046 measured the resulting 19 external / 6 branch-discharged / 13-under-11 rejected / 14 internal split and preserved the negative controls. OP-4 complete-state discharge runs first, preserving every existing base rejection. Removing exactly S2/S3 forms the unasserted judgment; v0.26's requirement bridge and O3 counterexample force the additional S4-blinded judgment so neither a requirement nor the command wrapper launders the same leaf. The exhaustive B/external-bit/U/parameter-datum partition assigns local leaves only to PRV-3 and call arguments only to PRV-2, including entries, with no fallback check or runtime change. R4 and DIAG-1/DIAG-2 derive the exact protected-node attribution, residual, provenance chain, and two legal repairs. | A claim remains legal in its own right; this rule constrains only its downstream authorization of the offset in `i < len(P)`. Bounds, bases, target addresses, and unrelated operands remain outside the gate by the evidence-selected subject rule. |

## OWN-1 amendment: tag-only enums are copy (2026-07-10)

Derivation: a tag-only enum value (every variant nullary) is resource-free —
no heap, no drop obligation, no interior storage a borrow could alias — so
affine classification buys zero safety (T1 unaffected) while taxing every
boolean expression: an owned `Bool` cannot be loop-carried state (match
consumes it, OWN-13), cannot flow through `band`/`bnot` dataflow (operands
consumed), and forces integer-typed workarounds whose lowering degrades
vectorization (measured: the chunk-summary wc classifier vectorizes at width
2x4 as an i64 recurrence vs width 16 for the i1 form — a 1.6-1.8x kernel
gap; experiments/port-study/wc-chunk-summary). The original affinity of Bool
was minimality-selected (R3 provisional: uniform enum rule), not
evidence-selected; this amendment is the evidence-driven correction (P0 via
vectorizable i1 recurrences; W1 via removing a writer tax with no safety
payoff). Companion FORM-1 discipline: `move` on a copy value is now a hard
error (one spelling per meaning: copies are used bare).

## v0.9 amendment — canonical frontend entrance closure (2026-07-21)

Specification binding:
`spec/kernel-spec-v0.9.md`, SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.

The candidate's conditional authority sentence is part of those exact bytes:
the reviewed document becomes authoritative only when installed through the
guarded procedure. No post-approval status-header rewrite is permitted.

Earlier entries remain dated history. For v0.9, this amendment supersedes an
earlier entry only for the changed fact it states; all other status and debt
continue. The original header was a historical v0.3 accounting statement.
Immediately before this amendment the ledger contains 91 unique rule rows: 50 derived, 41
derived-existence-only, and zero underived. This amendment changes 21 existing
rules and adds PROG-2. The v0.9 total is therefore **50 derived · 42
derived-existence-only · 0 underived** across 92 rules.

`derived` means existence and form have a constitutional or evidence-selected
chain. `derived_existence_only` means at least one surviving form choice remains
minimality-selected and R3-provisional. A selected correctness repair does not
erase an independent older form debt.

### Rule amendments

The entries cover all changed rules. This row exposes new PROG-2 to META-6:

| Rule | v0.9 status |
|---|---|
| PROG-2 | derived_existence_only |

#### CONST-1 — derived

- **Delta and selection ground:** the unchanged `const := "[0-9]+" | IDENT`
  production moves from GRAM-3 into CONST-1, its unique semantic owner. This is
  a correctness-selected META-4 repair; it changes no accepted source shape.
- **Continuing chain and debt:** the existing closed-const-language chain from
  R2, R4, TYPE-2, and FN-2 continues. Const arithmetic remains a deferred
  specification addition, and the earlier weak-writer/external-generation debt
  remains open. No v0.8 discrepancy record is claimed closed by this move.

#### CONST-2 — derived

- **Delta and selection ground:** the unchanged `cvalue` production moves from
  GRAM-3 into CONST-2, its unique semantic owner. This is the same
  correctness-selected META-4 repair and changes no value semantics.
- **Continuing chain and debt:** the T1 total-value, immutable-rodata, and P0
  read-only-fact chains continue. Struct- and enum-typed consts remain deferred.
  No v0.8 discrepancy record is claimed closed by this move.

#### DIAG-1 — derived

- **Delta and selection ground:** R4 requires a rule-citing rejection, W1
  requires a stable repair location, and W3 requires deterministic bytes. The
  closed `SourceBytes` / `SourceNode` / `BundleRoot` sum, frontend stage order,
  quote-aware raw spans, source-EBNF failure machine, expected-terminal order,
  and tree-owned FORM-2 gap location are correctness-selected ways to make
  those requirements truthful before and after a tree exists.
- **Evidence and debt:** independent candidate-bound frontend models must agree
  in `grammar-verifier/evidence/frontend-boundary-evidence.json`. Ordering for
  later semantic and target failures remains unselected. This amendment
  addresses `discrepancy:v0.8/diag1-pre-tree-node-path` only after v0.9 is
  installed; the v0.8 record remains immutable history.

#### EX-1 — derived_existence_only

- **Delta and selection ground:** only the enum declaration is re-rendered into
  the executable FORM-2 block shape. Program meaning and the selected match/give
  idiom do not change. This is a formatting-only correctness repair.
- **Continuing chain and debt:** EX-1 still inherits the R3-provisional surface
  debts of the constructs it demonstrates. It is part of the FORM-2 migration,
  not a new semantic claim or an expected-verdict change.

#### FN-1 — derived

- **Delta and selection ground:** the complete-signature chain from W1, P0,
  D1a, and the explicit-fact rulings continues. D24/A-01 additionally selects
  every top-level function signature as visible throughout the completed closed
  unit, so forward calls and mutual recursion do not depend on traversal order.
- **Continuing debt:** this does not make locals, regions, labels, generic
  parameters, or named constants globally visible. It does not resolve contract
  member semantics. A-01 is an owner ruling, not a v0.8 discrepancy ID.

#### FN-4 — derived

- **Delta and selection ground:** W3 requires every writer-stated law to be
  checked rather than trusted; R4 requires an unavailable discharge to fail
  closed; P0 and R0 are supported by the measured checked-law reassociation
  channel. Source acceptance therefore uses one mandatory, compiler-independent
  calculus: exact contract/member/conformance binding, an exact nongeneric
  two-`own D` pure signature, and a bound body containing only the direct
  `return iadd.sat<D>(p0, p1);` shape. The closed integer table defines totality,
  domain equality, the unsigned holds cells, the signed holds/refuted cells,
  and exact zero identity. No optional prover can accept another source shape.
- **Authority boundary:** a successful source discharge emits one canonical
  base derivation record but grants no optimization consequence. A law can
  affect optimization only through a separately approved optional proposition
  family whose independent verifier rederives the exact relation from the
  accepted artifact and binds the artifact, target, backend, proposition, and
  consequence. Absence or failure of that optional path leaves acceptance,
  semantic identity, checks, and empty-overlay lowering unchanged. The gated
  ledger remains a separate source of candidate propositions, not a source
  `conform` discharge. Static-proof menus, general source proof artifacts,
  runtime enumeration, and sampling are not v0.9 admission routes.
- **Evidence and debt:** the exact initial slice is grounded by
  `experiments/checked-law-channel/RESULTS.md` and the protected discharged,
  refuted-signedness, and undischarged FN-4 cases; the unshipped fact channel
  still lacks independent soundness tests. Additional operations and complete
  proof calculi are deferred specification additions. Most importantly,
  FN-4's local obligation relation does **not** define whole-conformance member
  completeness, extra/missing binding behavior, law-free conformance behavior,
  generic contract substitution, or behavior-parameterized calls.
  `discrepancy:v0.8/fn3-contract-member-semantics` remains open. This amendment
  addresses `discrepancy:v0.8/fn4-law-admission` only after installation.

#### FN-8 — derived_existence_only

- **Delta and selection ground:** grammar accepts ordinary typed `doc | stmt`
  children, then one early FN-8 pass requires ordinary lets followed by exactly
  one final check before recursively checking children. This correctness-
  selected boundary gives every excluded shape one deterministic owner without
  creating a requires-only parser. FORM-3 owns misuse of fixed `requires` as an
  identifier.
- **Continuing chain and debt:** the measured callee-entry semantics, retained
  check, and downstream proof chain continue. The block spelling remains
  R3-provisional, and contract/refinement use remains deferred. This contributes
  to resolving `discrepancy:v0.8/fn8-reserved-rule-attribution` after install.

#### FORM-2 — derived_existence_only

- **Delta and selection ground:** FORM-1 and W3 require a total byte format.
  The proposal makes it executable by rendering each source's ordered forest of
  top-level item subtrees from the one compilation-unit tree, with closed line,
  indentation, attachment, block, and source-boundary rules. The one-tree and
  per-source ownership mechanics are correctness- and evidence-selected.
- **Evidence and debt:** the primary and independent structural reports and the
  exact protected migration must bind the final candidate. The specific visual
  formatting conventions remain R3-provisional because no writer-tier format
  comparison selected them. This addresses
  `discrepancy:v0.8/form2-protected-conformance-spacing` only together with the
  separately approved protected migration; no verdict change is implied.

#### FORM-3 — derived_existence_only

- **Delta and selection ground:** deterministic grammar terminals and META-2
  require a context-free partition. IDENT now excludes the complete
  mechanically extracted set of exact fixed lowercase grammar words, removing
  fixed-word call/binding derivations without ordered-choice priority. The
  OPNAME explanation is aligned with that exclusion. The exclusion is evidence-
  selected by the two grammar engines and protected FORM-3 attribution.
- **Continuing debt:** casing and sigil choices retain their earlier lexicon and
  writer-tier debt. This contributes to resolving
  `discrepancy:v0.8/gram-terminal-ident-partition` and
  `discrepancy:v0.8/fn8-reserved-rule-attribution` after install.

#### FORM-4 — derived_existence_only

- **Delta and selection ground:** the `doc` production owner reference changes
  from GRAM-3 to its actual unique owner, GRAM-2. This is a minimal correctness
  erratum under META-4 and changes no construct.
- **Continuing debt:** the no-comments and doc-field choices remain
  R3-provisional. This addresses
  `discrepancy:v0.8/form4-doc-cross-reference` after install.

#### FORM-5 — derived_existence_only

- **Delta and selection ground:** one-spelling and no-undefined-value rules
  require a total host-independent finite-float contract. Exact rational input,
  signed zero, IEEE round-to-nearest-ties-to-even, shortest byte length, and
  unsigned-ASCII tie-breaking are correctness- and evidence-selected.
- **Evidence and debt:** exact-rational and independent Rust checks in
  `grammar-verifier/evidence/float-canonicality.json` must bind the final
  candidate. Decimal-only literals, no boolean literals, and other inherited
  surface choices remain R3-provisional. Together with FORM-7 this addresses
  `discrepancy:v0.8/form5-form7-float-canonical-spelling` after install.

#### FORM-7 — derived

- **Delta and selection ground:** T2, R4, W3, and FORM-1 already require finite,
  non-silent, canonical numeric values. Requiring the unique FORM-5 spelling
  closes the former contradiction and is correctness- and evidence-selected.
- **Continuing debt:** no host parser or formatter is language authority. Future
  literal-family additions remain separate specification deltas. This shares
  the FORM-5 discrepancy disposition above.

#### GIVE-1 — derived_existence_only

- **Delta and selection ground:** edge-case review found that the prior recursion
  could treat an inner value match, a may-trap operation, or an unproved loop as
  outward delivery. Correctness selects the exact repair: an inner value match
  delivers only to its own let; only a final statement match whose arms deliver
  relative to the same outer value match recurses outward; a may-trap check or
  call retains a continuing edge; no loop is assumed to diverge; and a break
  counts only when its resolved loop lexically encloses that same value match.
- **Continuing debt:** the explicit `give` surface remains R3-provisional. This
  is a semantic contradiction repair found during successor review, not a claim
  to close a separately registered v0.8 discrepancy ID.

#### GRAM-1 — derived

- **Delta and selection ground:** W3, FORM-1, DIAG-1, and META-2 require one
  context-free derivation. The exact maximal raw forms, predicate-valued terminal
  membership, full matching-predicate retention, pairwise-disjoint strong-LL(2)
  `SELECT_2` languages, and one-production/one-node rule are correctness-
  selected and independently executable. Quoted fixed atoms expand inside the
  specification to their unique raw-token sequence: `"&uniq"` counts as `&`
  then `uniq`, while `"->"` and `"=>"` each count as one compound token. This
  leaves exactly one pattern atom: quoted `"[0-9]+"` in `const` denotes one
  complete numeric-form token matching that pattern and is not a fixed atom.
  These self-contained rules make the two-token bound a statement about formed
  tokens and prevent an unreviewed tool-local atom table. Predicate priority
  and parser-local keyword lists remain forbidden.
- **Evidence and debt:** both grammar engines must rerun against the final
  candidate and agree on complete static and bounded generalized-parser
  evidence, including identical fixed-atom expansion and sole-pattern-atom
  classification; the generalized parser remains an evidence tool, never
  production authority. This contributes to resolving
  `discrepancy:v0.8/gram-terminal-ident-partition` and
  `discrepancy:v0.8/gram1-gram7-match-node-bijection` after install.

#### GRAM-2 — derived_existence_only

- **Delta and selection ground:** law arguments and requires entries parse as
  ordinary syntax before their semantic owners check them; `law_arg` and
  `requires_entry` make that boundary explicit. CONST-1 and CONST-2 become the
  unique owners of their unchanged grammar definitions. This factoring is
  correctness-selected where required for strong-LL(2), and otherwise
  minimality-selected to preserve one general parser.
- **Continuing debt:** contract/conform syntax and doc fields inherit their
  existing R3 debts. In particular, broad parsing plus FN-4 does not resolve
  `discrepancy:v0.8/fn3-contract-member-semantics`.

#### GRAM-3 — derived

- **Delta and selection ground:** the type, mode, targs, and targ productions
  keep their source shapes; duplicate `const` and `cvalue` definitions move to
  CONST-1 and CONST-2. Unique semantic ownership is correctness-selected under
  META-4.
- **Continuing debt:** region vocabulary and composite-type debts are unchanged.
  Fixed-terminal partition evidence is owned by GRAM-1/FORM-3, not by an
  invented GRAM-3 priority.

#### GRAM-4 — derived_existence_only

- **Delta and selection ground:** the complete let prefix precedes one explicit
  choice among `ordinary_let_rhs`, `try_let_rhs`, and `value_match`; statement
  and value matches are distinct productions. This factoring is evidence-
  selected by the strong-LL(2) and one-derivation requirements.
- **Continuing debt:** loop and match surface choices remain R3-provisional.
  Broad `requires_entry := doc | stmt` is checked by FN-8 and does not give
  excluded statements requires semantics. This contributes to the GRAM-7 node
  discrepancy disposition.

#### GRAM-7 — derived_existence_only

- **Delta and selection ground:** one-production/one-node requires distinct
  `match_stmt` and `value_match` node kinds in their disjoint source positions.
  This is correctness- and evidence-selected; a shared node kind is not an
  allowed normalization.
- **Continuing debt:** the contained let-initializer match and explicit `give`
  spelling remain R3-provisional. This addresses
  `discrepancy:v0.8/gram1-gram7-match-node-bijection` after install.

#### PRE-1 — derived_existence_only

- **Delta and selection ground:** only the byte layout of the existing prelude
  declarations changes so the normative fence is an exact FORM-2 rendering.
  Types, variants, fields, contracts, and conformer sets do not change.
- **Continuing debt:** Bool-as-prelude-enum and the other earlier prelude form
  debts remain open. This is formatting-only and proposes no semantic or verdict
  change.

#### PROG-1 — derived

- **Delta and selection ground:** the closed-world P0/W3 chain continues while
  PROG-2 becomes the unique unit-formation owner. The rule expressly preserves
  every ban: include, import, module, separate compilation, incremental semantic
  cache, internal ABI, dynamic loading, reflection, and source-path lookup.
- **Continuing debt:** a logical path contributes identity only and cannot
  become a namespace or lookup key. The whole-program check-loop latency debt
  remains. A-10 is an architecture question, not a v0.8 discrepancy ID.

#### PROG-2 — derived_existence_only

- **Delta and selection ground:** GRAM-1, PROG-1, DIAG-1, and A-01 require one
  exact answer for how several transported sources become one program. The rule
  therefore defines an ordered nonempty sequence of exact logical source
  records, portable paths, envelope failures, per-source derivation and FORM-2
  audit, no cross-record syntax, one flattened program root with
  `BundleRootExtent`, whole-unit declaration order, and identity-preserving
  empty records. Existence is architecture- and correctness-required; the
  precise path grammar and nonempty ordered-record form are minimality-selected.
- **Evidence and debt:** independent frontend-boundary evidence must cover
  invalid/duplicate paths, zero records, zero-byte versus one-LF sources,
  reorder/repartition distinctions, root extent, and cross-source isolation.
  Future modules or separate compilation would be new specification decisions.
  This resolves A-10 only after exact approval and installation; A-10 is not a
  v0.8 discrepancy ID.

#### TYPE-6 — derived_existence_only

- **Delta and selection ground:** D24/A-01 selects a total visibility table:
  every top-level function signature is visible throughout the completed closed
  unit, while every other declaration remains visible only after its lexical
  declaration. This removes traversal-order semantics from forward calls and
  mutual recursion without broadening other namespaces.
- **Continuing debt:** no-shadowing remains R3-provisional, and constructor/type
  collision questions remain separate. This does not authorize or define
  general contract-member resolution. A-01 has no v0.8 discrepancy ID.

### Discrepancy and evidence boundary

After, and only after, exact v0.9 installation plus its separately approved
protected migration, these eight immutable v0.8 records have v0.9 dispositions:

- `discrepancy:v0.8/diag1-pre-tree-node-path`;
- `discrepancy:v0.8/fn4-law-admission`;
- `discrepancy:v0.8/fn8-reserved-rule-attribution`;
- `discrepancy:v0.8/form2-protected-conformance-spacing`;
- `discrepancy:v0.8/form4-doc-cross-reference`;
- `discrepancy:v0.8/form5-form7-float-canonical-spelling`;
- `discrepancy:v0.8/gram-terminal-ident-partition`; and
- `discrepancy:v0.8/gram1-gram7-match-node-bijection`.

Their v0.8 records are never rewritten. A versioned v0.9 discrepancy set must
record the installed dispositions. The seven other registered v0.8 gaps remain
unresolved: affine-deref storage lifecycle, retained-check `proof_ref`, EFF-1
row canonicality, body-local region effects, FN-3 contract-member semantics,
`main` return spelling, and dotless-operation reservation. This amendment also
does not settle A-02 through A-09 or A-11 through A-18.

Before installation, all of the following must bind the exact final candidate
hash: the two grammar-engine report, float-canonicality report, primary and
independent FORM-2 reports, frontend-boundary report, protected-surface census,
and exact protected migration. A green development gate is evidence, not owner
approval, and no production parser receives authority from this ledger text.

## v0.10 and v0.11 amendments — resolver and semantic closure (2026-07-22)

`kernel-spec-v0.10.md` changed no rule IDs or derivation statuses. Its
grammar-selected namespaces and deterministic resolver diagnostics close
previously open implementation meaning under the existing TYPE-6, DIAG-1 and
FN-1 chains.

`kernel-spec-v0.11.md` also changed no rule IDs. It closes the first executable
semantic slice, changes the propagation terminal from `try` to `propagate`,
selects one private checked-program lowering authority, and reduces mandatory
runtime reporting to one exact trap record. DIAG-3 therefore moves from
`derived_existence_only` to `derived`; the other changed rules retain their
prior status. The current total is **51 derived · 41 existence-only · 0
underived** across 92 rules.

| v0.11 changed rule | current status |
|---|---|
| FORM-2 | derived_existence_only |
| GRAM-4 | derived_existence_only |
| SCOPE-4 | derived |
| TYPE-6 | derived_existence_only |
| STOR-3 | derived_existence_only |
| OP-1 | derived_existence_only |
| OP-2 | derived |
| OP-4 | derived |
| OP-5 | derived_existence_only |
| FN-1 | derived |
| FN-4 | derived |
| FN-7 | derived_existence_only |
| FN-8 | derived_existence_only |
| EFF-4 | derived |
| ERR-3 | derived |
| DIAG-1 | derived |
| DIAG-2 | derived |
| DIAG-3 | derived |

## v0.12 amendment — copy-place assignment and origin effects (2026-07-22)

`kernel-spec-v0.12.md` adds SET-1 and modifies fifteen existing rules without
changing grammar or terminal inventories. SET-1 is derived above from the real
mutation requirement plus the ownership, storage, ordering, effect, and checked
lowering invariants. EFF-2's ultimate-storage-origin projection closes a
correctness gap: a local region spelling cannot appear in an enclosing function
signature, while a borrow, reborrow, slice, or call must not erase the
caller-visible read or write origin. The other changed rules synchronize these
judgments, their diagnostics, and current governance references. No derivation
status is weakened. The current total is **52 derived · 41 existence-only · 0
underived** across 93 rules.

The active native `whitefoot-spec` gate checks exact v0.17 identity, unique
rule definitions, closed bracketed rule references, and one ledger row for
every active rule. It checks structural coverage; the truth of each derivation
chain remains a review responsibility.

## v0.13 amendment — Result propagation consumption (2026-07-22)

`kernel-spec-v0.13.md` modifies seven existing rules without changing grammar,
terminal inventories, runtime values, effects, or safety checks. ERR-3 now
states that a direct bare affine own-rooted Result place is a consuming
propagation operand, while explicit `move` remains valid. This aligns the sole
Result-forwarding writer form with OWN-13's existing structurally declared
consumption without expanding bare affine use in ordinary expressions.
TYPE-6, STOR-1, FN-4, DIAG-2, and DIAG-3 receive only synchronized current-
version references. No derivation status changes.

## v0.14 amendment — exact integer negation (2026-07-22)

`kernel-spec-v0.14.md` modifies six existing rules without changing grammar,
terminal inventories, operation names, effects, or safety checks. OP-2 closes
the semantics of the existing `ineg` rows: wrapping negation computes modulo
the signed width, trapping negation traps when the mathematical result is not
representable, and checked negation returns `Err(Overflow())` on that same
edge. Thus signed minimum remains minimum only in wrapping mode; all other
representable inputs produce their exact mathematical negation. This follows
the already-derived OP-2 mode partition and OP-8 totality requirements rather
than adding a new semantic mechanism. TYPE-6, STOR-1, FN-4, DIAG-2, and DIAG-3
receive only synchronized current-version references. No derivation status
changes.

## v0.15 amendment — selected-target layout (2026-07-23)

`kernel-spec-v0.15.md` adds STOR-6 and modifies eight existing rules without
changing grammar, terminals, writer-visible values, or language effects.
STOR-6 requires the compiler to select the exact target before target-dependent
output, reject statically unrepresentable complete layouts as a target failure,
and retain explicit target-domain obligations for runtime-sized allocation and
element addressing. Target lowering must discharge each obligation or emit an
exact no-wrap guard on both facts-on and facts-off paths. This closes the
undefined “frame limit” reference without inventing a numeric language cap,
hidden heap fallback, or new source-language rejection. The synchronized
TYPE-6, STOR-1, OP-4, OP-9, FN-4, DIAG-1, DIAG-2, and DIAG-3 text preserves
their derivations. The current total is **53 derived · 41 existence-only · 0
underived** across 94 rules.

## v0.16 amendment — static source-conformance closure (2026-07-23)

`kernel-spec-v0.16.md` modifies nine existing rules without changing grammar,
terminals, source constructs, runtime values, ABI, or language effects. FN-3
now defines one complete static family: a nongeneric source contract owns a
source-ordered unique signature table; a conformance has one exact concrete
subject and one coherent source-contract key; and its declared-order binding
vector is complete, nongeneric, `requires`-free, and signature-exact.
Callable-signature equality uses exact modes and types plus equality of the
normalized read, write, allocation, and trap capabilities after positional
region alpha-renaming. This preserves EFF-1 and EFF-2 as the independently
checked effect authority, ignores only semantically irrelevant repetition and
occurrence order, and admits neither effect subtyping nor an omitted or added
capability.

Every law-bearing validated conformance must then obtain FN-4's closed
discharge. A successful discharge produces one base record used for source
acceptance only. The record grants no dispatch, lowering, check-elision,
reassociation, or optimizer authority; any future optional law fact must
independently rederive the complete relation at a separately approved boundary.
`pure` remains only a required signature premise and does not establish
totality or any algebraic equation.

The independent-rederivation proposal in the preceding historical v0.16 text
is superseded by the active FN-4 derivation above and v0.40: the
originating semantic check now retains one checked fact for fixed consumers in
the same compilation, while its diagnostic derivation never runs a second
verification pass.

This closes `discrepancy:v0.8/fn3-contract-member-semantics` for whole static
conformance identity, binding completeness, signature compatibility,
law-free conformance, and mandatory law discharge. It does not add behavior
parameterization: source-contract generic bounds and contract member calls are
rejected or absent, and contracts, conformances, bindings, and law records
have no runtime or lowering representation. The callable behavior question is
an explicit future FN-5 delta rather than part of the closed discrepancy.
TYPE-6, STOR-1, FN-2, FN-3, FN-4, FN-5, DIAG-1, DIAG-2, and DIAG-3 retain
their derivation statuses. The total remains **53 derived · 41 existence-only
· 0 underived** across 94 rules.

## v0.17 amendment — direct returned-slice provenance (2026-07-23)

Specification binding:
`spec/kernel-spec-v0.17.md`, SHA-256
`19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93`.

v0.17 modifies thirteen existing rules without changing grammar, terminals,
source constructs, runtime values, ABI, operation inventory, effects, traps,
or required runtime safety checks. It closes one caller-visible semantic gap:
a returned direct slice may refer to more than one source even though its
region spelling is the same. T1/T2 and W3 require every possible source to
survive the call boundary so alias, lifetime, and effect checks cannot inspect
the wrong root. FN-1's settled signature-completeness boundary therefore
selects one finite origin ceiling derived from the written signature rather
than from callee bodies.

The finite-origin coverage proof is an induction over the closed producer set:

1. An incoming direct-slice parameter starts with one formal-slice origin.
2. `slice_of` starts with one resolved source place or `immutable-const`.
3. Binding, moving, passing, returning, borrowing the descriptor, and
   resolving through its holder preserve the complete set.
4. A call simultaneously substitutes each finite actual set for each formal
   supplier and takes their finite union plus `immutable-const`.

The induction is closed because every omitted producer receives an explicit
boundary. A slice-valued `value_match` is rejected rather than receiving an
undefined join. Region-bearing generic arguments and enumerated stored-content
positions are rejected under FN-2 and STOR-5 rather than hiding per-leaf
origins. A borrow-mode direct-slice result is rejected because it would require
both returned-descriptor provenance and underlying slice origins. A direct
own-slice body may return only origins in its signature ceiling, excluding raw
borrowed storage and callee-local arena storage. Recursive calls reuse the same
finite written ceiling and need no body-derived fixed point.

Each runtime slice descriptor still points to exactly one storage root. By the
induction above, that root is a member of its static set. OWN-5 and OWN-7 test
every possible origin for alias conflicts, and EFF-2 projects reads and writes
through every origin after formal-occurrence selection and before caller
region substitution. Therefore the static set is a sound conservative
overapproximation: no runtime branch, traversal order, callee body, or
optimizer fact may narrow it for acceptance.

The same signature-formation judgment applies to contract `fn_sig` members.
An own-slice member derives its ceiling from ordinal parameters; FN-3's exact
signature equality after positional region alpha-renaming makes a bound
function's ceiling correspond automatically. The member has no body judgment,
while the bound `fn_decl` remains independently subject to complete FN-1 body
and return-origin validation.

This amendment deliberately changes two hidden-leaf families that were
formerly valid but compiler-unsupported into source rejections:
region-bearing function and nominal generic arguments cite FN-2, and
region-bearing box or arena content—including `box_new<T>` and
`arena_new<'r, T>` substitutions—cites STOR-5. Direct
`slice<'s, slice<'r, T>>` type formation is not an enumerated stored-content
position and retains its language status; the current compiler may report the
unimplemented non-flat value path as unsupported, never as invalid source.
General per-leaf stored provenance, returned-borrow provenance, arena cleanup
transfer, and branch-produced slice joins remain separate amendments.

TYPE-6 and the current-version references in STOR-1, FN-3, FN-4, FN-5, DIAG-2,
and DIAG-3 stay synchronized without changing their underlying derivations.
OWN-5, OWN-7, STOR-5, FN-1, FN-2, and EFF-2 receive the correctness-selected
extensions above. No derivation status changes; the total remains **53 derived
· 41 existence-only · 0 underived** across 94 rules.

## v0.18 amendment — system-interface first command slice (2026-08-05)

Specification binding:
`spec/kernel-spec-v0.18.md`, SHA-256
`307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28`, installed
byte-for-byte from the candidate approved on 2026-08-05.

v0.18 adds twenty-five rules — `PROG-3`, `EFF-5`, `SYS-1..13`, `HOST-1..3`,
`PATH-1..2`, `QUAL-1..3`, `TRAP-1`, and `GATE-2` — and modifies thirteen:
GRAM-2, GRAM-11, TYPE-2, TYPE-6, OP-1, FN-3, FN-7, EFF-1, EFF-2, EFF-3, STOR-3,
PROG-1, and DIAG-1. Grammar productions +2 (`program_kind`, `input_label`),
modified 3; terminal spellings +3 (`as`, `external`, `blocks`); tokens +0;
exception clauses +0. The unlabelled entry and every v0.17-accepted program
behavior are unchanged, and no existing rule changes derivation status.

The selection ground is recorded in the rows above rather than repeated here:
the owner selected this architecture and the Route C declaration home on
2026-08-05 from the BOUND-1 dossier's alternative table after a thirty-one-issue
edge-case review, and the recorded fallback to a prelude extension lapsed
unexercised. Evidence lives in
`research/investigations/system-capability-architecture/` (dossier §5
alternatives, §12-§13 review results and owner decision, and the reviewed
decision records), in the live `mcts_mem/whitefoot/system-interface` node and
its `declaration-home` child with their rejected alternatives, and in the
`governance/APPROVALS.md` entry of the same date.

Statuses were assigned by one standard, stated here so a later reader can
re-apply it. The review's decisions are edge-case-review resolutions with
recorded grounds, not experiments. Where a decision fixed a semantic closure —
what happens on failure, which owner survives, what a target may not narrow —
it supports **derived**, because the argument is theorem-shaped and checkable by
reading it. Where a decision fixed a taste — a spelling, a class membership, a
field width, a home selection — it does not, and the rule is
**existence-only**. Four new rules fall on that side: SYS-1 (the declaration
home is an owner fork whose sole discriminator is a prospective reservation
cost), SYS-2 and SYS-7 (the operation list and the thirty-class `IoError`
membership are asserted rather than selected, and the two-field target detail is
reasoned but unmeasured), and SYS-4 (the kind taxonomy is argued classification
that nothing in v0.18 checks). The remaining twenty-one restate or instantiate
already-derived law — SCOPE-4, EFF-4, STOR-1, STOR-3, TYPE-4, ERR-2, OP-4, and
R6 — over a new domain.

Three honest weaknesses are recorded rather than resolved. `blocks` has no
consumer in v0.18 and is exactly coextensive with `external` across all eleven
operations; it is fixed now on a decidability ground plus a named successor
operation the slice does not contain, which is a live R1 tension. GATE-2 and
TRAP-1 are additions beyond the dossier's own delta inventory, ratified at exact
approval rather than reviewed as numbered rules, so both chains rest on the
retained law they restate and claim no review provenance. SYS-12 retains the
conservative stdout/stderr may-alias fact with no v0.18 consumer, on a
fail-closed ground.

The current total is **74 derived · 45 existence-only · 0 underived** across 119
rules.

## v0.20 amendment — non-argument reborrow disposition and same-node rejection ordering (2026-08-07)

Installed as `spec/kernel-spec-v0.20.md` at SHA-256
`b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1`,
byte-identical to the exact-approved candidate
(`governance/APPROVALS.md`, 2026-08-07).

The amendment adds OWN-14 (existence-only; row above) and modifies OWN-5,
OWN-6, OWN-9, OWN-13, and DIAG-1 with unchanged statuses — each row carries
its "v0.20" amendment note. Grammar productions +0; tokens +0;
terminal spellings +0; exception clauses +0. The evidence grounds are the
0024/0025 task records (the three return-position conformance cases and the
recorded TYPE-7/OWN-1 simultaneity instance), the 0027-unmasked fourth
witness `own13-pos-uniq-match-payloads` (the OWN-13/OWN-5 binder
contradiction, reproduced against the conforming compiler), the live
`mcts_mem/whitefoot/ownership/no-reborrow` decision node's carded relief
valve, and the subsumption arguments recorded in the OWN-13 and OWN-14 rows.

The v0.20 total is **74 derived · 46 existence-only · 0 underived** across
120 rules.

## v0.21 amendment — obligation discharge batch 1 (2026-08-07, candidate stage)

Specification binding:
`governance/spec-evolution/kernel-spec-v0.21-candidate.md` at SHA-256
`3c63a6274047ee2f7eceac7ec6b03d0b84d42fb87cc13da7e6b80ed5b934df9f`
(count-corrected before approval), assembled from the active v0.20 plus the approved batch-1
delta (`governance/spec-evolution/obligation-discharge-batch1-candidate.md`:
owner rulings O1–O16, edge-case-review fixes F1–F11, and the sitting
adoption of the second OP-1 modification are recorded there). Candidate
stage: these rows exist ahead of activation so the native `whitefoot-spec`
gate covers all 128 rule IDs; the `docs/WORKFLOW.md` step-4 exact-byte
approval and installation as `spec/kernel-spec-v0.21.md` are pending, and
this binding is restated at activation.

v0.21 adds eight rules — CLM-1, CLM-2, and ENT-1..ENT-6 (rows below) — and
modifies fifteen existing rules at sixteen enumerated modification sites:
FORM-2, FORM-5, GRAM-4, GIVE-1, OP-1 (two sites: the `index_get` row and the
adopted non-consuming place-operand sentence), OP-4 (rewritten to
discharge-or-reject), FN-1, FN-8, EFF-2 (three sites), SET-1, DIAG-1,
DIAG-2, DIAG-3, SYS-8 (two sites), and SYS-9 (three stated relations).
Grammar productions +1 (`claim_stmt`); tokens +2 (`claim`, `because`);
terminal spellings +2; operation table +1 row (`index_get`); exception
clauses +0. No existing rule changes derivation status; the modified rows'
amendment notes are activation work, not candidate work. Evidence grounds:
`research/investigations/obligation-discharge/` (DOSSIER.md §2/§3/§8,
SIMULATION.md, PROBE-W1.md rounds 1–2, PROBE-TAINT.md, PROBE-CODEGEN.md,
SYS-POSTCONDITIONS.md, and CANDIDATE-REVIEW.md with its re-verification),
the sixteen owner rulings of 2026-08-07, and `governance/APPROVALS.md`'s
sequencing amendment selecting atomic activation.

| Rule | Feature | Status | Derivation chain | Notes |
|---|---|---|---|---|
| CLM-1 | `claim name: e because "text";` — named runtime check, the writer's sole trap-stating statement | 🟡 existence-only | Existence: W3 keystone (no construct may introduce a fact without a proof or an executed runtime check — the SPARK `pragma Assume` door stays structurally closed; obligation-discharge DOSSIER §3, owner/assistant design record 2026-08-05..06) + R4 ladder (the failure path is language-authored and traps loudly with a named record; the writer authors only the predicate and the auditable justification — authorship factoring) + OP-5 lineage (semantics are exactly check-else-trap plus a name, and the executed check is what admits the fact into [ENT-3], so no assumption enters unexecuted). Construct-level evidence: PROBE-W1 rounds 1–2 (16/16 baseline and stress-test writers steered to honest shapes by the residual-printing loop) and SIMULATION.md (claim consolidation: ~13 claims cover 27 sites across three real programs; the corpus's existing test assertions map onto claims unchanged). Form NOT derived: the name-first spelling, mandatory `because` STRING, per-function name uniqueness outside every TYPE-6 domain, and the reservation exemption are owner-ruled (2026-08-07) without a comparative writer-form experiment. | Awaits: spelling comparison under W1 writers and the ledger tooling that consumes name-plus-predicate identity; fired-claim escalation is a toolchain contract. Redundancy and refutation live in CLM-2. |
| CLM-2 | Claim lifecycle: required redundancy advisory, refutation hard error, fired-claim escalation | ✅ derived | Version-monotonicity requirement (acceptance may not tighten when the checker strengthens; DOSSIER §2.7 keystone — 'acceptance monotonicity depends on redundant-claim being a warning') -> a provable claim must be a non-rejecting advisory in every later version, never an error. R4 shift-left -> a refuted claim (the predicate's exact negation derived in a non-contradictory state) is a program proven to trap on every execution reaching it, so compile-time rejection is the ladder's exact direction, deliberately enumerated as the lifecycle's one non-monotone edge in [ENT-1] (candidate review F4 forced the exception to be stated rather than implied). W3 -> the advisory channel cannot alter acceptance and the writer cannot suppress refutation. Fired-claim reclassification is a toolchain contract in the ERR-2 edit-list sense, not a language judgment. | Form is judgment semantics with no spelling axis. Advisory channel and encoding are implementation-owned this version; normative advisory bytes deferred (owner ruling 2026-08-07). |
| CLM-3 | opt-in transitive strict no-claim partition | 🟡 existence-only | Existence: W3 forbids assertion-backed authorization while the Stage 9a deterministic ledger measures a finite real claim population; FN-1 caller locality, the existing complete/U separation, and finite concrete ordinary-call SCCs therefore support an opt-in outgoing closure whose protected obligations and requirements must succeed without S2/S3, whose direct and imported claims are counted independent of reachability, and whose membership never propagates upward. SCC atomicity and one runtime body prevent a recursive or mixed-caller bypass. Form NOT derived: the `deny_claims` spelling, declaration-prefix placement, exact writer surface, and diagnostic tie-breaks are minimality-selected by the ACTIVE Stage 9b plan and have not had a comparative writer experiment. | Candidate v0.29 only. It reuses the existing U view, ordinary call graph, function-local derivation DAG, and failure-atomic checked-program batch; it adds no fact, solver, body, lowering, runtime check, effect, ABI field, foreign adapter, or ClaimLedger authority. |
| ENT-1 | L0 fragment authority: deterministic, spec-pinned, acceptance-bearing, TCB-resident, version-monotone | ✅ derived | W3 determinism (two conforming implementations must derive identical fact states and dispositions — the R0 delta of record: deterministic spec-pinned discharge versus SMT portfolios with timeout/replay instability, DOSSIER §5) + T1 (a wrong discharge compiles a raw out-of-bounds access, so the fragment joins the TCB beside the type and ownership checkers; owner ruling of record — a compiler bug class owned by testing, never language-level hedging, DOSSIER §4.3) + facts-off correctness law (the fragment is acceptance machinery, not an optional optimizer-fact family, and [SCOPE-2] is unchanged because every fact source is check-backed) -> determinism, per-FN-2-instantiation judgment, TCB placement, and the monotonicity law with CLM-2's enumerated refutation exception. | The monotonicity law keeps every later fragment strengthening a pure widening; the refutation carve-out is deliberate (review F4). |
| ENT-2 | Fragment terms and facts: tracked places, length terms, constants, difference bounds, disequalities | 🟡 existence-only | Existence: the fragment needs one closed fact language at released-spec precision (DOSSIER §4.2: kill rules, congruence, interval arithmetic all become normative text), and SIMULATION.md exercised exactly this L0 strength by hand — 57–59% of non-test bounds sites discharged outright on utf8parse, deflate-dynamic, and sha256, every residual one line. Determinism of the normal form (unique least closure over difference bounds) serves W3. Form NOT derived: the exact cuts — difference bounds only, declaration-anchored byte-identical place identity (review F2 repair), no index-segment places, the review-F5 term-root set — were hardened by edge-case review but were not comparatively selected against richer abstract domains. | Awaits: the preregistered acceptance run — the real checker must reproduce SIMULATION.md's per-program buckets — which is exactly the experiment the hand-simulation's single-analyst caveat defers to. |
| ENT-3 | Fact sources S1–S10: branch/match, check, claim, requires substitution, copy/cvt equalities, lengths, constant-offset arithmetic, midpoint family, const-array ranges, boundary counts | ✅ derived | Every source names an executed check, a declared allocation/type/operation contract, or a constant, preserving the W3 keystone (no assume anywhere), and each entered on measured evidence: dominating branch/match facts, allocation-length equality, const-array element ranges, and ±-constant arithmetic are exactly SIMULATION.md's validated L0 inventory on three real programs; S10's boundary count bounds are PROBE-TAINT.md's load-bearing finding (one structural claim in 723 wfgrep lines only because the read_once/host_copy count bounds hold) made normative through the SYS-8/SYS-9 postconditions and carried in the same [QUAL-1] contract trust class as S6's buffer_new length; the comparison-origin single-`let` cut is owner-ruled (2026-08-07) with its measured consequence recorded (the sha256 bucket restatement, review F8). | S4 requires-substitution is fail-closed on any non-comparison substituted shape; the S8 midpoint family is closed-shape and its unsigned arithmetic was verified sound in the edge-case review's retained-case record. |
| ENT-4 | Closure: least fixed point over difference-bound transitivity, disequality strengthening, subsumption; contradiction discharges everything | ✅ derived | Analytic, theorem-backed (the OP-2 div/rem class): given ENT-2's fact language the least closure exists, is unique and finite up to subsumption, and fixes every derivability answer, so the W3 two-implementation law holds by construction; the one-definition form with the reflexive implicit bound is review-forced (F7 removed a second, inequivalent shortest-path definition); contradiction-as-unreachable (every obligation discharged, no claim refuted) keeps CLM-2 refutation meaningful exactly where code is reachable in truth. | The contradictory-state disposition is owner-ruled (2026-08-07); a dedicated unreachable-code diagnostic remains open later work. |
| ENT-5 | Fact stability: kills by resolved-place overlap and effect-row projection, edge-ordered scope exits, joins, no-induction loop rule | ✅ derived | The kill architecture is the design's central derivation: exact bidirectionally-checked effect rows make the call kill a signature lookup (DOSSIER §2.6 — the analysis that drowns in aliasing elsewhere is modular here), reusing OWN-5/OWN-7 resolved-place overlap and EFF-2 boundary projection rather than inventing alias machinery (R1); edge-case review established the form — review F2 forced scope-exit kills to be edge events ordered before joins, and the retained-case record (buffer reassignment, element writes versus length facts, borrow aliasing, writes hidden from rows) is retained in CANDIDATE-REVIEW.md; the no-induction loop rule is the version-pinned L0 floor whose upgrade value SIMULATION.md priced (loop induction discharges ~11 of 13 structural claims), monotone under ENT-1. | TYPE-2's allocation-fixed lengths are what exempt length terms from element-write kills; the empty-join disposition for a break-less loop follows ENT-4's unreachability posture and is flagged for confirmation at the approval sitting. |
| ENT-6 | Obligations, discharge, residual: index bounds prove-or-reject with a printed one-line residual | ✅ derived | R4 shift-left is the rule itself: an unproven bound becomes a compile-time rejection carrying a rule citation and a pasteable residual instead of a runtime trap, and PROBE-W1 rounds 1–2 measured exactly this loop steering 16/16 writers to honest shapes (the checker-as-teacher argument, DOSSIER §3); P0 through the OP-4 lineage: 'proof means a deterministic checker derivation' was already OP-4 law, and discharge deletes the checked-site cost the simulation quantified (sha256's hottest loop at 5 checks per iteration consolidates under claims; PROBE-CODEGEN.md: the claim shape equals today's fused check shape by construction); for a current-function-local theorem, the zero-skill fallback — rebind, then claim the printed residual — closes the writable site after review F5 widened the term roots. The v0.34 amendment excludes every user-call and system-call result from that fallback. | The residual rendering schema (whole obligation, fixed bytes) is owner-ruled (2026-08-07) over the reduced-frontier alternative without a comparative test — this row's one untested residue. |

The current total is **80 derived · 48 existence-only · 0 underived** across
128 rules (120 active v0.20 rules plus the eight v0.21-candidate additions).

## v0.22 amendment — index surface settlement (2026-08-07, activated)

Binding: `spec/kernel-spec-v0.22.md`, SHA-256
`b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8`,
128 rules; no rows added or removed. ENT-3's S8 midpoint source is PARKED
(zero corpus demand; monotone re-add when a real site appears) and ENT-1's
removal prohibition is scoped to checker strengthening per the owner's
version-compatibility deferral. Statistics unchanged: 80 derived · 48
existence-only · 0 underived.

## v0.23 amendment — FLOOR-5 spelling relief (activated 2026-08-09)

Specification binding at activation:
`spec/kernel-spec-v0.23.md` at SHA-256
`e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5`, assembled
from the active v0.22 plus the approved FLOOR-5 delta
(`governance/spec-evolution/spelling-relief-candidate.md`: SWEEP rows A1, A3,
A4, and C1, with the owner rulings and edge-case-review fixes recorded
there). Installed as the active language authority on 2026-08-09 after the
owner's exact-byte approval, byte-identical to the approved candidate. After
the v0.24 stable-path switchover, these bytes remain the immutable outgoing
archive; compiler identity no longer depends on a parallel full-spec candidate
copy.

v0.23 adds and removes no rules. It modifies thirty-four existing rules at
sixty-two verbatim-anchored sites: FORM-2, FORM-3, GRAM-1, GRAM-4, GRAM-5,
GRAM-6, GRAM-7, GRAM-9, GIVE-1, TYPE-5, OWN-5, OWN-13, STOR-2, STOR-5, OP-1,
OP-2, OP-4, OP-7, OP-8, OP-9, ERR-2, ERR-3, FN-1, FN-4, FN-8, EFF-2, DIAG-1,
DIAG-3, SYS-13, ENT-2, ENT-3, ENT-5, ENT-6, and EX-1. Grammar productions grow
by four (`if_stmt`, `value_if`, `infix_tail`, `infix_op`; total 69) and the
fixed terminal inventory by seventeen (`if` and the sixteen `infix_op`
spellings; total 93 terminal predicates). Statistics unchanged: 80 derived ·
48 existence-only · 0 underived.

Amended 2026-08-08 by owner decision: the infix comparison spellings are
cancelled and all six integer comparisons keep their named calls, so the
delta loses two sites ([GRAM-1]'s compound-token sentence and [ENT-3] S1's
comparison-origin clause, both byte-identical to v0.22 again) and four
terminal spellings. The rule list and the production count are unchanged.

## v0.24 amendment — ENT-5 continuing-loop kills (activated 2026-08-09)

Specification binding: active `spec/kernel-spec.md`, headed v0.24, at SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
The superseded v0.23 bytes remain immutable at
`spec/kernel-spec-v0.23.md`.

v0.24 adds and removes no rules. It modifies one site in ENT-5: a loop-head
summary now includes only kills on paths that can continue to a later
iteration of that same loop. A kill on a path that returns, propagates an
error, or breaks out of the target loop does not invalidate pre-loop facts on
the continuing path. This refinement is derived from FN-1's structured normal
control graph together with ENT-5's existing edge-local kill and join rules;
it does not add reachability, arithmetic, or path-sensitivity beyond that
graph.

The grammar and rule inventories are unchanged: 69 productions, 84 decisions,
93 terminal predicates, and 128 rules. Pre-activation
frozen acceptance measured 22 of 33 UTF-8 obligations, 0 of 9 SHA-256
obligations, and 11 of 29 deflate obligations as claim-independent. Installed
post-activation confirmation belongs to the task closure evidence rather than
this derivation claim. Statistics remain 80 derived · 48 existence-only · 0
underived.

## v0.25 amendment — counted ascending u64 ranges (activated 2026-08-09)

Specification binding: active `spec/kernel-spec.md`, headed v0.25, at SHA-256
`c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.
The superseded v0.24 bytes are immutable at
`spec/kernel-spec-v0.24.md`, SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.

v0.25 adds and removes no numbered rules. It modifies twenty existing rules
at thirty anchored sites, plus one R3-PROVISIONAL register clarification, to
admit one ascending, unit-stride, half-open counted form over once-captured
`own u64` endpoints. S11 supplies only the compiler-executed structural facts
`lower_capture <= binder < upper_capture`; existing ENT-4 closure and S7
derive safe predecessor indices, while ordinary loops retain their
no-induction rule. The form is evidence-selected by the real SHA-256 consumer:
exactly three index loops replace four claims and discharge all nine array
obligations without a runtime trap in the pure compression function.

The native grammar inventory is 70 productions, 85 decisions, 96 terminal
predicates, and 128 rules. Fixed spellings add `for`, `in`, and `..`; the
scoped 448-file census found no accepted identifier collision. Derivation
statuses and totals remain 80 derived · 48 existence-only · 0 underived.

## v0.26 amendment — `requires` as one atomic call-site goal (activated 2026-08-09)

Specification binding: active `spec/kernel-spec.md`, headed v0.26, at SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
The superseded v0.25 bytes are immutable at
`spec/kernel-spec-v0.25.md`, SHA-256
`c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.

v0.26 adds and removes no numbered rules. It modifies fifteen existing rules
at twenty-two anchored sites: OP-5, FN-1, FN-8, EFF-2, PROG-3, DIAG-1,
DIAG-2, DIAG-3, GATE-1, and ENT-1 through ENT-6. The existing FN-8 clause
surface alpha-expands into one finite typed goal. Every ordinary call proves
the exact instantiated goal before transfer and callee effects; the body
receives it through S4, and no ordinary callee executes a fallback requirement
check. The two implemented process entries remain checked dynamic boundaries.
The same amendment makes a requirement effectless, admits otherwise-pure
required bodies and generic requirement templates, and adds signed opaque goal
facts without Boolean decomposition or a general theorem prover.

The amendment is evidence-selected by the obligation-discharge direction's O3
counterexample: an unconditional callee trap could hide a protected bounds
leaf from any later caller-side subject gate. The checked program therefore
retains a finite subject-only requirement-to-leaf bridge and its full,
unasserted, and S4-blinded rewalk evidence, but v0.26 emits no provenance
rejection and activates no held provenance class. This preserves T1's checked
safety boundary, W3's proof-only check authority, and the established FN-8
declaration surface while removing the structural bypass.

The grammar, token, operation-row, source-construct, and rule inventories are
unchanged: 70 productions, 85 decisions, 96 terminal predicates, and 128
rules. Derivation statuses and totals remain 80 derived · 48 existence-only ·
0 underived.

## v0.27 amendment — finite provenance and constrained-subject gate (activated 2026-08-10)

Specification binding: active `spec/kernel-spec.md`, headed v0.27, at SHA-256
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
The superseded v0.26 bytes are immutable at `spec/kernel-spec-v0.26.md`,
SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`
and byte-identical to the exact outgoing v0.26 authority.

v0.27 adds three rules and removes none: PRV-1, PRV-2, and PRV-3,
whose independently reviewed derivation rows appear in the ledger above. It
modifies nine existing rules at sixteen verbatim-anchored sites: OP-4, FN-1,
FN-8, SYS-2, DIAG-2, CLM-1, ENT-1, ENT-3, and ENT-6. The exact stable-file
diff is +62/-18 lines. Tokens +0/-0; terminal spellings +0/-0; grammar
productions +0/-0; operation-table rows +0/-0; source constructs +0/-0;
exception clauses +0/-0; and sections +0. The native inventories remain 70
productions, 85 decisions, and 96 terminal predicates. The rule inventory is
131.

The accepted set narrows only after the existing OP-4 or FN-8 full-state
judgment succeeds: an external constrained offset may not rely solely on S2
`check`, S3 `claim`, or an S4 requirement bridge. A real branch/value outcome
or removal of that external value from constrained-subject position remains
the repair. The gate widens no acceptance and changes no runtime operation,
effect rule, trusted assertion, or optimizer authority. Its exact current
subject is only the offset `i` in the protected leaf `i < len(P)`. Internal
subjects keep ordinary entailment; external values used only as bounds, bases,
write addresses, or unrelated operands remain outside the gate.

The selected classifier is deliberately finite explicit dataflow. Control
choice and write-address choice add no edge; storage is flow-insensitive per
whole root; payload selectors are direct rather than recursive; `len` remains
internal; and there is no path-sensitive storage, implicit-flow analysis,
Boolean decomposition, induction, arithmetic theorem prover, writer-spelled
provenance annotation, new goal language, or foreign adapter. The two fixed
points and diagnostic witnesses reuse the finite checked metadata already
retained by v0.26. These limits are the amendment's boundary, not a
noninterference claim.

The active totals are **83 derived · 48 existence-only · 0 underived** across
131 rules.

## v0.28 amendment — verified normal-return postconditions (activated 2026-08-15)

Specification binding: active `spec/kernel-spec.md`, headed v0.28, at SHA-256
`08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`.
The superseded v0.27 bytes are immutable at
`spec/kernel-spec-v0.27.md`, SHA-256
`bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`,
and byte-identical to the exact outgoing v0.27 authority. The specification,
compiler, five real consumers, fourteen additive protected cases, canonical
runner identity, approval chain, and derived material were installed as one
owner-approved atomic activation.

v0.28 adds FN-9 and removes no numbered rule. It modifies twenty existing
rules at forty verbatim-anchored sites: FORM-2, GRAM-2, GIVE-1,
GRAM-10, TYPE-6, OP-1, OP-5, FN-1, FN-3, FN-4, EFF-2, ERR-3, DIAG-1,
DIAG-2, and ENT-1 through ENT-6. The following rows bind every modified rule
without reclassifying any prior row:

| Rule | v0.28 status | Amendment role |
|---|---|---|
| FORM-2 | derived_existence_only | Adds the one canonical placement and rendering of the optional `ensures` block. |
| GRAM-2 | derived_existence_only | Adds `ensures_block`, `ensures_selector`, and `ensures_entry`, and one optional function-clause decision. |
| GIVE-1 | derived_existence_only | Admits relation delivery only from an eligible bare atom through `value_if`; `value_match` remains a closed negative. |
| GRAM-10 | derived_existence_only | Keeps zero, one, and multiple selector fields parseable while assigning their owner/member judgment exclusively to FN-9. |
| TYPE-6 | derived_existence_only | Retains a selector binder as provisional until FN-9 admits one block-local symbolic result datum. |
| OP-1 | derived_existence_only | Applies the existing reservation boundary to selector and clause-local candidate binders. |
| OP-5 | derived_existence_only | Classifies the final ensures check as a proof obligation with no execution or dynamic-boundary behavior. |
| FN-1 | derived | Extends the verified callable summary and keeps `fn_sig` template-free; only a function electing `ensures` receives the stricter result shape. |
| FN-3 | derived_existence_only | Excludes an ensures-bearing function from a static conformance binding. |
| FN-4 | derived | Excludes an ensures-bearing function from the existing exact law-discharge shape. |
| FN-9 | derived_existence_only | Defines selector admission, one output-bearing L0 relation, nonempty selected exits, entry-image stability, complete/U/B proof, callee-before-caller SCC publication, B-summary-first evidence selection, exact caller substitution and route cuts, and PRV-atomic publication. |
| EFF-2 | derived | Makes an ensures block proof-only and effectless. |
| ERR-3 | derived | Makes an automatically propagated `Err` unselected for an `Ok` postcondition and publishes no relation there. |
| DIAG-1 | derived | Fixes structural admission, selector ownership and same-block shadow rejection, stage order, exact residuals, and deterministic rejection selection. |
| DIAG-2 | derived | Extends the one function-local derivation DAG with S7, exit, aggregate, call, receiver, delivery, join, and atomic-publication roots. |
| ENT-1 | derived | Adds FN-9/S12 and bounded delivery to the same deterministic acceptance-bearing fragment without another proof authority. |
| ENT-2 | derived_existence_only | Adds the template-only symbolic result datum, view-independent parameter entry-image stability boundary, and Z's single S7 mathematical-zero disequality role. |
| ENT-3 | derived | Adds only the measured unsigned `iand` bounds, closed `ishl.wrap(one, count)` nonzero source, and verified S12 source. |
| ENT-4 | derived | Fixes complete exclusive dispositions for L0 relations, including equality, disequality, and one-bound exact negation. |
| ENT-5 | derived | Fixes support, kill, call/receiver order, forward delivery substitution, weakest-bound joins, and entry-image invalidation. |
| ENT-6 | derived | Carries candidate facts independently in complete/U/B after PRV-1 freezes and finalizes or discards the whole batch under PRV-2/PRV-3. |

The owner selected the high-level narrow selected-payload receiver and
`value_if`-only correction. Within that active plan, the lead semantic freeze
selected the `ensures` and selector spelling, explicit non-vacuity rejection,
and remaining exact closed route cuts. The complete 14/20 caller map and A10
delivery case select the existence and bounded semantics, but no function,
source, corpus, or test identity selects a language rule. FN-9 therefore adds
one existence-only row: the totals become **83 derived · 49 existence-only ·
0 underived** across 132 rules.

Grammar arithmetic from v0.27 is exact: 70 + 3 = 73 productions; 85 + 5 =
90 decisions (function `ensures?`, entries `*`, selector choice, optional
fieldbind list, and entry `doc | stmt`); 88 + 1 = 89 fixed terminal spellings;
and 96 + 1 = 97 terminal predicates. The only new spelling is `ensures`.

The frozen pre-implementation identity preflight used the active-v0.27 compiler,
not a candidate-branch build that self-embeds these candidate specification
bytes. Against the exact outgoing archive, baseline versus baseline exits zero
with `grammar-preserving candidate verified by the active compiler: 70
productions, 85 decisions, 96 terminal predicates`. Baseline versus this
candidate exits one with only `candidate changes the lexer or source grammar
of the baseline but does not match the compiler's embedded frontend contract`.
That identity mismatch was the expected freeze-boundary control. The installed
frontend now makes the archive-to-active verifier pass at 73 productions, 90
decisions, and 97 terminal predicates; the archive-to-archive v0.27 control
remains 70/85/96.

The amendment adds no runtime check, fallback, trusted assertion, optimizer
license, effect, cleanup behavior, error behavior, provenance class, host or
runtime ABI rule, serialized identity, or alternate lowering path. The private
`read_bits` signature and call-typing migration is ordinary source/compiler
synchronization. The installed protected matrix is additive: 437 cases, 30
unchanged annotations, and 132/132 rule coverage; it records rather than
selects the specification semantics above.

## v0.29 amendment — opt-in strict no-claim partition (activated 2026-08-15)

At the v0.33 closure that carried this amendment forward, the then-active
`spec/kernel-spec.md`, headed v0.33, had SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.
The superseded v0.28 bytes are immutable at
`spec/kernel-spec-v0.28.md`, SHA-256
`08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`,
and byte-identical to the exact outgoing v0.28 authority.

v0.29 adds CLM-3 and removes no numbered rule. It modifies twelve existing
rules: GRAM-2, OP-4, FN-1, FN-7, FN-8, FN-9, PROG-3, DIAG-1, DIAG-2, ENT-1,
ENT-3, and ENT-6. The following rows bind every modified rule without
reclassifying any prior row:

| Rule | v0.29 status | Amendment role |
|---|---|---|
| GRAM-2 | derived_existence_only | Adds the one optional fixed `deny_claims` prefix directly to `fn_decl`, before the optional program kind, with no new production. |
| OP-4 | derived | Rejudges each demanded protected leaf in the existing U view after ordinary complete and provenance success, preserving OP-4 at the actual `psuffix`. |
| FN-1 | derived | Exposes the declaration policy and derived strict summary at the caller boundary while retaining one signature, body, call graph, and lowering. |
| FN-7 | derived_existence_only | Admits the marker before either existing entry form without creating a third entry form, kind trigger, input, or adapter. |
| FN-8 | derived_existence_only | Rejudges demanded and outside-to-root call requirements in caller U, and marked program-start requirements before the retained wrapper check, at the existing FN-8 nodes. |
| FN-9 | derived_existence_only | Keeps the unchanged S12 and delivery candidates unpublished after PRV success until the strict partition also succeeds. |
| PROG-3 | derived | Requires the static marked-entry U judgment before, but never instead of, the one retained runtime wrapper check. |
| DIAG-1 | derived | Preserves every v0.28 verdict first, then fixes direct-claim, imported-claim, strict OP-4/FN-8, marked-entry, root, instance, and call ordering. |
| DIAG-2 | derived | Retains exact strict roots, SCC summaries, call and claim identities, successful U roots, and program-start disposition in the existing DAG and sole finalization. |
| ENT-1 | derived | Adds one fixed deterministic opt-in acceptance judgment over the existing concrete graph, U view, and derivation authority. |
| ENT-3 | derived | Removes PRV-only publication sufficiency from S12: the candidate source remains unchanged, but authority waits for total CLM-3 success. |
| ENT-6 | derived | Fixes the exact existing-U queries and preserves one failure-atomic S12, delivery, strict, and checked-program batch through the additional gate. |
| CLM-3 | derived_existence_only | Defines the finite outgoing SCC closure, exact direct and may-claim sets, strict-U success, deterministic direct/import ownership, and no-upward-propagation boundary. |

Existence is selected by the ACTIVE Stage 9b plan and Stage 9a's measured
finite claim population: an opt-in partition must close transitive assertion
bypasses rather than inspect only one marked body. W3 forbids either a claim or
an executed body check from becoming unchecked authority; the existing U view,
concrete ordinary-call graph, SCC condensation, and function-local derivation
DAG provide the smallest finite implementation ground. One checked runtime
body and an outgoing-only closure preserve FN-1 locality and ordinary callers.
The `deny_claims` spelling, declaration-prefix placement, and exact diagnostic
tie-break remain minimality-selected rather than experimentally derived, so
CLM-3 is existence-only.

Grammar arithmetic from v0.28 is exact: 73 productions remain 73; the optional
marker adds one decision, 90 to 91; the fixed spelling inventory adds only
`deny_claims`, 89 to 90; terminal predicates rise from 97 to 98. The rule
inventory is 133. CLM-3 adds one existence-only row, so the candidate totals
are **83 derived · 50 existence-only · 0 underived**.

The preimplementation verifier control was built from committed v0.28 bytes,
not from a candidate rebuild that automatically embeds the edited stable spec.
Archive versus archive exits zero with `grammar-preserving candidate verified
by the active compiler: 73 productions, 90 decisions, 97 terminal predicates`.
Archive versus candidate exits one with only `candidate changes the lexer or
source grammar of the baseline but does not match the compiler's embedded
frontend contract`. This is a preimplementation verifier limitation, not a
green frontend result. The candidate inventories above are a static one-token,
one-optional-node audit; frontend and generated-table agreement became green
only with the installed active identity.

The accepted byte set widens through the optional marked declaration and
reserves `deny_claims` away from IDENT. Relative to the same function without
the marker, the new rule only narrows acceptance: every direct or imported
claim is forbidden, and every protected leaf, ordinary required call, and
program-start requirement in the demanded outgoing closure must additionally
discharge in U. The closure includes complete concrete generic instances and
whole recursive components but never incoming unrelated callers. Unmarked
source not using the new spelling keeps v0.28 acceptance, diagnostics, runtime
checks, claims, effects, cleanup, lowering, ABI, and facts-on/off behavior.

The proposed protected matrix is purely additive: nine runnable cases take the
corpus from 437 to 446 while leaving 30 annotations unchanged and projecting
133/133 rule coverage. Two positives and seven negatives freeze direct and
imported claims, concrete generics, upward non-propagation, mutual recursion,
marked program start, strict OP-4, strict FN-8, transitive strict failure, and
the real value-branch repair shape. The authentic wfgrep candidate adds only
the marker to `report_failure`; its body, callers, output, error, cleanup,
status, runtime-check, and facts-off oracles remain activation obligations.
These corpus and consumer bytes are protected non-authoritative candidates and
do not select the rule semantics recorded above.

## v0.30 carry-forward — structured representation profile (activated 2026-08-17)

Specification binding: immutable `spec/kernel-spec-v0.30.md`, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`.
The outgoing v0.29 authority is immutable at SHA-256
`0b7aa8ccee958ba85613c51535165dcbf7ac12db556b2210d2f1aac0d39e6cc3`.

v0.30 changes no language semantics, numbered rule, grammar production,
operation-table row, or derivation status. It moves the same authority into the
selected structured Markdown profile: sentence-per-line normative prose,
`wf-` fences, ENT-3 sub-anchors, no embedded version history, and repaired
stale self-references and plan vocabulary. The activation audit found 124 rule
bodies with byte changes, 126 operation-table rows byte-identical, and zero
unexplained semantic delta. Every existing ledger row therefore carries
forward unchanged: **83 derived · 50 existence-only · 0 underived** across 133
rules.

## v0.31 amendment — affine replacement and proof extensions (activated 2026-08-18)

Specification binding: immutable `spec/kernel-spec-v0.31.md`, SHA-256
`ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c`.
The outgoing v0.30 bytes are the binding above.

v0.31 adds derived rule SET-2 and modifies exactly these 29 existing rules:
CLM-2, CLM-3, CONST-1, CONST-2, DIAG-2, DIAG-3, EFF-2, ENT-1, ENT-3,
ENT-4, ENT-5, ENT-6, FN-1, FN-8, FORM-2, GRAM-4, OP-1, OP-2, OP-9,
OWN-1, OWN-5, OWN-6, OWN-9, OWN-12, OWN-14, STOR-1, STOR-3, TYPE-2, and
TYPE-5. SET-2's row in the main ledger records the ownership-hole proof and
rejected alternatives. The companion deltas extend the same derived
mechanisms rather than introduce new authority: constant-operand integer
obligations use ENT-6; signed Boolean decomposition stays in L0; reborrow
composition preserves resolved-place exclusivity and declaration provenance;
one-operation const arithmetic remains compiler-checked; affine buffer
elements move only through the SET-2 exchange. No prior row is reclassified.
The totals become **84 derived · 50 existence-only · 0 underived** across 134
rules.

## v0.32 amendment — trap endpoints and traversal (activated 2026-08-18)

Specification binding: immutable `spec/kernel-spec-v0.32.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.
The immutable outgoing v0.31 bytes are the binding above.

v0.32 adds existence-only rule SYS-14 and modifies exactly these 35 existing
rules: CLM-2, CLM-3, DIAG-1, DIAG-2, DIAG-3, EFF-2, ENT-1, ENT-2, ENT-3,
ENT-6, EX-1, FN-1, FN-8, FN-9, FORM-5, GIVE-1, GRAM-2, GRAM-4, OP-1,
OP-2, OP-4, OP-5, OWN-6, PATH-2, PROG-3, PRV-2, PRV-3, QUAL-2, SYS-2,
SYS-4, SYS-5, SYS-6, SYS-8, SYS-10, and SYS-13. The check-dissolution
delta moves the surviving contract final into FN-8/FN-9 and leaves claim as
the ordinary body assertion; the division family extends the same finite
ENT-6 proof authority; declaration-site borrow-result provenance closes an
OWN-6 ambiguity; SYS-14 supplies the measured traversal gap with the
existence-only form debt recorded in its main row. No previous status changes.
The totals become **84 derived · 51 existence-only · 0 underived** across 135
rules.

## v0.33 amendment — claim-only runtime trap surface (activated 2026-08-20)

Specification binding at v0.33 activation: the then-active
`spec/kernel-spec.md`, headed v0.33, had SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.
The exact owner-approved candidate was SHA-256
`024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2`;
activation changed only its declared status line. Its outgoing v0.32 authority
is the immutable archive at the binding above.

v0.33 adds and removes no numbered rule. It modifies exactly 64 rules,
grouped below by one derivation move; the groups are disjoint and their union
is the complete archive-to-candidate changed-rule set:

| Move | Exact changed rules | Continuing derivation |
|---|---|---|
| Claim-only writer runtime boundary | SCOPE-2, SCOPE-4, ERR-4, TRAP-1, DIAG-3, CLM-1 | W3 forbids a writer-selectable unreviewed trust or abort channel; R4 keeps one explicit named fail-stop boundary when a proof is unavailable. Removing operation-specific traps narrows writer power without weakening a reachable safety obligation. |
| Canonical surface and mandatory result names | FORM-2, FORM-3, FORM-5, GRAM-1, GRAM-2, GRAM-4, GRAM-5, GRAM-10, TYPE-5, TYPE-6, OP-1, OP-5 | FORM-1/R3 select one result-binding and contract spelling. A result name is symbolic proof identity, not storage; retired `trap` remains unwritable. The grammar keeps 74 productions by replacing six obsolete productions with six owners of the new structure. |
| Unified erased static contracts | FN-1, FN-2, FN-3, FN-4, FN-8, FN-9, EFF-2, ERR-3, GIVE-1, GATE-1 | FN-1 locality and W3 force caller proof rather than a hidden callee check. `define` is alpha expansion only; plural requirements share one pre-transfer state, plural relations are independent and publish per SCC atomically. Named whole results and routed payloads preserve the existing finite relation calculus. |
| Sole command entry and global system domain | FN-7, PROG-3, SYS-1, SYS-3, QUAL-2, QUAL-3 | PROG-1's closed world needs one entry, not a declaration with both ordinary-call and process-start roles. Removing main contracts deletes the only external requirement exception. Global reservation removes a prospective kind-visibility fork; explicit command capabilities preserve least authority. |
| Exact integer domains | OP-2, OP-7, OP-8, CONST-1, CLM-2 | T2 forbids partial LLVM operations without proof. Total `.defined` predicates expose exactly each operation's mathematical domain to branches, requirements, and claims; checked/wrap/saturating forms stay ordinary total values. The exact predicate, not a second component language, is the sole goal identity. |
| One finite obligation and provenance authority | ENT-1, ENT-2, ENT-3, ENT-4, ENT-5, ENT-6, DIAG-1, DIAG-2, CLM-3, PRV-1, PRV-2, PRV-3, OP-4 | Existing L0 plus exact opaque goals already supplies the smallest deterministic proof engine. IntegerDomain, AllocationFit, and SystemRange are families in that same occurrence/root/view machinery; component normalizations are alternate derivations, never trusted facts or independent obligations. Contradictory requirements are safe dead code because every reachable caller must still prove every clause; checked metadata and unreachable lowering prevent an ex-falso body from reaching IR. |
| Static allocation fit | OP-9, STOR-6, OWN-1 | T2 forbids under-allocation. A target-independent conservative layout ceiling gives source a writable total goal; target qualification separately proves actual layout lies below that ceiling. The pair authorizes one non-overflowing multiplication, with no language trap or target-dependent source truth. |
| Static half-open system ranges | SYS-2, SYS-6, SYS-8, SYS-9, SYS-10, SYS-11, SYS-12, SYS-14 | Half-open `start,end` converts the former three-term offset/count condition into two existing L0 relations. Absolute `next` payloads preserve monotone cursor facts without unchecked addition, while expected host/path/content failure remains typed. Linux enumeration has a facility but no approved mapping, so qualification fails rather than inventing one. |
| Exhaustiveness follow-through | EX-1 | Removing the retired statement and operation spellings updates the independently generated legal-node and table-row census without changing its existence/form chain. |

The contradictory-requirement disposition changes no rule status. A
contradiction may make a function uncallable, which is safe in this closed
ordinary-call world; it is not promoted to a global theorem or published
postcondition. The ABI-shaped `unreachable` body is a lowering consequence of
that checker proof, not a new source construct or trusted escape.

No prior form debt is erased by replacing its mechanism. The new `contract`,
`define`, `when`, mandatory result-binding, `.defined`, and half-open endpoint
spellings remain under the same FORM-1/R3 existence-only debt carried by their
owning rows; derived semantic safety relations remain derived. Totals therefore
stay **84 derived · 51 existence-only · 0 underived** across 135 rules.

Installed inventory: the native frontend verifier records 74 grammar
productions, 93 predictive decisions, and 105 terminal predicates. Six old
productions are replaced by six new ones; the unique fixed lowercase atom set
removes `check` and standalone `trap`, adds `command`, `define`, and `when`, for
net +1; writer operation spellings add thirteen and remove four, while the five
bare exact infix spellings remain. The protected corpus contains 499 source
cases in exact manifest bijection, covers 135/135 rules, and the canonical
adapter reports `Pass=498 Skip=1 Fail=0`. The owner-approved candidate was
SHA-256 `024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2`;
the installed v0.33 active specification was
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`,
and the activation chain then ended at v0.33.

## v0.34 amendment — residual-only local claim authority (active 2026-08-22)

Specification binding: the active v0.34 bytes at `spec/kernel-spec.md` have
SHA-256 `cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`
and carry v0.33's exact predecessor identity. The outgoing v0.33 authority is
preserved as the immutable `spec/kernel-spec-v0.33.md` archive at SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.

v0.34 adds and removes no numbered rule. It modifies exactly these fifteen
existing rules: CLM-1, CLM-2, CLM-3, DIAG-1, DIAG-2, ENT-1, ENT-2, ENT-3,
ENT-4, ENT-5, ENT-6, EX-1, OP-4, PRV-3, and SCOPE-2. The groups below are
disjoint and their union is that complete archive-to-active rule set:

| Move | Exact changed rules | Continuing derivation |
|---|---|---|
| Residual-only claim contract | SCOPE-2, CLM-1, CLM-2 | W3 permits no writer-stated fact without machine proof or one executed reviewed boundary, while R4 makes a provably false or avoidable runtime failure a source error. A five-field proof record, fact-free canonical formation, contradiction-before-lifecycle-signs, exact/component unknownness, reconstruction, and fixed-set component/occurrence necessity make the surviving check a genuine checker residual rather than an assertion, oracle, deliberate abort, redundant check, or unused theorem. Runtime retention is unchanged. |
| Deterministic evidence and diagnostics | DIAG-1, DIAG-2 | W3 review requires stable source identity and exact proof ancestry, and R4 requires the earliest owning rejection rather than a later runtime symptom. Source-schema-before-instance ordering, component ordinals, failure-atomic scratch, stable terminal witnesses, and the ban on scratch identities are the finite evidence form of that requirement. |
| Finite proof, authority, and consumer judgment | ENT-1, ENT-2, ENT-3, ENT-4, ENT-5, ENT-6, OP-4, PRV-3 | The existing deterministic L0/opaque-goal state supplies the unique D/S/F and contribution vocabulary. W3 forbids a component proved, refuted, externally assertion-authorized, or unsupported by an authentic admission root from entering S3; fresh Full-minus runs over one fixed Eligible set prove exactly that each retained component and occurrence is checker-relative load-bearing without claiming mathematical minimality. The same finite support and normal-control graph carries the independent claim-authority observation described below. |
| Strict and canonical-example follow-through | CLM-3, EX-1 | A strict closure may inspect only claims that survived the ordinary admission judgment, and the canonical complete example must not use a claim as an output oracle. Reusing the retained-claim set preserves CLM-3's existing SCC boundary, and ordinary value control preserves EX-1's behavior without manufacturing a theorem. |

The claim-locality correction follows directly from FN-1's modular callable
boundary and W3's no-hidden-trust premise. An ordinary caller is allowed to
consult verified requirement and normal-result summaries and never a callee
body. Therefore a caller claim about a user-call result would be an unwritten
postcondition: changing the callee body without changing its boundary could
invalidate distant prose while the runtime trap merely delays the defect. The
trap still protects the immediate partial operation, but it cannot supply the
missing proof authority. A user result property crosses the boundary only as an
FN-9-verified S12 relation; a system result property crosses only as a
specification-fixed fact or typed outcome.

The strict `BoundaryResult` transfer is the smallest closed rule that makes
that boundary non-bypassable. Every user and system result component is seeded
regardless of PRV externality or an existing `ensures`; all explicit value,
holder, storage, and control paths preserve or join the seed. Copy,
conversion, `imin`, `+wrap 0`, aggregate wrapping, payload extraction, selecting
constants on boundary-controlled arms, and storage round trips are therefore
not ad hoc forbidden spellings but instances of one exhaustive transfer. An
`ensures` proves its exact relation and no more, so treating it as whole-value
declassification would recreate the stronger-hidden-postcondition hole.
PRV-1 remains an explicit external-dataflow judgment; claim authority is an
independent proof-ownership judgment and its control dependence changes no PRV
class.

Component sensitivity and strong replacement prevent the conservative rule
from inventing a different prohibition. A local aggregate sibling remains
local when an exact other field is boundary-derived. An unconditional exact
whole-value or exact-field write replaces the current value and may clear an
old marker, while a boundary-controlled write, shared element summary,
possible-overlap write, or control-flow join cannot. This is the ordinary
reaching-value distinction between replacement and join, not writer-directed
declassification. A returned aggregate is different: the call boundary seeds
all its components before any projection.

Ordinary and command-entry parameters are not call-result events, so this
amendment does not seed them; PRV and claim truth/residuality continue to judge
their uses. Likewise, a callee write through `&uniq` storage is not silently
added to this return-result correction. That neighboring accepted-set question
requires its own amendment; explicit writes of already boundary-derived values
continue to propagate normally. These exclusions prevent a return-authority
repair from becoming an unreviewed general information-flow policy.

ENT-1 applies the same formation and locality admission to every nongeneric
body, every generic source schema including an uninstantiated one, and every
inhabited concrete replay. The authority tree uses resolved source components,
not the restricted generic L0 term vocabulary. DIAG-1's source occurrence,
least component, earliest boundary call, and first same-witness support-carrier
order makes the result invariant under dense IDs, unrelated instances, and
traversal order.
The authority observation is independent of S3 and is computed once before U,
B, Eligible, or any counterfactual mask.

This amendment supersedes the original ENT-6 fallback sentence for non-local
values: rebinding closes a claim site only when the residual is a true theorem
over current-function-local authority. Rebinding,
converting, or otherwise renaming a user-call or system-call result never makes
it local. A cross-function fact instead comes from FN-9/S12, a
specification-fixed system fact, or ordinary typed control; when the current
contract language cannot express the needed result relation, that program is
not repaired by a caller claim.

No existing derivation status changes. CLM-1 retains its independent historical
writer-form debt; locality adds no spelling, fact source, operation, effect,
runtime check, trap family, grammar production, or unsafe authority. Totals
remain **84 derived · 51 existence-only · 0 underived** across 135 rules.

## v0.35 candidate amendment — permitted execution overlap (2026-08-23)

Candidate binding: `spec/kernel-spec.md`, headed v0.35 CANDIDATE, currently at
SHA-256 `73d647c8945ad3d51eea3ed030714b433d6171e0d36b0869dd91366238cbd8f5`.
Its outgoing authority is active v0.34 at the binding above, and the activation
chain still ends at v0.34. These candidate bytes are not language authority
until exact-byte owner approval and the separate activation step. Candidate
stage: the PAR-1 row above exists ahead of activation so the native
`whitefoot-spec` gate covers all 136 rule IDs.

v0.35 adds exactly one rule, existence-only PAR-1 (row above), and modifies no
existing rule. The section-13 heading widens from `Capabilities` to
`Capabilities and execution overlap` so the new law is not filed under the
capability stub; CAP-1's own bytes are unchanged, and PAR-1 binds neither of its
predicates. Grammar productions +0 (74 remain), tokens +0, spellings +0,
operations +0, exception clauses +0. No existing rule changes derivation
status. The totals become **84 derived · 52 existence-only · 0 underived**
across 136 rules.

The candidate's PAR-1 still carries a claim-closure condition — no function
reachable from either callee contains a `claim_stmt` — which the compiler no
longer applies. The owner's chartering direction of 2026-08-23 withdrew that
gate and `f6c55a9d` deleted it from both judgments; the matching specification
edit is the [PAR-1] v2 recipe in `docs/ongoing/0078-loop-permission.md`, which
the owner applies at activation and which no branch commit applies here, so the
in-tree bytes and the landed-archive gate stay stable.

Two things retire the condition, and the recipe records both. A claim is an
always-true reviewed lemma and a false executed claim is the sole
writer-reachable runtime contract violation [SCOPE-4], so the condition bought
a stable record for erroneous executions by refusing to overlap correct ones;
the guarantee becomes conditional on contract compliance exactly as [SCOPE-3]'s
is, and an erroneous execution still gets one complete record whose claim
identity is the single thing a schedule may select. And the condition was never
new law: CLM-3 already forms `MayClaims(K)` over a component and its strictly
outgoing callees, and v0.34 retains every accepted claim, so the condition was
`MayClaims` of the callee's component being empty — a second claim-reachability
analysis that would have had to agree with CLM-3 forever. The recipe deletes
the clause rather than restating it in CLM-3's vocabulary.

Evidence ground: `research/investigations/proof-derived-parallelism/DESIGN.md`
(the batch-0074 design contract and its two owner rulings of 2026-08-21 — claims
are always-true lemmas, so a trap under overlap is an audit failure rather than
a semantics question; permission comes from proofs on ordinary code while
actualization and every resource stay outside the language), the owner's
chartering direction of 2026-08-23 withdrawing the claim-free gate, PAL.md
beside it, and the batch's measured compute lane. The corpus delta is zero cases because
the rule changes no accepted program and no verdict; PAR-1 nevertheless needs
its own coverage annotation in the protected manifest before the repository
gate's coverage denominator is satisfied. Both annotations landed with the
v0.35 activation as the owner-approved protected addition their records
prepared.

## v0.37 amendment: unified state and completion I/O (activated 2026-08-27)

Specification binding: `spec/kernel-spec.md`, headed v0.37 ACTIVE, SHA-256
`ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619`,
superseding the exact v0.36 digest named above, whose outgoing bytes are
archived as `spec/kernel-spec-v0.36.md`. Its complete first-principles chain
is `research/investigations/io-model/FIRST-PRINCIPLES.md`; the selected concrete
surface is `research/investigations/io-model/DESIGN.md`; the discarded branch
evidence and clean-core measurements are `IMPLEMENTATION-AUDIT.md` and
`RESULTS.md` beside it. These bytes are the activation-chain tail; merging them
into `main` still requires the normal exact-revision owner approval recorded in
`governance/APPROVALS.md`.

No numbered rule is added or removed and no derivation status changes. The
new `effect_path` grammar production raises the grammar count from 74 to 75.
The amendment changes EFF-1, EFF-2, EFF-3, EFF-5, FN-1, FN-3, FN-7, CAP-1,
PAR-1, PAR-2, STOR-3, and the existing system rules. The following chains
supersede both the older active-row text and the earlier capability candidate.

**One observable-state model.** If the runtime and target boundary obey the
same ownership contract as Whitefoot code, there is no semantic reason to
classify a file write separately from a memory write. The public effect is a
transition of the `Output` value supplied by the caller. Runtime records,
kernel queues, registers, and target-private buffers are implementation state,
just as stack temporaries inside an ordinary function are implementation
state. W3 requires every caller-observable transition to appear at the
boundary, but it does not require the signature to enumerate those internal
representations. EFF-1 therefore has one `reads`/`writes` system over state
paths and no `world`, `external`, capability, root, family, or fragment domain.

**Paths name state; regions name loan duration.** The earlier region-granular
form was never evidence-selected. It fails when two parameters share one
lifetime, and it cannot name an owned state parameter with no borrow lifetime.
The selected subject is a formal parameter followed by zero or more static
struct fields. A field path may preserve facts about sibling fields, but it
never narrows a whole-object loan. Dynamic elements and ranges remain deferred
until their overlap proof is designed. FN-3 compares paths by parameter and
field ordinals, while region alpha-renaming remains confined to modes, types,
and arena allocation.

**Existing ownership supplies every permission.** A shared borrow promises
that the represented state remains stable for the loan. A clock, Output,
directory enumeration cursor, or changing directory namespace observation
therefore cannot mutate behind `&`; the operation takes `&uniq` or consumes
`own` and declares `writes(parameter)`. Immutable argument and host-string
snapshots, plus a position-explicit `ReadFile` observation which advances no
resource state, may use `&` plus `reads(parameter)`. This is the same rule used
for ordinary Whitefoot state. No system-type exception or interior-mutation
label is added.

**Calls use existing identity.** EFF-2 substitutes a callee's formal path
through the actual place. Borrows, reborrows, slices, fields, and moves retain
the identities already needed by OWN-5, OWN-6, and move checking. A fresh local
owner frames out. In the canonical factory shape,
`reserve_file<'r>(factory: &uniq 'r factory)` contributes `writes(factory)`
directly; an open with `permit: move permit` writes only that local permit; and later
actions on the fresh returned file remain local to that function. No
child-to-factory ancestry, origin graph, or new move feature is required.
Framing those actions out of the enclosing signature does not prove them dead:
the checked inner call retains its nonempty instantiated effect, and deletion
still requires the ordinary closed-state, escape, result, release, and
no-observer proof. The direct factory or permit write keeps the enclosing call
non-pure. No system-only observability tag is added.

**Resource creation cannot be ambient.** Every operation that creates a
system resource or consumes quota receives the changed factory, namespace,
allocator, or permit as an ordinary parameter and writes it directly. This
prevents the fresh-local framing rule from hiding observable creation. It also
keeps the API honest: the finite thing that changes is present in the source
signature instead of a shared pool hidden behind interior mutation.

**Ordinary loans decide overlap.** PAR-1 and PAR-2 project state paths to
actual places and apply OWN-5/OWN-7. Two writes through distinct Output owners
may overlap. Two calls through one `&uniq Output` cannot overlap, so per-output
order needs no Ordered relation or edge graph. The same rule applies to
ReadFile, DirectoryRead, and DirectorySource. Hard links, descriptor
duplication, redirection, and another process may make distinct values contact
one host object, but those environment aliases do not change Whitefoot place
identity. This supersedes v0.36's blanket order, the measured global-world
experiment, and the unimplemented Free/Ordered/Exclusive candidate.

**Completion remains target metadata.** A finite operation records
`result-ready`, one `loan-released(path)` fact for every retained borrow, and
`terminal`. The first system slice publishes all applicable facts together,
while later target contracts may separate them.
`wait-capacity` transfers no loan to the target and lets another ready frame
run. Native completion, readiness, polling, interrupts, and bounded
target-only helpers remain qualification choices. A helper never executes a
writer function. The target adapter may access one retained argument only
before its `loan-released(path)` fact; this is the trusted ABI implementation of the ordinary
borrow contract, not a new authority system.

**Trap consequence.** A retained claim is a reviewed true theorem; a false
execution is defective and cannot occur in a correct program. T3 therefore
forbids a claim-free eligibility gate, trap-latch submission read, epoch,
metadata field, queue transition, wake, copy, or ordering cost on the correct
path. A defective execution may leave whichever already permitted state
transitions became visible before process termination. Memory safety remains
unconditional through ordinary operation ownership and qualified process
teardown.

**Current evidence and open form debt.** The bounded core measures about 35 ns
for claim, publish, drain, and consume on the recorded M4 host. Its matched
park/resume median is 1.708 us against 1.667 us for a raw condition variable.
Wrapping a cached 4 KiB pread with zero-helper completion adds about 53 to 64
ns, selecting a direct/inline depth-one specialization; helper handoff does
not cross over for cached operations and remains only a target fallback for
real waits. A real Linux io_uring positioned-I/O probe passes. The exact
`x86_64-pc-windows-msvc` row now executes the IOCP core and adapter, bounded
multi-waiter wake and capacity boundaries, namespace facilities, the complete
runtime/driver link, one real `.wf` IOCP program, the wider-code-unit
HostString raw-byte route, and the emitted sequential `--par` world in native
run [33305475906](https://github.com/mbbill/Whitefoot/actions/runs/33305475906)
on `f04e15c9`. Windows compute-pool and performance evidence remain open.
Selective stackless continuation across arbitrary user call chains remains
the principal implementation and measurement debt. These open items affect
realization and performance evidence, not the derivation of one
completion-only source model. API expansion to networking, timers, and more
file operations must keep the same rule: evolving state is an ordinary unique
or owned parameter, and immutable state alone is shared.

## v0.38 amendment — the staged loop permission (activated 2026-08-27)

**One added rule, PAR-3, and one amended sentence in SYS-2.** The counted permission
PAR-2 divides an index range and recombines one accumulator; its unit is a
subrange, which is why it needs a trip count, a compiler-owned binder, and an
enumerated exactly-associative operation set, and why it admits only a
`for_stmt`. A loop that performs I/O needs a different overlap and none of that
apparatus: what makes it fast is keeping several target operations outstanding,
which is a property of the *iteration*, not of the index. Amending PAR-2 to
carry both would have put two permissions that share no condition under one
rule id; PAR-3 states the second one on its own, and the two rules are
independent — a loop may hold either, both, or neither.

The SYS-2 amendment adds no rule and no record. It fills one entry in the
milestone structure that rule already carries: the loan on the name an open
borrows is released before target transfer, because forming the request copies
the admitted code-unit range into compiler-owned storage and that copy is the
operation's last access to the caller's buffer. Its derivation is the
milestone's own — a borrow lives until the contract releases it, and the
contract knows the release when it stops reading the buffer — and both shipped
adapters already publish it there. PAR-3's condition on retained borrows does
not consume it: the judgment reads every retained borrow to `terminal`, which
is conservative against this contract and refuses only permission.

**What it is derived from.** Nothing in PAR-3 is new authority. The cut is a
dominator and post-dominator query on the body's own control-flow graph. Every
footprint and every loan is PAR-1's, including the loan column v0.36 added.
The four dispositions are OWN-5 exclusivity and OWN-7 place overlap read over
the two segments the cut produces. The exit condition is FN-1, GIVE-1, and
ERR-3's own edge classification. The replication condition consumes SYS-8's
buffer disposition and EFF-2's call-boundary projection and nothing else — and
it consumes them in the direction each states. SYS-8 fixes for the
range-bearing operations only which bytes of a buffer *may* have changed, which
is an upper bound and establishes no written byte; only its copy pair states a
change exactly. The rule says so in its own words, so its coverage condition
cannot read an upper bound as the lower bound it needs.

**The two schedule restrictions are load-bearing.** Prologues execute in index
order and never overlap one another; that sentence, and not an exemption, is
what admits SYS-10's short unique `FileFactory` loan, because at every program
point exactly one such loan is live. The remainder's accesses to storage
rooted outside the body that the body writes — its reads as well as its writes
— occur in iteration order; that sentence, and not an algebraic property, is
what admits an ordinary `set sum = sum +wrap e` with no identity element, no
combination tree, and no associativity — which is strictly more general than
PAR-2's accumulator apparatus and reaches folds it never can. The read half is
stated because the disposition it licenses assumes it: a place only the
remainder reaches, but the body writes, is granted on the ground that the
remainder serializes it, and a schedule that ordered only the writes would let
one iteration read what an earlier one has not yet written.

**The resource clause.** SYS-10 already states that reserving a permit promises
no native descriptor, handle-table entry, kernel memory, or host quota, and
that host exhaustion is the ordinary typed `ResourceExhausted` result. PAR-1 and
PAR-2 excuse only the execution resources an implementation spends on
overlapping. Descriptors the program's own opens consume are not those, so a
deeper schedule could turn an `Ok` into an `Err` and move published bytes.
PAR-3 therefore requires an overlap holding more such resources at once than
the source-order execution holds to complete the earlier iterations and perform
the operation again at the source-order footprint before delivering any
outcome. The cost is paid only on the exhausted path, so T3 holds.

**The cut is a program point, not a statement.** The prologue ends at the
argument evaluation and submission; the *outcome* of that submission is joined
only by the remainder. An edge the submitting statement takes on that outcome —
a `let_stmt` selecting `propagate_let_rhs` whose right-hand side is the cut
submission — is therefore an edge of the remainder, and the exit condition
refuses it. Without that sentence the same early exit would be admitted when
written as a `propagate` and refused when written as a `match` arm returning on
`Err`, which is one capability decided by source shape, and it would let an
iteration decide to leave after later prologues had already submitted
operations the source-order execution never performs.

**Open derivation debt.** The replication condition is stated in full and is
discharged in this version only for storage the body itself constructs; the
derived byte-range analysis that discharges it for a hoisted place is designed
and not built, and it will be the first place in this implementation where a
byte-correctness argument consumes the entailment fact state. A body whose
first I/O error returns is refused outright by the exit condition, and closing
that needs a closed-state proof extended to a call whose result is a fresh
resource. Neither affects the derivation of the permission; both are recorded
so no reader mistakes the stated rule for the implemented fragment.

## v0.39 amendment — claim authority selects definitions, not positions (activated 2026-08-28)

**No added rule. One narrowed clause in CLM-1's claim-authority state.** Claim
authority answers one question: can a reviewer validate this claim by reading
this function, or does its truth import an unstated property of something a
callable boundary returned? v0.38 answered that question partly by position: a
selector's witness joined every binder, delivered value, and storage write
whose reaching definition was selected by its edge, and "post-join state" was
named among them. The differential-fuzz campaign of
`docs/done/0097-differential-fuzz.md` showed what that answer costs. All 63 of
its rejections over 2004 generated programs were one shape: a `match` on a
system-call result whose `Err` arm returned, followed by definitions built
entirely from literals, whose claim was refused although nothing the system
returned reaches the claim's truth.

**What the narrowing derives from.** W3 forbids a writer-stated fact without
machine proof or one executed reviewed boundary, and CLM-1's locality condition
is what keeps that boundary reviewable: the reviewer must be able to establish
the predicate from what the function itself contains. A value whose every
operand is a literal, a named const, an ordinary parameter, or another such
value is exactly that — the reviewer reads it and is done — and the edge that
carried control to it changes none of its operands. So the position clause
protected nothing the locality condition needs, while refusing claims the
condition admits. What the reviewer genuinely cannot establish is a value the
selector *chose*: a payload binder the arm introduced, a value `value_if` or
`value_match` delivered, or a component whose reaching definition differs
between the incoming edges of a reconvergence, loop head, or loop exit. Those
remain non-local, and the amended rule states them as the complete list.

**Why the join is the selection.** A merge of two different definition
occurrences is where a boundary decision becomes a value: the writer cannot say
which definition arrives without knowing the selector's outcome. A merge whose
every incoming edge carries one definition of that component decides nothing
about it, so it adds no authority. This is a stronger reading of CLM-1's own
"whose reaching definition is selected by that edge" than the clause it
replaces, not a weaker one: v0.38 read *reaching* as *reached*, and the
amendment reads it as *chosen among*.

**Status unchanged.** CLM-1 keeps its existence-only status and its historical
derivation row; the component tree, the unconditional call-result seed, the
witness identity and its tie-break, the protected families, and the constrained
subjects are all untouched, as is PRV-1 provenance, which never included
control dependence. No other rule's derivation moves.

## v0.40 amendment — source-carried proof and deterministic proof flow (activated 2026-09-03)

**Two added rules, nine retired rules, and no compiler search.** INV-1 lets
source state one affine relation at a loop header or an ordinary program point
under the single spelling `invariant`. At a header placement the compiler
checks the base case and every arbitrary reachable backedge with a fixed finite
rule and exports only the separately justified exact-exhaustion consequence; at
a body placement it discharges the one ordinary program-point obligation. Either
placement erases before lowering. PRF-1 lets source name an affine target and
its ordered premises in that same declaration's `use` block. The compiler proves
every premise independently in the same pre-proof context and checks one fixed
coefficient-one sum in source order. The source selects propositions, premises,
and intermediate steps; the compiler does not guess an invariant, premise set,
coefficient, case, path, or lemma.

v0.40 retires SCOPE-4, DIAG-3, TRAP-1, CLM-1 through CLM-3, and PRV-1
through PRV-3. Their runtime assertion, assertion-authority, strict partition,
provenance mask, report, and latch machinery supplied no safety that a
successful pre-lowering proof does not supply, while it allowed accepted source
to stop at a writer-selected runtime site. A supported partial operation now has
only two dispositions: its exact domain goal is proved before lowering, or the
source is rejected. There is no inserted runtime fallback.

**Proof-carrying means Whitefoot source.** Contracts, invariants, and
`invariant`/`use` statements are the writer-controlled proof input. The syntax
tree, fact context, and checked program are ordinary compiler data in one
whole-unit compilation. A disagreement among them is a compiler bug to fix and
test, not a reason to serialize compiler-generated data or run a second
verification pass.
Proof authoring may use AI or offline search; compiler acceptance uses only the
committed source and specification-fixed terminating checkers, never SMT,
solver seeds, heuristics, or timeouts.

**One proof context, several finite domains.** Control flow, verified
requirements and postconditions, proved loop invariants, and successful source
proof steps populate one current `ProofContext`. A consumer submits its exact
goal through one deterministic entry; the entry may use the fixed equality,
difference-bound, affine, ownership/effect, layout/address, target, or parallel
domain that owns that proposition. The compiler never deletes one fact source
and reruns the body, builds an assertion-free view, or treats an explanation of
the first decision as a second decision. Multiple small checkers remain
separate algorithms behind one fact-and-goal boundary rather than becoming one
general solver.

**One retained bounds result authorizes the narrow map permission.** PAR-2's
single-binder affine map consumes the proved OP-4 outcome and exact value image
produced by that same source semantic check; it does not submit the subscript
for a second proof or recover its parser shape. FN-1 supplies distinct binder
values, and the fixed nonzero coefficient, direct-`set`, direct-own-root,
one-map-per-root, write-only, and no-overlapping-loan restrictions refine the
ordinary whole-root footprint to `[a*i+b, a*i+b+1)`. No premise, coefficient,
range, or injectivity search is added. Multiple maps of one root, multi-term or
dynamic ranges, borrowed roots, read/write transforms, and multiple
accumulators remain deferred, while the selected target's layout and
address-domain proof remains the separate STOR-6 obligation before emission.

The same version completes two transfers that the existing ENT-3 and ENT-5
rules already require. They do not add a template, fixed-point round, or runtime
check. A direct SET-1 write whose right-hand side is already a fixed S5 copy
source publishes the same equality that the corresponding `let` publishes:

```whitefoot
set x = 41_i32;       // publish x = 41
set x = y;            // publish x = y
set x = cvt<u8, u32>(byte); // total conversion: publish x = byte
```

The target is killed before the new equality is published. Consequently an old
fact about `x` cannot cross the write and combine with the new value. Indexed
targets, narrowing conversions, computed right-hand sides outside the fixed S5
table, and every other unsupported shape publish no image. This is the finite
operation-image discipline already selected by ENT-3, not a request for the
checker to discover an algebraic relation.

**Closure precedes lexical forgetting.** Consider a scope that contains
`next = i + 1`, proves `i < 3`, and copies `next` into an outer `out`. Before
leaving the scope the fixed entailment closure can derive `out < 4`. Killing
`next` first would discard the only path that justifies that surviving
conclusion. ENT-5 therefore closes the already finite state first, then removes
the lexical roots, and retains only conclusions whose support no longer names a
killed root. Conclusions that still mention `next` disappear. Contradiction,
when present, survives forgetting because no assignment to a forgotten local
can make an inconsistent predecessor state reachable.

**Derivation status.** INV-1 and PRF-1 are existence-only because their first
writer spellings and deliberately small affine calculi remain to be measured on
broader programs; the prove-before-lowering requirement and absence of a writer
runtime assertion follow directly from T2, W3, and R4. ENT-3's S5 copy-image
row is derived from the same assignment semantics and target-kill law as its
existing `let` image; ENT-4 is unchanged because no checked-program proof
object or runtime effect is added; ENT-5's ordering is derived from its stated
requirement that every surviving fact be a logical consequence of the
predecessor state rather than merely a fact textually present before a scope
boundary. The positive conformance cases exercise both transfers. The existing
kill-on-write negative case remains a canary that the new image cannot
resurrect stale target facts.

**Runtime and parallel consequences.** Contracts, invariants, proof statements,
and both transfer rules exist only in the checker. They emit no instruction,
branch, check, lock, schedule edge, or task dependency. STOR-6 consumes the
proved source length ceiling together with the selected target's actual stride
and address domain before emission, so the former target-domain runtime guard is
removed rather than retained as a fallback. `par` consumes the same checked
facts together with ownership, effects, exact iteration-index relations,
selected-target layout/address facts, and finite queue/completion bounds. It has
no second proof language and proof checking cannot serialize PAR-1, PAR-2, or
PAR-3 code.

External resource availability alone remains outside v0.40:
heap exhaustion, stack exhaustion, operating-system quotas, and runtime-start
resources may still fail at the declared host boundary, but their final
source-language model is not selected here. This is a temporary scope cut, not
a constitutional or derivation change. It does not defer layout, address,
target qualification, target-domain proof, parallel independence, or bounded
queue/completion protocols.

## v0.41 amendment — integer comparison symbols and the call-site delimiter (activated 2026-09-03)

Specification binding: active `spec/kernel-spec.md`, headed v0.41, at
SHA-256 `899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`,
superseding v0.40
(`15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168`), whose
bytes are archived at `spec/kernel-spec-v0.40.md`. The candidate bytes hashed
to `55ee571e7f342471b16078da05fa8b3bfdab11fbe6819a925baad83687966c06` until
the status line flipped to `ACTIVE v0.41`; no other byte changed at
activation. The owner rulings of 2026-09-03 and the rejected alternatives are
recorded in `governance/spec-evolution/comparison-symbols-v041-candidate.md`
and `research/investigations/spelling-relief/SWEEP.md`; the merge-time record
is in `governance/APPROVALS.md`.

v0.41 adds and removes no rules. It modifies 21 existing rules at
38 verbatim-anchored sites, plus the
title, status, META-5 delta, and selection-ground header: DIAG-1, EFF-2, ENT-3, ENT-6, EX-1, FN-9, FORM-2, GRAM-1, GRAM-4, GRAM-5, GRAM-6, INV-1, OP-1, OP-2, OP-7, OP-8, OP-9, PRF-1, STOR-2, SYS-2, TYPE-4.
Grammar productions +1 (`compare_op`; total 83); compound punctuation tokens
+5 (`==`, `!=`, `<=`, `>=`, `::`; total 8); the token alphabet gains `!`
inside `!=` only; the fixed terminal inventory grows by five (total 98 fixed,
106 predicates); the six integer comparison rows are respelled
`==` `!=` `<` `<=` `>` `>=` and leave `DotlessOperationNames`. The FLOOR-5
spelling rule (T1–T4) is the ground: token count, rule-count delta, two-token
preservation, and the uniqueness argument; `::` is admitted as a delimiter of
the expression-position type-argument list on the same ground as the `for`
header's parentheses. No rule changes derivation status: every modified row
keeps its chain, because the operation identities, comparison origins,
contract-clause roots, invariant relations, and diagnostic attributions are
unchanged under the new spellings. Statistics unchanged: 80 derived · 48
existence-only · 0 underived.

## v0.42 amendment — [FORM-8] canonical region spelling (activated 2026-09-03)

Specification binding: active `spec/kernel-spec.md`, headed v0.42, at
SHA-256 `6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`,
superseding v0.41
(`899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`), whose
bytes are archived at `spec/kernel-spec-v0.41.md`. The candidate bytes hashed
to `8cf0b9142eaee1e48734fd29d572d27f2e6f9faf0a6a6518d011124786bc2e37` until
the status line flipped to `ACTIVE v0.42`; no other byte changed at
activation. The merge-time record is in `governance/APPROVALS.md`.

This amendment binds the one row v0.42 adds; every other row's status carries
over from v0.41, because v0.42 changes which regions a writer spells and no
rule's judgment.

| Rule | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| FORM-8 | A REGIONID is written exactly where the document does not otherwise fix the region denoted: a declaration relates two of its own positions or names an output-position region no parameter determines; a `borrow_expr` names a region other than the innermost enclosing `region_stmt`'s; a `region_stmt` name survives a reference inside it; a call's `::` application names the callee region parameters no parameter position determines | 🟡 existence-only | Existence derived: FORM-1/R3 require exactly one spelling per semantic construct, and a written region at a position the surrounding text already fixes is a second spelling of one semantics — the same defect FORM-1 removes everywhere else. The derived-fact class is TYPE-5's derived `let` binder mode and OWN-13's derived match-binder mode: a fact the checker determines is not written, and a fact it cannot determine is. The optional-name-resolving-to-the-innermost-enclosing-construct form is TYPE-6's unlabeled `break`, already META-2-clean. Form NOT derived: the exact position partition (input positions determine, output positions are written) is minimality-selected against the 2026-09-03 owner ruling that whether a region is written must be decidable from the declaration text alone, and no weak-writer trial has measured it. | Registered: the writer trial that would select this form against the alternatives — writing every region as today, eliding at every position and inferring the result region from the body, or a per-declaration opt-in — has not run. Corpus evidence only: over `tests/programs`, `tests/conformance/cases`, `tests/snapshot/cases`, and `tests/codegen/cases` the rule takes the corpus's written region tokens down by about nine tenths, and no declaration outside a small set of conformance cases relates two positions at all. |

## v0.43 amendment — loop-body regions and the [ENT-6] join repair (activated 2026-09-04)

Specification binding: active `spec/kernel-spec.md`, headed v0.43, at
SHA-256 `037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`,
superseding v0.42
(`6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`), whose
bytes are archived at `spec/kernel-spec-v0.42.md`. The candidate bytes hashed
to `1708dd2b64b93c88d1dfc23acd340c853b03ca240b9b86ac43fbc91e1c0b2081` until
the status line flipped to `ACTIVE v0.43`; no other byte changed at
activation. The merge-time record is in `governance/APPROVALS.md`.

This amendment binds the rows v0.43 touches. No row is added or retired and no
status moves, so the statistics line above carries over from v0.42.

| Rule | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| OWN-11 / OWN-3 / FORM-8 | Every `loop_stmt` and `for_stmt` body is itself a region block: it introduces one unnamed local region whose block is that body, a `borrow_expr` written directly in the body takes that region and is written bare, and a `region_stmt` that is the body's only statement is a hard error citing FORM-8 | 🟡 existence-only | Existence derived: OWN-11 already fixed the extent — a borrow formed in a loop body ends with the iteration — and a construct that has an extent has a region [OWN-3], so naming that region a region block adds no judgment and removes the ceremony a writer had to write to obtain it. The rejection follows from FORM-1 through the same one-spelling argument v0.42's FORM-8 rows use: a block whose block is the body denotes the region the body already introduces. Form NOT derived: the line between a redundant block and a narrower one is minimality-selected at "the block is the body's only statement", the coarsest place where OWN-6's statement-scope judgment still tells the two extents apart; a stricter line (any block ending where the iteration ends) makes shapes OWN-6 needs unwritable, and a looser one leaves two spellings of the body's own region. | Registered: no writer trial has measured whether writers reach for the bare form or keep opening a block out of habit. Corpus evidence only: across `tests/programs`, `tests/conformance/cases`, `tests/snapshot/cases`, and `tests/codegen/cases` exactly four blocks are the body's only statement, so the rejection is narrow and the accepted-program change is almost entirely the newly admitted bare borrow. |
| ENT-6 | At a control-flow join every input image is first normalized by folding each delta atom an earlier join minted back into the constant interval it stands for; where the normalized inputs share one nonconstant form the join is that form plus one fresh delta atom over the hull of their constant intervals | 🟢 derived | Derived from ENT-6's own stated guarantee that image formation is source-structural and independent of proof success order. The v0.42 rule was not associative: a delta atom counted as an ordinary nonconstant term at the next join, so two joins in sequence lost an image one join over the same branches kept, and a three-way demux was accepted as a flat `match` and refused as nested `if`/`else` with no semantic difference between them. Folding is the unique normalization that makes the composed join equal the flat join while keeping every image sound: a delta atom stands for exactly its interval, so replacing it by that interval is an over-approximation that adds no fact. | None. The repair only adds images, so no program v0.42 rejected for another reason becomes accepted, and no program v0.42 accepted is refused. |

## v0.44 amendment — the fact machinery (activated 2026-09-04)

Specification binding: active `spec/kernel-spec.md`, headed v0.44, at
SHA-256 `5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049`,
superseding v0.43
(`037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`), whose
bytes are archived at `spec/kernel-spec-v0.43.md`. The candidate bytes hashed
to `1240d9ff604276f96b954f0524c973c8ab7490ef63b91c7f7c6b8c2d57181b3b` until
the status line flipped to `ACTIVE v0.44`; no other byte changed at
activation. The merge-time record is in `governance/APPROVALS.md`.

This amendment binds the four rows v0.44 adds; it retires none and moves no
existing row's status. Two of the four are derived (MSR-3, CALL-6) and two are
existence-only (MSR-5, CALL-4), which is what the statistics line above
counts; the candidate-time preamble said one derived and three
existence-only, which disagreed with its own table, and the table is the
authority.

| Rule | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| MSR-5 | A `requires_clause` and an `ensures_clause` take a `clause_expr` whose operands are an `atom`, a `call`, or a `construct`, so a measure of a place is a clause operand on either side of the comparison; the measure formers are table data with one row, `len(P)`, admitted exactly where ENT-2 clause (b) admits a length term | 🟡 existence-only | Existence derived: FORM-1/R3 admit one spelling per semantic construct, and before this rule one fact — a relation between two measures — had two spellings, a `contract_define` per measure that was admitted and a direct clause that was a GRAM-5 parse rejection. Removing the parse rejection removes the second spelling and the definition-per-measure cost with it. The judgment is unchanged: OP-5's condition and FN-8's pure-total operand admission already accept every operand this production adds, so the rule adds spellings and no authority. Form NOT derived: that the widened operand grammar is a clause-only production rather than a widening of `atom` or of `expr` is minimality-selected — widening `atom` would admit a nested call in every argument position and contradict GRAM-9's one-operation-over-two-atoms shape — and no writer trial has measured the clause surface. | Registered: no writer trial has measured how often a contract states a relation between two measures, so the size of the win is corpus evidence only. Probe `q7` of the containers-and-resources design session is the one measured rejection. |
| MSR-3 | One denotation per operand position, keyed on the parameter's mode: an `own` operand read at a caller denotes that call's compiler-owned call datum, a shared-borrow operand the live term, the result binder the result; and a `&uniq` parameter's measure is inadmissible in a source-declared `ensures` | 🟢 derived | Derived from L11 as the constitution's no-guessing premise carries it: the checker's knowledge of a measure comes from the type, an established fact with live support, a compiler-owned measure datum, or a verified contract relation, and a relation about a value a callee received names that value. An `own` parameter is a value the operation received, so its post-state is not a thing a caller can name and the only sound denotation is the value at transfer — which must therefore be a term with empty support, or the consume the same statement performs deletes the relation the contract was written to publish. The `&uniq` inadmissibility is the same sentence read from the other side: a source-declared body is a body, so a caller reading its post-state would be reading a claim about an object at a point the callee cannot name. Both halves were refuted before they were derived — the seventh falsifier round keyed the denotation on `writes` coverage and read `len(P) = len(P) + 1` out of one row, which discharges every goal in every loop from a contradiction. | Registered: the five measure-datum placements this version defers — entry, construct, rebind, enum payload binder, and destructuring binder — carry the same closure sentence and are not yet written, so a measured value still loses its measures at every naming event other than a call. |
| CALL-6 | Publication stated once: a declared relation is instantiated at the call under MSR-3's substitution, established on the call's normal continuation in ENT-5's order, restricted to its arm when it is routed rather than deferred to it, and supported by the substituted terms; and a contract whose published relations are contradictory at their establishment point is refused at the declaration | 🟢 derived | Derived from ENT-4's own least-closure semantics: at a contradictory point every L0 relation and both signs of every goal are derivable, so an inconsistent published set is not one wrong fact at one caller but every fact at every caller, including the subscript bounds SCOPE-2 exists to keep. Refusing it at the declaration is forced by the same sentence — the set is fixed at the declaration and no caller state repairs it. The instantiate-at-the-call/restrict-to-the-arm shape is derived from ENT-5's own call-boundary order together with the seventh falsifier round's refutation of the alternative: deferring a routed relation's establishment to the arm and killing it from that later point makes every write between the call and the arm precede the establishment and kill nothing, which that round used to hand back a run running past its extent in a program the resource judgment accepts. | Registered: the declaration-domain population of this source is empty in this version, because no compiler-owned row carries a declared relation set until the container declaration domain lands; the rule is written over both callee classes and exercised over one. |
| CALL-4 | Contract vocabulary over the result: a `fn_decl` declares exactly one `result_binding`, so a route names no ordinal and no ordinal binder is written, and the destinations stay ENT-3.S12's closed list | 🟡 existence-only | Existence derived: L16's one-denotation-per-position sentence requires that the position a route applies to be named exactly once, and with one result there is exactly one naming and no ambiguity to resolve. Form NOT derived: the ordinal binder this rule declines to write is the form a multi-result declaration would need, and no multi-result declaration exists to measure it against; the rule records the criterion rather than selecting a spelling. | Registered: a result of measured type, a measure over a result place, and a route over any variant of any returned enum are DEFERRED in the rule text with their deltas; each is an admission widening of FN-9 and each is what a container-returning helper needs before it can hand a run back with its measures. |

## v0.45 amendment — the admitted product's own interval (activated 2026-09-05)

Specification binding: active `spec/kernel-spec.md`, headed v0.45, at
SHA-256 `07238ec06058cd42933c4677b42234f0406ba4d8fd31c4bdd980035c159c90dd`,
superseding v0.44
(`5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049`), whose
bytes are archived at `spec/kernel-spec-v0.44.md`. This version amends and
activates in one change, so no candidate digest precedes it. The merge-time
record is in `governance/APPROVALS.md`.

This amendment adds no rule and retires none; it amends [ENT-6] and [ENT-3] in
place and adds [ENT-3]'s enumerated source S14. No row's status moves, so the
statistics line above is v0.44's unchanged.

| Amendment | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| ENT-6 / ENT-3.S14 | The fixed interval-product rule publishes the constant interval its four endpoint products bound, established by S14 on the value the admitted multiplication binds | 🟢 derived | Derived from ENT-1's own fact-source sentence read against what the rule already computes. ENT-6 proves an inclusive interval for each operand and forms the four endpoint products, and the multiplication is admitted exactly when all four lie in the result type; the least and greatest of those same four products therefore bound the value the operation produced, on the same premises, at the same point, with no further derivation. Withholding them was not a soundness boundary but a publication choice, and it is the choice R4 forbids: the shift-left ladder ranks a static rejection above a runtime check, and a writer whose admitted product carries no bound must hand the checker back the identical interval as an executed guard, which is a runtime branch standing in for a fact the compiler had already proved. P0 reads the same way — that guard is an emitted comparison and an unreachable arm on the hottest index path there is. Soundness is the ordinary ENT-5 rule rather than a new one: the published relations name only the bound value and the distinguished zero term, so their support is the bound value alone, which is what they mean — they describe a value already produced, so a write to an operand leaves them true and a write to the bound place kills them. The bound value never aliases an operand because a `let` binder is fresh and a commit value is compiler-owned [ENT-2]. Monotonicity is preserved in the permitted direction: the amendment only establishes facts, and no admission condition anywhere reads whether this source fired. | Registered: S14 states a constant interval, so it reaches an operand bounded by constants and not a bound relative to another runtime value — `p * n <= (k - 1) * n` with runtime `k` stays outside it, because the interval of `p < k` is the type range. That family is what a written non-literal `use` multiplier would reach, and no rule in this version admits one. |

## v0.46 amendment — a stated relation and the atom that discharges it (activated 2026-09-05)

Specification binding: active `spec/kernel-spec.md`, headed v0.46, at
SHA-256 `b853ae310215731d7c4353ec8fdf8ab906081a3f2baba076fb6433537ba3ce65`,
superseding v0.45
(`07238ec06058cd42933c4677b42234f0406ba4d8fd31c4bdd980035c159c90dd`), whose
bytes are archived at `spec/kernel-spec-v0.45.md`. This version amends and
activates in one change. The merge-time record is in
`governance/APPROVALS.md`.

This amendment adds no rule and retires none; it amends [FN-8], [ENT-3] and
[ENT-6] in place. No row's status moves, so the statistics line above is
v0.45's unchanged.

| Amendment | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| FN-8 clause rows | Exact addition, subtraction and multiplication are admitted in a clause and read over the mathematical integers; every other exact row stays inadmissible | 🟢 derived | Derived from what a clause is. FN-9 erases every clause before lowering, so a clause evaluates nothing and requests no operation; the ban existed because a clause is runtime-typed while an `affine_expr` is not, and [INV-1] already carries the carve-out that difference calls for. The three admitted rows are exactly those total over the mathematical integers, so reading them mathematically is a statement about values rather than a partial operation with no obligation to discharge. Division, remainder, negation, absolute value and the shifts are excluded by the same criterion, each having an input its own relation cannot state its way out of. R1 supplies the necessity: without this a size precondition is unwritable, so an expansion function's caller cannot be held to anything the function actually needs. | Registered: the arity of `clause_expr` is unchanged, so a relation still carries one operator and a second is written through a `contract_define`. |
| ENT-3 measure atom | A measure term is an affine atom: one atom per measured place, identified by that place's root binding, minted at its full u64 range, with the L0-to-affine index ranging over measure terms | 🟢 derived | This is the admission v0.44 recorded as DEFERRED, taken on the evidence that named it. Soundness is [ENT-5]'s own support rule rather than a new one: a measure is fixed at its object's creation and an element write never moves it, so the atom is stable exactly while the object is, and the write that removes it is the write to the root binding. The join keeps the atom only where every input agrees; a measure is not arithmetic-updated, so there is no spread for a join delta to stand for, and disagreement means a branch replaced the object. Identity by root binding rather than by an interned measure term is what lets a proof read the atom without interning, which keeps the read path free of the mutation the mint needs. | Registered: a measure of a projected place is not this atom and stays an ordinary clause operand; the projected case awaits a program that needs it. |
| ENT-6 affine route | The affine route discharges a comparison goal whose normalization is affine, whether or not that goal also projects to L0 | 🟢 derived | Derived from what the route proves. The affine target is the goal's own comparison normalized by [ENT-6], so proving the target proves the goal; the L0 projection was a property of the evidence record, not a premise of the proof. Requiring it was sound but incomplete, and the incompleteness was invisible while every affine-provable call goal also had a unit-coefficient projection — which is every such goal a version before this one could state. Without a projection the retained derivation is the affine consequence alone, the shape [ENT-6]'s interval-product rule already uses for its endpoint proofs, so no new evidence kind is introduced. Monotonicity moves in the permitted direction: the change only admits. | Registered: the route now reaches goals no corpus program wrote before this version, so the diagnostic for an unproved affine goal without a projection has no field evidence yet. |

## v0.47 amendment — the named const is the number it names (activated 2026-09-05)

Specification binding: active `spec/kernel-spec.md`, headed v0.47, at
SHA-256 `8c37b2c4401449487a9c1d35b39a7649087ce593f1c0096f1c93ce9e280f0aa5`,
superseding v0.46
(`b853ae310215731d7c4353ec8fdf8ab906081a3f2baba076fb6433537ba3ce65`), whose
bytes are archived at `spec/kernel-spec-v0.46.md`. This version amends and
activates in one change. The merge-time record is in
`governance/APPROVALS.md`.

This amendment adds no rule and retires none; it amends [INV-1] in place and
moves no row's status.

| Amendment | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| INV-1 named const atom | An integer-typed named const is an affine atom, folded at formation to the one closed value it declares; a const-generic parameter is symbolic and is not this admission | 🟢 derived | Derived from R3 and from [ENT-2] clause (c), which already names the mathematical value of an integer literal and of an integer-typed named const in the same breath. The const was therefore a constant term everywhere except in the relations written about it, which made one value have two spellings — the declared name in the body, the repeated digits in every invariant and every `use` — and R3 admits one way to say anything. W1 supplies the cost: the surviving spelling was the worse one, since a limit declared once had to be maintained in every site that reasoned about it, and a stale digit is a silent divergence between what the code enforces and what the proof states. Folding at formation is what keeps the admission free of consequence: no atom kind, image, kill, or join changes, and the same relation over a const and over its literal is byte-identical, including in a failure's rendered residual. The const-generic exclusion is the same sentence read from the other side: an affine factor is a number, and a symbolic constant has no number to fold to. | Registered: a const-generic parameter would need an atom of its own with the [ENT-2] symbolic-constant identity carried into the affine domain; no corpus program has asked for it. |

## v0.48 amendment — a use cites one premise (activated 2026-09-05)

Specification binding: active `spec/kernel-spec.md`, headed v0.48, at
SHA-256 `1df543b91d20e7f5800bc0fea79b5154ae7bf9bda265e45491e2170bf5a70dcc`,
superseding v0.47
(`8c37b2c4401449487a9c1d35b39a7649087ce593f1c0096f1c93ce9e280f0aa5`), whose
bytes are archived at `spec/kernel-spec-v0.47.md`. This version amends and
activates in one change. The merge-time record is in
`governance/APPROVALS.md`.

This amendment adds no rule and retires none; it amends [GRAM-4] and [PRF-1]
in place and moves no row's status.

| Amendment | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| GRAM-4 use premise | A `proof_use` cites one `use_premise` — an invariant name or a delimited relation — optionally prefixed by `N times`; the multiplicity is written as a bare decimal or a name | 🟢 derived | Derived from R3 and from what the form denotes. The multiplicity is how many times a premise is added into the certificate sum, and spelling it `*` claimed it was a multiplication whose right operand is a relation, which is not a thing the language has. Three defects followed from the one pose and each is removed by naming it: the form read as `n * bool`; a term multiplicity was undecidable in strong-LL(2) because after `use IDENT *` the token separating a certificate step from an affine relation source is arbitrarily far away; and the [FORM-2] stated space before `(` existed to carry a distinction the parser could not see, so the rendering rule was load-bearing for meaning. `times` is a fixed atom, so it can never be the second token of a premise name or of a relation, which decides every form within two tokens of `use`. Mandatory parentheses on the relation premise are the same sentence: the bare-when-unmultiplied split was a second consequence of the ambiguity, and R3 admits one way to say a premise. META-5 counts the new production and the new atom, and R3 selects the spelling by evidence under W1 — `times` has zero identifier uses in the corpus and the corpus's own doc strings already reach for the word to describe this construct. | Registered: `times` still reads as an operator binding looser than `*`, which is a pose the parentheses reduce but do not remove; `use 3 times x;` where `x` is a value and `use a + 3 times (b <= c);` are caught after parsing rather than by the grammar. |
| PRF-1 named multiplicity | A multiplicity may name an unsigned integer value; the certificate sum is then a degree-two polynomial whose nonlinear monomials must fold to admitted exact products before the residual forms | 🟢 derived | Derived from the verify-never-search principle and from R1. Scaling a written premise by a value is still verification: the writer names the premise and the multiplier, and the checker multiplies and adds, so acceptance stays a function of the written text and the [ENT-1] monotonicity obligation is discharged structurally. Two restrictions keep the addition free of anything new. The unsigned type is what makes the step sound without an obligation, because `m*p <= 0` follows from `p <= 0` exactly when `m` is nonnegative, and taking that from the written type keeps it structural rather than proved. The fold is what keeps the polynomial transient: a monomial is replaced by the value image an admitted exact multiplication already bound, which is an equality substitution. Folding the sum rather than expanding the target is the direction that stays inside the domain, and that is a termination argument rather than a preference. A sum holds finitely many monomials and each fold removes one, so folding is bounded by construction; expanding is not, because an operand may itself be a product — `let a = n * p; let b = a * q;` expands `b` to `a*q` and then `a` to `n*p`, reaching a degree nothing in [PRF-1] can hold. Expansion also rewrites a proposition that was already affine, so it can turn a provable residual into an unprovable one, while folding only ever removes monomials. What comes out is an ordinary affine inequality, so the residual, its integer tightenings, and the L0 route are the same ones a bare-decimal certificate uses, and no fact, published conclusion, invariant target, or `affine_expr` ever carries a nonlinear term. R1 supplies the necessity: a matrix multiply's inner index at a runtime stride is unwritable without it, and the identical certificate at a literal stride already compiled. | Registered, and measured wider than first recorded: the fold needs each operand image to be a single value image, so `let stride = width + padding; let base = stride * row;` records no product at all — binding the sum, which this row first offered as the remedy, does not help, because the binding's image is still the sum. The only workaround found is passing the operand across a function boundary so the checker holds it opaquely, which costs a whole extra function. A stride sweep measured this as landing on essentially every natural formula in that domain — padding, DIB rounding, tile area, channel count, alignment — so the restriction refuses the static route exactly where the feature was meant to serve. The repair is to fold by the operand's binding identity rather than by its expanded image, which is the rule [PRF-1] already uses one sentence away for a named premise. The route restriction that reads like a second gap is not one, and was measured rather than assumed: a product the finite L0 route admits has at least one compile-time-known operand, and its value image is then already a constant or an affine scaling, so no monomial is formed and there is nothing to fold. With both operands constant the target needs no certificate at all and a written one is the redundant-block rejection. |

## v0.49 amendment — fold by the declaration, not by the expansion (activated 2026-09-05)

Specification binding: active `spec/kernel-spec.md`, headed v0.49, at
SHA-256 `ba0d9e277c3c0d2d46123a6cc12869d8ffbbab7e9c922a03bb165259cc5ce696`,
superseding v0.48
(`1df543b91d20e7f5800bc0fea79b5154ae7bf9bda265e45491e2170bf5a70dcc`), whose
bytes are archived at `spec/kernel-spec-v0.48.md`. This version amends and
activates in one change. The merge-time record is in
`governance/APPROVALS.md`.

This amendment adds no rule and retires none; it amends [PRF-1] in place and
moves no row's status.

| Amendment | Statement | Status | Derivation | Open |
| --- | --- | --- | --- | --- |
| PRF-1 fold identity | A multiplication's operand and a written multiplicity name the same value when they name the same declaration; each such binding contributes one opaque handle, which exists between the fold and the residual and is replaced by its image before anything is proved | 🟢 derived | Derived from the rule [PRF-1] already states one sentence away. A named premise resolves to its declaration identity and is explicitly never reparsed from the current value of its spelling, for the same reason that arises here: a declaration is what the writer wrote, and what it currently expands to is a different thing. v0.48 folded by expansion and so refused every case where the two differ, which measurement showed is every case the feature was written for — a stride copied from a parameter accepted while `width + padding`, `width + 4`, and `2 * width` all rejected, and a stride is definitionally derived. R1 supplies the necessity: without this the feature reaches matrix multiply, where the stride happens to be a parameter, and nothing else in its own domain. The handle's transience is what keeps the admission free of consequence, and both alternatives were built and measured before it was chosen: publishing the handle's defining equality as a fact is invisible to the residual, which is the direct L0 route by rule, and replacing the binding's image with the handle makes every ordinary premise about that binding need the equality to prove. Naming it only between the fold and the residual leaves every other premise, the target, and the residual reading exactly what they read before, which is why no snapshot verdict and no conformance verdict moves. | Registered: an operand that is not a plain read of a binding names no declaration and so contributes no handle. This was drafted as a normative sentence and withdrawn before it landed, because it describes a path measurement could not reach: a field-read operand's multiplication does not discharge its own [OP-2] domain in the first place — `shape.stride *defined k` is undischarged with both operands bounded by `requires` — so it stops before any fold is attempted, and no other non-binding operand shape was found that reaches one. What the code does is settled; whether a writer can ever observe it is not, so it is registered here rather than stated as a rule. |
