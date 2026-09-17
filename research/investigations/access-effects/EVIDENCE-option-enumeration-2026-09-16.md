# Evidence: option enumeration for the ownership redesign, 2026-09-16

Five enumeration reports produced for [OPTIONS.md](OPTIONS.md) by research
agents under the primary agent's direction, one lens each: the AI writer (AI),
radical and adversarial (RAD), systems practice and compiler backends (SYS),
programming-language theory and verification (THE), and Whitefoot's own
recorded history (HIS). Every table row carries a stable ID in its first
cell (lens, section, ordinal); OPTIONS.md groups rows into families by those
IDs and a script checks that every ID appears there exactly once. Rows are
enumeration, not evaluation: nothing here is ranked, selected or refused.
Citations marked "(unverified)" were not confirmed against a venue; HIS rows
quote repository files verbatim, with paths. Remove this file when OPTIONS.md
is superseded.


# Lens file: numbered-ai.md


Scope: every option that changes what an LLM writer must **write**, **see**, or **repair** under a
deterministic checker in a diagnostic-driven repair loop. No ranking, no pruning. `(outlandish)` marks
options kept deliberately. Column key: **Loc** = diagnostic locality (`node` = one tree node, `fn` =
function-local, `cut` = at a declared cut point, `site` = call site, `WP` = whole-program/non-local);
**Tok** = writer token cost; **Weak** = failure mode of a weak model.

Evidence used: blind-writer REPORT 2026-08-28 (6 rejections, 3 with no message, 120 `region` blocks as
pure punctuation, 34/41 `len()` rebinds as boilerplate, 110 lines of re-declared prelude per file,
`--par-ledger` converting a denial into a grant in one edit); default-floor RESULTS (first-green in
0–2 repair rounds, 1.65x / 1.10x vs shipped Rust); frequency-study RESULTS (clustered relational proof
is where the win is, not single checks); proof-use-cost TSVs (checking time vs annotation count:
`fixed` ≈ 16 ms flat; `control`, growing contract premises, 18→41→166 ms for N=16→32→64, ≈ N^1.1–N^2.0;
`growing`, growing explicit `use` steps, 30→163→1689 ms, ≈ N^2.5–N^3.4). Spec size today:
`spec/kernel-spec.md` 533 KB ≈ 130k tokens, `docs/patterns.md` 89 KB ≈ 22k tokens, against the 48k
figure claimed in why-whitefoot §1.

---

## D0 The requirements themselves

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D0-01 | D0-a | Split M4 into three independent requirements: teachability, repairability, spec budget | M4 as written bundles them | — | — | — | Each needs its own acceptance test; M4 currently hides three different failure modes behind one bullet |
| AI-D0-02 | D0-b | R-teach: every mechanism must be statable in ≤ K spec tokens and one worked card | why-whitefoot §1 "48k budget"; patterns.md P1..P34 | — | — | spec truncation → invented rules | Pick K; today's spec is ≈ 130k tokens, ≈ 2.7x the claimed figure, so the constraint is already violated and unmeasured |
| AI-D0-03 | D0-c | R-repair: every rejection must be repairable from the diagnostic alone, without reading another file | blind-writer §6.3 (OWN-6 needed the example corpus) | node | — | infinite repair loop | Makes "mechanical fix" a rule property, not a per-rule courtesy; forbids any rule whose fix is non-local |
| AI-D0-04 | D0-d | R-latency as a first-class requirement, not folded into M1 | constitution: "termination alone is insufficient"; proof-use-cost | — | — | — | M1 says budget-free; the constitution says latency must permit iteration. The map states no latency requirement at all. Directly in tension with any per-statement annotation option below |
| AI-D0-05 | D0-e | R-latency stated as a scaling law, not a constant | proof-use-cost `growing` ≈ N^2.5–N^3.4 | — | — | — | Checking cost must be ≤ quadratic in writer-supplied proof steps; today it is not, which prices D7/D12 per-statement options out |
| AI-D0-06 | D0-f | R-abstraction: generics, nominals and higher-order code must carry identity/effect facts | map §7 "effect roots become identities … nominals carry identity parameters" | site | high | signature explosion | Capability-polymorphic signatures; the map defers this and it is where signature size becomes the cost |
| AI-D0-07 | D0-g | R-outcome: a requirement owning typed error outcomes and their routing | M2 mentions typed outcomes but no R owns them | fn | med | swallows errors | CALL-6 routing already exists; without an R, "typed outcome" is an escape valve with no discipline |
| AI-D0-08 | D0-h | R-nonvacuity: the writer may not discharge an obligation by weakening the spec or returning a default | AlphaVerus reward-hacking (2024); Clover consistency (2024); blind-writer `byte_at` returns `0_u8` out of range | fn | — | writes a plausible wrong default | The real escape hatch M3 does not close: safety survives, semantics do not. Needs an oracle, a totality obligation, or a non-triviality check |
| AI-D0-09 | D0-i | R-bounds: machine-verifiable memory/resource bounds as a standing requirement | constitution "must support machine-verifiable bounds" | cut | med | omits the bound | R2 only says "where bounded memory is promised"; the constitution is stronger |
| AI-D0-10 | D0-j | R-prelude: a program must not re-declare its own basic library | blind-writer §6.2 item 3 (110 identical lines, 30–60% of each file) | — | very high | copies a stale prelude | PROG-1's closed unit is a mechanism choice; a compiler-injected prelude or a spec-blessed one is an option that changes the writer's token bill more than any annotation choice here |
| AI-D0-11 | D0-k | R-evolution: a spec revision must come with a mechanical migration for existing sources | constitution "Compatibility and evolution" | — | — | rewrites from scratch | Ties D13 to D0; no requirement currently owns migration |
| AI-D0-12 | D0-l | Declare R4 and R5 non-independent; one footprint/distinctness requirement with two consumers | map §2 R4(b) and R5(b) request the same information | — | — | — | Halves the vocabulary a writer must learn; loses the "a missing fact costs speed, never correctness" asymmetry that R4(d) states |
| AI-D0-13 | D0-m | Declare R6 a sub-requirement of R4/R5 (same key, different consumer) | map §7 "one refinement vocabulary serves state, distinctness, footprints and kills" | — | — | — | R6 is a precision policy, not a hazard; keeping it separate inflates the taught vocabulary |
| AI-D0-14 | D0-n | Reclassify R7 as a chosen mechanism (FN-1) and state the underlying requirement as "checking cost is modular" | D0 prompt; PROG-1 closed world | — | — | — | The closed world makes whole-program inference legal; FN-1 forbids using it. Every "infer and pin" option below depends on this reclassification |
| AI-D0-15 | D0-o | M1 restated as "reproducible and auditable", allowing a deterministic decidable procedure | M1 today bans the tool, not the property | — | — | — | Would admit e.g. a fixed-qualifier Liquid-type fixpoint (Rondon et al., PLDI 2008) or a Houdini fixpoint (Flanagan & Leino, FME 2001), both terminating and specification-fixed |
| AI-D0-16 | D0-p | M2 restated as "no trap, no unpredicted branch", allowing a typed-outcome branch the writer wrote | M2's own carve-out | fn | low | — | Already the practice; stating it removes the appearance that all runtime branching is banned |
| AI-D0-17 | D0-q | M4 restated with the writer's real cost model: irregularity is expensive, verbosity is free, *retrieval* is the scarce resource | why-whitefoot premise; LeanDojo premise retrieval (NeurIPS 2023) | — | — | retrieves the wrong rule | Changes the spec's shape (D13) more than its size: rule-addressable cards beat a shorter monolith |
| AI-D0-18 | D0-r | M5 restated as a floor guarantee with a measured floor, not a policy | default-floor RESULTS (1.65x, 1.10x) | — | — | — | Gives M5 an acceptance test; the current wording ("failed permission leaves it sequential") is a rule about one mechanism |
| AI-D0-19 | D0-s | Add M6: diagnostics are deterministic, byte-stable and machine-parseable | why-whitefoot §1 follow-up; blind-writer §6.3 (all six are Rust `Debug`) | node | — | — | Precondition for any verifier-in-the-loop training (D13); currently true by accident, not by rule |
| AI-D0-20 | D0-t | Add M7: one spelling per construct, enforced as a checked property of the spec | why-whitefoot "one spelling per construct, to the byte" | — | — | picks the second spelling | Needs a spec-level lint; FORM-2 enforces bytes in source but nothing enforces uniqueness of forms in the spec |
| AI-D0-21 | D0-u | (outlandish) Requirements are themselves machine-checked: each R has an executable discriminating program in the conformance corpus, and a requirement with no program is deleted | Clover's mutual-consistency discipline | — | — | — | Turns §2 into a test suite; large upfront cost, kills prose drift |

## D1 The unit and naming of storage identity

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D1-01 | D1-a | Identity per allocation, named by a type parameter `'a` on the pointer | map §6 R1; Cyclone regions (USENIX 2002) | node | low | forgets a param on a nominal | Cheapest to teach; loses field/element precision, so D3 holes force reject |
| AI-D1-02 | D1-b | Identity per field path `'a.x`, written as a path expression | map §3 Step 1 | node | low | writes `'a` where `'a.x` is meant | Path algebra in the spec; refinement vocabulary must be closed |
| AI-D1-03 | D1-c | Identity per element `'v[i]` with an index proof | map §8 Q2 | node | med | index proof forgotten | Couples D1 to the proof system; every element pointer carries an obligation |
| AI-D1-04 | D1-d | Identity per proved range `'v[lo..hi]` | DPJ index-parameterized regions (OOPSLA 2009) | cut | med | wrong endpoint | Split/join algebra; best fit for parallel loops, heaviest vocabulary |
| AI-D1-05 | D1-e | Existential backing unpacked at pointer formation | map §8 Q2; Alias Types (ESOP 2000) | fn | med | never unpacks, uses the container | Gives containers one rule instead of two; existential packs are the construct models most often get wrong |
| AI-D1-06 | D1-f | Brand introduced by a scope (`brand 'b { … }`) | GhostCell (ICFP 2021); patterns.md P25 "name a generic store brand" | fn | low | brand leaks out of scope | Already taught as P25; brands are coarse, so D9 needs a second mechanism |
| AI-D1-07 | D1-g | Identity inferred everywhere, never written | whole-program inference in the closed world (PROG-1) | WP | zero | cannot see why a call failed | Contradicts FN-1; diagnostics become non-local, which is exactly M4's expensive case |
| AI-D1-08 | D1-h | Identity inferred, then **printed by the compiler and pasted by the writer** as a pinned certificate | Houdini (FME 2001) + Agda/Idris hole interaction + rustfix machine-applicable suggestions | site | high (generated, not composed) | pastes a stale certificate | The AI-writer inversion: the model is a superb copier and a poor inventor. Needs byte-exact re-check of the pasted text and a staleness diagnostic |
| AI-D1-09 | D1-i | Identity as an index into an explicit compile-time storage table declared at the top of the unit | Vault/Fugue keys; ESC/Java ghost fields | fn | high | table drifts from use sites | Makes every identity greppable and enumerable; a table is a second global namespace to keep consistent |
| AI-D1-10 | D1-j | Nothing: pointers name no storage, safety by construction (values only, projections instead of pointers) | Hylo/Val mutable value semantics (JOT 2022); Swift SE-0176 | node | zero | cannot express an object graph | Removes D1 entirely and most of D2/D4; forbids aliased graphs, which D9 requires |
| AI-D1-11 | D1-k | Two-level: coarse identity in signatures, fine identity inside bodies | map §7 granularity coupling | cut | med | mixes the levels | One vocabulary at cut points, another in bodies; doubles the spec-side vocabulary M4 pays for |
| AI-D1-12 | D1-l | Identity carries a generation counter, statically tracked | generational references (Vale); slot-map generations | node | low | ignores the generation | Free at runtime only if the generation is a proof term; otherwise violates M2 |
| AI-D1-13 | D1-m | (outlandish) Identity is content-addressed: a storage's name is the hash of the expression that created it, so identical constructions share a name and the writer never invents one | Unison content-addressed codebase | node | zero | cannot read the name in a diagnostic | Eliminates naming errors entirely; diagnostics become hashes unless the compiler maintains a display map |
| AI-D1-14 | D1-n | (outlandish) The writer never names identities; it names *intentions* (`the buffer I just filled`) and a fixed deterministic resolver maps intention to identity, printing the resolution | monitor-guided decoding (NeurIPS 2023) applied to names | site | low | intention is ambiguous | Needs an ambiguity error with the full candidate set (the FN-7 diagnostic in blind-writer is the model: it printed the whole legal set and the writer fixed it instantly) |

## D2 Where permission and state live

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D2-01 | D2-a | Checker context facts only, implicit | map §6 R1 | fn | zero | cannot see the current state | Nothing to write; nothing to read either, which is the repair problem |
| AI-D2-02 | D2-b | Explicit ghost linear tokens threaded by hand | Verus `Tracked<PointsTo<T>>` (OOPSLA 2023); L3 (TLCA 2005); Mezzo permissions (TOPLAS 2016) | node | very high | drops or duplicates a token | The canonical "humans hate it, models do not" inversion; every call site threads permissions, which is verbosity the constitution licenses |
| AI-D2-03 | D2-c | Tokens with sugar that elides the common thread | Mezzo's implicit permission context | fn | med | writes the sugar where the explicit form is required | Two forms violate one-spelling (M7); the sugar's desugaring must be printable |
| AI-D2-04 | D2-d | Permission in the reference type | Rust | site | low | fights the borrow checker non-locally | The known baseline; blind-writer's OWN-6 wall is exactly this failure |
| AI-D2-05 | D2-e | Permission in the access path per call | Hylo/Val; Swift `inout` exclusivity | site | low | overlapping paths at one call | Small vocabulary, no loans; forbids graphs |
| AI-D2-06 | D2-f | Global table computed for the whole closed program | PROG-1 | WP | zero | non-local rejection | Cheapest to write, worst to repair; the map's M4 names this as the expensive direction |
| AI-D2-07 | D2-g | Both: facts by default, tokens available where the writer wants to be explicit | Verus (ghost + ordinary) | mixed | opt-in | never opts in, then hits a wall | Needs a rule for when the token form is *required*, or the writer defers the cost until it is unpayable |
| AI-D2-08 | D2-h | **The checker prints the state, the writer pastes it** as a `state` block at each cut point | Agda holes; Dafny `assert`-to-see-state; VeriFast symbolic-state printing | cut | high (generated) | pastes a state that no longer holds | The single highest-leverage AI-writer mechanism here: no invention, only transcription; needs byte-exact re-derivation and a "state moved" diagnostic |
| AI-D2-09 | D2-i | State lives in a compiler-maintained sidecar file the writer never edits | proof-carrying code (Necula, POPL 1997); certifying compilers | — | zero | ignores the sidecar, edits source, gets a stale-cert error | Source stops carrying the fact, which contradicts the why-whitefoot premise; but it removes the whole authoring cost |
| AI-D2-10 | D2-j | Runtime tags with proofs eliminating the checks | typestate with residual checks | node | low | relies on the tag | Violates M2 unless every check is provably removed |
| AI-D2-11 | D2-k | Permission as an effect on the *statement*, not on the value | effect systems; EFF-1/EFF-2 today | node | med | over-declares | Rows are already checked both ways and blind-writer got all five programs' rows right first try — strong evidence this shape suits the writer |
| AI-D2-12 | D2-l | (outlandish) Permission state is a first-class printable value the program can `debug_state()` at compile time, and the compiler's answer is inlined into the source as a comment the checker verifies | Lean `#check`/`#print`; Idris `:t` at a hole | cut | med | comment rots | Makes the invisible visible in the artifact the model reads back; needs the comment to be checked, not decorative |
| AI-D2-13 | D2-m | (outlandish) No permission concept at all: the writer submits a *state machine* and the compiler derives access from reachability | state-machine-shaped source (below, D12) | fn | high | wrong transition table | Reverses the whole design; every access becomes a transition and D3's vocabulary becomes user-declared |

## D3 The state vocabulary of a storage

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D3-01 | D3-a | `Init` / `Uninit` / `Gone` only | map §3 | node | zero | none — three states are learnable | Minimum spec cost; cannot express partial init of an array |
| AI-D3-02 | D3-b | Plus `PartiallyInit(range)` | buffer-initialization evidence; LLVM `initializes` attr (EVIDENCE line 541) | node | low | wrong range endpoint | Range algebra; the cleanest unexploited fact per the survey |
| AI-D3-03 | D3-c | Plus `Frozen`, `ExclusivelyLoaned`, `Suspended` | today's OWN-5 | node | low | confuses frozen with loaned | The two-axis vocabulary LEX-1 deferred; each state is a new rejection the writer must learn |
| AI-D3-04 | D3-d | Plus layout/generation | D1-l | node | low | ignores it | Needed only if relocation is implicit |
| AI-D3-05 | D3-e | User-declared typestates per nominal | Typestate (Strom & Yemini, TSE 1986); Plaid (OOPSLA 2009); Vault | fn | high | declares states it never transitions | Open vocabulary: spec stays small, each program's vocabulary grows. Excellent for R10 external resources |
| AI-D3-06 | D3-f | Predicates drawn from `invariant` clauses | RefinedC (PLDI 2021); Liquid types (PLDI 2008) | cut | med | writes an unprovable predicate | Needs a fixed qualifier set to stay deterministic |
| AI-D3-07 | D3-g | States as terms over captured branch conditions | map §6 refinement; CASES.md B08 | fn | low | term blows up at joins | Atom growth is the open question (map §8 Q10); an unbounded term is a latency risk under D0-e |
| AI-D3-08 | D3-h | A lattice with an explicit join | dataflow analysis | fn | zero | surprised by the join result | Writer never sees the join; precision loss is invisible, which is the worst repair signal |
| AI-D3-09 | D3-i | No states: validity by construction (no holes, no partial init) | functional cores; Cogent (ICFP 2016) | node | zero | cannot write in-place algorithms | Removes D3 and most of D1; costs the in-place shapes the floor depends on |
| AI-D3-10 | D3-j | Compiler prints the state vocabulary *actually used* by a program and the writer pins it in a header | Houdini candidate sets | fn | med | pins a stale set | Makes the per-program vocabulary explicit and greppable |
| AI-D3-11 | D3-k | (outlandish) The state vocabulary is fixed at three, and everything else is a *typed outcome* in ordinary control flow | M2's own carve-out taken to the limit | fn | high | branch explosion | Trades state vocabulary for branch count, which models enumerate well and humans do not |
| AI-D3-12 | D3-l | (outlandish) Each state is a teaching card with a worked repair, and the diagnostic cites the card number | patterns.md P-numbers; `rustc --explain` | node | — | — | Makes the spec retrievable rather than short; costs a card per state, which caps D3's growth by a budget rather than a rule |

## D4 Sequential aliasing policy

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D4-01 | D4-a | Allow several writable pointers; state carries safety | map §3 Step 0 | node | zero | writes through a stale pointer with no warning | Facts weaken only where identities coincide; the writer never fights the checker sequentially — the single largest reduction in blind-writer's measured repair load (120 region blocks, OWN-6 wall) |
| AI-D4-02 | D4-b | Forbid: shared-xor-mutable | Rust | site | med | non-local borrow errors | The measured wall; every gain in D4-a is a loss here |
| AI-D4-03 | D4-c | Allow with declared coincidence (`alias 'a 'b;`) | map §7 "aliasing declared" | site | low | forgets the declaration | One line per coinciding pair; the call site must discharge distinctness for every other pair |
| AI-D4-04 | D4-d | Reads may alias, writes may not | Clean/Futhark uniqueness | node | low | copies to satisfy the rule | Familiar and small; still forbids the mutation-while-traversing shape patterns.md lists as a known gap |
| AI-D4-05 | D4-e | Read-only as a type flag, not a loan | map §7 "read-only interface on pointers" | node | zero | omits the flag, loses the fact | Flag is an interface fact only; no duration, no region |
| AI-D4-06 | D4-f | Per-scope exclusivity windows the writer opens explicitly | Swift exclusivity; today's `region` blocks | fn | med | one-statement-region trap | blind-writer §4: the OWN-6 one-statement region is the rule the writer *could not apply from the spec at all* |
| AI-D4-07 | D4-g | Exclusivity requested only where a fact is wanted, as an optimization annotation | `restrict` but checked | cut | opt-in | never requests, program is slow but correct | Cleanly separates safety from speed; M5's floor then rests on teaching, not on the checker |
| AI-D4-08 | D4-h | Compiler proposes the exclusivity annotations it could prove and the writer pins them | Houdini; rustfix suggestions; `--par-ledger`'s proven-useful shape | cut | high (generated) | pins without understanding | `--par-ledger` already demonstrates this loop working: one denial message, one edit, permission granted |
| AI-D4-09 | D4-i | (outlandish) Aliasing is legal but every aliased write emits a *fact-loss report* the writer must acknowledge in source | compiler-emitted budget reports | fn | med | acknowledges everything | Makes the performance cost of a shape visible at authoring time rather than in a profile |
| AI-D4-10 | D4-j | (outlandish) Two dialects in one file: a strict no-alias dialect for hot code, a permissive one elsewhere, declared per function | SPARK proof levels; `#![forbid]` scoping | fn | low | picks the wrong dialect | Violates one-spelling; but it is the only option that prices exclusivity per function |

## D5 Interference facts for parallel permission and the optimizer

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D5-01 | D5-a | Effect footprints over identities, writer-written | map §6 R5; EFF-1/EFF-2 today | cut | med | over- or under-declares | Checked both ways today and blind-writer got 5/5 programs right first try: the best-evidenced writer-facing mechanism in the repository |
| AI-D5-02 | D5-b | Footprints derived from bodies, printed, pinned by the writer | Houdini + PCC | cut | high (generated) | stale pin | Requires D0-n (FN-1 reclassified) or a body-to-signature derivation step |
| AI-D5-03 | D5-c | Footprints derived and *not* written (closed world) | PROG-1 | WP | zero | cannot localize a denial | Zero tokens, worst diagnostics |
| AI-D5-04 | D5-d | Loans on references | Rust; today's PAR-1 loan clause | site | low | non-local | map §8 Q14 keeps this open |
| AI-D5-05 | D5-e | Fractional/counting permissions | Boyland (SAS 2003); Chalice; Viper | cut | med | fraction arithmetic errors | Decidable only under a fixed split discipline |
| AI-D5-06 | D5-f | Region partitions | DPJ (OOPSLA 2009) | cut | med | wrong partition | Index-parameterized regions are the best match for proved ranges (D1-d) |
| AI-D5-07 | D5-g | Ownership transfer into the parallel construct | Rust `rayon`/`scope`, Pony (AGERE 2015) | fn | low | cannot express read sharing | Simple; forces copying |
| AI-D5-08 | D5-h | Purity annotations | Haskell, Koka effects | fn | low | claims purity falsely (checked, so rejected) | Coarse but very cheap to write |
| AI-D5-09 | D5-i | Declare the *determinism guarantee* per construct: source-order equality vs race freedom vs invariant preservation | CAP-1 today gives source-order equality only | cut | low | picks the strongest and gets denied | map §8 Q11 says this is a decision to record, not derive; the writer currently has one level and no way to ask for less |
| AI-D5-10 | D5-j | Permission ledger as a first-class output the writer iterates against | `--par-ledger`, blind-writer §7 | cut | zero (read-only) | ignores it | Already the best diagnostic surface in the toolchain; the option is to make it *normative* (a pinned expectation in source) rather than advisory |
| AI-D5-11 | D5-k | (outlandish) The writer writes no footprints; it writes an expected *ledger verdict* per loop (`@scan: permitted staged`) and the compiler checks its own report against the claim | preregistration; golden-output testing | cut | low | claims the wrong verdict | Inverts annotation into assertion-about-the-compiler; makes performance regressions a compile error |

## D6 Distinctness of identity parameters at function boundaries

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D6-01 | D6-a | Distinct by default, call site proves | map §6/§7 | site | med per call | cannot prove two args from one container | Inverts OWN-7's current default; sound only if every pair, sub-identities included, is discharged |
| AI-D6-02 | D6-b | May-alias by default, callee proves what it needs | C semantics + explicit `restrict` | fn | low | never asks, loses every fact | Safe default, poor floor |
| AI-D6-03 | D6-c | Declared per pair in the signature (`distinct 'a 'b;`) | Fortran/`restrict` contracts; ACSL `\separated` | site | low | forgets a pair | Linear in pairs; a 6-pointer signature is 15 clauses, which models write fine and humans do not |
| AI-D6-04 | D6-d | Inferred from the closed world | PROG-1 | WP | zero | non-local | Contradicts FN-1 |
| AI-D6-05 | D6-e | Both, switched per function | SPARK modes | fn | low | picks wrong | Two rules to teach |
| AI-D6-06 | D6-f | Distinctness as a *result* of the caller's identity naming (two different names are two different storages, by fiat) | nominal identity | node | zero | names the same storage twice | Makes distinctness syntactic and free; requires an aliasing declaration to be the only way to coincide (D4-c) |
| AI-D6-07 | D6-g | Compiler emits the exact `distinct` clause set a signature needs and the writer pastes it | rustfix; AutoVerus proof-fix knowledge base (2024) | site | high (generated) | pastes an over-strong set that call sites cannot meet | Needs the emitted set to be minimal, or the writer over-constrains its own API |
| AI-D6-08 | D6-h | Exhaustive enumeration: every signature lists the full pairwise matrix, no defaults | the "models handle boilerplate" inversion | site | very high | transcription error in a long matrix | O(n^2) tokens per signature; totally regular, no default to remember, and a missing entry is a local error |
| AI-D6-09 | D6-i | (outlandish) Signatures are generated from bodies by the compiler and stored beside the source; the writer edits bodies only | certifying compiler; Unison | fn | zero | cannot design an API | Removes signature authoring entirely; kills modular checking and API intent |
| AI-D6-10 | D6-j | (outlandish) Distinctness is a *proof obligation the caller discharges by construction*: arguments must be syntactically rooted in different `let` allocations, checked by a grammar rule rather than a proof | syntactic separation; Cogent | site | zero | restructures awkwardly | Zero proof cost, strong restriction; would make D6 a parser-level rule with node-local diagnostics |

## D7 The contract vocabulary at call boundaries

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D7-01 | D7-a | `requires`/`ensures` over identity states | Dafny (LPAR 2010); map §6 R7 | site | med | omits an `ensures`, caller cannot proceed | Contract premise count drives checking cost ≈ N^1.1–N^2.0 (proof-use-cost `control`) |
| AI-D7-02 | D7-b | `modifies`/`reads` clauses | Dafny; Low* (ICFP 2017) | site | low | over-declares | Best-in-class notation per EVIDENCE line 1083; the discharge must be a syntactic footprint algebra, not set entailment |
| AI-D7-03 | D7-c | Effect rows written by the writer, checked both ways | EFF-2 today | site | low | exhibits an undeclared effect | Measured cheap: blind-writer guessed all rows correctly across five programs |
| AI-D7-04 | D7-d | Effect rows derived from bodies, writer confirms by pinning | Houdini (FME 2001); Daikon-style candidate generation (TSE 2007) | site | high (generated) | confirms a wrong row | Requires D0-n; keeps FN-1 for *checking* while removing it for *authoring* |
| AI-D7-05 | D7-e | Existential results (`returns fresh 'r`) | Alias Types (ESOP 2000); L3 | site | med | never unpacks | Needed for allocation-returning functions |
| AI-D7-06 | D7-f | States routed by result variant | CALL-6 today | fn | med | one arm's state unhandled | Exhaustive match makes the omission a local error — an AI-writer strength |
| AI-D7-07 | D7-g | Prophecies (the future value of a mutable argument) | RustBelt/RefinedRust; Creusot prophecies | cut | high | cannot reason forward | Powerful for `&uniq` returns; conceptually the hardest item here for a weak model |
| AI-D7-08 | D7-h | No contracts: whole-program inference | PROG-1 | WP | zero | non-local | The closed world makes it legal; M4's diagnostic-locality cost makes it expensive |
| AI-D7-09 | D7-i | Contracts inferred from bodies, printed as a diff, writer accepts or overrides | LSP code actions; `cargo fix` | site | high (generated) | accepts everything | The confirm-don't-compose shape; needs an "I meant something stronger" override that the body must then meet |
| AI-D7-10 | D7-j | Typed-outcome-first API convention: every partial operation returns a variant, contracts only for the facts the optimizer needs | M2; blind-writer's zero-claim result | fn | med | default arm returns a wrong value (D0-h) | The measured path of least resistance: five programs, zero claims, every partial op discharged by `if` and `len` |
| AI-D7-11 | D7-k | Contract *cards*: a closed catalog of contract shapes, each with a card number, and a signature must instantiate one | patterns.md P-numbers; design patterns as a checked vocabulary | site | low | picks the wrong card | Caps contract-language growth by construction; forbids a contract nobody carded |
| AI-D7-12 | D7-l | Contract size budget: a signature above K clauses is rejected | D0-e latency law | site | — | splits a function badly | Turns the latency law into a local, node-level diagnostic |
| AI-D7-13 | D7-m | (outlandish) Two contracts per function: one the writer wrote and one the compiler derived; a mismatch is an error and the diff is the diagnostic | Clover's code/spec/doc consistency triangle (2024) | site | double | writes a vacuous contract that matches trivially | Directly attacks D0-h vacuity: a derived contract that is *stronger* than the written one flags an under-specified API |
| AI-D7-14 | D7-n | (outlandish) The contract is written in the *caller*, not the callee: each call site declares what it assumes, and the compiler checks the union against the body once | assume-guarantee; Dijkstra's wp read backwards | site | very high | assumptions drift between sites | Perfect locality at the site the diagnostic fires; O(sites) tokens |

## D8 Storage placement, relocation and moves

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D8-01 | D8-a | `own` aggregates travel as handles; storage stays put | map §6 R3 | site | zero | assumes a copy | Contracts must say whether the callee ends the storage; changes the calling convention |
| AI-D8-02 | D8-b | Aggregates travel as bytes; every pass may relocate | today | site | zero | interior pointer invalidated silently | Interior pointers become nearly unusable |
| AI-D8-03 | D8-c | Explicit relocating `move` is the only thing that ends storage | map §6 R3 | node | low | forgets `move` (measured: blind-writer attempts 3 and 4) | `OWN-1` already emits the exact mechanical fix string; the writer still hit it twice, so the *generalization* is the weak point, not the diagnostic |
| AI-D8-04 | D8-d | No moves: copy-only or place-only | mutable value semantics; Fortran | node | zero | copies a large aggregate | Removes D8 and much of D1; costs the zero-copy shapes |
| AI-D8-05 | D8-e | Reallocation as a contract-declared replacement with a conditional exit state | map §3 Step 3 | site | med | cannot state the conditional | map §8 Q5: whether one `ensures` can state a reallocating push is open |
| AI-D8-06 | D8-f | Pinning (`pin 'a;` forbids relocation for a scope) | Rust `Pin`; C++ | fn | low | pins everything | A single verb, node-local errors |
| AI-D8-07 | D8-g | Projections instead of interior pointers | Hylo/Val; Swift `_read`/`_modify` | node | low | cannot hold a view across statements | The clean answer to interior access without a general reference type |
| AI-D8-08 | D8-h | Every object in a pool; relocation is an index update | patterns.md P2 struct-of-arrays pool | fn | med | index/generation confusion | Already the blessed encoding for self-referential data (patterns known gaps) |
| AI-D8-09 | D8-i | The set of storage-ending operations is a closed, enumerated table in the spec, and the diagnostic prints the whole table | FN-7's entry table, which blind-writer called "excellent … prints the whole closed set" | node | zero | — | Directly copies the one diagnostic shape the writer trial rated highest; map §8 Q1 asks whether the set is complete |
| AI-D8-10 | D8-j | (outlandish) The writer declares placement per binding (`frame`, `heap`, `arena 'a`, `inline`) with no defaults, and the compiler rejects an omitted placement | explicit allocators; Zig | node | high | picks the wrong one and never revisits | Fully explicit, totally regular; the placement becomes a greppable performance decision rather than an inferred one |

## D9 Container elements and backings

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D9-01 | D9-a | Path identities `'v[i]` | map §8 Q2 | node | med | forgets the index proof | Index disjointness first appears at Step 2 |
| AI-D9-02 | D9-b | Existential backing per formation | Alias Types; map §8 Q2 | fn | med | never unpacks | One rule for push/reserve/realloc |
| AI-D9-03 | D9-c | Both, with the path as sugar | map §8 Q2 | mixed | med | mixes them | Two vocabularies |
| AI-D9-04 | D9-d | Per-element ghost tokens | Verus `PointsTo` per cell | node | very high | drops a token in a loop | Exact, unreasonably verbose — which the constitution licenses and this lens rates as tolerable |
| AI-D9-05 | D9-e | Generations per element | slot maps | node | low | ignores generation | Runtime cost unless erased |
| AI-D9-06 | D9-f | Index-only access with bounds proofs, no element pointers | blind-writer's `byte_at` accessor pattern | node | low | wrong default on the out-of-range arm (D0-h) | Measured: one safe accessor per program made "every bounds problem downstream vanish" |
| AI-D9-07 | D9-g | Iterator objects with proved invariants | Java/C++ iterators + separation logic | fn | med | invalidates the iterator | Needs an iterator-invalidation contract, which is D8-e again |
| AI-D9-08 | D9-h | Split-and-join tokens for ranges | DPJ; `split_at_mut`; patterns.md known gap "split_uniq disjoint views" | cut | med | splits at the wrong point | Best fit for parallel loops; the split point is an arithmetic proof |
| AI-D9-09 | D9-i | Length facts survive callee element writes (an S6-style rule) | blind-writer §6.2 finding 1 and finding 3 | node | −34 rebinds | — | Measured cost of *not* doing this: 34 of 41 `len()` bindings in five programs exist only to re-establish a killed fact. The largest single measured boilerplate tax in the corpus |
| AI-D9-10 | D9-j | A `writes` path that can name element storage rather than the whole parameter | blind-writer §6.2 finding 1 | site | low | writes the coarse path | Same tax, attacked from the effect-row side instead of the kill side |
| AI-D9-11 | D9-k | (outlandish) Containers are not a language concept: every container is a user nominal with declared typestates and contracts, and the spec has no container rules at all | Cogent; ATS views | fn | high | writes an unsound container (rejected, not accepted) | Maximum spec-size saving; maximum per-program token cost; the library becomes the spec |

## D10 Shared mutation across threads and the concurrency story

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D10-01 | D10-a | Locks as custodians of identity state | map §6 R9 | site | low | forgets which identities a lock holds | Synchronization cost lands on the write, not the hold; two calls on one lock deny each other under PAR-1 (map §8 Q11) |
| AI-D10-02 | D10-b | A second, weaker permission level without source-order equality | map §8 Q11 | cut | low | picks the weak level everywhere | Excluded by CAP-1 today; a decision to record |
| AI-D10-03 | D10-c | Protocol-governed regions with explicit `use` steps | CAP (ECOOP 2010); Iris; Verus atomics | cut | very high | cannot construct the protocol | EVIDENCE §: "a second model, not an addition" |
| AI-D10-04 | D10-d | Phase/epoch only | epoch-based reclamation; catalog §8 | fn | low | wrong phase boundary | Beats `RwLock` on the read path at zero reader cost |
| AI-D10-05 | D10-e | Manifest sharing | Balzer & Pfenning (ICFP 2017) | fn | high | acquire/release discipline errors | Session-typed; a second sub-language |
| AI-D10-06 | D10-f | Message passing only | Erlang, Pony | fn | med | copies too much | Removes D10's hard cases; costs shared-buffer performance |
| AI-D10-07 | D10-g | No threads: process-level parallelism | Unix | — | zero | — | Removes the requirement; costs the target projects (kernels, browsers) |
| AI-D10-08 | D10-h | Deterministic parallelism only | DPJ; PAR-1/PAR-2 today | cut | low | cannot express a server | Keeps source-order equality, the strongest guarantee and the strongest restriction |
| AI-D10-09 | D10-i | Linear `Task<'ids>` handle carrying the child's exit contract, consumed by join | Chalice `fork`/`join` (ESOP 2009); Verus `JoinHandle` | site | med | never joins, obligations leak | map §8 Q13: the never-joined child is unresolved |
| AI-D10-10 | D10-j | The concurrency story is a closed catalog of carded architectures; anything else is rejected | why-whitefoot "a closed, taught catalog of program architectures" | fn | low | needs an uncarded shape | Caps spec size and teaches by example; the floor mechanism taken literally |
| AI-D10-11 | D10-k | (outlandish) Concurrency is expressed only as a declared state machine over shared identities, and the compiler derives the locking | Verus state-machine sharding (OSDI 2023); TLA+-shaped source | fn | high | wrong transition set | Source shape models produce well (enumerated transitions); the derivation must stay deterministic |

## D11 External resources

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D11-01 | D11-a | Identities with a `foreign` qualifier over interference | map §8 Q12 | node | low | omits the qualifier | Spelled over interference, not origin; whether an internal object shared with a thread carries it decides constitutionality |
| AI-D11-02 | D11-b | Capability tokens per resource | today's PROV-6; L3 | node | med | drops the capability | Already in the language and already taught |
| AI-D11-03 | D11-c | Effect categories (`io`, `net`) | Koka, Eff | fn | low | mis-categorizes | effects.md rejects mechanism categories; constitution forbids an external role granting an exception |
| AI-D11-04 | D11-d | Trusted adapters outside the language | FFI everywhere | — | zero | trusts wrongly | Violates M3 for the adapter author; an escape hatch by another name |
| AI-D11-05 | D11-e | States as ordinary enums with declared typestates | D3-e | fn | med | wrong transition | The constitutionally cleanest option: no rule distinguishes host-crossing values |
| AI-D11-06 | D11-f | The whole external surface is one closed table the diagnostic prints | FN-7 entry table; blind-writer §2 | node | zero | — | Rated "excellent" by the writer trial; also the source of the "no stdin, Unix filter genre unwritable" finding |
| AI-D11-07 | D11-g | Opaque-contents rule: an external identity's bytes may change between operations unless a contract says otherwise | map §6 R10 | node | zero | assumes stability | Minimal; kills every retained fact across an external call |
| AI-D11-08 | D11-h | Library-supplied resource cards, one per resource kind, in the taught catalog | patterns.md P12, P34 | fn | low | uses the wrong card | Teaching, not mechanism; moves cost out of the spec |
| AI-D11-09 | D11-i | (outlandish) External operations are *proof-obligation generators*: the compiler emits the obligations the host must meet as a machine-readable manifest checked against the host at link time | seL4 assumptions; contract-based linking | — | zero | — | Makes the constitution's "external conditions" clause executable rather than prose |

## D12 Surface form and sugar

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D12-01 | D12-a | `&`/`&uniq` as sugar over pointer plus state clauses | map §8 Q9 | node | low | uses sugar where the explicit form is needed | Two spellings violates M7; migration path for existing corpus |
| AI-D12-02 | D12-b | Explicit-only forms, no sugar at all | the verbosity inversion | node | high | verbose but regular — the writer's best case | One spelling; every fact greppable; humans would refuse it |
| AI-D12-03 | D12-c | Per-cut-point annotations only (signatures, loop heads, joins) | Dafny/Verus loop invariants; today's `for (... invariant ...)` | cut | med | omits the loop-head invariant | Bounded annotation count → bounded checking cost (D0-e); the only option consistent with the measured N^2.5–N^3.4 growth of per-step proofs |
| AI-D12-04 | D12-d | Per-statement annotations | VeriFast `open`/`close`; today's `use` steps | node | very high | proof-step explosion → 1.7 s checks at N=64 | Perfect locality, measured superlinear cost; the two properties trade directly |
| AI-D12-05 | D12-e | SSA/ANF-shaped source: every intermediate named, no nested calls | today's GRAM-9 flat three-address form | node | high | writes a nested call anyway (blind-writer attempt 1) | Already the rule; the *diagnostic* for violating it carries no message, which is the fixable half |
| AI-D12-06 | D12-f | Projectional/structured source: the writer emits a tree through a tool API, never bytes | JetBrains MPS; structured editors | node | low | cannot emit the tree shape | Makes GRAM-9 and FORM-2 rejections unrepresentable — 2 of the 6 measured rejections, both message-less |
| AI-D12-07 | D12-g | Total canonical formatter: formatting is normalized, never rejected | `gofmt`; `rustfmt` | — | zero | — | Removes FORM-2 as an error class entirely; blind-writer lost a full compile round to one double space |
| AI-D12-08 | D12-h | Grammar-constrained decoding: the writer physically cannot emit an ill-formed program | Synchromesh (ICLR 2022); grammar-constrained decoding (EMNLP 2023) | node | zero | — | Needs the grammar exported as a decoding artifact; eliminates the syntax class of repairs |
| AI-D12-09 | D12-i | Typestate-aware constrained decoding: the checker's state is consulted at generation time, so a use-after-move is unsampleable | Monitor-Guided Decoding (NeurIPS 2023) | node | zero | — | Needs the checker to answer incremental queries at token granularity; the strongest possible form of "irregularity is expensive" relief |
| AI-D12-10 | D12-j | Typed holes: an incomplete program still checks, and the checker reports the goal at each hole | Hazelnut (POPL 2017); Agda/Idris interaction | node | zero | leaves holes | Turns the repair loop into a fill-in-the-blank task, the shape models handle best |
| AI-D12-11 | D12-k | State-machine-shaped source: functions are transition tables over declared states | D3-e; Verus sharding | fn | high | wrong table | Enumerative, regular, verbose; exactly the human-hostile/model-friendly inversion |
| AI-D12-12 | D12-l | Diagnostics carry a machine-applicable fix on *every* rule, with an applicability level | `rustc` machine-applicable suggestions; RustAssistant (ICSE 2024) | node | zero | applies a `MaybeIncorrect` fix blindly | blind-writer §6.3: the 3 rules with a `kind:` payload cost seconds, the 3 with only a byte offset cost the session |
| AI-D12-13 | D12-m | Diagnostics print the offending source line and the whole legal set | FN-7 (praised), OWN-6 (the wall) | node | zero | — | Cheapest measured improvement in the corpus; no language change at all |
| AI-D12-14 | D12-n | Every diagnostic cites a teaching card, and the card is retrievable by ID | `rustc --explain`; patterns.md P-numbers; LeanDojo retrieval (NeurIPS 2023) | node | zero | retrieves nothing and guesses | Makes spec size a retrieval problem instead of a context-window problem |
| AI-D12-15 | D12-o | Region syntax retired entirely; `'s` keeps identity meaning only | map §6 "PROV-1 spelling with identity meaning only" | node | −120 blocks | reuses region intuition from Rust | Measured: 120 `region` blocks across five programs, "most carry no design intent at all; they are punctuation" |
| AI-D12-16 | D12-p | (outlandish) Two synchronized surfaces: a verbose canonical form the checker reads and a compact form the model reads, with a total bidirectional mapping | bidirectional transformations; Lean's `set_option pp.all` | node | low | edits the wrong surface | One spelling per construct is preserved only if the mapping is total and checked |
| AI-D12-17 | D12-q | (outlandish) The source file contains the compiler's own state dump as checked comments, regenerated on every accept | literate proof scripts; `.expected` golden files | cut | high (generated) | edits the dump | The writer's context window then contains the checker's view without a tool round-trip |

## D13 Validation and migration strategy

| id | Option | One-line description | Prior art or source | Loc | Tok | Weak | What it would need; obvious tensions |
|---|---|---|---|---|---|---|
| AI-D13-01 | D13-a | Exhaustive small-model enumeration | `research/experiments/access-state/RESULTS.md`; map §8 Q8 | — | — | — | Already the repository's method; bounded and machine-checked |
| AI-D13-02 | D13-b | Formal model in a proof assistant | RustBelt; RefinedC (PLDI 2021) | — | — | — | High cost, highest confidence; map §8 Q8 defers it |
| AI-D13-03 | D13-c | Differential testing against Rust/C | `differential-fuzz` experiment | — | — | — | Catches semantic divergence, not soundness |
| AI-D13-04 | D13-d | Fuzzing the checker for unsound accepts | `differential-fuzz` | — | — | — | Finds accepts that should be rejects; needs an oracle |
| AI-D13-05 | D13-e | A writer-trial benchmark: N tasks, fixed model, count compile attempts and repair rounds per candidate | blind-writer protocol; DafnyBench (2024); default-floor PROTOCOL | — | — | — | The only method in this list that discriminates candidates *on this lens*; needs a frozen model and a frozen spec per arm |
| AI-D13-06 | D13-f | Repair-loop metrics as an acceptance criterion: a candidate that needs more than K rounds on the corpus is rejected | "Is Self-Repair a Silver Bullet?" (ICLR 2024): feedback quality dominates | — | — | — | Makes D0-c executable; requires a stable task corpus and honest first-green freezing |
| AI-D13-07 | D13-g | Diagnostic-quality audit: every rule must have a test asserting its message names the construct and the fix | blind-writer §6.3 scorecard | node | — | — | A gate target, not prose; 3 of 6 rules would fail it today |
| AI-D13-08 | D13-h | Spec-token budget enforced by `make check` | D0-b; M4 | — | — | — | Needs a tokenizer in the gate; the spec is ≈ 130k tokens against a claimed 48k |
| AI-D13-09 | D13-i | Per-mechanism spec-size accounting: each candidate reports the tokens it adds to spec + cards | M4 | — | — | — | Makes "the spec stays small" a comparable number across candidates instead of a slogan |
| AI-D13-10 | D13-j | Compile-latency gate per candidate on a fixed corpus | proof-use-cost harness (already native, already paired) | — | — | — | The harness exists; the option is to make its numbers a selection criterion, per D0-d/D0-e |
| AI-D13-11 | D13-k | Vacuity/anti-reward-hacking corpus: programs that are accepted but wrong because a contract was weakened or a default returned | AlphaVerus (2024); Clover (2024); D0-h | — | — | — | Needs behavior oracles, not compiler verdicts; the failure mode the current gate cannot see |
| AI-D13-12 | D13-l | Patch spec section 5 in place vs a new section | map §9 | — | — | — | A new section duplicates vocabulary; a patch risks a half-migrated corpus |
| AI-D13-13 | D13-m | Pattern cards migrate mechanically: each P-number gets a candidate-specific rewrite as part of the candidate | patterns.md P1..P34 | — | — | — | 34 cards; a candidate that cannot rewrite them all is under-specified for the writer |
| AI-D13-14 | D13-n | Discriminating programs: tree-walk utility (OWN-6/region tax), in-place buffer shift (D4), reallocating push with an interior pointer (D8-e), two-argument-from-one-container call (D6-a), per-iteration-output loop (PAR-3, blind-writer §7.3) | blind-writer §4, §5, §7.3; map §3 | — | — | — | Five programs that each isolate one coupling from map §7; all five are already written or nearly so |
| AI-D13-15 | D13-o | (outlandish) Verifier-in-the-loop fine-tuning as a validation instrument: train on the checker and measure which candidate a model learns fastest | AutoVerus (2024); AlphaVerus (2024); Baldur (FSE 2023) | — | — | — | Requires M6 (byte-stable diagnostics) and a large synthetic corpus; "learnability" becomes a measurable language property rather than an argument |

---

## Requirements missing or misstated (for D0)

| id | # | Finding | Ground | Consequence for the option space |
|---|---|---|---|
| AI-REQ-01 | 1 | No latency requirement exists; M1's "budget-free" reads as "cost-free" | constitution: "guaranteed termination alone is insufficient"; proof-use-cost `growing` 30→163→1689 ms for N=16→32→64 | Per-statement proof options (D12-d, D9-d, D2-b) have a measured superlinear price that no requirement currently names |
| AI-REQ-02 | 2 | Teachability, repairability and spec size are three requirements bundled into M4 | M4's single bullet | Each has a different acceptance test; a candidate can be regular (teachable) and still unrepairable (OWN-6) |
| AI-REQ-03 | 3 | M4's "the specification stays small" has no number | spec is 533 KB ≈ 130k tokens vs why-whitefoot's 48k claim | The constraint is currently unmeasured and, against its own stated budget, violated; D13-h/D13-i make it comparable |
| AI-REQ-04 | 4 | M3 "no unsafe escape" covers mechanisms, not semantics | blind-writer's `byte_at` returns `0_u8` outside the range; AlphaVerus reward hacking | The writer's real escape hatch is a wrong default or a weakened `requires`; no requirement forbids it and no gate detects it |
| AI-REQ-05 | 5 | R7 is FN-1, a mechanism, not a requirement | D0 prompt; PROG-1 closed world | Reclassifying it opens every "infer, print, pin" option (D1-h, D5-b, D6-g, D7-d/i) without giving up signature-only *checking* |
| AI-REQ-06 | 6 | R4 and R5 request the same information for different consumers | map §2 R4(b) vs R5(b) | One footprint vocabulary, two consumers, halves what the writer must learn |
| AI-REQ-07 | 7 | R6 is a precision policy keyed identically to R4/R5 | map §7 | Presenting it as a third requirement inflates the taught surface for no writer-visible gain |
| AI-REQ-08 | 8 | No requirement owns the standard library / prelude | blind-writer §6.2 item 3: 110 identical lines, 30–60% of each file | The largest single token cost measured in the corpus is invisible to §2 |
| AI-REQ-09 | 9 | No requirement owns generics, nominals or higher-order code carrying identity/effect facts | map §7 "nominals carry identity parameters"; §8 leaves it open | Signature size for generic code is the likeliest place a candidate becomes unwritable |
| AI-REQ-10 | 10 | No requirement owns typed error outcomes, though M2 depends on them | M2 | "Typed outcome" is the pressure valve for every partial operation and has no discipline attached |
| AI-REQ-11 | 11 | Resource bounds are conditional in R2 but unconditional in the constitution | R2(a) "where bounded memory is promised" vs constitution "must support machine-verifiable bounds" | A candidate can satisfy R2 and not the constitution |
| AI-REQ-12 | 12 | M5 is a rule about one mechanism, not a floor requirement | M5 text vs default-floor RESULTS | With a measured floor, M5 becomes testable against candidates rather than assumed |

## Decision points missing from this list

| id | # | Missing decision | Why it is a decision, not a detail | Whose options it changes |
|---|---|---|---|
| AI-DPX-01 | D14 | The diagnostic contract: what every rejection must contain (rule, node, rendered source, legal set, mechanical fix, card ID, applicability) | blind-writer measured 3 of 6 rules failing this and one of them ending the session; it is the writer's entire feedback channel | All of D1–D12: a mechanism's cost is its diagnostic, not its rule text |
| AI-DPX-02 | D15 | The unit of compilation and the library/prelude story | PROG-1's closed world is assumed by D6-d, D7-h and every inference option, and it is what forces the 110-line prelude | D0, D6, D7, D13 |
| AI-DPX-03 | D16 | Spec delivery form: monolith vs rule-addressable cards vs retrieval index | M4 treats spec cost as size; for an LLM it is retrieval accuracy | D0, D3-l, D7-k, D12-n |
| AI-DPX-04 | D17 | Where inference is allowed to run and how its result enters the source (nowhere / printed and pinned / sidecar / silent) | This is orthogonal to every mechanism choice and changes the token bill more than any of them | D1-h, D2-h/i, D4-h, D5-b, D6-g, D7-d/i |
| AI-DPX-05 | D18 | The generation interface: text, constrained decoding, structured/tree API, or typed holes | Decides which error classes exist at all; 2 of 6 measured rejections vanish under D12-f/g/h | D12, D13-f |
| AI-DPX-06 | D19 | The anti-vacuity story: how the project detects an accepted-but-wrong program | No current gate distinguishes "proved" from "proved trivially"; this is the one hazard M1–M5 leave open | D0-h, D7-m, D13-k |
| AI-DPX-07 | D20 | The proof-cost budget: whether a per-function annotation/step ceiling is a language rule | Makes the D0-e latency law local and enforceable instead of an aggregate property | D7-l, D12-c vs D12-d |
| AI-DPX-08 | D21 | Migration mechanics for the existing corpus and the 34 pattern cards | A candidate that cannot mechanically rewrite `tests/programs/` and patterns.md is not deliverable | D13-l, D13-m |


# Lens file: numbered-radical.md


Lens: challenge every premise. No ranking, no scoring, no pruning. Prior art is
named only where it exists; `(unverified)` marks a recollection I could not check
here; `(none known)` marks an option with no prior art I can name.
Premises treated as revisable: PROG-1's closed world, FN-1's signature-only
judgment, M1's budget-free determinism, M2's no-runtime-check, M3's no-escape,
M5's fast-shapes-are-accepted-shapes, and the requirement list itself.

## D0 The requirements themselves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D0-01 | D0-1 Delete R4 | Alias facts are a performance instrument, not a requirement; the optimizer takes only what proofs already imply | CompCert emits no alias metadata and is still correct | Constitution ranks runtime performance high; deleting R4 concedes the why-whitefoot measurements |
| RAD-D0-02 | D0-2 R7 is a mechanism | "Modular, signature-only" is a compile-cost choice; PROG-1 makes whole-program the default | MLton, Stalin whole-program compilation | Recheck cost per edit becomes the real requirement; large-project latency is untested |
| RAD-D0-03 | D0-3 R6 is checker-internal | Frame retention is entailment-engine precision, not a property of accepted programs | ENT-5 is a compiler rule, not a language rule | Removing it from the requirement list hides a real cost driver |
| RAD-D0-04 | D0-4 R9 is not independent | Several holders is R1 precision sequentially and R5 concurrently | The map's own (d) clauses | Loses the "occasional write, long-lived holder" shape as a first-class case |
| RAD-D0-05 | D0-5 R2 folds into R3 | "Released exactly once" is linearity of a resource value, not a separate requirement | Austral, Linear Haskell multiplicities, Wadler 1990 | Bounded-memory proofs still need a home |
| RAD-D0-06 | D0-6 R3 folds into R1 | A transfer is a state event on storage; value classes are a spelling of state | The map's own hazard ladder, Step 1 | Copy/affine/linear still needs a type-level carrier |
| RAD-D0-07 | D0-7 R4 and R5 are one relation | Distinctness and interference are the same fact read by two consumers | DPJ effects serve both; CSL parallel rule | One relation must serve both exactness directions |
| RAD-D0-08 | D0-8 M1 is a mechanism | The requirement is reproducible, predictable acceptance; "no solver" is one way to get it | Polonius Datalog fixpoint; RAML/AARA's LP (Hoffmann et al) | A fixed-iteration Datalog or LP is deterministic and terminating but is "a solver" |
| RAD-D0-09 | D0-9 M2 is a mechanism | Its own escape clause (typed outcome with real control flow) already admits generational and epoch checks | Vale generational references; Swift SE-0176 dynamic exclusivity | Needs a cost rule for which checks are cheap enough; erodes the "no check" slogan |
| RAD-D0-10 | D0-10 M3 is a mechanism | "No unsafe escape" could be "no escape without a machine-checked obligation at the boundary" | Verus trusted specs, Low* `assume val`, SCOPE-3's linked definitions | The TCB boundary moves into source; who audits it |
| RAD-D0-11 | D0-11 M4 is misstated | "The writer is an AI" implies a context-window budget, not merely verbosity tolerance | Constitution's teachability aim; practice with long specs | A measurable "fits in one prompt" criterion does not exist yet |
| RAD-D0-12 | D0-12 M5 is a mechanism | It presumes the writer chooses the shape; a schedule language lets the writer choose shape and mapping separately | Halide, Exo | Two artifacts to check instead of one |
| RAD-D0-13 | D0-13 FN-1 is a mechanism inside R7 | Enumerate the whole-program alternative explicitly rather than assuming signatures | PROG-1 itself | Separate compilation is then permanently excluded |
| RAD-D0-14 | D0-14 PROG-1 is unspent | No stated requirement currently needs the closed world; if it is kept, some requirement should spend it | PROG-1; MLton | Spending it forecloses libraries and incremental builds |
| RAD-D0-15 | D0-15 Missing: resource bounds | Machine-verifiable memory and hardware-resource bounds are a constitutional requirement with no R | Constitution, Safety; RAML, SPARK, COSTA | Needs a cost semantics and a deterministic bound derivation |
| RAD-D0-16 | D0-16 Missing: environment-failure behavior | Allocation failure, stack exhaustion, device error must have defined behavior | Constitution, Safety | Interacts with linearity: what discharges obligations on a failed path |
| RAD-D0-17 | D0-17 Missing: evolution and compatibility | Signature, proof and spec evolution is a constitutional section with no requirement row | Constitution, Compatibility and evolution | Certificates and archives must survive version changes |
| RAD-D0-18 | D0-18 Missing: compile latency at scale | "Guaranteed termination alone is insufficient" is constitutional; M1 says nothing about cost | Constitution, Performance | Conflicts with whole-program options D0-2, D0-13 |
| RAD-D0-19 | D0-19 Missing: repairable rejection | OWN-8 rejects sound programs, so the named restructuring is part of the requirement, not a courtesy | OWN-8; DIAG-1 | Needs a machine-readable repair form for an AI writer |
| RAD-D0-20 | D0-20 Missing: proof stability under edit | A one-token change must not reopen every proof | Incremental verification practice (unverified) | Cuts against whole-program inference |
| RAD-D0-21 | D0-21 Missing: abstraction | R1..R10 are entirely first-order: no closures, generics over identity, or dynamic dispatch | FN-2 generics exist in the spec; closure cost is live on this branch | Effect rows and identity parameters must be higher-order |
| RAD-D0-22 | D0-22 Missing: termination | EFF-3 already needs a termination proof v0 does not have | EFF-3 text | A termination checker is a new deterministic family |
| RAD-D0-23 | D0-23 Missing: observability | Debuggers, profilers and core dumps read storage the model never admits as an alias | (none known as a language rule) | Either the model admits a foreign reader or debug builds are outside it |
| RAD-D0-24 | D0-24 Missing: cost model exposure | The AI must predict performance to choose shapes; nothing requires the language to expose one | Constitution, Performance; Halide's explicit schedules | A published cost model constrains the backend |
| RAD-D0-25 | D0-25 Missing: optimizer authority | May the backend use facts the source never stated? R4 assumes retained facts only | R4 (c) "never re-derived by analysis" | Forbidding re-derivation gives up free wins; allowing it weakens the fact channel's purpose |

## D1 The unit and naming of storage identity

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D1-01 | D1-1 No identity at all | The writer names nothing; whole-program points-to analysis supplies every fact | Andersen, Steensgaard; MLton | Drops FN-1; diagnostics become non-local, which M4 calls expensive |
| RAD-D1-02 | D1-2 Identity is the SSA definition point | Source is in SSA/ANF, so every storage is named by the instruction that created it | Swift SIL, MLIR, Appel "SSA is functional programming" | Mutation must be modelled as memory ops anyway; loops need phi identities |
| RAD-D1-03 | D1-3 Identity is an iteration vector | Storage named by allocation site plus a polyhedral iteration vector | Polyhedral model (Feautrier), Pluto, ISL | Exact only for affine control; non-affine code has no name |
| RAD-D1-04 | D1-4 Identity is a table row key | No pointers exist; every object is a row and every field a column | Relational databases, Apache Arrow, DuckDB | Pointer-chasing data structures must be re-expressed as joins |
| RAD-D1-05 | D1-5 Identity is an ECS entity id | The language is entities plus component columns; systems iterate archetypes | Unity DOTS, Bevy, flecs (practice; no formal prior art known) | (outlandish) No general graph structure; kernels and compilers do not decompose this way |
| RAD-D1-06 | D1-6 Identity is a generation counter | The handle carries a generation; deref compares and yields a typed outcome | Vale generational references | M2 only if the outcome is typed control flow, not a trap; a compare per deref |
| RAD-D1-07 | D1-7 Identity is a hardware capability | The type names a CHERI capability with bounds and a validity tag | CHERI (Watson, Woodruff et al) | Ties the language to one ISA; revocation still needs a sweep (CHERIvoke, unverified) |
| RAD-D1-08 | D1-8 Identity is a scope brand | A rank-2 brand introduced by a scope; no identity variables in signatures | GhostCell (Yanovski et al 2021), Haskell `ST` | Brand-coarse: no distinctness inside a brand |
| RAD-D1-09 | D1-9 Identity is a compiler-assigned integer | PROG-1 lets the compiler build one global storage table; source writes the integer or a generated symbol | (none known in this form) | Source becomes machine-generated and unstable under edit |
| RAD-D1-10 | D1-10 Identity is the creating proof term | The witness that produced the storage is its name | Dependent pairs in Coq/F* | (outlandish) Every identity is a proof object; erasure must be total |
| RAD-D1-11 | D1-11 Identity is a link-time address range | No dynamic allocation; every storage is a linker symbol with a fixed extent | JPL Power of Ten (Holzmann 2006), Ravenscar profile, SCADE code generation | Excludes containers, recursion depth and dynamic workloads |
| RAD-D1-12 | D1-12 Identity is an owner path | Storage named by its position in an ownership tree | Ownership types (Clarke, Potter, Noble 1998), Universe types | Tree ownership forbids intrusive lists and back pointers |
| RAD-D1-13 | D1-13 Identity is a channel | Storage is only reachable through a protocol endpoint | Session types (Honda et al), Singularity channel contracts | Every access becomes a message; cost model is unclear for hot loops |
| RAD-D1-14 | D1-14 No identity because nothing mutates | Immutable values only; "the same storage" has no meaning | Okasaki, Clean, pure functional cores | Needs D8-8-style proved reuse to stay fast |
| RAD-D1-15 | D1-15 One identity: the world | The whole heap is one linear value threaded through the program | Wadler 1990, Clean `*World`, Alias Types store types | Zero parallelism from identities; the store type is the entire program state |
| RAD-D1-16 | D1-16 Identity is a dependent index into one byte array | The heap is `Array<Byte>` and every access carries a refinement proof | ATS, Liquid Haskell, Flux | (outlandish) Types lose all structure; proof burden is arithmetic everywhere |
| RAD-D1-17 | D1-17 Identity inferred, writer confirms | The compiler derives identities whole-program and emits a digest the writer checks in | (none known); Unison's codebase-as-database as the storage analogue | A generated artifact becomes part of the source of truth |
| RAD-D1-18 | D1-18 Identity is a fresh nominal type per site | A staging meta-level mints a distinct type per allocation site, so distinctness is type equality | MetaML (Taha & Sheard), Terra (DeVito et al 2013), LMS (Rompf & Odersky) | Code size and type-count explosion; loops need bounded unrolling |

## D2 Where permission and state live

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D2-01 | D2-1 Nowhere | No permission concept exists; whole-program analysis answers every question on demand | MLton, Infer biabduction (Calcagno et al 2011) | Acceptance depends on analysis precision, which M1 forbids as a selector |
| RAD-D2-02 | D2-2 In a separate proof object | The AI emits a certificate beside the program; a small kernel checks it | Proof-carrying code (Necula & Lee), Foundational PCC (Appel 2001), DRAT/LFSC checkers | The certificate format is a second language; the kernel is the new TCB |
| RAD-D2-03 | D2-3 In a Datalog database | Facts are an EDB, the rules are the spec, acceptance is a fixpoint | Polonius, Doop, Soufflé | Deterministic and terminating, but "a solver" under M1's current wording |
| RAD-D2-04 | D2-4 In a BDD over condition atoms | State is a boolean function; joins are BDD operations with a fixed variable order | bddbddb (Whaley & Lam 2004), symbolic model checking (McMillan) | Canonical and deterministic; blowup is data-dependent, so cost is not bounded by syntax |
| RAD-D2-05 | D2-5 In hardware tags | Permission lives in memory tags; static analysis only elides checks | CHERI, ARM MTE, Mondrian MMP (Witchel et al 2002) | M2 relaxed to "trap-free typed outcomes"; portability lost |
| RAD-D2-06 | D2-6 In the runtime scheduler | Static summaries are published; the runtime derives the dependence graph | Legion (Bauer et al 2012), Regent, StarPU, OmpSs | M2 and M5 relaxed for scheduling only; deterministic results still provable |
| RAD-D2-07 | D2-7 In an undo log | Permission is "you may write what you logged"; rollback restores state | WAM trail (Warren), STM Haskell (Harris et al 2005), rr | (outlandish) Log cost on every write; but exact for aliased mutation |
| RAD-D2-08 | D2-8 In the continuation's type | CPS/answer-type modification: permission is what the continuation expects | Answer-type modification (Danvy & Filinski, unverified for this use) | (outlandish) Unfamiliar to every backend; erasure unclear |
| RAD-D2-09 | D2-9 In effect handlers | Every access goes through a handler that carries the capability | Effekt (Brachthäuser et al), Koka (Leijen), OCaml 5 effects | Handler dispatch cost unless fully staged away |
| RAD-D2-10 | D2-10 In the value | Generation or epoch field stored with the object | Vale, epoch-based reclamation (Fraser 2004), RCU | Per-object space cost; typed outcome at every deref |
| RAD-D2-11 | D2-11 In the allocation plan | A whole-program placement pass computes every storage's lifetime up front | Futhark memory allocation, Halide bounds inference | Only works where extents are computable; no data-dependent lifetimes |
| RAD-D2-12 | D2-12 In a checked-in lock file | The compiler writes the derived facts; the writer approves; the compiler rechecks | (none known) | Generated file becomes reviewable state; merge conflicts on facts |
| RAD-D2-13 | D2-13 In debug builds only | Checks compiled in for testing, erased in release under a static proof | Gradual verification (Bader, Aldrich, Tanter 2018), Gradual C0 (unverified) | Two semantics; M2 holds only for release |
| RAD-D2-14 | D2-14 In a threaded store type | The heap's type is a value passed and returned by every function | Alias Types (Smith, Walker, Morrisett 2000), Cogent | Signatures name unaffected storage; abstraction needed for scale |
| RAD-D2-15 | D2-15 Split: pointer plus ghost token | Identity in the reference, permission in a separate linear token the writer threads | Verus `Tracked<PointsTo>`, L3 (Ahmed, Fluet, Morrisett) | Verbose but explicit; M4 says verbosity is cheap |
| RAD-D2-16 | D2-16 In the schedule | The algorithm is pure; a schedule artifact declares in-place updates and their legality | Halide, Exo (Ikarashi et al 2022), Elevate/RISE (Hagedorn et al) | Two checkers; the legality rules are the real ownership system |

## D3 The state vocabulary of a storage

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D3-01 | D3-1 No states | Validity by construction: no holes, no free, nothing ever ends | Arena-only languages, pure functional cores | Needs D8-1 or D8-2; no per-object release |
| RAD-D3-02 | D3-2 One state: typed | Physical type safety only; temporal safety by never reusing storage for another type | CCured (Necula et al 2002), type-stable memory | Memory growth unbounded without reuse |
| RAD-D3-03 | D3-3 Writer-declared typestate automata | Each nominal declares its states and legal transitions | Vault (DeLine & Fähndrich 2001), Fugue, Strom & Yemini 1986 | Precision collapses under aliasing unless identity carries it |
| RAD-D3-04 | D3-4 Protocol states on a channel | State is a session type advancing on each operation | Honda et al, Balzer & Pfenning 2017 | Every access is a protocol step; hot loops pay syntax |
| RAD-D3-05 | D3-5 Full separation assertions | State is a separation-logic formula with explicit fold and unfold | Iris, VST, Low*/F* | M1 needs a fixed, search-free discharge; the writer supplies every step |
| RAD-D3-06 | D3-6 Boolean functions over atoms | State is a BDD over captured condition atoms, canonical by fixed variable order | bddbddb, symbolic model checking | Canonical joins for free; blowup is data-dependent |
| RAD-D3-07 | D3-7 Affine constraint sets | State is a polyhedron over indices; join is a convex hull | Polyhedral analysis (Feautrier), abstract interpretation (Cousot & Cousot) | Exact only for affine fragments; widening is a budget in disguise |
| RAD-D3-08 | D3-8 Writer-declared lattice plus finiteness proof | The writer supplies a lattice and proves finite height; the checker runs the fixpoint | Abstract interpretation frameworks; (writer-supplied-with-proof form: none known) | A second sub-language; M4 cost |
| RAD-D3-09 | D3-9 State is the history | Storage carries an append-only operation log; every query is a fold | Event sourcing, Dedalus/Bloom | (outlandish) Unbounded ghost state; erasure must be total |
| RAD-D3-10 | D3-10 Resource-algebra elements | State is a ghost RA element with frame-preserving updates | Iris ghost state | M1 needs a fixed update checker; RA composition is a search |
| RAD-D3-11 | D3-11 Shape graphs | State is a 3-valued shape abstraction computed whole-program | TVLA (Sagiv, Reps, Wilhelm), Space Invader (Distefano, O'Hearn, Yang) | Precision decides acceptance, which M1 forbids |
| RAD-D3-12 | D3-12 State as a type index | The state is part of the type; transitions are type changes | ATS, Idris, F* | Dependent typing in the surface language |
| RAD-D3-13 | D3-13 No Uninit | Everything is initialized to a total value at creation; holes are unrepresentable | Java/Go zero-init semantics | Costs a write per allocation; `malloc` without `memset` becomes inexpressible |
| RAD-D3-14 | D3-14 Generations replace Gone | "Ended" is a counter mismatch, checked as a typed outcome | Vale, slot maps | One compare per deref; storage never returned to the OS |
| RAD-D3-15 | D3-15 Per-bit-range state | State is tracked per bit range, not per field, for packed and union layouts | (none known as a language rule) | Vocabulary grows; needed for bit fields and unions anyway |

## D4 Sequential aliasing policy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D4-01 | D4-1 No references exist | Mutable value semantics: parameters are `let`/`inout`/`sink`, never aliases | Hylo/Val (Racordon et al., mutable value semantics), Swift value types | Interior pointers and intrusive structures must be indices; copies must be proved away |
| RAD-D4-02 | D4-2 Unrestricted aliasing, facts from proofs only | Any number of writable pointers; the optimizer gets only what the writer proved | Alias Types, L3 unrestricted pointers | R4 quality becomes a writer-effort question, not a language guarantee |
| RAD-D4-03 | D4-3 Unrestricted aliasing, exclusivity re-derived | Aliasing is legal; the compiler recovers exclusivity whole-program as a derived fact | Infer, Doop, Andersen | Facts vary with analysis precision; M1 tension |
| RAD-D4-04 | D4-4 Dynamic exclusivity where static fails | A typed-outcome exclusivity check at the few places static proof cannot reach | Swift SE-0176 exclusivity enforcement | M2 relaxed; the check must be a branch the source can see |
| RAD-D4-05 | D4-5 Hardware-enforced exclusivity | Capability or tag hardware enforces it; static proof only elides | CHERI, ARM MTE | Portability; no facts for the optimizer unless statically proved anyway |
| RAD-D4-06 | D4-6 No writes in source | The program is pure; in-place update is a proved compiler transformation | Futhark uniqueness (Henriksen et al 2017), Dex (Paszke et al 2021), Perceus FBIP | Systems code (device registers, in-place parsing) resists purity |
| RAD-D4-07 | D4-7 Writes only in transactions | Aliased mutation is legal inside a transaction; conflicts analyzed statically | AME (Isard & Birrell 2007), Atomos (Carlstrom et al 2006), STM Haskell | Transaction cost; static conflict analysis is itself an alias analysis |
| RAD-D4-08 | D4-8 One global write token | All mutation requires the single linear world token | Wadler 1990, Clean `*World` | Serializes everything; parallelism needs token splitting, i.e. regions again |
| RAD-D4-09 | D4-9 Copy-on-write with proved uniqueness | Aliasing is always safe because a write to a shared value copies | Swift arrays + ARC, Lean 4 RC (Ullrich & de Moura 2019), Perceus | Static RC elision or a runtime count; M2 tension where the count is dynamic |
| RAD-D4-10 | D4-10 Declared coincidence sets | The writer declares which names may be the same storage; the checker joins their facts | (none known as a language rule); C99 `restrict` as the inverse | Precision is lost exactly where the writer asked for it |
| RAD-D4-11 | D4-11 Per-scope exclusivity windows | Exclusivity is a property of a block, not of a reference | Swift accessors and yield-once coroutines, Hylo subscripts | Interior pointers may not escape the window |
| RAD-D4-12 | D4-12 Aliased writes plus commutativity proofs | Overlapping writes are legal if they commute | Commutativity analysis (Rinard & Diniz 1997) | A commutativity proof obligation per operator pair |
| RAD-D4-13 | D4-13 Logged, reversible mutation | Every write is logged; a failed proof rolls back rather than rejecting | Janus reversible language, WAM trail | (outlandish) Contradicts "no runtime cost"; but removes whole classes of proof |
| RAD-D4-14 | D4-14 Column-level exclusivity | Shared-xor-mutable applies to columns, not objects: one system writes a column, others read it | ISPC/Arrow SoA, ECS system scheduling (practice) | Requires D9-2; object-oriented shapes vanish |

## D5 Interference facts for parallel permission and the optimizer

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D5-01 | D5-1 Parallelism is not in the source | A separate typed schedule parallelizes a pure algorithm; legality is checked on the schedule | Halide (Ragan-Kelley et al 2013), Exo, Elevate/RISE | Two artifacts; the schedule language is where all the hard rules move |
| RAD-D5-02 | D5-2 Polyhedral dependence as the fixed family | Exact, decidable dependence testing for affine loop nests, deterministic and terminating | Feautrier, Pluto, ISL | Only affine control; non-affine code gets nothing |
| RAD-D5-03 | D5-3 Dataflow or streaming semantics | Independence is structural: nodes communicate only through channels | StreamIt (Thies et al), Lustre/Signal, Kahn process networks, SDF (Lee & Messerschmitt) | Irregular pointer code does not fit; buffer sizing becomes the new proof |
| RAD-D5-04 | D5-4 Disentanglement | The invariant is that parallel tasks do not reach each other's newly allocated data | MPL disentanglement (Westrick et al 2020) | A different invariant from footprint disjointness; needs a matching memory manager |
| RAD-D5-05 | D5-5 Monotone shared state | Shared state only grows in a lattice, so interference cannot change results | LVars (Kuper & Newton 2013), CRDTs (Shapiro et al 2011) | In-place mutation of buffers is not monotone |
| RAD-D5-06 | D5-6 Runtime dependence with static summaries | The compiler publishes footprints; the runtime builds the task graph | Legion, Regent, StarPU, OmpSs | M2/M5 relaxed for scheduling; determinism still provable |
| RAD-D5-07 | D5-7 Static conflict analysis over transactions | Everything is atomic; the analysis decides which atomics can run together | AME, Atomos | Transaction machinery in the runtime |
| RAD-D5-08 | D5-8 Commutativity instead of disjointness | Two statements may overlap if their effects commute, even on the same storage | Rinard & Diniz 1997 | Per-operator proof obligations; source-order equality is lost, replaced by result equality |
| RAD-D5-09 | D5-9 Effect rows derived, never written | The closed world derives every row; signatures carry none | Lucassen & Gifford 1988 as the base; whole-program effect inference | Drops FN-1; rows stop being a writer-visible contract |
| RAD-D5-10 | D5-10 Phases only | Bulk-synchronous supersteps; no intra-phase sharing at all | BSP (Valiant 1990), GPU kernel model | Fine-grained task parallelism is inexpressible |
| RAD-D5-11 | D5-11 Distributed memory only | No shared address space; workers exchange messages | MPI, PGAS (UPC, Chapel, X10 places) | Copying cost; shared-memory performance is given up |
| RAD-D5-12 | D5-12 Guarantee: race freedom only | Permission promises no data race, not source-order equality | CSL (O'Hearn 2007), Rust `Send`/`Sync` | Results become schedule-dependent; CAP-1 excludes this today |
| RAD-D5-13 | D5-13 Guarantee: observational determinism | Any interleaving yields the same observable outputs | DPJ (Bocchino et al 2009); Deterministic Process Groups (Bergan et al, unverified) | Needs an observation model; stronger than race freedom, weaker than source order |
| RAD-D5-14 | D5-14 Guarantee: invariant preservation | Overlap is legal when each participant preserves a declared invariant | Rely-guarantee (Jones), Iris | A second proof vocabulary; no equality claim at all |
| RAD-D5-15 | D5-15 Expose a weak memory model | Source names release/acquire and the proof rules are stated against a formal weak model | RC11 (Lahav et al 2017), Promising semantics (Kang et al 2017) | Large spec growth; M4 cost |
| RAD-D5-16 | D5-16 Time partitioning | A static schedule assigns disjoint time slots, so footprints need not be disjoint | ARINC 653, time-triggered architecture | (outlandish for a general language) Needs a real-time runtime |

## D6 Distinctness of identity parameters at function boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D6-01 | D6-1 No function boundaries | A staged meta level generates one monolithic object program; nothing crosses a boundary | MetaML, Terra, LMS, Zig comptime | Code size; debugging the generated program; recursion needs a base case |
| RAD-D6-02 | D6-2 Inferred globally, confirmed by the writer | Whole-program inference derives distinctness; signatures are a generated cache the writer approves | MLton as the inference base; (confirmation form: none known) | Generated signatures in source control |
| RAD-D6-03 | D6-3 Datalog fixpoint per call site | Distinctness is an IDB relation computed once over the closed world | Polonius, Doop | Deterministic but not syntax-directed; M1 wording |
| RAD-D6-04 | D6-4 Distinctness by construction | Every argument is an index into one named table; two arguments alias iff their indices are equal | ECS, relational programs | Requires D1-4/D9-2; equality proofs become integer refinements |
| RAD-D6-05 | D6-5 Fresh nominal types per allocation site | Distinctness is type inequality, decided by name | (unverified) Terra/LMS staging idioms | Type explosion; no polymorphism over sites |
| RAD-D6-06 | D6-6 Hardware bounds at the boundary | Capability bounds are compared where static proof is absent | CHERI | A dynamic compare; M2 relaxed |
| RAD-D6-07 | D6-7 May-alias by default, callee handles overlap | The callee has a typed-outcome overlap path, like `memmove` versus `memcpy` | C `memmove`; Fortran's contrary rule | Every callee pays a branch or a specialization |
| RAD-D6-08 | D6-8 Declared per call site | The caller writes and proves a `restrict`-style claim; the signature says nothing | C99 `restrict`, Fortran argument aliasing | Proof burden moves entirely to callers; unsound without full discharge |
| RAD-D6-09 | D6-9 Integer refinements | With indices instead of pointers, distinctness is `i != j` in a refinement | Liquid Haskell, Flux, ATS | Arithmetic proofs everywhere; a decidable fragment must be fixed |
| RAD-D6-10 | D6-10 Copies remove the question | Arguments are values; the callee cannot observe aliasing | Hylo/Val mutable value semantics | Copy cost unless proved away |
| RAD-D6-11 | D6-11 Per monomorphic instance only | PROG-1 closes the world, so distinctness is proved per instantiation, never polymorphically | MLton defunctorization, monomorphization | Instance count explosion; no separate compilation |
| RAD-D6-12 | D6-12 Existential unpack at the call | The callee unpacks two fresh identities; distinctness is by construction inside the body | Alias Types `exists`, L3 | An unpack step per call; the caller still owes the pack |

## D7 The contract vocabulary at call boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D7-01 | D7-1 No contracts | Whole-program inference; FN-1 retired entirely | MLton, Stalin, Infer summaries | Non-local diagnostics; no separate compilation ever |
| RAD-D7-02 | D7-2 Derived contracts, writer-confirmed | The compiler writes the contract into a checked-in digest; the writer approves it | (none known); generated-interface practice (unverified) | The digest is reviewable state and a merge-conflict surface |
| RAD-D7-03 | D7-3 A separate proof object | The AI emits program and certificate; the checker validates the pair | PCC (Necula & Lee), Foundational PCC (Appel), typed assembly (Morrisett et al) | Certificate language, kernel checker, and proof portability across spec versions |
| RAD-D7-04 | D7-4 Source is a proof script | The writer writes proofs; the program is extracted | Coq extraction, F*, Isabelle code generation, CakeML | Performance of extracted code; a whole proof assistant in the spec |
| RAD-D7-05 | D7-5 Session types per resource | A protocol, not a pre/post pair, governs each resource across many calls | Honda et al, Singularity channel contracts, Balzer & Pfenning | Protocol state must be threaded; hot-path syntax cost |
| RAD-D7-06 | D7-6 Typed schedules | The contract is "this algorithm, under this schedule"; legality rules replace pre/post | Halide, Exo | Two languages; interprocedural schedules are unsolved (unverified) |
| RAD-D7-07 | D7-7 Refinement types on indices | No storage vocabulary at all; contracts constrain integers and lengths | Liquid Haskell, Flux, Stainless, ATS | Requires D9-1; decidable fragment must be fixed for M1 |
| RAD-D7-08 | D7-8 Relational views | A function is a query; its contract is a view definition over tables | Datalog, LogicBlox, DBSP (unverified) | (outlandish) Imperative systems code must be re-expressed |
| RAD-D7-09 | D7-9 Boundary checks in debug, proofs in release | Contracts are executable in debug builds and erased under proof in release | Gradual verification (Bader, Aldrich, Tanter), SPARK's mixed mode | Two semantics; M2 holds only for release builds |
| RAD-D7-10 | D7-10 No calls: explicit state machines | The program is transitions; a "call" is a transition, so contracts are the transition relation | Esterel/SCADE, P (Desai et al), Statecharts (Harel) | Unfamiliar structure for compilers and kernels |
| RAD-D7-11 | D7-11 Hash-keyed summaries | Each contract carries a content hash so rechecking is incremental and cache-keyed | Unison content-addressed code | Renaming becomes free; identity of a function is its hash |
| RAD-D7-12 | D7-12 Capabilities instead of contracts | The caller passes capabilities; the callee names no storage and can do nothing else | Effekt, Scala capture checking, E language | Capability granularity is the new precision question |
| RAD-D7-13 | D7-13 Prophecy contracts | A mutable argument's final value is named in the precondition | RustHorn (2020), Creusot, Aeneas | M1 poor as usually discharged; unnecessary if state is checked per operation |
| RAD-D7-14 | D7-14 Mixed regime | Contracts only at chosen cut points; everything inside a cut is whole-program | (none known as a language rule); LTO and inlining as practice | The cut becomes a tuning knob affecting acceptance, which M1 dislikes |

## D8 Storage placement, relocation and moves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D8-01 | D8-1 No heap | All storage statically allocated; sizes fixed at link time | JPL Power of Ten (Holzmann 2006), Ravenscar, MISRA profiles, SCADE | Excludes dynamic workloads; browsers and compilers do not fit |
| RAD-D8-02 | D8-2 Arena only with region inference | The compiler infers regions; no per-object free exists | Tofte & Talpin 1997, MLKit, Cyclone | Coarse lifetimes leak; inference precision selects acceptance |
| RAD-D8-03 | D8-3 No moves | Copy-only; linear resources are non-movable handles into pools | Slot maps, handle-based engines (practice) | Copy cost; linearity must be carried by the handle, not the storage |
| RAD-D8-04 | D8-4 Nothing relocates | Every object is pinned in a slot; containers hold indices | ECS, slot maps, `std::deque`-style chunking | Cache locality suffers for pointer-heavy structures |
| RAD-D8-05 | D8-5 Columns instead of structs | A struct is decomposed into per-field arrays; an object is an index | Apache Arrow, DuckDB, ISPC SOA, data-oriented design (Acton) | Struct-shaped APIs disappear; FFI and layout control get harder |
| RAD-D8-06 | D8-6 Compiler chooses all placement | The writer never names a storage class; a whole-program pass allocates | Futhark memory allocation, Halide bounds inference | Cost model becomes opaque, contradicting D0-24 |
| RAD-D8-07 | D8-7 Destination-passing style | Every function writes its result into a caller-supplied destination | Shaikhha et al 2017, Futhark, C out-parameters | Every signature gains a destination parameter; composition gets verbose |
| RAD-D8-08 | D8-8 Persistent structures plus proved reuse | Values are immutable; the compiler proves in-place reuse where uniqueness holds | Perceus (Reinking, Xie, de Moura, Leijen 2021), Lean 4 (Ullrich & de Moura), Okasaki | Reuse analysis quality selects performance, and often uses a runtime count |
| RAD-D8-09 | D8-9 Heap as a threaded linear value | Relocation is a rewriting of the store type; no separate placement rules | Alias Types, Cogent, Capability Calculus (Crary, Walker, Morrisett 1999) | Signatures name the whole store; abstraction is mandatory |
| RAD-D8-10 | D8-10 SSA rebinding replaces moves | Nothing survives a move because names are definitions, not storage | SSA, Swift SIL | Memory ops still need a store model; only the surface changes |
| RAD-D8-11 | D8-11 Compacting collector at proved safe points | Objects move; no interior pointers exist across a safe point | Semispace/compacting GC, Immix (unverified for this framing) | (outlandish under M2) A runtime with pauses; but removes fragmentation proofs |
| RAD-D8-12 | D8-12 Explicit two-space arenas | Copying between arenas is a source operation with a proved name change | L3's `realloc` typing, Cyclone dynamic regions | Every derived pointer is re-derived after the copy |
| RAD-D8-13 | D8-13 Stack only, escape analysis | Escape analysis proves the heap unnecessary for the program's shapes | Escape analysis (Choi et al, unverified), Go escape analysis | Data-dependent lifetimes force heap anyway |
| RAD-D8-14 | D8-14 Chunked growth | Containers grow by chaining fixed blocks, so element storage never relocates | `std::deque`, chunked arrays, rope structures | Indexing indirection; contiguity for SIMD is lost |

## D9 Container elements and backings

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D9-01 | D9-1 No interior pointers, ever | Access is by index plus a bounds proof; nothing points into a container | Java/C# semantics, handle-based engines, why-whitefoot section 10 handles | One indirection per access; iterator patterns are re-expressed |
| RAD-D9-02 | D9-2 Containers are relations | Elements are rows; each field is a column; access is a query | Arrow, DuckDB, relational model | Row-at-a-time code becomes awkward; joins replace pointer chasing |
| RAD-D9-03 | D9-3 ECS as the container model | The program is systems over component tables with archetype iteration | Unity DOTS, Bevy, flecs (practice; no formal prior art known) | (outlandish) No general containers; scheduling is the language |
| RAD-D9-04 | D9-4 Proved affine index maps only | Elements are reachable only through affine index expressions | Polyhedral model, DPJ index-parameterized arrays | Hash tables and irregular structures are inexpressible |
| RAD-D9-05 | D9-5 Index sets as types | Iteration is a total map over an index type; out-of-range is unrepresentable | Dex (Paszke et al 2021), Futhark, APL-family languages | Dynamic-length containers need existential index types |
| RAD-D9-06 | D9-6 Per-element generations | Each slot carries a generation; a stale handle yields a typed outcome | Vale, slot map crates | Space and a compare per access; M2 relaxed to typed outcomes |
| RAD-D9-07 | D9-7 Split-and-join range tokens | A range capability splits into disjoint sub-capabilities and rejoins | Rust `split_at_mut`, Regent partitions, fractional permissions (Boyland 2003) | A split algebra with a fixed, search-free discipline for M1 |
| RAD-D9-08 | D9-8 Existential backing per formation | Pointer formation unpacks the backing; container calls repack it | Alias Types `exists`, L3 | An unpack/repack step at every boundary |
| RAD-D9-09 | D9-9 Backing epochs | Interior pointers carry the backing epoch; reallocation bumps it | Epoch-based reclamation (Fraser 2004), RCU | Either a compare (typed outcome) or a static `len < cap` proof, as today |
| RAD-D9-10 | D9-10 Snapshot plus delta log | Readers see an immutable snapshot; writers append deltas | LSM trees, MVCC, persistent vectors | Memory overhead; reads see stale data by design |
| RAD-D9-11 | D9-11 Bulk operations only | Elements are never individually named; only map, scan, reduce and friends | APL, Futhark, SQL | In-place algorithms with two cursors resist this |
| RAD-D9-12 | D9-12 Iterators as typed schedules | Traversal is a schedule with proved invariants, not a pointer that advances | Halide loop IR, Exo | The schedule language must cover irregular traversal |
| RAD-D9-13 | D9-13 Dependent indexed types | `Vec n T` with proofs; the index type carries the bound | ATS, Idris, F* | Dependent types in the surface; proof burden is arithmetic |
| RAD-D9-14 | D9-14 One global pool per type | Containers hold indices into a per-type pool; elements are interned | String interning, per-type arenas (practice) | Cross-type locality lost; pool lifetime is global |

## D10 Shared mutation across threads and the concurrency story

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D10-01 | D10-1 No threads | Isolated processes with separate heaps and message passing | Erlang, Singularity SIPs (Hunt & Larus), seL4 | Copy cost; shared-memory throughput given up |
| RAD-D10-02 | D10-2 Deterministic parallelism only | Nondeterminism is not expressible at all | DPJ, MPL, LVars | Servers and event loops resist determinism |
| RAD-D10-03 | D10-3 Everything is a transaction | Shared operations are atomic; conflicts analyzed statically | AME (Isard & Birrell 2007), Atomos (Carlstrom et al 2006) | Runtime transaction support; static conflict analysis is an alias analysis |
| RAD-D10-04 | D10-4 Ownership as scheduler input | Static summaries feed a runtime that derives the dependence graph | Legion, Regent, StarPU | M2/M5 relaxed for scheduling only |
| RAD-D10-05 | D10-5 Phases and epochs only | Bulk-synchronous supersteps; readers pay nothing between phases | BSP (Valiant), RCU grace periods, GPU kernels | Latency-sensitive fine-grained sharing does not fit |
| RAD-D10-06 | D10-6 Typed reference capabilities | Sharing is decided by the type of the reference, checked syntactically | Pony (Clebsch et al 2015) | A capability lattice to teach; M4 cost |
| RAD-D10-07 | D10-7 Session-typed channels only | All sharing is through protocol endpoints, with manifest sharing for acquire/release | Honda et al, Balzer & Pfenning 2017 | Protocol overhead on every interaction |
| RAD-D10-08 | D10-8 Single-writer protocols as constructs | Seqlock and RCU patterns are language constructs with proved protocols | RCU (McKenney), seqlock, hazard pointers (Michael 2004) | A weak memory model in the spec; reader-side invariants |
| RAD-D10-09 | D10-9 CRDTs | Concurrent writes merge by construction; exclusion is never needed | Shapiro et al 2011 | (outlandish for systems code) Merge semantics for arbitrary state |
| RAD-D10-10 | D10-10 Hardware transactional memory | Atomicity from the ISA; aborts are a typed outcome | Intel TSX, IBM HTM | Portability; abort paths must be written |
| RAD-D10-11 | D10-11 Time partitioning | A static schedule proves accessors never overlap in time | ARINC 653, time-triggered architecture | Real-time runtime; not a general answer |
| RAD-D10-12 | D10-12 Distributed memory only | No shared address space; workers exchange messages | MPI, PGAS (UPC, Chapel, X10) | Copying; NUMA becomes explicit everywhere |
| RAD-D10-13 | D10-13 Exposed weak memory model | Release/acquire are source-level facts and the proof rules quote a formal model | RC11 (Lahav et al), Promising (Kang et al) | Large spec growth; M4 cost |
| RAD-D10-14 | D10-14 Everything shared is foreign | Any cross-thread storage keeps no contents fact across any operation | The map's `foreign` qualifier, generalized | All optimization on shared data is lost; correctness is cheap |

## D11 External resources

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D11-01 | D11-1 The host is an adversarial thread | External writes are modelled as interference from an unscheduled participant | Rely-guarantee (Jones); the map's `foreign` qualifier | No contents facts across any operation on such storage |
| RAD-D11-02 | D11-2 The host is a source state machine | The resource's protocol is declared in source; the implementation is linked | PRE-1 signatures, Low* `assume val`, Verus trusted specs | The declaration is TCB; who validates it against the real OS |
| RAD-D11-03 | D11-3 The world is a linear value | A `World` token is threaded through every effectful call | Wadler 1990, Clean `*World`, GHC `State# RealWorld` | Serializes I/O unless the token splits, which reintroduces regions |
| RAD-D11-04 | D11-4 Pure core, imperative shell | The program is a function from inputs to outputs; the runtime performs I/O | Elm architecture, Haskell `IO` at the edge | Kernels and drivers are all shell |
| RAD-D11-05 | D11-5 Explicit capabilities, no ambient authority | Every external operation needs a capability value in scope | E language, Joe-E, Effekt, Scala capture checking | A second vocabulary beside identities |
| RAD-D11-06 | D11-6 Protocol types per resource | open/read/close is a session type, not a set of pre/post pairs | Session types, Singularity channel contracts | Protocol threading in hot I/O loops |
| RAD-D11-07 | D11-7 Compensating transactions | External failure is handled by declared compensation rather than typed outcomes | Sagas (Garcia-Molina & Salem) | (outlandish for a systems language) Compensation is not always possible |
| RAD-D11-08 | D11-8 Refinement against a verified OS spec | Contracts are stated against a machine-checked kernel specification | seL4, CertiKOS | Ties the language to one verified platform |
| RAD-D11-09 | D11-9 MMIO as typed maps with fences | Device registers are typed storage with explicit ordering effects | Low* MMIO models (unverified), Rust volatile access | Ordering must enter the effect vocabulary |
| RAD-D11-10 | D11-10 DMA as a fenced foreign window | A window is foreign for a bounded span delimited by proved fences | (none known as a language rule) | Fence placement becomes a proof obligation |
| RAD-D11-11 | D11-11 Mandatory typed failure outcomes | Every external operation returns a failure variant; there is no other path | SPARK, Rust `Result`-only I/O | Matches the constitution's environment-failure requirement directly |
| RAD-D11-12 | D11-12 External resources outside the language | A trusted adapter layer implements them; source never mentions them | libc as the TCB, Miri shims | Contradicts the constitution's "ordinary objects" section head-on |

## D12 Surface form and sugar

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D12-01 | D12-1 No surface syntax | The source is a canonical serialized AST the agent emits; no parser, no formatting rules | Unison codebase-as-database, JetBrains MPS projectional editing | Human review needs a renderer; diffs become structural |
| RAD-D12-02 | D12-2 Source in SSA or ANF | Every binding is a definition, so identity is syntactic | Swift SIL, MLIR, Appel "SSA is functional programming" | Memory operations still need a store model; loops need phi identities |
| RAD-D12-03 | D12-3 Source is a proof script | The writer writes proofs; the program is extracted | Coq, F*, Isabelle, CakeML | Extracted-code performance; the spec absorbs a proof assistant |
| RAD-D12-04 | D12-4 Source is Datalog | The program is rules; the compiler schedules evaluation | Soufflé, Bloom/Dedalus, LogicBlox | (outlandish) Imperative control and layout are inexpressible |
| RAD-D12-05 | D12-5 Source is a state machine table | Transitions, not statements | Esterel/SCADE, P (Desai et al), Ragel, Statecharts | Verbose for straight-line computation |
| RAD-D12-06 | D12-6 Algorithm plus schedule | Two artifacts with two checkers, composed by a legality relation | Halide, Exo, Elevate | Interprocedural scheduling is unsolved (unverified) |
| RAD-D12-07 | D12-7 Two-level staging | A meta program generates the checked object program | MetaML, Terra, LMS, Zig comptime | Only the object program is checked; meta errors are late |
| RAD-D12-08 | D12-8 Source is the specification | The writer writes requirements; the compiler synthesizes code nobody edits | Synthesis; Dafny-to-code (unverified) | (outlandish) Performance control disappears |
| RAD-D12-09 | D12-9 Content-addressed declarations | A declaration's identity is the hash of its checked form; renaming is free | Unison | Diagnostics and diffs must be rendered, not read |
| RAD-D12-10 | D12-10 Diagnostics as repair patches | A rejection ships a machine-applicable edit, not prose | rustfix, clippy autofix (practice); (as a language rule: none known) | Every rule needs a repair function; DIAG-1 grows |
| RAD-D12-11 | D12-11 One spelling per meaning, rendered views | No sugar in the file; the tooling renders a readable view | MPS projectional editing | Tooling becomes mandatory for humans |
| RAD-D12-12 | D12-12 `&`/`&uniq` as pure sugar | Retained as an expansion to pointer plus state clauses, printed in diagnostics | The map's migration option | Two mental models coexist during migration |
| RAD-D12-13 | D12-13 Region syntax with no lattice | `'s` names identity only; outlives, liveness and nesting are deleted outright | The map's section 6 retirement list, taken to its limit | Everything OWN-3/OWN-4/OWN-10 did must be re-expressed as state |
| RAD-D12-14 | D12-14 Written proof budgets | Each declaration states a proof-cost budget and the compiler reports the actual | (none known) | Makes compile latency a checked fact; risks becoming a budget that selects acceptance (M1) |

## D13 Validation and migration strategy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| RAD-D13-01 | D13-1 Mechanized metatheory first | Build the model and prove soundness before implementing | RustBelt (Jung et al), Iris | Slow; but the only method that answers "is it sound" |
| RAD-D13-02 | D13-2 Exhaustive small-model enumeration | Enumerate all programs within a small scope and compare against a reference semantics | Alloy (Jackson), Nitpick; the repo's own access-state experiment | Small-scope hypothesis is an assumption, not a proof |
| RAD-D13-03 | D13-3 Certificate-producing checking | The checker emits a proof a small kernel re-checks; the kernel is the TCB | PCC (Necula & Lee), DRAT/LFSC, de Bruijn criterion | A second format to maintain across spec versions |
| RAD-D13-04 | D13-4 Executable semantics as oracle | An independent interpreter defines behavior; the compiler is tested against it | K framework, PLT Redex, Miri for Rust | Two definitions can disagree; which is normative |
| RAD-D13-05 | D13-5 Differential testing against Miri | Translate the corpus to Rust and compare against Stacked/Tree Borrows verdicts | Miri, Stacked Borrows (Jung et al 2020), Tree Borrows | Only covers the intersection of the two models |
| RAD-D13-06 | D13-6 Random program generation | Generate programs plus a reference interpreter and compare | Csmith (Yang et al 2011), EMI (Le, Afshari, Su 2014) | A generator that produces provable programs is itself hard |
| RAD-D13-07 | D13-7 Bounded model checking of the checker | Verify the entailment engine's own properties within a bound | CBMC, Kani, SAW | Bounds are a budget; but on the checker, not on acceptance |
| RAD-D13-08 | D13-8 Verified front end | Extract the type checker from a proof | CompCert, CakeML | Extracted checkers are slow; large engineering cost |
| RAD-D13-09 | D13-9 Dogfooding | Write the compiler in Whitefoot; its own passes are the discriminating corpus | Self-hosting practice | Bootstrap cost; slow feedback |
| RAD-D13-10 | D13-10 Discriminator: push with a live element pointer | `len < cap` decides validity of an interior pointer across a `push` | The map's ladder Step 3 | Separates Rust-style invalidation from proof-based validity |
| RAD-D13-11 | D13-11 Discriminator: intrusive doubly linked list | O(1) unlink with back pointers and no tree ownership | Kernel `list_head`; Rust's difficulty here is well known | Separates ownership-tree models from identity-plus-state models |
| RAD-D13-12 | D13-12 Discriminator: hash map with tombstones and in-place rehash | Holes, relocation and index arithmetic in one structure | Standard open-addressing maps | Exercises D3 holes and D9 backings simultaneously |
| RAD-D13-13 | D13-13 Discriminator: an arena allocator written in the language | The program hands out interior storage it will later invalidate | Cyclone, Zig arenas, region libraries | Separates capability/provider models from plain state models |
| RAD-D13-14 | D13-14 Discriminator: zero-copy parser over an mmap | Slices into storage another agent may write | Practice; the map's `foreign` question | Exercises D11 foreign qualifier and D9 backings |
| RAD-D13-15 | D13-15 Discriminator: union-find with path compression | Aliased mutation of shared nodes with no tree structure | Standard algorithm | Separates models that require exclusivity from those that do not |
| RAD-D13-16 | D13-16 Discriminator: work-stealing deque | Single writer, multiple readers, weak memory | Chase-Lev deque | Exercises D10-8 and D5-15 |
| RAD-D13-17 | D13-17 Discriminator: free list threaded through free pages | Storage that is simultaneously dead as data and live as metadata | Kernel page allocators | Exercises D3's state vocabulary at its limit |
| RAD-D13-18 | D13-18 Discriminator: in-place sort with two cursors | Two writable paths into one backing, provably disjoint by arithmetic | Quicksort partition | Exercises D4 and D9-7 |
| RAD-D13-19 | D13-19 Migration by fork | Run both models on one corpus in parallel and delete the loser | (none known as a repository practice) | Doubles maintenance during the overlap |
| RAD-D13-20 | D13-20 Migration by second normative chapter | Add the candidate as a parallel chapter with a translation; retire one after both pass | (none known); spec archive rules make it cheap | Two normative texts violate "supersede in place" unless time-boxed |

## Decision points missing from this list

| id | Missing decision point | Why it is a decision, not a detail | Where it bites |
|---|---|---|
| RAD-DPX-01 | M0 Memory and concurrency model | SC-only, a proved weak model, or no atomics at all is a language choice nothing in D0..D13 pins | D5, D10; every lock and epoch option |
| RAD-DPX-02 | M1' Closures and higher-order functions | Where captured identities live and how a closure's row is written | FN-5 already exists; this branch measures closure cost |
| RAD-DPX-03 | M2' Generics over identities and effects | Variance, instantiation, monomorphization versus dictionaries | D6, D7; `region_params` today |
| RAD-DPX-04 | M3' Dynamic dispatch and existential objects | A vtable call's effect row and identity parameters have no answer | D7; opaque nominals today |
| RAD-DPX-05 | M4' Recursive and cyclic structures | Identity for self-referential and cyclic data; back pointers | D1, D9 |
| RAD-DPX-06 | M5' Termination and totality | EFF-3 already needs a termination proof v0 lacks | D0-22; parallel and pure-call optimization |
| RAD-DPX-07 | M6' Resource bounds | Memory, stack depth, allocation counts as verifiable facts | Constitution's Safety section; RAML-style derivation |
| RAD-DPX-08 | M7' Failure and unwinding | Whether any abnormal termination exists, and what discharges linear obligations on such a path | D0-16; R2 |
| RAD-DPX-09 | M8' Allocation failure | Typed outcome, provider-declared bound, or abort | D8, D11 |
| RAD-DPX-10 | M9' Arithmetic overflow policy | Which deterministic family proves domains; wrapping types versus proofs | Spec excludes silent overflow; mechanism is open |
| RAD-DPX-11 | M10' Data layout control | Unions, bit fields, padding, `repr`, uninitialized padding bytes | D3-15; FFI |
| RAD-DPX-12 | M11' Volatile and ordering | What the optimizer may move across a device access | D11-9 |
| RAD-DPX-13 | M12' Separate compilation and incrementality | Directly contradicts or spends PROG-1 | D0-18, D7 |
| RAD-DPX-14 | M13' Compile-time evaluation and staging | Whether proofs and code may run at compile time | D12-7 |
| RAD-DPX-15 | M14' Debuggers and profilers | External readers of storage the model does not admit | D0-23 |
| RAD-DPX-16 | M15' Pointer equality and object identity | Whether identity is observable program semantics or only a checker notion | D1 entirely |
| RAD-DPX-17 | M16' Zero-copy deserialization | Typing bytes an external agent wrote | D11, D9 |
| RAD-DPX-18 | M17' Proof and certificate evolution | What happens to a checked-in certificate when the spec version changes | D7-3, D13-3 |
| RAD-DPX-19 | M18' TCB boundary | What a linked definition may assume under SCOPE-3 | D11-2, D0-10 |
| RAD-DPX-20 | M19' Diagnostics as machine-readable repairs | The form a rejection takes for an AI writer | D12-10, OWN-8 |
| RAD-DPX-21 | M20' Cost model exposure | What the writer may assume about layout, inlining and allocation | D0-24, D8-6 |
| RAD-DPX-22 | M21' Optimizer authority | Whether the backend may use facts the source never stated | R4 (c) |

## Requirements missing or misstated (for D0)

| id | Requirement | Missing, misstated, or dependent | Ground |
|---|---|---|
| RAD-REQ-01 | Bounded resource usage | Missing entirely from R1..R10 | Constitution, Safety: "machine-verifiable bounds on memory and other hardware resource usage" |
| RAD-REQ-02 | Defined behavior on environment failure | Missing | Constitution, Safety: "Expected input and environment failures must have defined program behavior" |
| RAD-REQ-03 | Evolution and compatibility | Missing | Constitution has a whole section; signatures, proofs and certificates all evolve |
| RAD-REQ-04 | Compile latency at project scale | Missing; M1 covers determinism but not cost | Constitution, Performance: "guaranteed termination alone is insufficient" |
| RAD-REQ-05 | Abstraction (generics, closures, dispatch) | Missing; R1..R10 are entirely first-order | Constitution: construction at the scale of kernels, compilers, browsers |
| RAD-REQ-06 | Repairable rejection | Missing; OWN-8 rejects sound programs, so the named restructuring is part of the requirement | OWN-8, DIAG-1 |
| RAD-REQ-07 | Proof stability under edit | Missing; an AI writer edits constantly and must not reopen every proof | Constitution: maintenance and evolution at scale |
| RAD-REQ-08 | Teachability within a context budget | M4 states verbosity tolerance, not the actual constraint | Constitution: the harness is for AI agents |
| RAD-REQ-09 | R4 | Misstated as a requirement; its own (d) clause says a missing fact costs speed, never correctness | Map section 2, R4 (d) |
| RAD-REQ-10 | R6 | Misstated; frame retention is checker precision, a property of the implementation | Map section 2, R6 (d) |
| RAD-REQ-11 | R7 | Misstated; "signature-only" is a mechanism, and the requirement is bounded recheck cost under edit | FN-1 is a rule, PROG-1 is a premise |
| RAD-REQ-12 | R9 | Not independent: R1 precision sequentially, R5 concurrently | Map section 2, R9 (a) |
| RAD-REQ-13 | R2 | Not independent of R3 once resources are linear values | Austral, Linear Haskell |
| RAD-REQ-14 | R3 | Not independent of R1 by the map's own ladder: a transfer is a state event | Map section 3, Step 1 |
| RAD-REQ-15 | R5 and R4 | One relation with two consumers, stated twice | Map sections 2 and 5 |
| RAD-REQ-16 | M1 | A mechanism: the requirement is reproducible, predictable acceptance | Polonius, RAML: deterministic and terminating but solver-shaped |
| RAD-REQ-17 | M2 | A mechanism: its own typed-outcome clause admits generational and epoch checks | M2's own wording; Vale, Swift |
| RAD-REQ-18 | M3 | A mechanism: "no escape" could be "no escape without a machine-checked obligation" | SCOPE-3 already trusts linked definitions |
| RAD-REQ-19 | M5 | A mechanism: it presumes the writer chooses the shape rather than a schedule | Halide separates algorithm from schedule |
| RAD-REQ-20 | FN-1 versus PROG-1 | Two premises in tension: a closed world makes signature-only checking a cost choice | FN-1, PROG-1 |


# Lens file: numbered-systems.md


Enumeration only. No ranking, no scoring, no recommendation, no pruning.
Lens: how kernels, databases, game engines, HPC/GPU code, allocators, browsers
and JITs manage identity, aliasing, lifetime, relocation, sharing and
parallelism *without* a type system, what a language could reify from that, and
what LLVM and other backends can actually consume.

Builds on EVIDENCE part B (LLVM fact forms, D1–D12) and part D (threads, locks,
external) rather than repeating them. Where a row restates a part-B/part-D fact
it is because a *different* source discipline arrives at the same construct.

"Backend fact" names what the option would let the emitter state. "(outlandish)"
marks options kept deliberately. "(unverified)" marks a citation or claim I did
not re-check.

---

## D0 The requirements themselves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D0-01 | R11 Layout and ABI control as a stated requirement | Padding, alignment, cache-line placement, false-sharing avoidance and SoA/AoS choice are correctness-adjacent performance requirements no R1–R10 row owns | Linux `____cacheline_aligned_in_smp`, DPDK `__rte_cache_aligned`, ISPC `soa<N>` types, Arrow buffer alignment | Backend fact: `align(n)`, vectorization legality, struct layout. Tension: R8 says "where a value lives" but never "how wide the line is" |
| SYS-D0-02 | R12 Memory ordering as a stated requirement | Nothing in R1–R10 mentions acquire/release, fences, or what a load may be reordered past | Linux `Documentation/memory-barriers.txt` and the LKMM (Alglave et al.), C11/LLVM atomic orderings, ARM/POWER models | Needs an ordering vocabulary before D10 can be decided. Tension: R5 promises race freedom yet the word "fence" appears nowhere |
| SYS-D0-03 | R13 Allocation failure and bounded allocation | "Every allocation released once" says nothing about allocation that *fails*, or about contexts where allocation is forbidden | Linux `GFP_ATOMIC` vs `GFP_KERNEL`, `__must_check`, realtime no-allocate rules, seL4's no-kernel-heap design | Needs a typed-outcome contract and a "may not allocate here" effect. Tension with M2: the failure is representable, the *context rule* is not |
| SYS-D0-04 | R14 Secret erasure and side-channel discipline | Zeroization must survive dead-store elimination; constant-time code must survive branch/select rewriting | `memzero_explicit`, `explicit_bzero`, OpenSSL `OPENSSL_cleanse`, Rust `zeroize`, `-fzero-call-used-regs` | Directly collides with "proofs may authorize check removal and optimization": the erasure is a store the optimizer has every right to delete |
| SYS-D0-05 | R15 Compile-time cost as a first-class requirement | M1 constrains *determinism*, not *cost*; a closed-world whole-program check is an unbounded cost even when perfectly deterministic | LLVM ThinLTO summaries exist precisely because full LTO did not scale; Polly ships `--polly-dependences-computeout` | Makes PROG-1 a measurable liability. Tension: M1 forbids budgets selecting acceptance but says nothing about a two-hour accepting run |
| SYS-D0-06 | R16 Debuggability when the checker is wrong | With no traps and no unsafe, a soundness bug in the checker becomes a silent miscompile with no shadow state to catch it | Rust disabled `noalias` on `&mut` for years over LLVM miscompiles (unverified: re-enabled ~1.54); Miri exists for exactly this | Needs a sanitizer-parity or translation-validation story. Tension with erasure: the facts are gone before the bug is observable |
| SYS-D0-07 | R17 Memoization, lazy init and caches | The no-`RefCell` doctrine deletes every shape used for a cache, a memo table, an interned symbol, or a lazily built index | `once_cell`/`OnceLock`, Linux `static_key`/jump labels, JIT inline caches, DBMS plan caches | Needs either a first-class "write-once then frozen" state or an admission that these live outside. Tension: why-whitefoot §5 makes the absence a performance argument |
| SYS-D0-08 | R18 Non-lexical loans to a non-program agent | A buffer handed to the kernel at submission and returned at completion is a loan with no lexical scope and no program-ordered end | io_uring SQE/CQE, `IORING_REGISTER_BUFFERS`, POSIX AIO, `dma_map_single`/`dma_unmap_single` | Needs an escrow state plus a linear completion token. R2 assumes every obligation has a program-ordered discharge point |
| SYS-D0-09 | R19 Multiple address spaces | A pointer's *space* is part of what it names; dereference legality and aliasing both follow from it | CUDA `__global__`/`__shared__`/`__constant__`/`__local__`, NVVM addrspace(1/3/4/5), OpenCL/SYCL qualifiers, Linux `__user`/`__iomem` | Backend fact: address-space-based AA (target-dependent, e.g. `AMDGPUAliasAnalysis`; general LLVM does *not* assume cross-space non-aliasing — unverified) |
| SYS-D0-10 | R20 Relocation by a third party | R8 covers relocation the program performs; nothing covers a compacting collector, a defragmenting allocator, or `mremap` moving storage under the program | Blink Oilpan moving GC, V8 handle scopes, mimalloc/jemalloc purge, `mremap`, LeanStore eviction | Requires handles, not pointers, at every escape point. Tension: interior pointers (R8) and third-party relocation are mutually exclusive without a fixup channel |
| SYS-D0-11 | R21 Termination, deadlock freedom and deadlines | R2 mentions bounded memory; nothing mentions bounded time, blocking, or lock ordering | Linux lockdep, Chalice `waitlevel`, WCET analysis, `might_sleep()`, Clang `[[clang::nonblocking]]` function effects (paper number unverified) | Needs an effect axis orthogonal to memory. Tension: M2's "no trap" says nothing about a program that never returns |
| SYS-D0-12 | R22 The allocator writes inside "dead" storage | Free-list links, slab metadata and poison patterns live *inside* storage the program considers Gone; "Gone means untouched" is false in every real allocator | Linux SLUB free pointer in the object, glibc tcache `next` in the chunk, `SLAB_POISON` | The model must state whether Gone storage may be written by the runtime, and whether `noalias`-return still holds. Tension with R1's Gone being terminal and inert |
| SYS-D0-13 | R23 A weaker determinism level than source-order equality | Many systems need only reproducible *output*, not statement-order equality; PAR-1 promises the strongest form available and pays for it in denied overlap | Intel MKL Conditional Numerical Reproducibility, ReproBLAS, MPI reproducible reductions, `-ffp-contract` | Would let D5 offer a menu of guarantees. Tension: CAP-1 fixes one guarantee for everything |
| SYS-D0-14 | Challenge M2: no runtime check is a mechanism, not a requirement | The requirement is presumably "no unpredictable trap and no cost the writer did not choose"; a perfectly predicted never-taken branch, or a hardware bounds check, is not that cost | CHERI bounds checks in the load/store unit at ~no IPC cost; branch predictors on invariant conditions; ARM MTE tag compare in the LSU | Restates the whole space: the disqualification of Vale generational references and of Mezzo adoption in MAP §5 rests on M2 read literally |
| SYS-D0-15 | Challenge M1: "budget-free" vs the closed world | A whole-program, deterministic, terminating analysis can still be exponential; ISL's Presburger arithmetic is deterministic *and* needs a compute-out in practice | Polly/ISL `--polly-dependences-computeout`, whole-program alias analysis cost curves | Either M1 must add a complexity bound or the closed world must go. The two premises push opposite ways |
| SYS-D0-16 | Challenge FN-1 vs PROG-1 | Signature-only judgement and closed-world compilation are each a design choice; holding both means paying modularity's cost without taking modularity's benefit (separate compilation) | LLVM does the opposite at each level: modular `declare` attributes *plus* ThinLTO summaries that cross the boundary | Pick which one is load-bearing, or state why both. Real backends treat the signature as a cache of what whole-program analysis would find |
| SYS-D0-17 | Non-independence: R4 and R5 are one fact-base | Distinctness-of-footprints serves the optimizer and the scheduler; only the consumer differs (LLVM metadata vs a spawn judgement) | Legion privileges feed both a type check and a runtime DAG; SYCL accessors feed both | If they are one requirement, D5's menu collapses into D4's |
| SYS-D0-18 | Non-independence: R6 is R4 run backwards | "Which facts die at a write" and "which accesses cannot alias" are the same disjointness query at two program points | `!alias.scope`/`!noalias` tag *accesses*; ENT-5 kills *facts*; both key on the same identity set | If they are one requirement, identity granularity (D1) is the only real knob |

## D1 The unit and naming of storage identity

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D1-01 | Allocation named by its call site | Identity is the `alloc` that produced it; freshness is the whole fact | LLVM `noalias` return, `allockind`/`allocsize`/`allocptr` allocator-function attributes (spelling unverified) | Backend fact: `noalias` on the returned pointer. Says nothing about two elements of one allocation |
| SYS-D1-02 | (cache, slot) from a type-stable pool | Identity is the slab plus the slot index; the object may be reused, the *type* of the storage never changes | Linux `kmem_cache`, `SLAB_TYPESAFE_BY_RCU` ("memory may be reused by an object of the same type"), tcmalloc size classes | Backend fact: `!tbaa` root per cache; cross-cache non-aliasing for free. Tension: identity survives the object, so R1's Gone is not terminal |
| SYS-D1-03 | Handle = (index, generation) | Identity is a plain copyable value; staleness is detected by comparison, not by the type | Unity DOTS `Entity{Index,Version}`, EnTT/Bevy entities, Rust `slotmap`, why-whitefoot §10 | Runtime mechanism: one compare per lookup. Backend fact: none directly; enables `noalias` on the whole pool |
| SYS-D1-04 | Index only, into one named pool | No generation; reuse is avoided by never recycling (append-only) | why-whitefoot §10 current witness, arena-indexed ASTs, Rust `Vec`-index folklore | Backend fact: the pool is one `noalias` object; elements need index disjointness. Tension: no reclamation |
| SYS-D1-05 | Base+bounds+permissions in the pointer | Identity, extent and rights all travel in the pointer word, enforced by hardware | CHERI capabilities (base, length, perms, otype, tag bit), CheriBSD | Runtime mechanism: 128-bit pointers, tagged memory. Backend fact: bounds are a machine fact, not an IR one. Tension: M2 read literally |
| SYS-D1-06 | Capability-table slot | The program holds an index into a table it cannot forge or dereference; the kernel owns the object | seL4 CNode/CSpace, `seL4_CNode_Move`, file descriptors, Capsicum | Backend fact: none; identity is not a pointer at all. Tension: no interior pointers, ever |
| SYS-D1-07 | Address space plus pointer | Identity = (space, value); legality of dereference and disambiguation both follow from the space | CUDA `__shared__`/`__constant__`, NVVM addrspaces, Linux sparse `__user`/`__iomem`/`__percpu` | Backend fact: address-space AA where the target supports it (`AMDGPUAliasAnalysis`); `addrspacecast` at boundaries |
| SYS-D1-08 | Logical region + partition color | Identity is a node in a region tree; disjointness comes from the *kind* of the partition that made it | Legion/Regent logical regions and disjoint/aliased partitions (Bauer SC 2012, Slaughter SC 2015) | Backend fact: per-subregion `noalias`. Tension: Legion computes partition disjointness *dynamically*, which is the half WF forbids |
| SYS-D1-09 | (page_id, frame) with a pin | Identity is a database page; a pointer into it is valid only while the pin count is nonzero | PostgreSQL buffer manager, InnoDB buffer pool, LeanStore | Runtime mechanism: pin/unpin. Backend fact: `dereferenceable` valid exactly for the pin extent |
| SYS-D1-10 | Epoch-scoped identity | The identity exists only inside a read-side critical section; outside it, nothing is named | RCU `rcu_read_lock()`/`rcu_dereference`, crossbeam-epoch, folly hazptr | Backend fact: `!invariant.group` per epoch window is a near-fit; needs `launder` at the boundary |
| SYS-D1-11 | (symbol, CPU) per-CPU identity | Identity includes which core owns it; distinctness from other cores is structural | Linux `DEFINE_PER_CPU`, `this_cpu_ptr`, DPDK per-lcore, Seastar shards | Backend fact: no atomics needed; every per-CPU access is `noalias` against every other core's. Tension: needs a context effect ("preemption disabled") |
| SYS-D1-12 | Per-column identity in SoA | The struct has no identity; each field-array does | why-whitefoot §5 `Cols`, Arrow record batches, ClickHouse columns, ISPC `soa<N>` | Backend fact: exactly EVIDENCE D1 — one `!alias.scope` per column on loaded data pointers |
| SYS-D1-13 | Half-open range `[lo,hi)` as identity | The named thing is a byte or element extent, not an object | LLVM `initializes((Lo,Hi))` is the only range-shaped attribute; Halide/TVM bounds inference; `omp simd safelen` | Backend fact: `initializes`, per-tile `!alias.scope`. Tension: LLVM has no range-shaped `readonly` |
| SYS-D1-14 | Tile identity derived from a schedule | The algorithm names no identity; a separate schedule language splits, reorders and names tiles | Halide `split`/`tile`/`vectorize`, TVM `te.Schedule`, Dex index sets | Backend fact: `llvm.experimental.noalias.scope.decl` inside the loop body, duplicated on unroll |
| SYS-D1-15 | Stack slot with an explicit live range | Identity is an `alloca` plus a lifetime interval the source states | `llvm.lifetime.start`/`end`, stack coloring — per EVIDENCE §A.6 the one *live* backend consumer today | Backend fact: slot reuse. Cheapest identity kind to add and the only one already consumed |
| SYS-D1-16 | Brand introduced by a scope (generativity) | A scope mints a fresh type-level name; only values branded with it may index the container | Haskell `runST`, Rust `generativity`/`indexing` crates, `GhostCell` | Backend fact: bounds checks vanish with no `!range` and no proof obligation at use. Tension: brands do not survive storage in a struct |
| SYS-D1-17 | Existential identity per formation | Opening a container yields a fresh unusable-after-change identity | L3/Alias Types `exists`, LeanStore optimistic version *without* the runtime recheck | Backend fact: `!invariant.group` window + `llvm.launder.invariant.group` at the change. Both experimental in LLVM |
| SYS-D1-18 | The MMU mapping is the identity | Identity is a VMA/page; two virtual addresses may name one physical page deliberately | `mmap(MAP_SHARED)` double mapping, ring-buffer mirror trick, `mremap` | Backend fact: none — and it *breaks* provenance-based AA, since two distinct pointers legitimately alias (unverified as a source-level mechanism) |
| SYS-D1-19 | Nothing: identity by runtime pointer comparison | The baseline every C program uses; `p == q` is the only aliasing fact | C, and every dynamic alias check LLVM's loop versioning emits | Runtime mechanism: 29 guards in the measured Rust SoA kernel (why-whitefoot §5). Retained as the null option |
| SYS-D1-20 | (outlandish) Cache-line color as identity | The type names which cache set the object lives in; false sharing becomes a type error | Page coloring in OS research, `____cacheline_aligned_in_smp` as the manual version | Needs allocator cooperation and a layout algebra. Backend fact: `align`, plus a scheduling fact no IR can express |
| SYS-D1-21 | (outlandish) Hardware memory tag as identity | The pointer carries a 4-bit tag; relocation retags; an aliasing violation faults in the LSU | ARM MTE (`irg`, `stg`), SPARC ADI, HWASan | Runtime mechanism: tagged memory. 16 identities before reuse; tag collisions are probabilistic, which no static model tolerates |
| SYS-D1-22 | (outlandish) NUMA node + allocation site | Identity carries placement, so effect rows also say *where* and the backend can emit affinity | `numa_alloc_onnode`, `hugetlbfs`, `cudaMallocManaged` prefetch hints | Backend fact: scheduling info, not IR metadata. Turns D1 into a placement language |

## D2 Where permission and state live

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D2-01 | In an external checker keyed on per-function annotations | The type system is untouched; a separate deterministic tool holds the state and the source carries qualifiers | Linux `sparse` (`__acquires`, `__releases`, `__must_hold`, `__user`, `__rcu`) over ~30M lines, no SMT | Deployed proof that this scales. Tension: the checker is advisory, so it has never had to be sound |
| SYS-D2-02 | In Microsoft-style declaration annotations | Pre/post state per parameter written as attributes, checked by a deterministic engine | SAL2 `_In_`, `_Out_writes_(n)`, `_When_`, `_Acquires_lock_`, `_Must_inspect_result_`; Windows Static Driver Verifier | Direct match for entry/exit states (R7). Tension: SAL's checker is heuristic, unlike M1 |
| SYS-D2-03 | In explicit ghost linear tokens | The writer threads a permission value through every call | Verus `Tracked<PointsTo>`, L3, Chalice `fork tk`/`join tk` | Erased; no runtime cost. Tension: M4 says verbosity is cheap, which makes this the *cheapest* option under WF's own premises |
| SYS-D2-04 | In the reference itself (a mode bit) | Rust `&`/`&mut`; WF today | Rust, WF OWN-2 | Backend fact: parameter `noalias`/`readonly` only. Per EVIDENCE §B.2, cannot name a loaded pointer |
| SYS-D2-05 | In the pointer at runtime, enforced by hardware | Permission bits travel in the capability; the load unit checks them | CHERI perms (Load, Store, Execute, LoadCap, StoreCap), `CSetBounds`, `CAndPerm` | No compiler proof needed at all. Tension: M2 literally; but the check costs no branch and no IPC |
| SYS-D2-06 | In the allocator's own metadata | The state already exists where the allocator keeps it: slab freelists, `page->flags`, buddy order | Linux SLUB/buddy, jemalloc extent state, glibc chunk flags | Zero new mechanism; the model just names what is there. Tension: it is runtime state, so proofs would have to read it |
| SYS-D2-07 | In a guard object whose destruction restores state | RAII: acquiring yields a value; dropping it releases | C++ `lock_guard`, Rust `MutexGuard`, Linux `guard(mutex)(&lock)` cleanup attribute, Part D §B.3's linear `Guard<'meta>` | Compile-time analogue is a linear guard. Direct fit for D10 |
| SYS-D2-08 | In a per-thread context word | "In an IRQ", "preemption disabled", "in an RCU read section" are ambient facts a register holds | Linux `preempt_count`, `in_atomic()`, `might_sleep()`, lockdep | A *static* context effect would replace a runtime counter. Nothing in R1–R10 has a place for ambient context |
| SYS-D2-09 | In the calling convention | Permission is a parameter convention, not a type: `in`/`out`/`inout`/`consuming`/`borrowing` | Swift/Hylo parameter conventions, Ada `in out`, Fortran `intent(inout)` | Backend fact: maps straight to `byval`/`sret`/`readonly`/`writeonly`/`initializes`. 40 years of deployment |
| SYS-D2-10 | In an accessor object created per launch | The accessor *is* the permission, and a runtime builds the dependency graph from the set of accessors | SYCL `accessor<T,D,access::mode::read_write,target::device>`, oneAPI buffers | Runtime mechanism: a task DAG. Backend fact: scheduling info, not IR |
| SYS-D2-11 | In a job's declared component set | A system declares which component arrays it reads and writes; a scheduler derives conflicts | Unity DOTS `[ReadOnly] ComponentTypeHandle<T>`, Bevy system-param conflict detection | In a closed world (PROG-1) the schedule could be computed at compile time instead of startup |
| SYS-D2-12 | In a whole-program table | One global store typing computed by the compiler; nothing in the source | LTO/ThinLTO summaries, whole-program alias analysis, Legion's runtime as the dynamic version | Tension with R15/M1: cost, and with FN-1: no signature needed |
| SYS-D2-13 | In an OS-owned descriptor table | The program holds no permission, only an index; the kernel holds the state | seL4 CSpace, POSIX fds, Windows HANDLE table | No source mechanism at all for in-program memory. Fits D11 exactly |
| SYS-D2-14 | In shadow memory, materialized only in debug builds | Release erases it; debug keeps a byte-per-byte state map | ASan/MSan/TSan shadow, Valgrind V-bits, HWASan | A validation channel (D13), not an acceptance one. Gives R16 its missing answer |
| SYS-D2-15 | In the IR metadata only, nowhere in the source | The source has no permission notion; the emitter manufactures scopes from resolved paths | The retired `democ` prototype (EVIDENCE §B.2: "it manufactured a scope from a static path, on the fly, at emission") | Already demonstrated. Tension: the fact is not cross-procedural and not in a signature |
| SYS-D2-16 | In a compiler-enforced custom lint outside the type system | A domain rule the type system cannot state, enforced by a bespoke pass | Servo's `unrooted_must_root` lint for GC rooting, Rust clippy-style plugins | Precedent for putting one hard rule outside the type system rather than distorting it |
| SYS-D2-17 | (outlandish) In page-table permission bits | Taking a unique loan `mprotect`s every other mapping read-only for the loan window | GC write barriers via `mprotect` (Boehm), Windows guard pages, hardware watchpoints | Runtime mechanism: TLB shootdowns. Enforcement at page granularity, which no loan respects |
| SYS-D2-18 | (outlandish) In memory tags flipped at loan boundaries | A loan retags the region; a stale pointer's tag mismatches and faults | ARM MTE `irg`/`stg`, HWASan's tagging discipline | Probabilistic (1/16 false accept). Runtime mechanism only; no static fact reaches the optimizer |

## D3 The state vocabulary of a storage

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D3-01 | Init / Uninit / Gone | The minimum the hazard ladder needs | MECHANISM-MAP §3 | Backend fact: `initializes` from Uninit→Init. Baseline |
| SYS-D3-02 | Add Poison / Undef | Bytes that exist but whose value is not merely unknown — reading them is worse than reading garbage | LLVM `poison` vs `undef` vs `freeze`, MSan's uninitialized shadow | Needed to state what the backend may assume after a partial write. Tension: WF has no UB, so `poison` has no source meaning today |
| SYS-D3-03 | Add Pinned | The storage may not be relocated while pinned, because something points into it | Rust `Pin`/`pin_init` and Rust-for-Linux, `pin_user_pages()` vs `get_user_pages()`, `mlock`, buffer-pool pins | Precondition for interior pointers surviving anything. Backend fact: forbids the relocating memcpy |
| SYS-D3-04 | Add Reserved / Committed / Decommitted | Address space may exist without backing pages; backing may vanish without the mapping going away | `VirtualAlloc(MEM_RESERVE/MEM_COMMIT)`, `madvise(MADV_DONTNEED)`, `MADV_FREE` (contents may vanish, mapping stays) | A genuinely third state Init/Uninit/Gone cannot express. Enables the reserve-huge-then-commit container (see D8) |
| SYS-D3-05 | Add Pinned-in-pool / Evictable | The storage is resident because someone holds a pin; otherwise it may be evicted | PostgreSQL `PinBuffer`/`UnpinBuffer`, InnoDB, LeanStore | Runtime mechanism: a pin count. Static version is a linear pin token |
| SYS-D3-06 | Add Published / Unpublished | The storage is or is not visible to concurrent readers | `rcu_assign_pointer` (release store), `__rcu` sparse qualifier, Java safe publication | Backend fact: the publishing store must not be reordered. Bridges D3 and D12's ordering gap |
| SYS-D3-07 | Add Quiescing (freed but not reclaimable) | Storage logically dead but still reachable by in-flight readers | `call_rcu`/`synchronize_rcu` grace periods, epoch reclamation, hazard pointers, `SLAB_TYPESAFE_BY_RCU` | Every lock-free allocator needs this state. R1's binary Gone cannot express it |
| SYS-D3-08 | Add Dirty / Clean | A write creates a write-back obligation to some other tier | Page cache, buffer pool, GC card tables, `msync` | Turns R2's release accounting into a two-tier obligation. Backend fact: none; scheduling and I/O |
| SYS-D3-09 | Add Frozen | Write-once then permanently immutable | `memfd` `F_SEAL_WRITE`, Java `final` freeze semantics, `Object.freeze`, CONST-2 | Backend fact: `!invariant.load` (program-wide) — per EVIDENCE D8 this is the *only* WF fact strong enough |
| SYS-D3-10 | Add Zeroed | Stronger than Uninit, weaker than Init; the security-relevant middle state | `calloc`, `__GFP_ZERO`, BSS, `INIT_ON_ALLOC_DEFAULT_ON`, `-ftrivial-auto-var-init=zero` | Lets the backend elide a memset and lets R14 state its requirement |
| SYS-D3-11 | Add Device-owned | Between map and unmap the storage belongs to a DMA engine and the CPU may not touch it | `dma_map_single`/`dma_unmap_single`, `dma_sync_single_for_cpu`/`for_device` | An affine transfer to a non-program agent. Nothing in R1–R10 has this shape |
| SYS-D3-12 | User-declared typestate automata | Each nominal declares its own state machine and transitions | Vault/Fugue, Windows SDV driver state rules, `sk_state` in the kernel's sockets | M4 cost: one automaton per type to teach. Backend fact: none directly; enables `memory()` per state |
| SYS-D3-13 | Layout-version / generation counter | A monotone counter distinguishes this occupant of the storage from the last | ECS `Version`, DBMS LSN, LeanStore version, `seqcount_t` | Runtime mechanism unless statically eliminated. Bridges to D9's handles |
| SYS-D3-14 | States as terms over captured conditions | `ite(c, Init, Uninit)` rather than a lattice join | CASES.md refinement; closest systems analogue is a driver's error-unwind ladder | Cost in condition atoms; collapse at thread joins (part D #3) |
| SYS-D3-15 | A lattice with a Top for "unknown" | Classical abstract interpretation | Any dataflow framework; LLVM's own `LatticeVal` | Loses exactness; the collapse rule is automatic rather than a decision |
| SYS-D3-16 | No states: validity by construction | Arrays are fully defined by a map; a hole is unrepresentable | Futhark, Dex, APL/J lineage, Halide's pure functional algorithm layer | Removes R1's whole first clause. Tension: in-place update needs uniqueness typing instead |
| SYS-D3-17 | Per-byte state | State at byte or bit granularity rather than per storage | MSan shadow, Valgrind V-bits, `initializes` byte ranges | Exact for partial init. Explodes in the checker; `initializes` is the one backend form that wants it |
| SYS-D3-18 | Validity as a separate parallel array | "Partially initialized" is not a state — it is a second container of bits | Apache Arrow validity bitmaps, Parquet definition levels | Dissolves partial initialization into ordinary data. Tension: every read must consult it, which is a runtime check |
| SYS-D3-19 | (outlandish) States indexed by coherence state | The type says whether the line is Modified, Exclusive, Shared or Invalid in this core's cache | MESI/MOESI protocols; the closest language analogue is Pony's capability denial set | No architecture exposes this. Would make false sharing a type error |
| SYS-D3-20 | (outlandish) States indexed by storage tier | Register / L1 / DRAM / NVM / disk, with explicit staging transitions in the source | Sequoia (Fatahalian et al., SC 2006), CUDA shared-memory staging, `__pipeline_memcpy_async` | A real HPC discipline written by hand today. Turns D3 into a memory-hierarchy language |

## D4 Sequential aliasing policy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D4-01 | Allow aliased writes; facts weaken where identities coincide | The C default, with the loss localized to the storages the writer let coincide | C without `restrict`; MECHANISM-MAP §7 row | Backend fact: none, exactly where coincidence occurs. M5 tension: the obvious shape stops being the fast shape |
| SYS-D4-02 | Forbid: shared-xor-mutable | Rust's rule | Rust, WF OWN-5 | Backend fact: parameter `noalias` only, per EVIDENCE §B.2 |
| SYS-D4-03 | Declared coincidence, checked | `restrict` inverted: the writer declares where aliasing *is* allowed and the checker verifies everywhere else | C99 `restrict` (trusted), CUDA `const T* __restrict__` (trusted), `__attribute__((may_alias))` for the exceptions | Backend fact: `noalias` by default with explicit escapes. Tension: every call site must discharge distinctness (MAP §7) |
| SYS-D4-04 | Read-only flag on the pointer type, no duration | `const`-style, not a loan | C `const`, LLVM parameter `readonly`, Linux `const struct` conventions | Backend fact: `readonly` on the parameter, and read-read aliasing stays free |
| SYS-D4-05 | Per-scope exclusivity windows | Exclusivity claimed at a block head and released at its end, not tied to a reference's life | C99 block-scoped `restrict`; LLVM's own `llvm.experimental.noalias.scope.decl` marks exactly this point | Backend fact: `!alias.scope`/`!noalias` valid over the block, duplicated on unroll. The closest IR construct to a "loan" that is not a reference |
| SYS-D4-06 | Forbid only cross-type aliasing | Different types never alias; same type freely may | C/C++ strict aliasing, LLVM `!tbaa` access tags with a Parent relation | Backend fact: `!tbaa`. Historically the most miscompile-prone rule in C; Linux compiles with `-fno-strict-aliasing` |
| SYS-D4-07 | Pay-per-fact: prove distinctness only where a fact is wanted | Aliasing is unrestricted; the writer discharges an obligation at the loop that needs the fact | `__builtin_assume`, `#pragma ivdep`, `omp simd safelen(n)`, ISPC `uniform` — all trusted today | Backend fact: the fact appears only at annotated sites. M5 tension: the obvious shape needs an annotation |
| SYS-D4-08 | Distinctness only at container granularity (the SoA rule) | Columns never alias; anything inside one column may | ISPC `soa<N>`, Unity DOTS chunks, Arrow record batches, why-whitefoot §5 `Cols` | Backend fact: EVIDENCE D1's per-column `!alias.scope`. Cheap, and covers the measured kernel exactly |
| SYS-D4-09 | Static where provable, dynamic where not | Ship the check only at sites the checker could not discharge | Swift `-enforce-exclusivity=checked` (dynamic for class properties and globals in release) | Runtime mechanism: one flag per dynamic site. Direct M2 violation, kept as the shipped-industry baseline |
| SYS-D4-10 | Aliased writes allowed to proved-disjoint byte ranges | Two writable pointers are fine when arithmetic proves the ranges do not meet | Fortran array sections, `omp simd`, Halide bounds inference, Polly dependence vectors | Backend fact: per-range `!alias.scope`, `initializes`. Needs an arithmetic fragment (see D5) |
| SYS-D4-11 | Aliasing allowed, but every access is a fresh load | No fact is retained across any aliasing access | `volatile`, `READ_ONCE`/`WRITE_ONCE`, `std::atomic` relaxed | Backend fact: negative — it *forbids* CSE, hoisting and vectorization. The honest cost of unrestricted aliasing |
| SYS-D4-12 | Non-aliasing asserted by the language, unchecked | The standard simply declares arguments do not alias | Fortran 77 argument aliasing rules; a documented source of real miscompiles | Baseline showing what "trusted, not checked" costs |
| SYS-D4-13 | Union-style declared aliasing with an active member | Two types share storage and the source says which is live | C unions, `__attribute__((__may_alias__))`, Linux type-punning macros | Needs a discriminant. Backend fact: `!tbaa` root merge, i.e. giving up TBAA for that storage |
| SYS-D4-14 | Emission policy rather than a typing rule | The type system permits aliasing; the emitter picks the strongest fact it can prove per loop | LLVM's own `llvm.loop.parallel_accesses` + loop versioning cascade | M5's floor becomes "whatever the emitter proved". Tension: the guarantee is no longer stable across compiler versions |
| SYS-D4-15 | (outlandish) Exclusivity enforced by page protection | A unique loan `mprotect`s the other mapping; a violating write faults | Boehm GC write barriers, Windows `PAGE_GUARD`, CRIU-style dirty tracking | Runtime mechanism: TLB shootdowns per loan. Page granularity vs field granularity |
| SYS-D4-16 | (outlandish) Exclusivity by address-space split | The writable alias and the readable alias live in different address spaces, so AA is exact by construction | CUDA `__constant__` vs `__global__`, OpenCL qualifiers | Backend fact: address-space AA. Needs an `addrspacecast` at every conversion and a target that honors it |

## D5 The source of interference facts for parallel permission and the optimizer

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D5-01 | Effect footprints over identities | Read/write sets keyed by identity, compared for disjointness | DPJ (Bocchino OOPSLA 2009), the CSL parallel rule, MECHANISM-MAP §6 | Backend fact: scheduling info + per-span `!alias.scope`. Baseline |
| SYS-D5-02 | Loans on references | Today's PAR-1 loan clause | WF PAR-1/PAR-2 | EVIDENCE §E: the loan channel is "the whole of the loss" against footprints |
| SYS-D5-03 | Fractional or counting permissions | Split a permission, recombine to write | Boyland 2003, Chalice, Viper | Bookkeeping; part D notes WF's lexical loans never need counting recombination |
| SYS-D5-04 | Declared region partitions with a disjointness kind | The partition itself asserts disjoint or aliased, and subregions inherit it | Legion `partition` (disjoint/aliased), Regent privileges, `__demand(__parallel)` (spelling unverified) | Backend fact: per-subregion `noalias` into the task. Tension: Legion checks partition disjointness at runtime |
| SYS-D5-05 | Accessor-declared access modes with a runtime DAG | Every buffer access declares its mode; a runtime derives the dependence graph | SYCL accessors, oneAPI, StarPU, OmpSs | Runtime mechanism: task DAG construction. M2/M5 tension for permission; fine as a *lowering* |
| SYS-D5-06 | Job-system component declarations checked at schedule build | Systems declare component read/write sets; conflicts are found when the schedule is built | Unity DOTS `[ReadOnly]`, Bevy system-param conflicts, Unreal task graph | In a closed world this can move from startup to compile time — the strongest argument PROG-1 has |
| SYS-D5-07 | Affine/polyhedral dependence analysis | Dependence vectors computed from index expressions over a Presburger fragment | ISL, Polly, Pluto, the Omega test | Deterministic and terminating, unlike SMT — but Polly ships a compute-out, which is exactly M1's forbidden budget |
| SYS-D5-08 | Schedule legality against a pure algorithm | The algorithm is functional; parallelism is a schedule directive checked against the dependence graph | Halide `parallel`/`vectorize`, TVM, Tiramisu | Separates "what" from "where". Backend fact: the loop nest itself, plus `llvm.loop.parallel_accesses` |
| SYS-D5-09 | Ownership transfer only | Nothing is shared; interference is impossible because only one party holds the data | Erlang, Pony `iso`, seL4 IPC, Seastar cross-shard messages | Backend fact: everything is `noalias`. Cost: copies or moves at every boundary |
| SYS-D5-10 | Purity of deep code with intents returned as values | Deep code is `pure` or `reads`; one shallow applier writes | why-whitefoot §9 command buffer, ECS command buffers, Redux/Elm-style update loops | Backend fact: `memory(none)`/`memory(argmem: read)` over most of the program |
| SYS-D5-11 | Runtime dependency tracking | The runtime discovers interference as tasks are issued | Legion, StarPU, OmpSs, Intel TBB flow graph | Explicitly excluded as permission; kept as a lowering or a fallback |
| SYS-D5-12 | Shard-by-construction | Every datum has one owning core; cross-core work is a message | Seastar/ScyllaDB shared-nothing, DPDK per-lcore, nginx workers | Backend fact: no atomics, no fences, no alias facts needed. Cost: partitioning is a program-structure decision |
| SYS-D5-13 | Phase / epoch separation | Readers and writers are separated in time, not in space | RCU read phases, game-engine double buffering, frame N/N+1 flip, CONCURRENCY-CATALOG §8 | Zero reader cost measured against ~15–25 ns for an `RwLock` read. Cost: publish latency |
| SYS-D5-14 | Declared reductions over a proved monoid | Concurrent writes to one accumulator are *permitted* because the operator is associative | `omp reduction(+:x)`, Futhark `reduce`, Legion `reduces` privilege, CUDA `atomicAdd` with per-block privatization | The most common real parallel write pattern, and no footprint model covers it. Backend fact: `llvm.vector.reduce.*`, `atomicrmw` |
| SYS-D5-15 | Declared commutativity / reduction instances | Interference allowed when order does not matter, with private copies folded at the end | Legion reduction instances, MapReduce combiners, per-CPU counters folded by `percpu_counter_sum` | Weakens determinism to "same value, unspecified order" — or keeps it if the fold order is fixed |
| SYS-D5-16 | Writer-declared loop independence consumed directly | No type system involvement: the loop says its accesses are independent | LLVM `llvm.loop.parallel_accesses`, `#pragma omp simd`, `#pragma ivdep` | Backend fact: the exact metadata LLVM consumes. Trusted in every existing deployment; WF would check it |
| SYS-D5-17 | Hardware transactional memory | Speculate, detect conflict in hardware, fall back to a serial path | Intel TSX/RTM, POWER HTM, DBMS optimistic concurrency | Runtime mechanism. Not an inserted safety check for an unproved operation — a scheduling choice. Determinism lost |
| SYS-D5-18 | An explicit determinism level per site | Choose source-order equality, race freedom, invariant preservation, or bitwise FP reproducibility | Intel MKL CNR, ReproBLAS, `-ffp-contract=off`, MPI reproducible reduce | Would make part D #6's "second permission level" one point on a declared scale rather than an exception |

## D6 Distinctness of identity parameters at function boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D6-01 | Distinct by default, proved at the call | The C `restrict` contract, but discharged rather than trusted | C99 `restrict`, LLVM parameter `noalias` | Backend fact: `noalias` on every pointer parameter. EVIDENCE §E's soundness trap: two brand *variables* must not default to distinct |
| SYS-D6-02 | May-alias by default, callee proves what it needs | Nothing is assumed; the callee asks | C default, WF OWN-7's current default | Backend fact: none unless requested. Safest, and the reason C is slow |
| SYS-D6-03 | Declared per pair | `noalias(a, b)` as an explicit relation, not a per-parameter flag | CUDA `__restrict__` per parameter, Fortran-ish conventions, SAL `_Post_satisfies_` (unverified) | Backend fact: one `!alias.scope` domain per declared pair. Quadratic in signature size |
| SYS-D6-04 | Inferred from the closed world | Whole-program alias analysis supplies the answer; signatures say nothing | LTO, ThinLTO summaries, Legion's dynamic version | Tension with FN-1 and R15 |
| SYS-D6-05 | Per-function switch | Each function opts into strict or permissive | `-fstrict-aliasing` per TU, `__attribute__((optimize))`, `#pragma GCC optimize` | Two dialects to teach; M4 cost |
| SYS-D6-06 | Distinctness from the address space | Parameters in different spaces cannot alias, for free | CUDA/OpenCL/SYCL qualifiers, `AMDGPUAliasAnalysis` | Backend fact: address-space AA. Target-dependent in LLVM generally (unverified) |
| SYS-D6-07 | Distinctness from the allocation class | Objects from different slab caches or size classes never alias | Linux `kmem_cache`, tcmalloc size classes, `!tbaa` root trees | Backend fact: `!tbaa` with distinct roots. Fails for two objects of the same type |
| SYS-D6-08 | Distinctness inherited from a partition | The arguments are subregions of a partition the caller declared disjoint | Legion partitions, Regent task privileges | The callee gets the fact free from the partition tree. Tension: Legion computes it dynamically |
| SYS-D6-09 | Distinctness only over the modified subset | LLVM's actual rule: `noalias` "only holds for memory locations that are modified" | LangRef, quoted in EVIDENCE §B | Two read-only parameters may alias freely and still both be marked. Under-exploited today |
| SYS-D6-10 | Split-and-join tokens | A `split` primitive returns two identities distinct by construction; no proof at the call | Rust `split_at_mut`, Rayon `join`, Legion `partition`, Futhark slicing | Backend fact: `noalias` on both halves. Covers the common case and nothing else |
| SYS-D6-11 | Caller-run versioning, written once | The writer writes `if (a != b) fast else slow` once instead of LLVM emitting 29 guards per loop | LLVM loop versioning (measured: 29 guards, 2132 asm lines in why-whitefoot §5) | Runtime mechanism, but authored and auditable. M5 tension: the obvious shape branches |
| SYS-D6-12 | Fresh `noalias` domain per instantiation | Distinctness is minted per call site, not per parameter position | LLVM's inliner: "a new domain is used every time the function is inlined" (LangRef, EVIDENCE §B) | Exactly mirrors what the backend already does. The source construct is "instantiate at a fresh domain" |
| SYS-D6-13 | (outlandish) Distinctness by arena | Each parameter is typed with the arena it came from; arenas never overlap | jemalloc arenas, per-transaction arenas (Postgres MemoryContexts), nginx pools, LLVM `BumpPtrAllocator` | Backend fact: `noalias` from the arena, not the parameter. Coarse but free |
| SYS-D6-14 | (outlandish) Distinctness by physical page coloring | The allocator guarantees the two parameters land in disjoint page sets and the hardware enforces it | OS page coloring research, `hugetlbfs` partitioning | No proof at all, at the cost of a specialized allocator and coarse granularity |

## D7 The contract vocabulary at call boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D7-01 | requires/ensures over identity states | Entry and exit state per identity | DESIGN.md Candidate A, Vault, Mezzo `consumes` | Baseline for the decomposition |
| SYS-D7-02 | modifies/reads clauses | Footprint sets with `fresh` and `old` | Dafny, Low\* `modifies loc h0 h1` | The notation fits M4; the SMT discharge does not fit M1 |
| SYS-D7-03 | Effect rows, written or derived | Today's EFF-1/2 | WF spec | Backend fact: `memory(argmem: read/write/readwrite)` directly |
| SYS-D7-04 | Exactly what LLVM can consume, nothing more | The contract vocabulary is `memory(...)`, `readonly`, `writeonly`, `initializes`, `captures`, `dereferenceable`, `align` | LLVM LangRef parameter/function attributes | Deliberately incomplete but every clause pays for itself at emission |
| SYS-D7-05 | `initializes((lo,hi))`-style out-parameter clauses | The callee promises to fill a byte range without reading it first | LLVM `initializes`, SAL `_Out_writes_(n)`, Windows driver annotations | EVIDENCE D5 calls this "the cleanest unexploited fact in the current spec" |
| SYS-D7-06 | SAL-style annotation language | A full deployed contract vocabulary for C, checked deterministically | SAL2 (`_In_`, `_Inout_`, `_When_`, `_Acquires_lock_`, `_Must_inspect_result_`), Windows SDV | Industrially proven at Windows scale. Tension: heuristic checker, not a sound one |
| SYS-D7-07 | sparse-style qualifier annotations | Per-function lock and address-space contracts as attributes on declarations | Linux `__acquires(lock)`, `__releases(lock)`, `__must_hold`, `__user`, `__rcu`, `__percpu` | Deterministic, no SMT, ~30M lines. The single strongest existence proof for a non-solver contract checker |
| SYS-D7-08 | `__must_check`-style outcome obligations | The result carries an obligation to be inspected | `warn_unused_result`, Rust `#[must_use]`, `_Must_inspect_result_` | The error-outcome half of R7 that no row states |
| SYS-D7-09 | Existential results for fresh storage | The callee returns a new identity the caller unpacks | Alias Types `exists`, LLVM `noalias` return, `allockind("alloc,uninitialized")` (spelling unverified) | Backend fact: `noalias` return + `allocsize`. Clean fit |
| SYS-D7-10 | States routed by result variant | The exit state depends on which variant came back | CALL-6, `errno`/`-EAGAIN` conventions, io_uring CQE `res` | Matches every real syscall. Backend fact: per-branch `memory()` is not expressible, so this is checker-only |
| SYS-D7-11 | Ownership-transfer conventions as the whole contract | The contract is only "who owns what afterwards" | GObject-Introspection `(transfer full)`/`(transfer none)`, Objective-C ARC method families, COM `AddRef`/`Release` | Deployed across enormous C API surfaces. Says nothing about aliasing or state |
| SYS-D7-12 | Capability pre/post over a slot table | The contract is which cap slots are consumed and which are filled | seL4 API (`seL4_CNode_Move`, `seL4_Untyped_Retype`), fd-passing via `SCM_RIGHTS` | No memory contract at all; identity is table-shaped |
| SYS-D7-13 | Prophecies / backward functions | The final value of a mutable borrow named at the call | RustHorn, Creusot, Aeneas | M1 tension; unnecessary if access checks state per operation |
| SYS-D7-14 | Contracts over blocking and time | What the callee may do to the scheduler, not to memory | `might_sleep()`, `[[clang::nonblocking]]` function effects, realtime audio "no allocation" rules | An orthogonal effect axis missing from R1–R10 entirely |
| SYS-D7-15 | Contracts over memory ordering | Which orderings the callee performs and which it requires of the caller | C11 `memory_order`, `rcu_dereference` requiring `rcu_read_lock()`, `smp_store_release` pairing | Needed before D10 is decidable |
| SYS-D7-16 | No contracts: whole-program inference | The compiler derives `memory()` and `noalias` itself in the closed world | LTO/ThinLTO function summaries, GCC IPA-modref | Directly contradicts FN-1. Backend fact: identical output, different authorship |
| SYS-D7-17 | (outlandish) The contract is the schedule | The callee declares the loop nest it will execute so the caller can fuse or tile across the boundary | Halide `compute_at`/`store_at`, TVM `cache_read`, MLIR linalg fusion | Would make cross-procedural fusion possible, which no attribute vocabulary reaches |

## D8 Storage placement, relocation and moves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D8-01 | Aggregates as handles vs as bytes | The calling convention decides whether identity survives a call | MECHANISM-MAP §6 R3; LLVM `byval` vs `sret` vs plain pointer | Backend fact: the ABI itself. Handle passing is what keeps interior pointers valid |
| SYS-D8-02 | Explicit relocating move ends the storage | A `move` is a state event; every derived pointer dies | Ladder Step 1; Rust move-as-memcpy | Backend fact: `llvm.lifetime.end` on the source slot |
| SYS-D8-03 | No moves: place-only, in-place construction | Objects are constructed where they will live and never relocate | C++ placement new, Rust-for-Linux `pin_init!`/`impl PinInit`, intrusive `list_head` | Removes the whole relocation problem. Tension: constructors become two-phase |
| SYS-D8-04 | Pinning as an explicit state | Relocation is forbidden while pinned | Rust `Pin`, `pin_user_pages()`, buffer-pool pins, `mlock` | Backend fact: `dereferenceable` stays valid across the pin extent. See D3 |
| SYS-D8-05 | Reallocation as a contract-declared replacement | `push` either preserves the backing or names a fresh one, conditional on `len < cap` | L3 `realloc: cap(ρ) → ∃ρ'. cap(ρ')`, ladder Step 3 | The most explainable form. Every derived pointer must be re-derived |
| SYS-D8-06 | Relocation with a fixup channel | A compacting collector rewrites handles; the program never sees an address | Blink Oilpan (tracing, moving), V8 `Handle`/`HandleScope` with a handle table, Boehm GC | Requires handles at every escape. Backend fact: `!invariant.group` becomes unusable across a move |
| SYS-D8-07 | Every object in an append-only pool | No relocation and no reuse; indices are stable forever | why-whitefoot §10 current witness, arena-indexed ASTs | Cost: no reclamation. Backend fact: the pool is one `noalias` allocation |
| SYS-D8-08 | Size-class / slab allocation: relocation impossible | The allocator never moves an object, by construction | Linux SLUB, jemalloc, tcmalloc, mimalloc | Backend fact: `noalias` return, stable `dereferenceable`. The dominant real-world answer |
| SYS-D8-09 | Arena with whole-arena release | Individual objects are never freed or moved; the arena goes at once | Postgres MemoryContexts, nginx pools, LLVM `BumpPtrAllocator`, per-request/per-transaction arenas | R2's obligation becomes one per arena. Tension: per-object `dispose` disappears |
| SYS-D8-10 | Reserve address space, commit on growth | Reserve a huge virtual range so growth never relocates | `VirtualAlloc(MEM_RESERVE)` then commit, Linux overcommit, `io_uring` fixed buffers, "stable vector" idiom | Makes push *unconditionally* non-relocating, deleting the `len < cap` proof obligation. Costs address space, not memory |
| SYS-D8-11 | Chunked / segmented containers | Growth adds chunks; existing elements never move | `std::deque`, Unity DOTS 16 KiB archetype chunks, Postgres tuple pages, Arrow record batches | Interior pointers survive push for free. Backend fact: per-chunk `noalias`; indexing costs one extra indirection |
| SYS-D8-12 | Pointer swizzling | A stored reference is a page id that becomes a pointer only while pinned | LeanStore (Leis et al. ICDE 2018), ObjectStore/Texas | Relocation becomes "unswizzle". Runtime mechanism: a tagged word and a pin |
| SYS-D8-13 | Move as a byte copy the compiler may elide | The source form is a copy; NRVO/copy elision remove it | C++17 guaranteed copy elision, Rust moves, LLVM `memcpy` + `lifetime` markers | Backend fact: the ABI. Tension: elision is an optimizer promise, not a language one |
| SYS-D8-14 | A "has interior pointer" bit forbids relocation | Any type someone points into is statically non-relocatable | Linux structs containing `list_head` are never memcpy'd; Rust `!Unpin` | Cheap and coarse. Propagates through composition |
| SYS-D8-15 | Stack slots with explicit live ranges | The source states where a slot begins and ends; the backend colors slots | `llvm.lifetime.start`/`end`, stack coloring — per EVIDENCE §A.6 the one live backend consumer | The lowest-risk placement mechanism available today |
| SYS-D8-16 | Placement and tier declared at allocation | NUMA node, huge pages, device memory stated where the storage is made | `numa_alloc_onnode`, `MADV_HUGEPAGE`, `cudaMalloc`/`cudaMallocManaged`, `hugetlbfs` | Backend fact: none in IR; a runtime and scheduling fact |
| SYS-D8-17 | (outlandish) Relocation by remapping, not copying | The bytes never move; the virtual address does | `mremap`, `MAP_FIXED`, the ring-buffer mirror trick, CRIU | Interior pointers survive "relocation" only if they are re-derived. Page granularity |
| SYS-D8-18 | (outlandish) Placement chosen by a schedule language | The algorithm never states where data lives; a schedule assigns stack / arena / GPU shared memory | Halide `store_at`/`compute_at`, TVM `cache_read`/`cache_write`, Sequoia | Two-file authoring (see D12). An AI writer would not mind |

## D9 Container elements and backings

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D9-01 | Path identity `'v[i]` | The element is named by the container plus the index | MECHANISM-MAP §6, ladder Step 2 | Needs index disjointness proofs. Backend fact: per-element `!alias.scope` only for constant indices |
| SYS-D9-02 | Existential backing per formation | Forming a pointer unpacks a fresh backing identity | Alias Types `exists`, DESIGN.md | Backend fact: `!invariant.group` window; `launder` at reallocation |
| SYS-D9-03 | Per-element ghost tokens | One erased linear permission per slot | Verus `PointsTo`, L3 | Explodes for large arrays unless quantified |
| SYS-D9-04 | Generation per slot | Reuse is detected by comparing a counter | Unity DOTS `Entity.Version`, `slotmap`, Vale generational references | Runtime mechanism: one compare per lookup. why-whitefoot §10 proposes it as a typed outcome |
| SYS-D9-05 | Index-only access with bounds proofs | No element identity; the bound is discharged | Futhark, WF today | Backend fact: `!range` on the loaded length, no bounds branch |
| SYS-D9-06 | Iterator objects carrying proved invariants | The iterator's type states what it has consumed and what remains | C++ iterators, Rust `IterMut`, Java `ListIterator` | Needs a splitting story to parallelize. Backend fact: induction-variable facts |
| SYS-D9-07 | Split-and-join tokens over ranges | A split yields two disjoint range identities, rejoined later | Rust `split_at_mut`, Rayon, Legion partitions, `chunks_mut` | Backend fact: `noalias` per half. Matches PAR-2 tiles exactly |
| SYS-D9-08 | Branded index (generativity) | A scope mints a brand; only brand-matching indices may subscript | Haskell `runST`, Rust `generativity`/`indexing`, `GhostCell` | Bounds check disappears at the type level with no proof at each use. Brands cannot be stored in structs |
| SYS-D9-09 | (chunk, lane) identity | The container is a list of fixed chunks; identity is chunk plus lane | Unity DOTS archetype chunks, Arrow record batches, ClickHouse blocks, Vectorwise | Structural disjointness per chunk: parallelism needs no index arithmetic. Backend fact: per-chunk `noalias` |
| SYS-D9-10 | One identity per column, not per container | An SoA container is N independent backings | why-whitefoot §5 `Cols`, Arrow, ISPC `soa<N>`, ECS SoA chunks | Backend fact: EVIDENCE D1 exactly — the measured 0-guard 121-line kernel |
| SYS-D9-11 | Validity bitmap as a separate container | Partial initialization is a second array of bits, not a state of the element | Apache Arrow validity buffers, Parquet definition levels, `bitmap` in Postgres tuples | Dissolves D3's partial-init problem. Cost: a runtime consult per access |
| SYS-D9-12 | Free list stored inside freed slots | Freed elements are not inert: the allocator writes links into them | Linux SLUB free pointer, glibc tcache, every intrusive free list | Forces the model to say whether Gone storage may be written. See D0 R22 |
| SYS-D9-13 | Ring buffer with modular index arithmetic | Head and tail indices; disjointness is modular, not affine | io_uring SQ/CQ rings, DPDK `rte_ring`, `kfifo`, LMAX Disruptor | No current mechanism proves modular disjointness. Backend fact: the mirror-mapping trick removes the wrap branch |
| SYS-D9-14 | Open-addressed table with SIMD group probe | Element identity is a probe sequence; distinctness only from key inequality | Abseil Swiss tables, folly F14, the measured key/value pairing in why-whitefoot §9 | Backend fact: the group load is the vectorization; layout is pinned by measurement, not doctrine |
| SYS-D9-15 | Extents inferred from a schedule | The writer never states a range; bounds inference computes every intermediate buffer's extent | Halide bounds inference, TVM, Tiramisu | Removes most index proof obligations. Requires the algorithm to be pure |
| SYS-D9-16 | No element identity: functional update only | Elements are values; mutation is `scatter`/`gather` over whole arrays | Futhark uniqueness types (`*[]f64`), Dex index sets as types, APL/J | Backend fact: in-place update licensed by uniqueness. Tension: no interior pointers exist to name |
| SYS-D9-17 | (outlandish) Backing is a file or mapping | An element is (fd, offset); memory and storage use one mechanism | `mmap(MAP_SHARED)`, DBMS pages, Boost.Interprocess `offset_ptr`, `memfd` | Foreign identity (D11) and container identity become the same question |
| SYS-D9-18 | (outlandish) Element identity is a SIMD lane | The type tracks lane masks; `'v[i]` refines to a vector-register lane | ISPC `uniform`/`varying`, AVX-512 mask registers, CUDA warp lanes | Backend fact: vectorization becomes a source-level guarantee, not an optimizer hope |

## D10 Shared mutation across threads and the concurrency story

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D10-01 | Lock as custodian of identity state | Acquire adds the custody facts at the invariant; release erases every fact about the custody set | CSL resource invariants, Chalice monitors, Mezzo locks, part D §B.3 sketch | Runtime: one mutex. Part D §B.4's refusal: two `alloc` calls both `writes(lock)` and deny overlap |
| SYS-D10-02 | A second, weaker permission level | Race freedom plus invariant preservation, without source-order equality | Part D #6; DPJ and Regent both decline it | Explicitly a decision, not a derivation. CAP-1 conflict |
| SYS-D10-03 | Protocol / STS shared regions | Only facts stable under the other party's action set may be retained | CAP (ECOOP 2010), Iris STS, Verus atomics | A second model. Part D marks it "recommend against" |
| SYS-D10-04 | Phase / epoch only | Readers and writers separated in time | RCU, CONCURRENCY-CATALOG §8 (zero reader cost vs 15–25 ns `RwLock` read) | Publish latency bounded by read-phase length. No new vocabulary |
| SYS-D10-05 | Message passing only | Nothing is shared; ownership travels in messages | Erlang, Pony actors, seL4 IPC, Go channels (by convention) | Backend fact: everything `noalias`. Copies at boundaries |
| SYS-D10-06 | No threads: process-level parallelism | Isolation comes from the address space, not the type system | nginx workers, Chrome site isolation, Redis single-threaded, PostgreSQL backends | Zero language mechanism. Cost: IPC, and shared state needs D11 anyway |
| SYS-D10-07 | Deterministic parallelism only | Overlap is admitted only when the result equals the sequential one | DPJ (OOPSLA 2009), PAR-1/PAR-2 today | The current promise. Excludes every lock-based allocator |
| SYS-D10-08 | Shard-by-construction | Each core owns a partition; cross-shard work is an explicit message | Seastar/ScyllaDB, DPDK per-lcore, Kafka partitions | Removes locks entirely. Partitioning becomes an architecture requirement |
| SYS-D10-09 | Per-CPU data plus a context discipline | Exclusivity from the execution context (preemption/IRQ disabled), not from a lock | Linux `this_cpu_ptr` under `preempt_disable()`, `local_irq_save`, `percpu_counter` | Needs a static context effect (`in_irq`, `preempt_disabled`), which nothing in R1–R10 provides |
| SYS-D10-10 | Seqlock: writers lock, readers retry | Readers re-read on a version change; the retry is a loop the source writes | Linux `seqcount_t`, `read_seqbegin`/`read_seqretry`, `u64_stats_sync` | The retry is a *typed outcome with real control flow* — M2-compatible in a way a hidden check is not |
| SYS-D10-11 | Hazard pointers / epoch reclamation | Reclamation deferred until no reader can hold a reference | crossbeam-epoch, folly `hazptr`, Michael's hazard pointers | Needs D3's Quiescing state. Runtime: a per-thread hazard slot |
| SYS-D10-12 | RCU with a checked `__rcu` qualifier | A qualified pointer may only be dereferenced inside a read-side section | Linux `__rcu` + sparse, `rcu_dereference`, `rcu_assign_pointer` | A deployed, deterministic, non-SMT static discipline for shared mutation at kernel scale |
| SYS-D10-13 | Copy-on-write with an atomic publish | Writers build a new version; readers see one version or the other | RCU's update side, persistent data structures, Clojure refs, `git` objects | Reduces D10 to "immutable value plus atomic owner swap". Cost: allocation per update |
| SYS-D10-14 | Double buffering / frame flip | State N is read-only for everyone; state N+1 has exactly one writer; swap at the boundary | Game-engine frame state, graphics swap chains, ping-pong buffers in HPC | No synchronization inside a phase. Memory doubled |
| SYS-D10-15 | Compile-time computed schedule | The closed world lets the conflict graph and the topological schedule be built at compile time | Bevy/Unity DOTS do it at startup; PROG-1 permits doing it earlier | Backend fact: the emitted schedule. The clearest concrete payoff PROG-1 offers |
| SYS-D10-16 | Static lock order | Deadlock freedom from a total order on lock types, checked statically | Chalice `waitlevel`, Linux lockdep (dynamic), Windows SDV IrqlLockOrder rules | Checkable without SMT. Answers R21 partially |
| SYS-D10-17 | Transactional memory with a fallback path | Speculate; on conflict, take a serial path | Intel TSX/RTM, POWER HTM, STM libraries | Runtime mechanism; determinism lost; the fallback path is ordinary code |
| SYS-D10-18 | Typed protocols on channels | The channel's type is a session protocol both ends must follow | Session types, Pony actor behaviours, `io_uring` opcode/CQE pairing | Backend fact: none; a checker mechanism. Composes with message passing |
| SYS-D10-19 | SPSC ring as the only channel | One producer, one consumer, publication by a release store | io_uring SQ/CQ, LMAX Disruptor, DPDK `rte_ring`, `kfifo` | The proof is modular arithmetic plus a release/acquire pair. Narrow, and industrially ubiquitous |
| SYS-D10-20 | (outlandish) Compile the closed world to bare cores | Static per-core programs, no preemption, no scheduler, so no locks exist | Time-triggered architectures (Giotto), bare-metal DPDK pipelines, unikernels | Removes every concurrency mechanism by removing the scheduler |
| SYS-D10-21 | (outlandish) GPU launch model as the whole story | No threads: only kernel launches over index spaces with barriers and explicit staging | CUDA `__syncthreads()`, SYCL nd-range, Halide GPU schedules | Determinism from barrier structure. Shared mutation between launches is a D11 problem |

## D11 External resources

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D11-01 | Identity with a `foreign` qualifier | No contents fact is retained across any operation; extent facts stay program-ordered | Part D §C.4; Low\* monotonic refs, VST's first-order external world | Backend fact: every access must behave like a fresh load. Minimal by construction |
| SYS-D11-02 | Capability tokens | Access requires a consumable permit | `HandleFactory` permits, seL4 caps, Capsicum, OpenBSD `pledge`/`unveil` | Composes with R2's linearity. Says nothing about contents facts |
| SYS-D11-03 | Effect categories | A separate effect kind for I/O | Rejected by `design/language/effects.md` | Kept for completeness; violates the no-host-distinction rule |
| SYS-D11-04 | Trusted adapters outside the language | The boundary is a hand-audited TCB module | Rust `std`'s `unsafe` core, kernel driver APIs, the JVM's native methods | Explicitly what M3 forbids inside the language; the question is whether it exists outside it |
| SYS-D11-05 | States as ordinary enums | The descriptor is a nominal with an ordinary state machine | Part D §C.2 fd/socket tables | Works for fd and socket with zero additions, per part D #9 |
| SYS-D11-06 | Address-space qualifiers that forbid dereference | A `__user` pointer cannot be loaded at all; access goes through `copy_from_user` | Linux sparse `__user`, `__iomem`, `__percpu`; deployed at ~30M lines | The strongest deployed prior art for D11. Deterministic, no solver, checked mechanically |
| SYS-D11-07 | LLVM address spaces as the mechanism | Foreign storage lives in `addrspace(N)`; `target_mem#` exists in `memory(...)` for exactly this | LangRef `memory(target_mem#: ...)`, NVVM spaces, `AMDGPUAliasAnalysis` | Backend fact: the emitter already has a slot for it. Target-dependent semantics |
| SYS-D11-08 | `volatile` as the retention rule | "Retain no fact" is precisely what volatile means to LLVM | C `volatile`, Linux `READ_ONCE`/`WRITE_ONCE`, `readl`/`writel` | Backend fact: the exact existing construct. Cost: no CSE, no hoisting, no vectorization on that storage |
| SYS-D11-09 | MMIO with ordering requirements | Device registers need no reordering, no coalescing, and explicit barriers | `readl`/`writel` (barriered), `readl_relaxed`, `iowrite32`, PCIe posted-write rules | Volatile alone is insufficient; needs D0's R12 ordering vocabulary |
| SYS-D11-10 | Device-owned handoff | The buffer belongs to a DMA engine between map and unmap | `dma_map_single`/`dma_unmap_single`, `dma_sync_single_for_cpu`/`for_device`, `IOMMU` domains | An affine transfer to a non-program agent plus a D3 state. Nothing in R10 has this shape |
| SYS-D11-11 | Escrow with a completion token | The kernel holds the buffer from submission to completion; a CQE returns it | io_uring SQE/CQE, `IORING_REGISTER_BUFFERS`, POSIX AIO, Windows OVERLAPPED | Needs a linear completion token and a non-lexical loan. R2 assumes program-ordered discharge |
| SYS-D11-12 | Descriptors as sealed capabilities | The resource can be made provably immutable by an OS operation | `memfd_create` + `F_SEAL_WRITE`/`F_SEAL_SHRINK`, read-only mappings | A foreign identity that *becomes* frozen; backend fact: `!invariant.load` legitimately applies afterwards |
| SYS-D11-13 | Ambient authority removed at startup | The program drops the ability to name resources it did not open | Capsicum `cap_enter`, `pledge`/`unveil`, seccomp, Landlock | Makes R10's "permit" model enforceable outside the language too |
| SYS-D11-14 | Partial and interrupted operations as first-class | `read()` returns a short count; `EINTR`, `EAGAIN`, `EWOULDBLOCK` are normal | POSIX, io_uring `-EAGAIN`, TCP short writes | A state table cannot express "half the transition happened". Missing from R10 entirely |
| SYS-D11-15 | (outlandish) Declared interference model per foreign identity | Instead of retaining nothing, retain what the external agent's declared transition relation cannot falsify | Low\* monotonic references/`witnessed` (library shape unverified), VST external world, hardware ring-buffer specs ("the device only advances the tail") | Makes the degenerate rule one end of a scale. Needs a stability judgement, which part D flags as a second model |
| SYS-D11-16 | (outlandish) Hardware compartments as the boundary | The foreign region is a separate CHERI compartment or seL4 protection domain | CHERI compartmentalization, seL4 CSpaces, Intel MPK/PKU | Enforcement in hardware; the language states only the boundary |

## D12 Surface form and sugar

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D12-01 | `&`/`&uniq` as sugar over pointer plus state clauses | Today's spelling desugars into the new model | MECHANISM-MAP §8 item 9 | Migration path; tension: sugar hides the fact the optimizer reads |
| SYS-D12-02 | Explicit-only forms | Every identity, state and footprint written out | Verus-style ghost threading; M4 says verbosity is cheap | Backend fact: unchanged. The cost lands entirely on diff size |
| SYS-D12-03 | Declaration attributes, type grammar untouched | Contracts as attributes beside the declaration rather than inside the type | Linux sparse (`__must_hold(&x->lock)`), SAL2, GCC `__attribute__`, Rust attributes | Deployed at the largest scale of any option here. Tension: attributes do not compose or abstract |
| SYS-D12-04 | C-style type qualifiers | `const`/`restrict`/`volatile`/address-space qualifiers rather than type parameters | C99, CUDA/OpenCL qualifiers, Linux `__rcu` | Cheap to read and write. Notoriously bad composition (`const char * restrict *`) |
| SYS-D12-05 | Parameter conventions instead of types | `in`/`out`/`inout`/`consuming`/`borrowing` as calling conventions | Swift/Hylo conventions, Ada `in out`, Fortran `intent()`, C# `ref`/`out`/`in` | Backend fact: maps directly to `byval`/`sret`/`readonly`/`writeonly`/`initializes` |
| SYS-D12-06 | Algorithm and schedule in separate files | Identity, placement and parallelism annotations live apart from the computation | Halide algorithm/schedule split, TVM, Tiramisu, MLIR transform dialect | An AI writer has no objection to two files. Tension: diagnostics must cite across them |
| SYS-D12-07 | Compiler writes the annotations back into the source | The writer states nothing; the checker materializes obligations as source text | `cargo fix`, Infer's `@Nullable` inference, clang-tidy fixits, Coverity annotations | Makes M4's verbosity free. Tension: the source is then a compiler output |
| SYS-D12-08 | Machine-readable interface files | Signatures with identity/state contracts live in a generated `.wfi` the compiler reads and writes | Rust `.rmeta`, Swift `.swiftinterface`, ThinLTO summaries, BTF | Makes FN-1 an artifact rather than a syntax question. Canonical-form §11 already wants this |
| SYS-D12-09 | Pragmas at cut points only | Annotate loops and calls, never types | `#pragma omp simd`, `#pragma ivdep`, `#pragma unroll`, `llvm.loop.*` metadata | Backend fact: exactly the metadata LLVM consumes. Tension: no modular guarantee |
| SYS-D12-10 | Region syntax reused with identity meaning | Keep `'s`, change what it denotes | MECHANISM-MAP §6 "PROV-1 spelling with identity meaning only" | Smallest diff; tension: a familiar spelling with new semantics is the worst case for a reader, and arguably for a writer |
| SYS-D12-11 | Spelling chosen so each fact has a unique tree node | One rule, one path, one diagnostic — the canonical-form dividend applied to the new vocabulary | why-whitefoot §11 | Constrains the grammar before the semantics is settled |
| SYS-D12-12 | (outlandish) No surface syntax at all | Identities and states are inferred in the closed world and only *printed* in diagnostics and interface dumps | Whole-program alias analysis, Zig comptime, `objdump`-style views | The source looks like C; every fact is a compiler artifact. Kills FN-1 |
| SYS-D12-13 | (outlandish) The IR is the surface form | The writer emits checked IR; human-readable text is a rendering | MLIR textual/binary dialects, WebAssembly text vs binary, LLVM `.ll`/`.bc` | An AI writer needs no concrete syntax. Tension: every human review path assumes text |

## D13 Validation and migration strategy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| SYS-D13-01 | Exhaustive small-model enumeration | Enumerate all programs under a size bound and compare against a reference semantics | `research/experiments/access-state/RESULTS.md`, Alloy-style bounded checking | Already the house method. Bound choice is the whole question |
| SYS-D13-02 | A formal model | Mechanized soundness for a core calculus | RustBelt (POPL 2018), Stacked/Tree Borrows, Cogent's refinement proofs | High cost; catches exactly the class of bug R16 names |
| SYS-D13-03 | Differential testing against Rust and C | Same program, three compilers, compare behavior | The committed SoA kernel in three Rust shapes (why-whitefoot §5) | Cheap and already partly built. Only finds disagreements on programs all three accept |
| SYS-D13-04 | Fuzzing | Random inputs against accepted programs | libFuzzer, AFL | Weak for a checker; strong for the backend |
| SYS-D13-05 | Static verdict vs dynamic sanitizer agreement | Run a corpus under ASan/MSan/TSan-equivalent semantics and check the checker agreed | Miri vs Stacked Borrows vs Tree Borrows — this exact experiment found real model bugs in Rust | Needs shadow state (D2), which release builds erase. Answers R16 directly |
| SYS-D13-06 | Emit the alias facts *and* verify them at runtime in a debug build | The `noalias` promise is checked dynamically where it is claimed | Rust's history of disabling `&mut` `noalias` over LLVM miscompiles (version unverified); TSan's happens-before checking | The only way to catch a wrong D4/D6 fact before it silently miscompiles |
| SYS-D13-07 | Random program generation with a reference interpreter | Csmith-style generation targeted at the ownership fragment | Csmith, YARPGen, CsmithEdge | Needs a reference semantics first. Finds checker bugs, not model bugs |
| SYS-D13-08 | Equivalence-modulo-inputs testing of the optimizer | Mutate dead regions of a program; the output must not change | EMI (Le et al. PLDI 2014), Orange4 | Tests whether a wrong alias fact changes observable behavior — the exact D4 risk |
| SYS-D13-09 | Translation validation on the emitted IR | Prove each lowering preserves semantics *under the claimed attributes* | Alive2, CompCert's validators, Sea of Nodes verification work | Directly validates the attributes D4/D6/D7 emit, which nothing else does |
| SYS-D13-10 | A fixed discriminating benchmark set | Programs chosen because candidates disagree on them | The SoA 8-column kernel (committed), plus: a slab allocator, an io_uring ring, an ECS iteration, a B+tree with pins, an arena JSON parser, a `memcpy` with `restrict` | Each names a mechanism: columns→D1, slab→D3/D8, ring→D9, ECS→D9/D10, B+tree→D3/D11, arena→D8 |
| SYS-D13-11 | Port a real kernel subsystem as the discriminator | If the model cannot express `list_head`, an RCU list, or a page allocator, that is the finding | Rust-for-Linux's own experience with `Pin`, intrusive lists and `Arc` | Expensive; the most informative single test available |
| SYS-D13-12 | Measure compile time on a large program | The closed-world premise is unvalidated until a whole-program check is timed at scale | ThinLTO exists because full LTO did not scale; Polly's compute-out | Turns D0's M1 challenge into a number |
| SYS-D13-13 | Measure guard count and code size, not just time | why-whitefoot §5's durable win is 0 guards and 121 vs 2132 lines, not a large-n speedup | The committed assembly scoreboard | A metric that survives redesign, unlike a wall-clock ratio |
| SYS-D13-14 | Dual acceptance with disagreement warnings | Run the old borrow checker and the new identity checker together; warn on every disagreement | Rust's NLL migrate mode ran both borrow checkers and warned on divergence (`-Zborrowck=migrate`, spelling unverified) | The exact problem at the exact scale; the strongest migration precedent available |
| SYS-D13-15 | Require an attribute-level end-to-end test per mechanism | Each accepted mechanism must name the backend attribute it emits and ship an asm test asserting it appears | EVIDENCE §B's table is already the checklist; `compiler/src/backend/tests/effect_attributes.rs` is the existing tripwire shape | Makes R4 falsifiable. Tension: today the emitter states no alias promises at all |
| SYS-D13-16 | Patch section 5 vs a new section | Whether the spec change is an edit or a replacement | Repository rule: active file retitled vN+1, outgoing bytes archived | Mechanical; the real question is whether conformance verdicts change |
| SYS-D13-17 | (outlandish) Validate on CHERI hardware | Compile the corpus for CheriBSD; any static claim the model makes should never fault at runtime | CHERI-RISC-V, CheriBSD running real software, Morello | Every violated assumption becomes a hardware trap with a counterexample, which erasure otherwise destroys |

---

## Decision points missing from this list

| id | Missing point | Why it is a separate decision | Which listed points it would otherwise distort |
|---|---|---|
| SYS-DPX-01 | D14 Memory model and ordering | Acquire/release, fences and what may be reordered past what; D10 cannot be decided without it | D3 (Published), D5, D7, D10, D11 |
| SYS-DPX-02 | D15 Layout, ABI and calling convention | `byval`/`sret`/handle, SoA/AoS, alignment and false sharing; "aggregates travel as handles" is an ABI decision made under D8 | D1 (per-column identity), D8, D9 |
| SYS-DPX-03 | D16 Error and partial-outcome vocabulary | Allocation failure, short reads, `EINTR`; a state table cannot say "half the transition happened" | D3, D7, D11 |
| SYS-DPX-04 | D17 Compile-time cost model and incremental build | PROG-1's closed world has never been costed; M1 constrains determinism, not complexity | D2 (global table), D5 (polyhedral), D6 (inferred), D7 (whole-program inference) |
| SYS-DPX-05 | D18 What erasure authorizes the backend to remove | Zeroization, `freeze`, poison propagation, and which proofs license which transform | D0 R14, D3 (Poison), D4, D13 |
| SYS-DPX-06 | D19 Generics and abstraction over identities | Identity polymorphism, existentials inside nominals, monomorphization; §7 notes nominals must carry identity parameters but no point owns the rule | D1, D6, D7, D12 |
| SYS-DPX-07 | D20 Observability when the checker is wrong | With no traps and no shadow state, a checker bug is a silent miscompile | D13 entirely, and D0 R16 |
| SYS-DPX-08 | D21 Reductions and commutative accumulation | The most common real parallel write is to a shared accumulator; no footprint model admits it | D5, D10 |
| SYS-DPX-09 | D22 Heterogeneous targets and address spaces | GPU spaces, MMIO, `__user`; identity arguably includes a space | D1, D6, D11 |
| SYS-DPX-10 | D23 Caching, memoization and lazy initialization | The no-`RefCell` doctrine removes every shape these use | D3 (Frozen/Zeroed), D4, D10 |
| SYS-DPX-11 | D24 The TCB boundary and how adapters are audited | M3 forbids an unsafe escape *inside* the language; part D calls a mutex a "TCB extension" | D10, D11, D13 |
| SYS-DPX-12 | D25 Non-lexical, non-program-ordered loans | DMA and io_uring hold storage across an unbounded window with no program-ordered end | D3, D8, D11 |
| SYS-DPX-13 | D26 Blocking, time and realtime effects | `might_sleep`, `nonblocking`, lock order, deadlines — an effect axis orthogonal to memory | D7, D10, D0 R21 |
| SYS-DPX-14 | D27 What the writer may *demand* of the backend | Whether "this loop vectorizes" is a checked guarantee or a hope; M5 asserts the accepted shape is the fast shape without a mechanism | D4, D5, D12, D13 |

## Requirements missing or misstated (D0)

| id | Item | Statement | Ground |
|---|---|---|
| SYS-REQ-01 | Missing: layout and ordering | R1–R10 name no layout, alignment, false-sharing or memory-ordering requirement, yet every systems program this lens covers is shaped by all four | Linux LKMM, `____cacheline_aligned_in_smp`, Arrow/ISPC layout rules |
| SYS-REQ-02 | Missing: failure and partiality | Allocation failure, short reads and interrupted operations are not "release exactly once" problems and have no home in R2 or R10 | `GFP_ATOMIC`, `EINTR`, TCP short writes |
| SYS-REQ-03 | Missing: the allocator's own writes | R1 treats Gone storage as inert; every production allocator writes free-list links and poison patterns into it | Linux SLUB, glibc tcache, `SLAB_POISON` |
| SYS-REQ-04 | Missing: third-party relocation | R8 covers relocation the program performs, not relocation a collector or the OS performs under it | Oilpan, `mremap`, LeanStore eviction |
| SYS-REQ-05 | Missing: reductions | R5 admits overlap only when footprints are disjoint; the commonest parallel write is a shared accumulator under an associative operator | `omp reduction`, Legion `reduces`, Futhark `reduce` |
| SYS-REQ-06 | Missing: compile-time cost | M1 forbids a budget selecting acceptance but permits an unbounded accepting run; a closed-world check is exactly where that bites | ThinLTO's existence; Polly's `computeout` |
| SYS-REQ-07 | Missing: observability | With no trap, no unsafe and full erasure, there is no channel through which a checker bug becomes visible before it miscompiles | Rust's `noalias` disablement history; Miri |
| SYS-REQ-08 | Not independent: R4 and R5 | Both are "are these two footprints disjoint", differing only in consumer (IR metadata vs a spawn judgement) | Legion and SYCL feed one fact to both; LLVM's inliner converts between the two forms |
| SYS-REQ-09 | Not independent: R4 and R6 | "Which accesses cannot alias" and "which facts a write kills" are the same disjointness query at two program points | ENT-5 keys kills the same way `!alias.scope` keys accesses |
| SYS-REQ-10 | Not independent: R2 and R8 | An arena makes release and placement one decision; a slab makes relocation and lifecycle one decision | Postgres MemoryContexts, nginx pools, SLUB |
| SYS-REQ-11 | Mechanism in disguise: M2 | "No runtime safety check" is a mechanism; the underlying requirement is plausibly "no unpredictable trap and no unrequested cost". Under the literal reading, CHERI bounds checking, a seqlock retry loop and a generational handle compare are all excluded despite costing no branch, being ordinary control flow, and being ordinary data comparison respectively | CHERI LSU checks; `read_seqretry`; ECS `Entity.Version`; MAP §5 disqualifies Vale and Mezzo on this reading alone |
| SYS-REQ-12 | Mechanism in disguise: FN-1 with PROG-1 | Signature-only judgement and whole-program compilation are each a mechanism, and holding both pays modularity's annotation cost without collecting separate compilation's benefit | LLVM holds both deliberately and uses summaries to cross the boundary; WF uses neither half of that |
| SYS-REQ-13 | Mechanism in disguise: M5's "accepted shapes are the fast shapes" | Stated as a meta-constraint but unenforceable without D27: nothing says what the writer may demand of the backend, and today the emitter states no alias promises at all | `compiler/src/backend/emitter.rs:4`; EVIDENCE §C.0 |
| SYS-REQ-14 | Misstated: R5's determinism promise | "Equality with the source-order result in every execution" is one of at least four useful guarantees, and it is the most expensive; the requirement should name the level, not fix it | Intel CNR, ReproBLAS, MPI reproducible reduce, DPJ vs Legion |


# Lens file: numbered-theory.md


No ranking, scoring, recommending or pruning. `[survey]` marks a family already
in EVIDENCE part C or MECHANISM-MAP section 5, kept only so the enumeration is
complete; everything else is an addition. `(outlandish)` marks options no one
would obviously ship. `(unverified)` marks a real artifact whose venue or
authorship I did not confirm.

## D0 The requirements themselves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D0-01 | Keep R1..R10 and M1..M5 verbatim | Treat the map's list as the frozen requirement set | MECHANISM-MAP section 2 | assumes independence that section 7's coupling table already contradicts |
| THE-D0-02 | Split R1 into three | Initialization, liveness/end-of-storage, and layout validity become separate requirements with separate mechanisms | ATS view kinds `T @ l` vs `T? @ l` (Zhu, Xi, PADL 2005) | three state vocabularies instead of one; may reveal that layout needs no state at all |
| THE-D0-03 | Merge R2 into R3 | Lifecycle accounting is the linear-multiplicity case of value classes; delete R2 | Wadler, IFIP TC2 1990; Tov, Pucella, POPL 2011 (Alms) | loses the "obligation vs uniqueness" distinction Clean makes; R2's join rule has no R3 counterpart |
| THE-D0-04 | Merge R6 into R7 | Framing is the other half of a signature; one contract vocabulary covers both | Prusti (Astrauskas et al., OOPSLA 2019); Flux (Lehmann et al., PLDI 2023) | intra-procedural kills (ENT-5) then need a signature-shaped statement for every write |
| THE-D0-05 | Merge R4 into R5 | Optimizer facts and parallel permission are one interference judgment at different granularities | DPJ (Bocchino et al., OOPSLA 2009) | R4 tolerates imprecision, R5 does not; merging makes a missing fact a rejection |
| THE-D0-06 | Add the ten missing requirements | R11 abstraction/generics/closures, R12 error outcomes, R13 termination and space bounds, R14 checking cost, R15 human auditability, R16 evolution, R17 recursive structures, R18 erasure guarantee, R19 memory model, R20 secret-independent timing | each is cited in the final section of this file | enumerated with grounds under "Requirements missing or misstated" below |
| THE-D0-07 | Add only the requirements a current experiment blocks on | Admit a missing requirement only when a program the project wants to compile fails without it | the repository's own priority order | risks discovering R11 and R17 late, after a mechanism has been selected on a first-order fragment |
| THE-D0-08 | Restate requirements as adequacy theorems | Each R becomes a statement about the operational semantics that the metatheory must prove | Wright, Felleisen, Inf. & Comp. 1994 | forces a semantics before a mechanism; slows D13 but makes independence checkable |
| THE-D0-09 | Restate requirements as a discriminating program suite | Each R is operationalized as programs that must compile and programs that must be refused | POPLmark (Aydemir et al., TPHOLs 2005) as a precedent for benchmark-driven design | requirements become test-shaped and may be under-specified between the programs |
| THE-D0-10 | Requirements as a refinement lattice | Order the R's by logical strength and derive which imply which | Back, von Wright, *Refinement Calculus*, Springer 1998 | reveals R9 as the negation of R4's exclusivity, a dependence the flat list hides |

## D1 The unit and naming of storage identity

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D1-01 | Abstract location name in a store typing | `ρ` names one storage; the store type maps `ρ` to a type | Alias Types (Smith, Walker, Morrisett, ESOP 2000) `[survey]` | location polymorphism plus pack/unpack at every abstraction boundary |
| THE-D1-02 | Linear capability keyed by a location | The name is copyable, the right to use it is linear | L3 (Ahmed, Fluet, Morrisett, TLCA 2005) `[survey]` | capability threading at every call; higher-order code needs capability polymorphism |
| THE-D1-03 | Singleton type of an address | `Ptr(a)` where `a` is a type-level value denoting one address | Dependent ML singletons (Xi, Pfenning, POPL 1999) | a type-level term language and a decidable equality on it |
| THE-D1-04 | Type-level nominal atom with freshness | Identities are atoms; distinctness is a freshness constraint `a # b` | Pitts, Inf. & Comp. 2003; FreshML (Shinwell, Pitts, Gabbay, ICFP 2003) | a freshness context in signatures; gives D6 a ready-made judgment |
| THE-D1-05 | Rank-2 generative brand | A scope introduces a fresh type variable no other scope can name | Launchbury, Peyton Jones, PLDI 1994 (`runST`); Kiselyov, Shan, ENTCS 2007 | branding is scope-shaped, so stored pointers escaping the scope are inexpressible |
| THE-D1-06 | Brand shared by many cells, permission elsewhere | Identity is a brand; the right is a separate token | GhostCell (Yanovski, Dang, Jung, Dreyer, ICFP 2021) `[survey]` | brand-coarse granularity; two sub-structures share one permission |
| THE-D1-07 | Generative module/functor identity | Each allocation site instantiates a generative functor producing an abstract storage type | SML generative functors; Harper, Lillibridge, POPL 1994 (unverified for this use) | ties identity to the module system; allocation in a loop needs one identity per iteration |
| THE-D1-08 | World or place modality | `A @ w`: a value lives at world `w`; movement between worlds is explicit | ML5 (Murphy VII, Crary, Harper, TGC 2007; CSL 2005) | modal syntax; was designed for distribution, reused here for storage placement |
| THE-D1-09 | Path expression as the identity | `'v.field[i]` is the name; equality is path equality up to a fixed normalizer | Dafny/JML `o.f` footprints; WF's resolved places | needs a normal form for paths and a decidable index fragment |
| THE-D1-10 | Index into a ghost permission map | Identity is a key into `Map<K, PointsTo>` | Verus permission maps (Lattuada et al., OOPSLA 2023) | map reasoning becomes the bottleneck; key disjointness is a set problem |
| THE-D1-11 | Existential package per formation | Every pointer formation unpacks `∃ρ. cap(ρ) ⊗ ptr(ρ)` | Alias Types `exists`; L3 realloc rule | an unpack step at every use; names multiply in loops |
| THE-D1-12 | Dependent pair over a heap index | `Σ (a : Addr). Points a T`, identity as data | Hoare Type Theory (Nanevski, Morrisett, Birkedal, ICFP 2006) | full dependent types; definitional equality must be decidable and cheap |
| THE-D1-13 | Region as the identity unit | Identity is the region, not the object; per-object facts are region-relative | Tofte, Talpin, Inf. & Comp. 1997 `[survey]` | coarse: same region does not mean same object, so R4 is weak |
| THE-D1-14 | Linear region value | The region itself is a linear value passed and consumed | Fluet, Morrisett, Ahmed, ESOP 2006 ("Linear regions are all you need") | region passing everywhere; but gives bounded-space guarantees for free |
| THE-D1-15 | Ownership context (owner-as-name) | Identity is "the owner of this object"; nesting gives a tree of identities | Clarke, Potter, Noble, OOPSLA 1998 | owners-as-dominators forbids the interior pointers R8 needs |
| THE-D1-16 | Ownership domain label | Identity is a declared domain; links between domains are declared | Aldrich, Chambers, ECOOP 2004 | two naming systems (domains and objects); policy declared per class |
| THE-D1-17 | Universe modifier | Identity is relative: `rep`, `peer`, `any` with respect to the current receiver | Dietl, Müller, JOT 2005; Dietl, Drossopoulou, Müller, ECOOP 2007 | relative naming has no absolute distinctness, which R4 wants |
| THE-D1-18 | Ghost identifier in an invariant algebra | Identity is an Iris-style `gname` allocated in ghost state | Iris (Jung et al., POPL 2015) | ghost allocation is a proof step; needs a fixed ghost-theory catalogue, not arbitrary RAs |
| THE-D1-19 | Identity as a term-level ghost natural | Identities are ghost integers with explicit distinctness proofs | Boogie/Dafny ghost ids (unverified for this use) | distinctness becomes arithmetic; pushes D6 into the index fragment |
| THE-D1-20 | Closed-world derived naming | Identity is the allocation site refined by context, or an entry in a global table computed once | k-CFA heap abstraction, Shivers 1991 (unverified for this use); Datalog points-to, Polonius (unverified, no paper) | PROG-1 permits it; violates FN-1, and the name is not writable in a signature |
| THE-D1-21 | Nothing: no pointer type at all | Identity questions disappear because references do not exist; only paths and indices | Hylo/Val mutable value semantics (Racordon et al., JOT 2022) `[survey]` | aliased object graphs are inexpressible; R9 has no answer |
| THE-D1-22 | Content-based identity (hash consing) | Two storages with equal contents are one identity | hash-consing, Filliâtre, Conchon, ML Workshop 2006 (unverified for this use) | (outlandish) mutation destroys the equality the identity rests on |
| THE-D1-23 | Identity by ordinal position in an ordered context | Ordered (non-commutative) logic makes storage order the name | Polakow, Pfenning, TLCA 1999 | (outlandish) stack-shaped only, but exactly right for contiguous ranges and split/join |
| THE-D1-24 | Identity as a separation-algebra element | A name is an element of a partial commutative monoid; `∗` is its join | Calcagno, O'Hearn, Yang, LICS 2007 (abstract separation logic) | (outlandish) makes D6 an algebra question rather than a naming question |
| THE-D1-25 | Prophetic identity | The identity names the storage the value *will* occupy after a future relocation | Jung, Lepigre, Parthasarathy, Rapoport, Timany, Dreyer, Vindum, POPL 2020 | (outlandish) needs a prophecy resolution rule; erasure must remove it |
| THE-D1-26 | Stage-0 identity | Identities exist only at compile-time stage 0 and are erased before stage 1 code | MetaML (Taha, Sheard, TCS 2000); Davies, Pfenning, JACM 2001 | (outlandish) makes erasure structural, but the writer programs in two stages |

## D2 Where permission and state live

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D2-01 | Implicit checker context | Facts about identities held by the checker, never written | WF ENT-5 today `[survey]` | invisible to the writer; diagnostics must reconstruct why a fact died |
| THE-D2-02 | Explicit linear ghost tokens | The writer threads permissions as first-class erased values | Verus `Tracked<PointsTo>`; L3 `[survey]` | verbose; higher-order code needs permission-polymorphic signatures |
| THE-D2-03 | In the reference type | Permission is the reference's mode | Rust `&`/`&mut` `[survey]` | couples R1, R4, R5, R9 into one concept; the failure mode the map documents |
| THE-D2-04 | In the access convention per call | Permission is `let`/`inout`/`sink` on the parameter | Hylo/Val (JOT 2022); Swift SE-0176 `[survey]` | no first-class references, so stored permissions are inexpressible |
| THE-D2-05 | Graded context | The typing context carries a grade per variable from a semiring; permission is the grade | Brunel, Gaboardi, Mazza, Zdancewic, ESOP 2014; Granule (Orchard, Liepelt, Eades, ICFP 2019) | semiring choice fixes the whole permission algebra; grade arithmetic must stay decidable |
| THE-D2-06 | Quantitative type theory | Multiplicities are part of the dependent typing judgment itself | Atkey, LICS 2018; McBride 2016 | full dependent types; 0-use ghost terms are erasure by construction |
| THE-D2-07 | Coeffect (context demand) | Permission is a *requirement on the context*, computed bottom-up, dual to an effect | Petricek, Orchard, Mycroft, ICFP 2014 | reverses the direction of inference; interacts oddly with FN-1's top-down signatures |
| THE-D2-08 | Comonadic resource wrapper | The value is `D A` for a comonad `D` carrying its access right | Uustalu, Vene, ENTCS 2008 (comonadic notions of computation) | (unverified for this use) elegant but no known checker of this shape at scale |
| THE-D2-09 | Adjoint-logic mode | Permission is the *mode* of a proposition; mode shifts are explicit up/down shifts | Reed 2009 (unverified); Licata, Shulman, Riley, FSCD 2017; Pruiksma, Pfenning (unverified) | one framework unifies linear/affine/unrestricted; modes are a new sort to teach |
| THE-D2-10 | Bunched context | The context is a tree with `,` (separating) and `;` (sharing) nodes | O'Hearn, Pym, Bull. Symbolic Logic 1999 | (outlandish for a source language) but makes D6 aliasing structural, per argument group |
| THE-D2-11 | Hoare-monad index | Pre- and post-state live in the type of the computation, not in a context | HTT (Nanevski et al., ICFP 2006); Ynot (ICFP 2008) | everything effectful becomes monadic; direct-style imperative code disappears |
| THE-D2-12 | Predicate-transformer index | The type carries a weakest-precondition function | Dijkstra monads (Swamy et al., PLDI 2013; Ahman et al., POPL 2017) | WP computation must be a fixed terminating normalizer to satisfy M1 |
| THE-D2-13 | Parameterised/indexed monad | Type `M i j A` where `i`, `j` are store states | Atkey, JFP 2009 | same monadic cost; states become type indices with a decidable equality |
| THE-D2-14 | Effect row on the arrow | Permission appears as a row of labelled effects on the function type | Leijen, POPL 2017 (Koka); Wand, LICS 1987 (rows) | rows unify effects and permissions; row unification must stay syntax-directed |
| THE-D2-15 | Second-class values | Permissions may be passed down but never stored or returned | Osvald, Essertel, Wu, Alvarez, Rompf, OOPSLA 2016 | stack discipline by construction; no stored capability, which R9 may need |
| THE-D2-16 | Capture set on the type | The type records which capabilities a value captures | Capturing Types (Boruch-Gruszecki, Odersky et al., TOPLAS 2023, unverified) | capture polymorphism; boxing rules at abstraction boundaries |
| THE-D2-17 | Capability in lexical scope | Permission is an ordinary term-level capability, passed as a normal argument | Effekt (Brachthäuser, Schuster, Ostermann, OOPSLA 2020) | capabilities are second-class in Effekt; first-class needs capture checking |
| THE-D2-18 | Certificate beside the binary | Permissions live in a proof object checked by a small independent kernel | Necula, POPL 1997; TAL (Morrisett, Walker, Crary, Glew, TOPLAS 1999) | a proof-checking kernel to trust; matches M1's "explicit steps" framing |
| THE-D2-19 | Answer-type/continuation index | Permission lives in the continuation's answer type; state changes retype the continuation | Danvy, Filinski, LFP 1990 (unverified for this use) | (outlandish) delimited control in the core; hard to lower |
| THE-D2-20 | Session endpoint | Permission lives in a channel endpoint's protocol state | Honda, CONCUR 1993; Caires, Pfenning, CONCUR 2010 | turns every storage into a communicating process; heavy for local code |
| THE-D2-21 | Ghost field of the object | Permission is a ghost component of the value itself, mutated by proofs | Iris ghost state (POPL 2015); JML `ghost` fields | erasure must remove the field; the object then has two states to keep consistent |
| THE-D2-22 | Global whole-program table | Permission computed once over the closed world | closed-world points-to analysis practice | conflicts with FN-1; no per-call statement to read in a diagnostic |
| THE-D2-23 | Runtime tag with proofs deleting the check | Permission is dynamic in the model but statically eliminated | Vale generational references (Ovadia, not peer-reviewed) `[survey]` | M2 forbids the residue; useful as a baseline only |

## D3 The state vocabulary of a storage

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D3-01 | `Init` / `Uninit` / `Gone` | Three states per identity | MECHANISM-MAP section 6 `[survey]` | partial initialization of aggregates needs a per-field refinement |
| THE-D3-02 | Full/empty slot state in the type | Record types change shape as fields are taken and put | Cogent `take`/`put` (O'Connor et al., ICFP 2016) `[survey]` | slot state multiplies with nesting; notation must compress it |
| THE-D3-03 | View assertions including uninitialized | `T @ l` and `T? @ l` as separate view constructors | ATS (Zhu, Xi, PADL 2005) `[survey]` | a view algebra as a second language |
| THE-D3-04 | Row of field presence flags | The state is a record row; presence is a row label | Wand, LICS 1987; Rémy 1994 (unverified) | row unification for states; gives partial initialization for free |
| THE-D3-05 | Bitmask index over bytes or fields | State is a type-level bitset of initialized components | Dependent ML index sorts (Xi, Pfenning, POPL 1999) for the index language | a decidable bitset algebra; explodes for large aggregates |
| THE-D3-06 | User-declared typestate automaton | Each nominal declares named states and legal transitions | Strom, Yemini, IEEE TSE 1986; DeLine, Fähndrich, ECOOP 2004 `[survey]` | needs an aliasing story underneath, which D2 supplies |
| THE-D3-07 | Typestate with alias permissions | States plus `unique`/`full`/`share`/`pure`/`immutable` per reference | Bierhoff, Aldrich, OOPSLA 2007 `[survey]` | five permission kinds plus splitting rules; regularity cost under M4 |
| THE-D3-08 | Gradual typestate | Some states checked statically, some deferred; a gradual boundary | Wolff, Garcia, Tanter, Aldrich, ECOOP 2011; Garcia et al., TOPLAS 2014 | the deferred half is a runtime check, which M2 forbids |
| THE-D3-09 | Abstract predicates | The state is an opaque predicate the module may fold and unfold | Parkinson, Bierman, POPL 2005 | explicit fold/unfold steps, which fits M1's "explicit `use` steps" |
| THE-D3-10 | Inductive separation-logic predicates | `list(x)`, `tree(x)` as the state of a recursive structure | Reynolds, LICS 2002 `[survey]` | unfolding search is where solvers enter; a fixed unfold discipline is needed |
| THE-D3-11 | Refinement/liquid state | State is a refinement predicate over the value and the heap | Rondon, Kawaguchi, Jhala, POPL 2010 (low-level liquid types); Flux, PLDI 2023 | liquid inference uses qualifiers plus a solver; a fixed qualifier set is the deterministic subset |
| THE-D3-12 | State as a term over captured conditions | `ite(c, s1, s2)` rather than a lattice join | MECHANISM-MAP section 6 refinement from CASES.md `[survey]` | atom growth; a collapse rule where the condition is unnameable |
| THE-D3-13 | Three-valued abstraction | States are definite-true / definite-false / unknown per property | Sagiv, Reps, Wilhelm, TOPLAS 2002 (TVLA) | the unknown value is a fail-closed default; precision depends on instrumentation predicates |
| THE-D3-14 | Fractional state | Initialization or ownership as a fraction in (0,1] | Boyland, SAS 2003 `[survey]` | fraction arithmetic; a fixed halving discipline is the decidable subset |
| THE-D3-15 | Counting permissions | An integer count of outstanding readers, with a deterministic algebra | Bornat, Calcagno, O'Hearn, Parkinson, POPL 2005 | counts are decidable but bookkeeping-heavy; matches M1 better than fractions |
| THE-D3-16 | Grades from a semiring | State is `0`, `1`, `ω` or a richer semiring element | Ghica, Smith, ESAP/ESOP 2014; Choudhury, Eades, Eisenberg, Weirich, POPL 2021 | one algebra covers copy/affine/linear and read counts; semiring choice is load-bearing |
| THE-D3-17 | Protocol/session state | The storage's state is a position in a session type | Balzer, Pfenning, ICFP 2017 `[survey]`; Pfenning, Griffith, FoSSaCS 2015 | session syntax for local memory; polarity/shift discipline to teach |
| THE-D3-18 | Layout as an index | The physical layout is a type index that operations may change | Dargent for Cogent (unverified); Ribbit (unverified) | makes "stale layout" a typing error rather than a state; separates D3 from D8 |
| THE-D3-19 | Ordered-context position | State is implicit in where the storage sits in a non-commutative context | Polakow, Pfenning, TLCA 1999 | (outlandish) no state vocabulary at all; the order *is* the state |
| THE-D3-20 | Temporal-logic obligation | State includes an LTL obligation such as "eventually released" | Manna, Pnueli 1992 (unverified for this use) | (outlandish) liveness in a type system; checking becomes model checking |
| THE-D3-21 | No states: validity by construction | Every constructor produces a valid value; holes are unrepresentable | pure functional data; Cogent's boxed discipline | forbids hole-and-refill, which R3 requires |
| THE-D3-22 | Modal necessity/possibility | `□A` = always valid here, `◇A` = valid somewhere | Pfenning, Davies, MSCS 2001 | (outlandish) modal states instead of enumerated states; unclear payoff for memory |

## D4 Sequential aliasing policy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D4-01 | Shared XOR mutable | One writer or many readers, enforced on references | Rust `[survey]` | R9 becomes unrepresentable, forcing an interior-mutability hole |
| THE-D4-02 | Unrestricted aliasing, state per storage | Any number of writable pointers; safety from identity state alone | L3 unrestricted pointers; MECHANISM-MAP section 6 `[survey]` | R4 facts weaken exactly where identities coincide |
| THE-D4-03 | Aliasing allowed, writes serialized by a declared order | Aliases legal but a total write order is declared and checked | (no direct prior art known) | (outlandish) the order is a second specification with no obvious checker |
| THE-D4-04 | Owners-as-dominators | Aliasing free inside an owner; no reference crosses the boundary | Clarke, Potter, Noble, OOPSLA 1998 | forbids interior pointers escaping the owner, which R8 needs |
| THE-D4-05 | Ownership domains with declared links | Aliasing policy is per-domain and declared, not derived from nesting | Aldrich, Chambers, ECOOP 2004 | a policy language beside the type system; M4 spec-size cost |
| THE-D4-06 | Owner-as-modifier (universes) | Anyone may reference, only the owner may modify | Dietl, Müller, JOT 2005 | separates read aliasing from write aliasing without a mode on the pointer |
| THE-D4-07 | External uniqueness | One unique reference from outside; internal aliasing unconstrained | Clarke, Wrigstad, ECOOP 2003 | gives a movable aggregate with an aliased interior, which handle-passing wants |
| THE-D4-08 | Islands, balloons, confined types | Aliasing bounded by a boundary object or a module, never crossing it | Hogg, OOPSLA 1991; Almeida, ECOOP 1997; Bokowski, Vitek, OOPSLA 1999 | coarse: encapsulation by construction, no per-field facts inside the boundary |
| THE-D4-09 | Alias burying | Aliases are allowed but become unusable when a unique reference is taken | Boyland, SPE 2001 | a "buried" state per alias; deterministic and flow-based, no runtime residue |
| THE-D4-10 | Destructive read | Reading a unique location empties it | Wadler 1990; Cogent `take` | exactly the hole-and-refill shape; forces explicit refill everywhere |
| THE-D4-11 | Unique / shared / immutable capability lattice | A small lattice of reference capabilities with viewpoint adaptation | Gordon, Parkinson, Parsons, Bromfield, Duffy, OOPSLA 2012 (Midori/M#) | isolation recovery rules are subtle; the lattice is more surface under M4 |
| THE-D4-12 | Deny capabilities | Capabilities are defined by what they deny others | Pony (Clebsch, Drossopoulou, Blessing, McNeil, AGERE! 2015) `[survey]` | six capabilities plus viewpoint adaptation; large matrix |
| THE-D4-13 | Reference immutability qualifiers | A separate read-only axis orthogonal to ownership | Javari (Tschantz, Ernst, OOPSLA 2005); ReIm (Huang, Milanova, Dietl, Ernst, OOPSLA 2012); IGJ (Zibin et al., 2007, unverified venue) | the "read-only flag, not a loan" option in D12, with three shipped precedents |
| THE-D4-14 | Fractional permission as the policy | Write needs fraction 1; reads split the fraction | Boyland, SAS 2003 `[survey]` | a fixed split discipline to keep M1 |
| THE-D4-15 | Counting permissions | An integer reader count; write requires count 0 | Bornat et al., POPL 2005 | deterministic arithmetic; verbose accounting |
| THE-D4-16 | Per-scope exclusivity windows | Exclusivity holds over an access span, not over a reference | Swift SE-0176 (2017) | Swift falls back to dynamic checks for class/global storage, which M2 forbids |
| THE-D4-17 | Projections with lexical spans | Interior access is a scoped projection; no reference outlives it | Hylo subscripts (JOT 2022); Swift `_modify` (unverified) | projections may not escape; stored interior pointers are inexpressible |
| THE-D4-18 | Second-class references | References pass down but never up or into storage | Osvald et al., OOPSLA 2016 | eliminates escape analysis; forbids stored references outright |
| THE-D4-19 | Capture-checked first-class references | References are first class but their capture sets are tracked | Capturing Types (TOPLAS 2023, unverified) | capture polymorphism at every abstraction boundary |
| THE-D4-20 | Declared coincidence | Two identities may be declared equal at a call or in a struct | MECHANISM-MAP D6 framing `[survey]` | the "may-alias" declaration must propagate through the footprint algebra |
| THE-D4-21 | Adoption and focus | Aliased objects are adopted into a group; `focus` grants temporary exclusivity | Fähndrich, DeLine, PLDI 2002 | focus blocks are a natural exclusivity window with a purely static form |
| THE-D4-22 | Region isolation with viewpoint adaptation | Objects live in isolated regions; a region has one entry point | Project Verona (unverified); Encore/Kappa (Castegren, Wrigstad, ECOOP 2016) | region entry/exit discipline; matches arenas in D8 |
| THE-D4-23 | Aliasing legal but facts degrade monotonically | Aliasing never rejects; it only weakens the optimizer's facts | M5's own framing generalized | needs a fact lattice with a defined weakening, and a way to report lost speed |
| THE-D4-24 | Gradual aliasing policy per module | Strict modules and permissive modules interoperate at a checked boundary | Siek, Taha, Scheme Workshop 2006 (gradual typing) | the boundary is where a runtime check would live, which M2 forbids |
| THE-D4-25 | Ordered contexts forbid aliasing structurally | Non-commutative context means no two names denote one storage | Polakow, Pfenning, TLCA 1999 | (outlandish) the strictest policy; stack-shaped programs only |

## D5 The source of interference facts for parallel permission and the optimizer

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D5-01 | Effect footprints over identities | Reads and writes per identity per statement | DPJ (Bocchino et al., OOPSLA 2009) `[survey]` | a decidable index algebra for arrays |
| THE-D5-02 | Loans on references | Exclusion rather than access | WF PAR-1 today `[survey]` | a loan denies overlap the footprint would grant |
| THE-D5-03 | Fractional / counting permissions | Amount-based read sharing | Boyland SAS 2003; Bornat et al. POPL 2005 `[survey]` | full arithmetic pulls in a solver |
| THE-D5-04 | Region partitions | Heap partitioned into named disjoint regions | DPJ; Regent (Slaughter et al., SC 2015) `[survey]` | partitions are a second naming system beside identities |
| THE-D5-05 | Access declarations per task | The task declares which objects it reads and writes; the runtime checks | Jade (Rinard, Lam, TOPLAS 1998) | Jade resolves dynamically; the static residue is the declaration form |
| THE-D5-06 | Fully strict fork/join series-parallel structure | Interference is decided by the series-parallel shape of the spawn tree | Cilk (Blumofe et al., PPoPP 1995); Feng, Leiserson, SPAA 1997 | structure gives determinacy-race *detection*, not static freedom |
| THE-D5-07 | Hyperobjects / reducers | Shared accumulators made race-free by an associative monoid view | Frigo, Halpern, Leiserson, Lewin-Berlin, SPAA 2009 | exactly PAR-2's accumulator, generalized; needs a declared monoid |
| THE-D5-08 | Async/finish with places and clocks | Interference bounded by place locality plus phase clocks | X10 (Charles et al., OOPSLA 2005) | place affinity is a distribution concept; maps onto arenas |
| THE-D5-09 | Phasers / permission regions | Dynamic phases plus statically declared permission regions per task | Habanero; Westbrook, Zhao, Budimlić, Sarkar, ECOOP 2012 (unverified venue) | phaser semantics is runtime; the permission-region half is static |
| THE-D5-10 | Domains and locales | Index domains are first-class and distributed | Chapel (Chamberlain, Callahan, Zima, IJHPCA 2007) | domain algebra must be decidable to give static disjointness |
| THE-D5-11 | Logical regions with a partition tree | Hierarchical partitions whose disjointness is declared once | Legion (Bauer, Treichler, Slaughter, Aiken, SC 2012) | Legion checks dynamically; the static subset is Regent's |
| THE-D5-12 | SPMD rank ownership | Each rank owns its slice by construction; interference is impossible | MPI practice; ISPC (Pharr, Mark, InPar 2012) | restricts the programming model to a fixed decomposition |
| THE-D5-13 | Flattening / nested data parallelism | Parallelism from the shape of the data, not from effects | NESL (Blelloch, CACM 1996) | reshapes the program; interior pointers do not survive flattening |
| THE-D5-14 | Kahn process networks | Determinism from blocking single-reader channels | Kahn, IFIP Congress 1974 | determinism is a theorem about the model, not a checked property of code |
| THE-D5-15 | Synchronous dataflow / clock calculus | A clock type per stream decides which computations coexist | Lustre, Esterel, Signal (Halbwachs et al., Proc. IEEE 1991) | clock calculus is a real decidable type system; fits M1 well, fits mutable memory poorly |
| THE-D5-16 | Stream graphs with static rates | Interference from declared push/pop rates | StreamIt (Thies, Karczmarek, Amarasinghe, CC 2002) | only for streaming shapes |
| THE-D5-17 | Task-graph dependence clauses | The writer states `in`/`out`/`inout` dependences per task | OpenMP 4 `depend`; StarPU/OmpSs (unverified as papers) | the dependence set is exactly a footprint; the discharge is the question |
| THE-D5-18 | Liquid effects | Refinement-typed read/write sets over index predicates | Kawaguchi, Rondon, Bakst, Jhala, PLDI 2012 ("Deterministic parallelism via liquid effects") | designed for exactly this problem; liquid inference needs a fixed qualifier set |
| THE-D5-19 | Commutativity declarations | Overlap is legal when the operations commute, proved or declared | Rinard, Diniz, PLDI 1996; Pingali et al., PLDI 2011 | changes the determinism promise from source-order equality to result equality |
| THE-D5-20 | Monotone stores (LVars) | Shared state is a lattice with only least-upper-bound writes | Kuper, Newton, FHPC 2013; Kuper, Turon, Krishnaswami, Newton, POPL 2014 | determinism without disjointness; requires all writes to be joins |
| THE-D5-21 | Isolation types with deterministic merge | Tasks fork isolated copies and merge by a declared rule | Burckhardt, Baldassin, Leijen, OOPSLA 2010; ESOP 2011 | copies cost memory; merge is a per-type obligation |
| THE-D5-22 | CRDT-shaped shared values | Shared values are join-semilattices, so order does not matter | Shapiro, Preguiça, Baquero, Zawirski, SSS 2011 | restricts value types; no general mutable structure |
| THE-D5-23 | Futures over pure functions | Interference impossible because there are no effects | Steele, POPL 1990 ("Making asynchronous parallelism safe for the world") | purity is the restriction the project has already refused for hot code |
| THE-D5-24 | Purity plus uniqueness | In-place update legal because the array is unique | Futhark (PLDI 2017); Cogent `[survey]` | array-shaped only |
| THE-D5-25 | Ownership types for race freedom | Every object has an owner lock; the type says which | Boyapati, Rinard, OOPSLA 2001; Boyapati, Lee, Rinard, OOPSLA 2002 | ties interference to locks, which CAP-1 currently excludes |
| THE-D5-26 | `guarded_by` annotations | Each field declares the lock that protects it | Flanagan, Freund, PLDI 2000 | a lock-centric answer; static and simple, but assumes locks exist |
| THE-D5-27 | Atomicity by reduction | Interference judged by mover types (left/right/both movers) | Lipton, CACM 1975; Flanagan, Qadeer, PLDI 2003 | a genuinely different vocabulary: not disjointness but commutation with the environment |
| THE-D5-28 | Rely-guarantee pairs | Each statement declares what it tolerates and what it promises | Jones, TOPLAS 1983; Vafeiadis, Parkinson, CONCUR 2007 (RGSep) | two relations per statement; the strongest non-disjointness option |
| THE-D5-29 | Polyhedral dependence stated in source | The writer states the affine schedule and the checker verifies legality | Feautrier 1991; Bondhugula et al., PLDI 2008 | verification of a given schedule is decidable even when search is not |
| THE-D5-30 | Proof-carrying non-interference certificate | The writer supplies an explicit finite proof of disjointness | Necula, POPL 1997, applied to schedules | matches M1 exactly; certificate format is the work |
| THE-D5-31 | Graded effects | Effects carry semiring quantities, so "written twice" is distinguishable | Katsumata, POPL 2014; Orchard, Petricek, Mycroft | a quantity algebra beyond read/write; more precise footprints |
| THE-D5-32 | Coeffect demand per iteration | Each iteration declares which inputs it demands | Petricek, Orchard, Mycroft, ICFP 2014 | dual direction from effects; unusual for a parallel-permission rule |
| THE-D5-33 | Session-typed pipeline stages | Overlap legal because stages communicate on linear channels | Caires, Pfenning, CONCUR 2010; Wadler, ICFP 2012 | pipeline parallelism only; no loop-iteration overlap |
| THE-D5-34 | Determinism promise: source-order equality | Parallel run equals the sequential source order exactly | WF CAP-1 today `[survey]` | strongest; denies commutative accumulators |
| THE-D5-35 | Determinism promise: final-state confluence | Any schedule reaches the same final state | Kahn 1974; LVars POPL 2014 | permits reordered accumulation; changes what PAR-1 must prove |
| THE-D5-36 | Determinism promise: quasi-determinism | Same result or an error, never a wrong result | Kuper et al., POPL 2014 | the error outcome is a typed failure, which M2 permits |
| THE-D5-37 | Determinism promise: race freedom only | No data race; interleaving observable | DRF-SC (Adve, Hill, ISCA 1990) | weakest; the "second permission level" the map's lock finding asks for |
| THE-D5-38 | Determinism promise: invariant preservation | Only a declared invariant is guaranteed across overlap | CSL resource invariants (O'Hearn, TCS 2007) | matches locks; loses reproducibility of results |
| THE-D5-39 | Determinism promise: modulo declared commutativity | Equality up to a declared associative-commutative operator | Bocchino et al., POPL 2011 (safe nondeterminism) | floating-point associativity becomes a stated, not hidden, license |

## D6 Distinctness of identity parameters at function boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D6-01 | Distinct by default, call-site proof | Two identity parameters are disjoint unless proved equal | MECHANISM-MAP section 7 `[survey]` | every call proves pairwise distinctness, sub-identities included |
| THE-D6-02 | May-alias by default, callee proof | Overlap permitted; the callee proves it copes | WF OWN-7 today `[survey]` | loses the clean-shape default M5 wants |
| THE-D6-03 | Separation by distinct linear names | Two distinct capabilities are disjoint by construction; aliasing is inexpressible without an explicit equality | Alias Types, ESOP 2000; L3, TLCA 2005 | aliased arguments must be passed as one name plus a projection |
| THE-D6-04 | Connective-chosen distinctness | `P ∗ Q` disjoint, `P ∧ Q` overlapping; the signature picks per conjunct | Reynolds, LICS 2002; Ishtiaq, O'Hearn, POPL 2001 | two conjunctions in the contract language; writers must choose correctly |
| THE-D6-05 | Bunched argument groups | The parameter list is a bunch: `,`-separated arguments are disjoint, `;`-separated may alias | O'Hearn, Pym, Bull. Symbolic Logic 1999 | (outlandish surface) but the cleanest per-group statement of the question |
| THE-D6-06 | Freshness constraints | Signature carries `a # b` freshness assumptions | Pitts, Inf. & Comp. 2003 | a freshness judgment; already decidable in nominal logic |
| THE-D6-07 | Constructive apartness | Distinctness is a positive relation `a # b`, not the negation of equality | Bishop-style constructive apartness (unverified for this use) | (outlandish) but avoids needing decidable equality on identities |
| THE-D6-08 | Overlapping conjunction / septraction | A connective for "these share an unknown part" | Gardner et al. overlapping conjunction (unverified); Vafeiadis septraction (unverified) | expressive but exactly where entailment search begins |
| THE-D6-09 | Fractional split at the call | The same identity passed twice at half permission each | Boyland, SAS 2003 | aliasing needs no distinctness declaration at all; reads only |
| THE-D6-10 | Counting split at the call | Same identity passed n times with a reader count | Bornat et al., POPL 2005 | decidable; write requires a proof that the count is zero |
| THE-D6-11 | Declared per pair | An `aliases(a, b)` or `disjoint(a, b)` clause in the signature | DPJ `disjoint` region clauses (OOPSLA 2009) | O(n²) clauses for n parameters; M4 verbosity is cheap but the spec grows |
| THE-D6-12 | Declared equivalence relation | The call site supplies a partition of the identity parameters | (no direct prior art known) | one declaration replaces the pairwise clauses; needs a partition algebra |
| THE-D6-13 | Three-valued may/must alias in the type | A parameter pair is `must-distinct`, `must-equal`, or `unknown` | may/must alias analysis practice (unverified for a source type) | the unknown case must fail closed for safety and open for optimization |
| THE-D6-14 | Aliasing polymorphism | The callee is polymorphic in the aliasing configuration; the call instantiates | (no direct prior art known; adjacent to region polymorphism in Cyclone, PLDI 2002) | one body must typecheck under every configuration, or be checked per instantiation |
| THE-D6-15 | Region-constraint subtyping | Signatures carry outlives and disjointness constraints over regions | Cyclone (Grossman et al., PLDI 2002) | expresses lifetime well, disjointness weakly |
| THE-D6-16 | Index-arithmetic distinctness | Distinctness reduced to a decidable arithmetic fragment over indices | Presburger/affine fragments; Xi, Pfenning, PLDI 1998 | the fragment choice is itself a decision point (see "missing decision points") |
| THE-D6-17 | Partition-passing | Two parameters are replaced by one proved-partitioned container | Regent partitions (SC 2015); Legion index space trees | reshapes APIs; removes the pairwise proof entirely |
| THE-D6-18 | Callee-side magic wand | The callee returns a wand restoring the caller's frame, so overlap never arises | separating implication (Reynolds, LICS 2002) | wand reasoning is where search enters; a fixed apply discipline is needed |
| THE-D6-19 | Whole-program alias oracle | Distinctness answered from the closed world at each call | PROG-1 premise; Datalog points-to (Polonius, unverified) | contradicts FN-1; diagnostics are non-local |
| THE-D6-20 | Distinctness degrades, never rejects | Unproved pairs are may-alias; only optimizer facts are lost | M5's own floor rule generalized | R5 cannot degrade: an unproved pair forces sequential execution |
| THE-D6-21 | Identity inequality as a type judgment | A type-level apartness judgment over singleton identity types | Dependent ML (Xi, Pfenning, POPL 1999) | needs decidable disequality, harder than decidable equality |

## D7 The contract vocabulary at call boundaries

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D7-01 | requires/ensures over identity states | Entry and exit state per named identity | DESIGN.md Candidate A; Vault (DeLine, Fähndrich, PLDI 2001) `[survey]` | every state-changing callee must say so |
| THE-D7-02 | modifies/reads clauses | Footprint sets on the signature | Dafny (Leino, LPAR 2010); Low* (Protzenko et al., ICFP 2017) `[survey]` | best notation, solver discharge; needs a syntactic footprint algebra |
| THE-D7-03 | Store type in, store type out | The signature is a rewrite rule on the store typing | Alias Types, ESOP 2000; Capability Calculus (Walker, Crary, Morrisett, TOPLAS 2000) `[survey]` | complete store types; abstraction needs existentials |
| THE-D7-04 | Signature as a linear implication | The whole contract is `A ⊸ B` in the store algebra | Girard, TCS 1987; L3 | elegant and fully syntax-directed; the writer reads linear logic |
| THE-D7-05 | Hoare type | The function's type *is* `{P} A {Q}`; bind composes contracts | HTT (Nanevski, Morrisett, Birkedal, ICFP 2006); Ynot (ICFP 2008) | monadic style; direct imperative code disappears |
| THE-D7-06 | Predicate transformer in the signature | The callee publishes a weakest-precondition function the caller applies | Dijkstra monads (Swamy et al., PLDI 2013; Ahman et al., POPL 2017); F* | WP normalization must terminate deterministically |
| THE-D7-07 | Indexed-monad pre/post | Types `M s1 s2 A` compose states by index equality | Atkey, JFP 2009 | states become indices; equality decides composition |
| THE-D7-08 | Graded/effect-row signature | Effects as a row or a semiring grade on the arrow | Leijen, POPL 2017; Katsumata, POPL 2014; Marino, Millstein, TLDI 2009 | rows unify EFF-1/2 with contracts; row unification must stay syntax-directed |
| THE-D7-09 | Algebraic effect signatures with handlers | The callee declares operations; the caller supplies a handler | Plotkin, Pretnar, ESOP 2009 | handlers are control operators; interacts with M2's "no hidden control flow" |
| THE-D7-10 | Abstract predicates in signatures | Contracts over module-owned predicates with explicit fold/unfold | Parkinson, Bierman, POPL 2005 | fold/unfold as writer-supplied `use` steps fits M1 |
| THE-D7-11 | Refinement types on arguments and results | Contracts as refinement predicates rather than state names | Liquid types (Rondon, Kawaguchi, Jhala, PLDI 2008); F* (Swamy et al., POPL 2016) | subtyping entailment is a solver unless the qualifier set is fixed |
| THE-D7-12 | Typestate pre/post per parameter | `@pre(open) @post(closed)` on each tracked argument | Vault (PLDI 2001); Fugue (DeLine, Fähndrich, ECOOP 2004) | finite-state and deterministic; does not express holes or reallocation |
| THE-D7-13 | Adoption and focus clauses | The signature says which objects it adopts and which it focuses | Fähndrich, DeLine, PLDI 2002 | gives temporary exclusivity in a signature without a loan concept |
| THE-D7-14 | Permission consume/produce clauses | Parameters annotated `consumes`; results produce permissions | Mezzo (Pottier, Protzenko, ICFP 2013) `[survey]` | Mezzo needed a dynamic escape; that is the cautionary datum |
| THE-D7-15 | Existential result identities | `∃ρ'. cap(ρ') ⊗ ptr(ρ')` for fresh or reallocated storage | Alias Types; L3 `[survey]` | an unpack step per call |
| THE-D7-16 | Prophecy / backward function | The contract names the borrow's final value | RustHorn (Matsushita, Tsukada, Kobayashi, ESOP 2020); Aeneas (Ho, Protzenko, ICFP 2022) `[survey]` | needs prophecy resolution; unnecessary if access checks state per operation |
| THE-D7-17 | Two-state rely-guarantee pair | The signature states what the callee tolerates and promises about concurrent change | Jones, TOPLAS 1983 | the natural contract shape for D11's "foreign" storage |
| THE-D7-18 | Atomic triple | `⟨P⟩ f ⟨Q⟩`: the effect appears atomic to the environment | TaDA (da Rocha Pinto, Dinsdale-Young, Gardner, ECOOP 2014) | the right shape for lock-free library boundaries; proof burden is high |
| THE-D7-19 | Subjective self/other split | The contract distinguishes this thread's contribution from the environment's | FCSL (Nanevski, Ley-Wild, Sergey, Delbianco, ESOP 2014) | two views of state per contract; compositional for concurrency |
| THE-D7-20 | Set-valued frame expressions | Footprints as ghost sets or region expressions with a separator relation | Kassios, FM 2006; Smans, Jacobs, Piessens, ECOOP 2009; Banerjee, Naumann, Rosenberg, ECOOP 2008 | more expressive than name separation; set entailment needs a solver unless a syntactic set algebra is fixed |
| THE-D7-21 | Interface automaton per module | The contract is an I/O automaton; composition checks compatibility | de Alfaro, Henzinger, ESEC/FSE 2001 | composition rather than subsumption; unusual for calls |
| THE-D7-22 | Resource usage expressions | The contract is a trace language over the resource's operations | Igarashi, Kobayashi, POPL 2002 | fits files and sockets (D11) better than memory |
| THE-D7-23 | Session type on the call | A call is a protocol with typed message order | Honda, Yoshida, Carbone, POPL 2008 | heavy for a local call; natural for external resources |
| THE-D7-24 | Specification statement (refinement calculus) | The signature is a nondeterministic specification the body refines | Morgan, TOPLAS 1988; Back, von Wright 1998 | refinement obligations instead of type checking; a different checker shape |
| THE-D7-25 | Contract-as-type-level function | The post-state is computed by a compile-time function on the pre-state | MetaML-style staging (Taha, Sheard, TCS 2000); dependent type-level computation | needs a terminating normalizer; makes contracts programmable |
| THE-D7-26 | Explicit frame parameter | The caller passes its frame in and receives it back; no frame rule at all | (no direct prior art known; the inverse of the frame rule) | (outlandish) makes framing explicit and syntactic, at severe verbosity |
| THE-D7-27 | Bi-abduced contracts | Contracts inferred from the body by abduction, then published | Calcagno, Distefano, O'Hearn, Yang, JACM 2011 | search-based, so M1 is violated; closed-world inference is the D7 "no contracts" option |
| THE-D7-28 | No contracts, whole-program inference | The closed world removes the need for signatures | PROG-1 premise `[survey]` | contradicts FN-1; makes every error non-local |

## D8 Storage placement, relocation and moves

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D8-01 | Relocation as a name change | `realloc : cap(ρ) → ∃ρ'. cap(ρ')` | L3 (TLCA 2005); Alias Types `[survey]` | derived pointers must be re-derived; the most explainable rule |
| THE-D8-02 | Handles for aggregates, storage fixed | `own` aggregates travel as handles; storage never moves | MECHANISM-MAP section 6 `[survey]` | contracts must say whether the callee ends the storage |
| THE-D8-03 | Take/put on inline storage | Inline slots with type-level full/empty state | Cogent (ICFP 2016); ATS `[survey]` | slot state multiplies with nesting |
| THE-D8-04 | Region/arena placement with a linear free capability | Allocation in a named region; one capability frees the whole region | Walker, Crary, Morrisett, TOPLAS 2000; Cyclone (PLDI 2002) `[survey]` | coarse freeing; needs a per-object mechanism alongside |
| THE-D8-05 | Linear region values | The region is a linear value passed and consumed; space bounded by construction | Fluet, Morrisett, Ahmed, ESOP 2006 | region passing everywhere; gives the strongest bounded-memory statement |
| THE-D8-06 | Region inference | Placement derived by the compiler, not declared | Tofte, Birkedal, TOPLAS 1998 (MLKit); Birkedal, Tofte, Vejlstrup, POPL 1996 | inference is exactly what M4 says produces non-local diagnostics |
| THE-D8-07 | Storage-mode analysis (`at` vs `atbot`) | Whether an allocation resets its region or extends it, as a derived mode | Birkedal, Tofte, Vejlstrup, POPL 1996 | a second derived attribute; instructive as the lowering target |
| THE-D8-08 | Escape analysis for stack placement | Placement decided by whether a value escapes | Blanchet, POPL 1999 | derived, non-local; conflicts with declared placement |
| THE-D8-09 | Layout as a type index | Physical layout is part of the type; relocation changes the index | Dargent (unverified); Ribbit (unverified); Chandra, Reps, PASTE 1999 (physical type checking) | separates "stale layout" from "storage ended"; a layout algebra to specify |
| THE-D8-10 | Projections instead of interior pointers | Interior access through a scoped accessor that yields and resumes | Hylo subscripts (JOT 2022); Swift `_read`/`_modify` (unverified) `[survey]` | projections may not escape; coroutine lowering |
| THE-D8-11 | Lenses / optics as first-class projections | Interior access is a composable first-class lens value | Foster, Greenwald, Moore, Pierce, Schmitt, POPL 2005 | composition algebra is nice; a lens stored in a struct reintroduces the escape problem |
| THE-D8-12 | Zipper representation | Interior focus is represented as a decomposition of the structure, not a pointer | Huet, JFP 1997 | (outlandish for a systems language) but eliminates interior pointers entirely |
| THE-D8-13 | Index handles into a pool | Every object lives in a pool; access is index plus bounds proof | why-whitefoot section 10 `[survey]` | one indirection; identity becomes the pool plus an index |
| THE-D8-14 | Generational index (slot map) | Handles carry a generation, checked or proved | Vale generational references (not peer-reviewed) `[survey]` | the check is runtime unless proved away, which M2 requires |
| THE-D8-15 | Pinning as a type | An immobile marker on the type | Rust `Pin` `[survey]` | expresses less than a state event; idiom burden |
| THE-D8-16 | Immobile arena | A region declared non-relocating; interior pointers legal inside it | (no direct prior art known; adjacent to Cyclone regions) | placement declaration decides pointer legality, which is clean but coarse |
| THE-D8-17 | Move as a primitive with an explicit old-name death | `move` ends the source identity and names the destination | MECHANISM-MAP Step 1 `[survey]` | needs relocation to be contract-visible at every call |
| THE-D8-18 | No moves: copy-only or place-only | Values are copied or constructed in place; nothing relocates | mutable value semantics (Hylo, JOT 2022) | large aggregates are copied or must be pool-allocated |
| THE-D8-19 | Copy elision proved, not heuristic | In-place construction guaranteed by a checked rule rather than an optimization | C++17 guaranteed copy elision (unverified as a paper) | the guarantee must be in the semantics, not the optimizer |
| THE-D8-20 | Amortized potential annotations | Types carry potential; placement and reallocation costs are accounted | Hofmann, Jost, POPL 2003; RaML (TOPLAS 2012) | adds numeric annotations; gives a real bounded-memory theorem |
| THE-D8-21 | Ownership-typed IR with placement | Placement and ownership carried into the IR, verified by a syntactic checker | Swift SIL OSSA (unverified); MLIR bufferization (unverified) | a lowering precedent, not a source design |
| THE-D8-22 | World/place modality for placement | `A @ w` where `w` is stack, heap, arena, or a container slot | ML5 (Murphy VII, Crary, Harper, TGC 2007) | modal discipline; movement between places is an explicit term |
| THE-D8-23 | Two-level staging for placement | Stage 0 decides placement; stage 1 is the residual program | MetaML (Taha, Sheard, TCS 2000); Davies, Pfenning, JACM 2001 | (outlandish) the writer programs the layout decisions |
| THE-D8-24 | Flattening changes representation | The compiler is licensed to change layout, with proofs preserving semantics | NESL (Blelloch, CACM 1996) | interior identities do not survive; a very different R8 |

## D9 Container elements and backings

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D9-01 | Path identity `'v[i]` | Elements named by path with an index term | MECHANISM-MAP section 6 `[survey]` | index equality and disequality in a decidable fragment |
| THE-D9-02 | Existential backing per formation | The backing is unpacked at pointer formation | Alias Types `exists` `[survey]` | one unpack per formation; names accumulate in loops |
| THE-D9-03 | Per-element ghost tokens | A `PointsTo` permission per element, held in a ghost map | Verus (OOPSLA 2023) `[survey]` | map-level reasoning; key disjointness becomes a set problem |
| THE-D9-04 | Iterated separating conjunction | `⊛_{i∈S} elem(i)` over a decidable index set | separation-logic array predicates (Reynolds, LICS 2002 lineage) | a decidable set algebra over index sets; splitting lemmas as fixed rules |
| THE-D9-05 | Split-and-join range tokens | A range token splits into disjoint sub-ranges and rejoins | Rust `split_at_mut` pattern; WF `ProvedRangePartition` `[survey]` | join must restore exactly; partial rejoin is the hard case |
| THE-D9-06 | Dependent index refinement | Array access typed by a proof that the index is in bounds | Xi, Pfenning, PLDI 1998 ("Eliminating array bound checking") | a fixed index sort; the canonical M2-compatible bounds story |
| THE-D9-07 | Low-level liquid types | Heap refinements over C-like arrays with index predicates | Rondon, Kawaguchi, Jhala, POPL 2010 | liquid inference needs a fixed qualifier set to satisfy M1 |
| THE-D9-08 | Index sets as types | The index set is a first-class type; iteration is total over it | Dex (Paszke et al., ICFP 2021) | very regular, parallelism-preserving; reshapes container APIs |
| THE-D9-09 | Size-indexed arrays | Length in the type, symbolic or literal; bounds hold by construction | Agda `Vec`; GADT practice; Futhark size types (unverified as a paper) | type-level naturals and syntax-directed size unification; resizing changes the type |
| THE-D9-10 | Index-parameterized regions | Element regions indexed by a loop variable, disjoint by construction | DPJ (OOPSLA 2009) `[survey]` | the disjointness fragment must be fixed; affine index maps only |
| THE-D9-11 | Hierarchical partition tree | Backings decompose into a declared tree of disjoint partitions | Legion (SC 2012); Regent (SC 2015) | the tree is declared once and reused; static subset only |
| THE-D9-12 | Generative index brand | Each array gets a fresh brand; indices carry the brand as a proof of bounds | Kiselyov, Shan, ENTCS 2007 ("Lightweight static capabilities") | branding is scope-shaped; a stored index needs the brand stored too |
| THE-D9-13 | Element identity as a nominal atom | Insertion mints a fresh atom; the container is a finite map from atoms to storages | Pitts, Inf. & Comp. 2003 | identity is stable under relocation, which is attractive; but atoms are ghost data |
| THE-D9-14 | Focus on one element | A block grants temporary exclusive access to one element of an aliased group | Fähndrich, DeLine, PLDI 2002 | exactly the "one hot element" case; static and scope-shaped |
| THE-D9-15 | Abstract container predicate | The backing is an opaque predicate with fold/unfold as explicit steps | Parkinson, Bierman, POPL 2005 | hides the element structure; interior pointers need a focus rule |
| THE-D9-16 | Cursor as a linear value | An iterator is a linear token carrying its own invariant | Rust iterator pattern; session-typed iterators (unverified) | invalidation is a type error; nested iteration needs token splitting |
| THE-D9-17 | Ordered context for contiguous ranges | Ranges are ordered-context segments; split and join are the ordered rules | Polakow, Pfenning, TLCA 1999 | (outlandish) exactly matches contiguity, awkward for scatter |
| THE-D9-18 | Polyhedral index footprints | Element footprints are Presburger sets; disjointness is emptiness of an intersection | Feautrier 1991; Bondhugula et al., PLDI 2008 | Presburger is decidable but expensive; a sub-fragment must be fixed |
| THE-D9-19 | Permission distributed over shape | `cap(Array ρ n)` distributes to `∀i<n. cap(ρ[i])` by a fixed rule | (no single prior art; the distribution law in array separation logics) | the distribution law and its inverse must both be rules, not searched |
| THE-D9-20 | Per-element generation words | Elements carry generations; stale handles detected | Vale (not peer-reviewed) `[survey]` | runtime residue, M2 violated unless proved away |
| THE-D9-21 | Index-only access, no element pointers | Elements are never addressed; only `get`/`set` with a bounds proof | why-whitefoot handles `[survey]` | removes D9 almost entirely; costs interior-pointer expressiveness |
| THE-D9-22 | Backing as a capacity-indexed existential | `∃β. backing(v) = β ∧ cap(β) ∧ len ≤ capacity` with a conditional push contract | MECHANISM-MAP Step 3 `[survey]` | one contract clause states reallocation; `len < cap` becomes a proof obligation |
| THE-D9-23 | Prophetic element identity | Element identity names where the element will live after the next growth | Jung et al., POPL 2020 | (outlandish) survives reallocation by construction; erasure must remove it |

## D10 Shared mutation across threads and the concurrency story

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D10-01 | Resource invariants owned by a lock | Acquire yields the storage's facts; release checks the invariant | O'Hearn, CONCUR 2004 / TCS 2007; Brookes, CONCUR 2004 `[survey]` | invariant precision against WF's fact model is unchecked; two calls on one lock deny overlap |
| THE-D10-02 | Rely-guarantee | Each thread declares what it tolerates and promises | Jones, TOPLAS 1983 | two relations per thread; no disjointness needed, which is the point |
| THE-D10-03 | RGSep / SAGL / LRG | Rely-guarantee combined with separation: local state private, shared state governed | Vafeiadis, Parkinson, CONCUR 2007; Feng, Ferreira, Shao, ESOP 2007; Feng, POPL 2009 | a shared/local split in every assertion; a second reasoning mode |
| THE-D10-04 | Concurrent abstract predicates, first and higher order | Shared state abstracted per role with protocol-governed transitions; iCAP lets protocols quantify over protocols | Dinsdale-Young, Dodds, Gardner, Parkinson, Vafeiadis, ECOOP 2010; Svendsen, Birkedal, ESOP 2014 | proof search, and impredicativity in the higher-order form; a fixed protocol checker is the deterministic subset |
| THE-D10-05 | TaDA atomic triples | A library's operation appears atomic to clients | da Rocha Pinto, Dinsdale-Young, Gardner, ECOOP 2014 | the right client-facing contract for lock-free code; heavy proofs |
| THE-D10-06 | FCSL subjective state | Self and other contributions separated in every assertion | Nanevski, Ley-Wild, Sergey, Delbianco, ESOP 2014 | compositional; a new assertion shape for the writer |
| THE-D10-07 | Iris invariants plus ghost state | A general framework: invariants, view shifts, resource algebras | Jung et al., POPL 2015; JFP 2018 `[survey]` | needs a fixed catalogue of resource algebras to satisfy M1 |
| THE-D10-08 | Views framework | Concurrency reasoning parameterized by a chosen view monoid | Dinsdale-Young, Birkedal, Gardner, Parkinson, Yang, POPL 2013 | choosing the monoid is the design act; could fix one monoid per language feature |
| THE-D10-09 | GPS / RSL / FSL for relaxed memory | Ghosts, protocols and separation for release/acquire atomics | Turon, Vafeiadis, Dreyer, OOPSLA 2014; Vafeiadis, Narayan, OOPSLA 2013 | requires committing to a memory model (see missing decision points) |
| THE-D10-10 | Linearizability as the contract | Concurrent objects specified by an equivalent sequential object | Herlihy, Wing, TOPLAS 1990 | the specification is clean; proving it is not a fixed family |
| THE-D10-11 | Atomicity by movers | Overlap judged by whether operations commute with the environment | Lipton, CACM 1975; Flanagan, Qadeer, PLDI 2003 | mover types are a small regular vocabulary and fully static |
| THE-D10-12 | Type-based race detection | Every field declares its guarding lock | Flanagan, Freund, PLDI 2000 | simple and deterministic; assumes locks, denies lock-free code |
| THE-D10-13 | Ownership types with lock ownership | Objects have owners; owners have locks; lock order prevents deadlock | Boyapati, Rinard, OOPSLA 2001; Boyapati, Lee, Rinard, OOPSLA 2002 | gives deadlock freedom too, a property nothing else here provides |
| THE-D10-14 | Deny capabilities / actors | Sendability from a capability lattice; no shared mutable state between actors | Pony (AGERE! 2015) `[survey]`; Agha 1986 | actor granularity; Pony relies on per-actor GC |
| THE-D10-15 | Reference capabilities for regions | Isolated regions with a single entry point transferred between threads | Castegren, Wrigstad, ECOOP 2016 (Kappa); Encore (Brandauer et al., SFM 2015); Verona (unverified) | region entry discipline; matches arenas and handle passing |
| THE-D10-16 | Uniqueness and immutability for parallelism | `isolated`, `readable`, `immutable` with isolation recovery | Gordon, Parkinson, Parsons, Bromfield, Duffy, OOPSLA 2012 | isolation recovery is subtle but fully static, and shipped at scale |
| THE-D10-17 | Binary session types | Sharing replaced by linear channels with typed protocols | Honda, CONCUR 1993; Caires, Pfenning, CONCUR 2010 | no shared memory; copies or transfers instead |
| THE-D10-18 | Multiparty session types | A global protocol projected to per-role local types | Honda, Yoshida, Carbone, POPL 2008 | projection is decidable; global types are a second specification |
| THE-D10-19 | Manifest sharing | Acquire/release visible in the session type | Balzer, Pfenning, ICFP 2017 `[survey]` | acquire is a typed outcome with real control flow, which M2 accepts |
| THE-D10-20 | Linear channels in the pi-calculus | Channel usage typed as linear, giving race freedom | Kobayashi, Pierce, Turner, POPL 1996 / TOPLAS 1999 | a process calculus core; distant from imperative memory |
| THE-D10-21 | Kahn networks | Determinism from blocking single-reader channels | Kahn, IFIP Congress 1974 | determinism by model, not by check; no shared mutation at all |
| THE-D10-22 | Synchronous clocks | Determinism by a clock calculus; no threads, only reactions | Lustre/Esterel/Signal (Halbwachs et al., Proc. IEEE 1991) | a real decidable type system; restricts the programming model sharply |
| THE-D10-23 | Software transactional memory | Shared mutation inside typed transactions | Harris, Marlow, Peyton Jones, Herlihy, PPoPP 2005 | retry is runtime control flow, not a safety trap; M2's boundary is a judgment call |
| THE-D10-24 | Epoch / phase discipline | Readers in a phase, writers between phases; no per-read cost | RCU verification (Tassarotti, Dreyer, Vafeiadis, PLDI 2015) | the map's own catalogue already measures this pattern; needs phase facts in the checker |
| THE-D10-25 | Immutable sharing only | Shared data is immutable; mutation requires exclusive ownership | Pony `val`; functional practice | expressive loss for in-place algorithms |
| THE-D10-26 | Monotone shared state | Shared writes are lattice joins only | LVars (FHPC 2013; POPL 2014) | determinism with sharing, at the cost of arbitrary mutation |
| THE-D10-27 | CRDT values | Shared values converge regardless of order | Shapiro et al., SSS 2011 | restricts value types |
| THE-D10-28 | SCOOP processor tags | Each object has a processor; cross-processor calls are queued | Meyer, Eiffel SCOOP (unverified as a paper) | queuing is runtime; the tag discipline is static |
| THE-D10-29 | Separate thread level with weaker permission | A second permission whose promise is race freedom, not source-order equality | MECHANISM-MAP open question 11 `[survey]` | CAP-1 excludes it today; a recorded decision, not a derivation |
| THE-D10-30 | No threads at all | Process-level parallelism with message passing outside the language | (design stance) | removes D10; pushes the problem to D11 |
| THE-D10-31 | Deterministic parallelism only | Only statically disjoint overlap, never general sharing | DPJ (OOPSLA 2009); Regent (SC 2015) `[survey]` | the allocator/lock case in evidence part D has no answer under this stance |

## D11 External resources

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D11-01 | Foreign identity qualifier | Storage another agent may write without an edge in this program's order | MECHANISM-MAP section 5 `[survey]` | no contents fact survives any operation; extent facts stay ordinary |
| THE-D11-02 | Rely condition for the environment | The external agent's permitted steps stated as a rely relation | Jones, TOPLAS 1983 | the general form of "foreign"; two-state contracts at every operation |
| THE-D11-03 | Havoc between calls | External storage is arbitrarily reassigned between operations | Boogie modelling practice (Barnett et al., FMCO 2005) | the coarse deterministic form of the rely condition |
| THE-D11-04 | Unique world token | The outside world is a unique value threaded through effectful code | Clean `*World` (Barendsen, Smetsers, MSCS 1996) | makes I/O ordering a data dependency; verbose but fully static |
| THE-D11-05 | IO monad / state-transformer | External effects sequenced by a monad | Peyton Jones, Wadler, POPL 1993 | monadic style; ordering by bind rather than by identity state |
| THE-D11-06 | Hoare-monadic external state | Pre/post over an external heap | HTT (ICFP 2006); Low* external state (ICFP 2017) `[survey]` | the external heap is a second heap with its own frame rule |
| THE-D11-07 | Algebraic effect operations | External operations declared as effect signatures; handlers interpret them | Plotkin, Pretnar, ESOP 2009; Koka (Leijen, POPL 2017) | handlers reintroduce control flow; a handler-free subset is possible |
| THE-D11-08 | Capability-passing effects | Effects are capabilities passed as arguments, second-class | Effekt (Brachthäuser, Schuster, Ostermann, OOPSLA 2020) | second-class capabilities cannot be stored; a file handle in a struct is then illegal |
| THE-D11-09 | Capture-checked capabilities | Capabilities are first-class with tracked capture sets | Capturing Types (TOPLAS 2023, unverified) | capture polymorphism at every abstraction boundary |
| THE-D11-10 | Object-capability discipline | Authority comes only from references you were given | Miller 2006 dissertation; Maffeis, Mitchell, Taly, IEEE S&P 2010; Joe-E (NDSS 2010) | ambient authority must be removed from the language, including globals |
| THE-D11-11 | Resource usage types | A usage expression per resource describing legal operation sequences | Igarashi, Kobayashi, POPL 2002 | inference is constraint-based; a checked-only fragment is deterministic |
| THE-D11-12 | Typestate protocol per resource kind | File and socket protocols as declared automata | Vault (PLDI 2001); Fugue (ECOOP 2004); ESP (Das, Lerner, Seigle, PLDI 2002) | finite state, deterministic, local diagnostics; no contents reasoning |
| THE-D11-13 | Session type per external endpoint | The OS API is a protocol partner | Honda, Yoshida, Carbone, POPL 2008 | protocol conformance for the whole API surface; large specification |
| THE-D11-14 | Interaction trees as the semantics | External behaviour modelled as a coinductive interaction tree | Xia, Zakowski, He, Hur, Malecha, Pierce, Zdancewic, POPL 2020 | a semantics choice for D13, not a source mechanism |
| THE-D11-15 | Three identities per descriptor | Wrapper with the close obligation, open-file description with the cursor, contents any agent may write | evidence part D `[survey]` | needs no new vocabulary; contracts per operation |
| THE-D11-16 | States as ordinary enums | No special vocabulary; the resource's state is a value the program branches on | design stance `[survey]` | pushes safety onto the writer's control flow; M2-compatible by construction |
| THE-D11-17 | World-indexed modal types | `A @ w` where `w` is a device, a mapping or a peer | ML5 (TGC 2007) | modal discipline for locality; unusual for file descriptors |
| THE-D11-18 | Trusted adapter outside the language | A small trusted base written elsewhere with a proof obligation | Cogent's C FFI with refinement proofs (ICFP 2016); F* extraction | M3 says no writer-accessible escape; this puts the escape outside the writer's language |
| THE-D11-19 | Proof-carrying foreign module | The foreign side ships a certificate the compiler checks | Necula, POPL 1997 | a certificate format for foreign code; heavy but removes the trusted base |
| THE-D11-20 | Effect categories for host mechanisms | Labels like `external`, `blocks` on operations | early WF `external` `[survey]` | rejected by effects.md as mechanism labels rather than state |
| THE-D11-21 | Volatile as a no-frame storage class | Device registers are storages for which no contents fact is ever retained | C `volatile` semantics (unverified as a formal account) | the narrow special case of the foreign qualifier |
| THE-D11-22 | Angelic/demonic external outcome | Every external operation returns a typed sum, never a trap | refinement calculus (Back, von Wright 1998) | matches M2 exactly; every call site branches |

## D12 Surface form and sugar

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D12-01 | `&`/`&uniq` as sugar over pointer plus state clauses | Existing spellings desugar to the new core | MECHANISM-MAP open question 9 `[survey]` | the desugaring must be total, or the sugar leaks |
| THE-D12-02 | Explicit-only core, no sugar | Everything written; the AI writer pays the verbosity | Alias Types store types; Austral's stance | diagnostics are local; programs are long |
| THE-D12-03 | Untrusted elaborator to a checked core | The surface is an untrusted front end; only the core calculus is checked | proof-assistant architecture; de Bruijn criterion | matches M1 exactly; the core must be small and stable |
| THE-D12-04 | Tactic language producing explicit steps | Proofs written as deterministic scripts that expand to finite `use` steps | Coq/Lean tactic practice; Lithium in RefinedC (Sammler et al., PLDI 2021) | Lithium is the closest precedent for goal-directed *deterministic* proof search |
| THE-D12-05 | Bidirectional checking decides annotation sites | Annotations required exactly where checking cannot synthesize | Pierce, Turner, TOPLAS 2000 | gives a principled answer to "what must the writer write at cut points" |
| THE-D12-06 | Implicit arguments with deterministic resolution | Identity parameters implicit, resolved by a fixed instance search | Agda/Lean implicit arguments; type classes | resolution must be terminating and confluent to satisfy M1 |
| THE-D12-07 | Elision rules for identities | A generalization of lifetime elision to identity parameters | Rust lifetime elision (unverified as a paper) | elision hides exactly the thing diagnostics must name |
| THE-D12-08 | Separation-logic notation in source | `∗`, `⊸`, `▷` written directly | Reynolds, LICS 2002; Iris notation | small and precise; unfamiliar and hard to render in plain text |
| THE-D12-09 | Modal/graded annotation syntax | `[e]A`, `!A`, `□A` for permissions and grades | Granule (ICFP 2019); Linear Haskell multiplicities (POPL 2018) | one uniform annotation position; a semiring to teach |
| THE-D12-10 | Effect rows as record syntax | Effects spelled as a row `<reads r, writes w>` | Koka (Leijen, POPL 2017) | unifies EFF-1 with contracts; row syntax must not collide with records |
| THE-D12-11 | Ghost blocks in comment-like syntax | Contracts in a separate annotation layer | ACSL, JML | keeps the core grammar small; two grammars to maintain |
| THE-D12-12 | Contracts as first-class terms | Specifications are ordinary values in the same language | F*; Lean | reflection; the proof language and the program language coincide |
| THE-D12-13 | Two-level surface with a proof stage | Proofs at stage 0, program at stage 1 | MetaML (TCS 2000); Davies, Pfenning, JACM 2001 | (outlandish) makes erasure structural; the writer works in two languages |
| THE-D12-14 | Typed AST as the source of record | The AI writer emits a typed AST; text is a rendering | structure editors; (no PL-theory citation) | (outlandish) matches M4's premise directly; breaks every text-based tool |
| THE-D12-15 | Hygienic macros expanding to the core | Sugar defined in-language rather than in the compiler | Scheme `syntax-rules` (Kohlbecker et al., LFP 1986) | sugar becomes a library; diagnostics must map back through expansion |
| THE-D12-16 | Hole syntax for uninitialized slots | `T?` or a distinguished hole spelling per state | ATS `T?` views; Cogent `take` | one spelling per state keeps M4 regular; the state space must stay small |
| THE-D12-17 | Qualifier-prefixed types | `own`, `ref`, `foreign` as type qualifiers | C `const`/`restrict`; Cyclone qualifiers | familiar; qualifiers compose badly with generics |
| THE-D12-18 | Region syntax reused with identity meaning | `'s` kept but meaning identity, not duration | MECHANISM-MAP section 6 `[survey]` | familiarity helps; the changed meaning is a teaching hazard |
| THE-D12-19 | Region syntax retired entirely | New spellings so no reader carries the old meaning | (design stance) | churn in every existing test and pattern card |
| THE-D12-20 | Diagnostics as a specified artifact | Error messages are typed terms with a stable schema, part of the spec | (no PL-theory citation; adjacent to proof-state printing) | makes M4's locality requirement checkable rather than aspirational |
| THE-D12-21 | No surface pointers | Only paths, indices and projections | Hylo (JOT 2022) `[survey]` | removes most of D12; removes stored references too |

## D13 Validation and migration strategy

| id | Option | One-line description | Prior art or source | What it would need; obvious tensions |
|---|---|---|---|
| THE-D13-01 | Exhaustive small-model enumeration | Enumerate all programs under a size bound and compare checker to semantics | WF `research/experiments/access-state` `[survey]` | a reference semantics to compare against; bound must cover the hazard ladder |
| THE-D13-02 | Syntactic type soundness | Progress and preservation for a core calculus | Wright, Felleisen, Inf. & Comp. 1994 | a core calculus small enough to prove; the gap to the real language is the risk |
| THE-D13-03 | Semantic soundness by logical relations | Interpret types as predicates; prove the interpretation sound | RustBelt (Jung, Jourdan, Krebbers, Dreyer, POPL 2018) | allows a trusted core to be proved separately; Iris-scale effort |
| THE-D13-04 | Step-indexed logical relations | Handle recursive types and state by step indexing | Appel, McAllester, TOPLAS 2001; Ahmed, ESOP 2006 | the standard technique when identities are recursive |
| THE-D13-05 | Mechanized metatheory in a proof assistant | Coq/Rocq, Isabelle, Agda or Lean development of the type system | POPLmark (Aydemir et al., TPHOLs 2005) as the precedent | large effort; keeps pace with a changing spec poorly |
| THE-D13-06 | Executable semantics plus random testing | PLT Redex or K model, randomly tested against the checker | Felleisen, Findler, Flatt 2009; Roşu, Şerbănuţă 2010 (K, unverified venue) | a second semantics artifact to keep current |
| THE-D13-07 | Property-based testing of metatheory | Generate well-typed terms and test preservation | QuickChick (Paraskevopoulou, Hriţcu, Dénès, Lampropoulos, Pierce, ITP 2015); Hriţcu et al., ICFP 2013 | good generators for well-typed programs are the hard part |
| THE-D13-08 | Bounded model finding | Alloy or a SAT/SMT model finder over the rules, used at design time only | Jackson, TOSEM 2002 | solvers are forbidden for *acceptance*, not for metatheory; the distinction must be stated |
| THE-D13-09 | Differential acceptance testing | Run the old borrow checker and the new identity checker over the corpus and diff the accepted sets | (standard practice; no single citation) | the sharpest migration evidence available; needs both checkers alive at once |
| THE-D13-10 | Differential testing against Rust/C | Compare accepted/rejected programs and runtime behaviour | `[survey]` | Rust's `unsafe` and UB make it an imperfect oracle |
| THE-D13-11 | Dynamic monitor over a shadow heap | An interpreter carrying identity states and checking the discipline at runtime, as Miri does for Stacked/Tree Borrows | Stacked Borrows (POPL 2020); Tree Borrows (PLDI 2025) | a second implementation of the model; the cheapest dynamic oracle for R1 and R6 |
| THE-D13-12 | Race-detector oracle for R5 | Run permitted-overlap programs under a determinacy race detector | Feng, Leiserson, SPAA 1997; ThreadSanitizer (unverified as a paper) | validates parallel permission empirically rather than by proof |
| THE-D13-13 | Metamorphic testing of permission | Run the same program sequentially and in parallel and require identical results | (standard practice; no single citation) | directly tests the source-order-equality promise |
| THE-D13-14 | Translation validation of erasure | Prove per-compilation that erasing proofs preserves semantics | Pnueli, Siegel, Singerman, TACAS 1998 | cheaper than a verified compiler; validates R18 if adopted |
| THE-D13-15 | Verified compilation of the erasure step | Prove once that lowering preserves the checked facts | CompCert (Leroy, CACM 2009); CakeML (Kumar, Myreen, Norrish, Owens, POPL 2014) | large; probably out of scope but the right long-term target |
| THE-D13-16 | Proof-carrying acceptance with a kernel | The checker emits a certificate a small independent kernel rechecks | Necula, POPL 1997; de Bruijn criterion | makes M1 auditable; requires a certificate format for every derivation family |
| THE-D13-17 | Soundness by translation into a proved core | Define the language by elaboration into a calculus already proven sound | L3 (TLCA 2005) or Iris as the target | the elaboration itself must be proved type-preserving |
| THE-D13-18 | Csmith-style generation | Generate random valid programs to stress the checker and backend | Yang, Chen, Eide, Regehr, PLDI 2011 | generating *well-typed* programs under an ownership discipline is the hard part |
| THE-D13-19 | Equivalence modulo inputs | Mutate programs in ways that must not change behaviour | Le, Afshari, Su, PLDI 2014 | good for the backend; weak for acceptance-set questions |
| THE-D13-20 | Mutation testing of the checker | Delete or weaken a rule and require a test to fail | (standard practice; no single citation) | measures whether the conformance corpus actually pins the rules |
| THE-D13-21 | Hazard-ladder coverage metric | Every rung of section 3 has a discriminating program that must be refused | MECHANISM-MAP section 3 `[survey]` | makes coverage a checkable property of the corpus |
| THE-D13-22 | Mechanical source-to-source migration | A translator rewriting `&`/`&uniq` code into identity plus state clauses | (standard practice; adjacent to Rust's `cargo fix`) | the translator is throwaway; must be deleted after use under the hygiene rule |
| THE-D13-23 | Acceptance-set diff report per spec version | Each spec revision publishes which programs changed status | (no single citation; the rule-4 framing generalized) | ties D13 to the repository's existing merge rules |
| THE-D13-24 | Parallel section-5 rewrite vs patch | Whether the new mechanism lands as a patched section 5 or a new section | `[survey]` | a patched section risks two vocabularies coexisting mid-migration |

## Decision points missing from this list

| id | Missing point | Why it is a separate decision | Options it would enumerate |
|---|---|---|
| THE-DPX-01 | Recursive and cyclic structures | The ladder stops at arrays and vectors; recursive identity is where every system in the survey either adds inductive predicates or gives up | inductive predicates with explicit fold/unfold (Walker, Morrisett, TIC 2000); existential recursive store types; parent pointers as indices; arena-of-nodes with index handles; ownership-tree plus back-pointer weak identities |
| THE-DPX-02 | Closures and higher-order code | A closure captures identities and states; FN-1 signature-only checking says nothing about a closure's footprint | closure as an existential package of captured identities; effect/identity polymorphism on the closure type; second-class closures (OOPSLA 2016); capture sets (TOPLAS 2023, unverified); defunctionalization to a closed set of cases |
| THE-DPX-03 | Generics and polymorphism over identities and states | Whether nominals and functions may abstract over identity parameters, and with what variance; this is live work on the current branch | monomorphization vs dictionary passing; identity-parameter variance; bounded identity polymorphism; existential abstraction at module boundaries |
| THE-DPX-04 | Dynamic dispatch and behavioural subtyping | A trait object's contract must be a subtype of the implementation's | behavioural subtyping (Liskov, Wing, TOPLAS 1994); contract inheritance; no dispatch at all; closed-world devirtualization under PROG-1 |
| THE-DPX-05 | The arithmetic and index fragment | Every index-disjointness option in D5, D6 and D9 defers to "a fixed decidable fragment"; which one is itself a decision | intervals; affine/octagon; Presburger; DML index sorts (Xi, Pfenning, POPL 1999); bit-vector-free naturals; writer-supplied arithmetic `use` steps only |
| THE-DPX-06 | The equality judgment on identity terms | When two identity expressions denote the same storage: syntactic, normalized, or up to a decided theory | syntactic equality; confluent rewriting to a normal form; equality modulo a fixed path algebra; decidable disequality for D6 |
| THE-DPX-07 | Erasure and the fact channel to the backend | Which checked facts cross into the IR, in what encoding, and who trusts them; the map notes R4 has *no* channel today | LLVM `noalias`/`alias.scope`/`initializes`; a WF-specific fact IR; an ownership-typed IR with a syntactic verifier (Swift OSSA, unverified); certificates consumed by the backend |
| THE-DPX-08 | The trusted computing base | Which parts of the compiler must be trusted, and whether a recheck kernel exists | monolithic trusted checker; certificate plus small kernel; verified checker; differential double-check |
| THE-DPX-09 | Error outcomes, unwinding and linear obligations | A typed failure path must still discharge every obligation; the R set does not cover this | no unwinding; result types only; linear-safe unwinding with a per-frame release graph; obligations sunk into a typed error value |
| THE-DPX-10 | Concurrency memory model | D10 chooses a sharing discipline but never the model accepted programs obey | SC; DRF-SC (Adve, Hill, ISCA 1990); release/acquire with GPS-style protocols (OOPSLA 2014); no atomics at all |
| THE-DPX-11 | Allocator authorship | Whether a user can write an allocator; evidence part D found the lock/metadata case has no answer | allocator as a language primitive; allocator as a trusted library; allocator expressible with a second permission level; arenas only |
| THE-DPX-12 | Compile-time evaluation and staging | Whether proofs and identities may be computed at compile time, and whether const-eval participates | no const-eval; total const-eval; two-level staging (TCS 2000); type-level computation with a terminating normalizer |
| THE-DPX-13 | Incrementality and separate compilation | PROG-1 closes the world, but rechecking cost and caching are real constraints | whole-program only; per-function proof caching keyed by signature; certificate reuse; no incrementality |
| THE-DPX-14 | Subtyping and variance of states and identities | Whether `Init <: MaybeInit`, whether identity parameters are covariant, whether there is subtyping at all | no subtyping; state lattice with subsumption; variance annotations; subtyping only at declared coercion points |
| THE-DPX-15 | Proof-language identity | Whether the proof language is the program language (reflection) or a separate sub-language | reflection (F*, Lean); separate ghost sub-language (Verus, ACSL); no proof language, only fixed certificate forms |
| THE-DPX-16 | Globals, statics and initialization order | Ambient authority and initialization are untouched by R1..R10, and object capabilities forbid them outright | no globals; const-only globals; initialization-order proofs; globals as capabilities passed from `main` |

## Requirements missing or misstated (D0 detail)

| id | Item | Observation | Consequence |
|---|---|---|
| THE-REQ-01 | R11 abstraction is absent | R1..R10 never mention generics, closures, modules or dynamic dispatch, yet every store-typing system in the survey needs location polymorphism and existential packing to scale past one function | the mechanism comparison is being run on a first-order fragment; families that score well there may not survive abstraction |
| THE-REQ-02 | R12 error outcomes are absent | M2 converts falsifiable conditions into typed outcomes, which multiplies control-flow paths, and R2 must discharge obligations on every one of them | the R2 join rule and the M2 outcome rule interact and neither states the interaction |
| THE-REQ-03 | R13 bounds are absent | "Very high performance" is a premise but termination, stack depth and allocation bounds are never required | the language could accept programs whose memory is unbounded, which contradicts the "where bounded memory is promised" clause inside R2 |
| THE-REQ-04 | R14 checking cost is absent | M1 requires determinism and budget-freedom but permits an exponential fixed family | budget-free and cheap are different guarantees; a fixed family can be unusable |
| THE-REQ-05 | R15 human auditability is absent | M4 optimizes for an AI writer; the owner still reads diffs, diagnostics and design prose | verbosity cheap for the writer can be prohibitive for the reviewer, and nothing records that trade |
| THE-REQ-06 | R18 erasure is a premise, not a requirement | "Proofs are erased before lowering" appears in the constitution, never as a checkable requirement with a theorem | nothing in D13 is obliged to validate it |
| THE-REQ-07 | R4 and R5 are not independent | Both are consumers of the same distinctness facts; section 7 says so, section 2 presents them as separate | a mechanism change for R4 silently retunes R5's permission judgment |
| THE-REQ-08 | R6 and R7 are not independent | Framing is the intra-procedural half of what a signature states inter-procedurally; Prusti and Flux derive both from one structure | choosing a contract vocabulary largely chooses the kill rule |
| THE-REQ-09 | R2 is a special case of R3 | Linear multiplicity plus an obligation is exactly R2; Clean's uniqueness shows the one part that differs (last-reference vs obligation) | keeping them separate duplicates the join rule |
| THE-REQ-10 | R9 is the negation of part of R4 | R4 wants exclusivity facts; R9 wants several writers; R9(d) already admits the trade | the pair should be stated as one axis with a per-storage choice, not two requirements |
| THE-REQ-11 | R1 bundles three properties | Initialization, storage end, and layout currency have different mechanisms in ATS, Cogent and Dargent | one state vocabulary is being chosen for three questions that may want different answers |
| THE-REQ-12 | M1 "no SMT" is a mechanism | The underlying requirement is reproducible, explainable, non-flaky acceptance; a fixed-portfolio solver with a checked proof certificate would also satisfy that | the constraint as written forecloses Lithium-style deterministic goal-directed search (RefinedC, PLDI 2021), which is neither SMT nor unbounded |
| THE-REQ-13 | M2 "no runtime check" is a mechanism | The requirement is no hidden control flow and no unpredictable cost; a branch proved unreachable and a typed outcome both satisfy it, and STM retry arguably does | the stated form also forbids gradual and dynamic-boundary designs outright without arguing them |
| THE-REQ-14 | M3 "no unsafe escape" has a hole at R10 | Every external adapter is trusted code somewhere; the requirement is silent on where the trusted base lives | D11's "trusted adapter" option is either a violation or an admitted exception, and the text does not say which |
| THE-REQ-15 | M5 is a goal, not a checkable property | "The accepted shapes are the fast shapes" cannot be falsified by a checker | it can only be validated by measurement, which makes it a D13 obligation nobody has assigned |
| THE-REQ-16 | FN-1 plus PROG-1 are in tension | A closed world makes whole-program inference available; signature-only checking is therefore a deliberate restriction with its own grounds, not a consequence | D6's "inferred from the closed world" and D7's "no contracts" options are currently foreclosed by a premise rather than by an argument |
| THE-REQ-17 | "No runtime check even where a typed outcome is cheap" | The premise list states this as an open challenge; the requirement text (M2) does not distinguish a *safety* trap from an ordinary conditional the writer wrote | without that distinction, bounds-checked `get` returning an option is indistinguishable from a trap in the requirement's language |


# Lens file: numbered-history.md


| id | Convention | Meaning |
|---|---|
| Lens | Whitefoot's own record only: options this repository proposed, tried, deferred, refused or partly adopted, with the reason recorded at the time. No ranking, recommending or pruning. Quotes verbatim; paths relative to `/private/tmp/whitefoot-access-effects-research` |
| ADOPTED / DEFERRED / REFUSED | live in `spec/kernel-spec.md` or `design/` · recorded as deferred with a delta or re-entry trigger · recorded as rejected with its reason |
| SUPERSEDED / FLOATED | was live, replaced, reason kept in an `.alt/` node or a `Rejected:` line · surveyed or sketched, never selected |
| † · (m) | the recorded reason depends on permission riding on the reference (dependent premise in brackets) · the record cites a measurement, quoted |

---

## D0 The requirements themselves

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D0-01 | R1..R10 + M1..M5 as stated | ten independent requirements plus five meta-constraints | `research/investigations/access-effects/MECHANISM-MAP.md:26-150` | FLOATED | "Each requirement gives (a) what must hold, (b) the least a checker must know" |
| HIS-D0-02 | M1 no-SMT as a decision, not a given | fixed terminating families plus explicit certificates | `design/language/checks-and-proofs.md:2` | ADOPTED | "acceptance must never depend on solver state, a timeout, machine speed, or a work budget" |
| HIS-D0-03 | M2 no-trap as a decision with rivals | typed outcome instead of a runtime check | `design/language.md:1` | ADOPTED | "any writer-reachable escape becomes an unauditable failure edge in generated code" |
| HIS-D0-04 | Result-everywhere (every fallible op returns a value) | replaces traps with typed outcomes everywhere | `design/language/checks-and-proofs/obligation-discharge.md:51` | REFUSED | "each checker-incompleteness site would force an error arm for a condition that is impossible" |
| HIS-D0-05 | Global prove-or-handle as language law | unproved obligation is always a compile error | `design/language/checks-and-proofs/obligation-discharge.md:52` | REFUSED | "a deterministic no-search checker leaves a true-but-unprovable residue that would be camouflaged" |
| HIS-D0-06 | Assume-without-check (SPARK `pragma Assume`) | writer states a fact, no proof, no check | `research/investigations/obligation-discharge/DOSSIER.md:59` | REFUSED | "Categorically rejected; see the W3 keystone in §3" |
| HIS-D0-07 | Implicit retained checks on every unproved obligation | a trap-everything compilation mode | `design/language/checks-and-proofs/obligation-discharge.md:50` | REFUSED | "the trap surface was neither stated nor enumerable, a caller could not tell from a signature" |
| HIS-D0-08 | FN-1 signature-only checking | a call is judged by the callee's signature alone | `spec/kernel-spec.md` FN-1; `MECHANISM-MAP.md:96` | ADOPTED | "Not its job: inferring anything from bodies" |
| HIS-D0-09 | PROG-1 closed world | one compilation unit, no modules or separate compilation | `design/language.md:6` | ADOPTED | "a module or separately compiled interface is a fact-loss surface" |
| HIS-D0-10 | Exceptions / resumable effect handlers | a second control mechanism beside Result | `design/language.md:7` | REFUSED | "a second control mechanism would duplicate what those values already express" |
| HIS-D0-11 | No exponential checking work | compilation never grows exponentially in program or proof size | `design/language.md:2` | ADOPTED | "guaranteed termination alone does not make a checker usable" |
| HIS-D0-12 | Migration cost / corpus frequency as a ground | choose by what today's programs do | `design/language.md:5` | REFUSED | "the corpus was written to exercise the compiler rather than to represent real programs" |
| HIS-D0-13 | Frequency/prevalence as a veto on capability | rare need means no mechanism | `containers-and-resources/REASSESSMENT.md:701` | REFUSED | "Frequency determines optimization and investigation priority, not whether a needed capability may exist." |
| HIS-D0-14 | Closed pattern catalog as the writer contract | a fixed set of program architectures is taught | `design/language/pattern-doctrine.md:1` | ADOPTED | "this language's writers are agents with no installed base to appease" |
| HIS-D0-15 | Restriction without an accessible alternative | forbid a shape and leave no route | `design/language/pattern-doctrine.md:2` | REFUSED | "a gap in the catalog is a finding to investigate rather than authority to invent a rejection rule" |
| HIS-D0-16 | One observable behavior, no build modes | facts on/off may not change behavior | `design/language.md:3` | ADOPTED | "a source that behaves one way when checked and another way when built ... is not the program that runs" |
| HIS-D0-17 | Facts-on/facts-off acceptance pair | two builds as a soundness oracle for the fact channel | `design/log.md:168` | SUPERSEDED | "the owner's intent was never a facts-on and facts-off pair of builds" |
| HIS-D0-18 | M5 "accepted shapes are the fast shapes" | a failed permission leaves the program sequential | `design/language/parallelism.md:1` | ADOPTED | "whether to use it is a cost choice that must not change which programs are accepted" |
| HIS-D0-19 | Obvious shape is the fast shape | the property M5 asserts, as a current compiler claim | `docs/why-whitefoot.md:410` | FLOATED (m) | "reproducing that property in the current compiler remains open" |
| HIS-D0-20 | Proved parallel permission implies speedup | permission taken as a performance guarantee | `docs/ideas.md:536` | REFUSED | "Permission establishes independence, not profitable granularity." |
| HIS-D0-21 | More annotations make checking cheaper | verbosity as a cost argument | `docs/ideas.md:537` | REFUSED | "A larger vocabulary or a redundant certificate can be sound while still costing more" |

## D1 The unit and naming of storage identity

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D1-01 | Region as store identity (PROV-1) | a store's identity is a region, a component of every type it backs | `spec/kernel-spec.md:849`; `design/language/data-model.md` | ADOPTED | "a region names a store for the proof and nothing at run time" |
| HIS-D1-02 | Brand is not allocation, not generation | the brand names only the provider | `containers-and-resources/REASSESSMENT.md:712` | ADOPTED | "it does not identify an allocation or a recycled object generation" |
| HIS-D1-03 | Resolved place (root plus ordered steps) as the identity | fields and subscripts below a root binding | `spec/kernel-spec.md` OWN-7 | ADOPTED | "two field selections of different fields, or two subscripts whose offsets are both literals with unequal values" |
| HIS-D1-04 | Storage identity in the pointer type plus per-identity state | `ptr<'a, T>` naming a storage, state facts per identity | `MECHANISM-MAP.md:320`, `access-effects/DESIGN.md:141` | FLOATED | "M1: equality and finite states are fixed families; M2, M3 met; M4: local diagnostics" |
| HIS-D1-05 | Identity parameters distinct by default | two distinct identity parameters assumed disjoint | `MECHANISM-MAP.md:555` | FLOATED | "sound only if every call site discharges distinctness for every pair" |
| HIS-D1-06 | Existential identity for a fresh result | `exists P. Owner<P,T>` packed at allocation | `access-effects/DESIGN.md:143`, `MECHANISM-MAP.md:387` | FLOATED | "met" / "an unpack step at the caller" |
| HIS-D1-07 | Path identity for an element (`'v[i]`) | a sub-identity per element path | `MECHANISM-MAP.md:574` | FLOATED (open) | "What identity does an element or a backing carry" |
| HIS-D1-08 | Fresh existential allocation identities `ac`/`ap` | allocation mints unforgeable identity terms | `containers-and-resources/FOUNDATION.md:251` | FLOATED | "they cannot be manufactured from an integer, equal capacity or a reused address" |
| HIS-D1-09 | Plane identities inside one root allocation | two typed planes carved from one backing | `containers-and-resources/FOUNDATION.md:308` | FLOATED | "Plane identities are not independently freeable allocations." |
| HIS-D1-10 | Generation counter as object identity | a per-slot generation distinguishes reuse | `containers-and-resources/FOUNDATION.md:336` | FLOATED | "A finite generation must retire/refuse on exhaustion or justify safe reuse, not wrap" |
| HIS-D1-11 | Generation alone as store safety | generations without a store brand | `containers-and-resources/FOUNDATION.md:337` | REFUSED | "A generation alone does not prevent a handle being applied to the wrong store." |
| HIS-D1-12 | Runtime-validated references (Vale generational refs) | generation compare and trap at each access | `MECHANISM-MAP.md:324` | REFUSED | "M2 violated" / "disqualified; a baseline only" |
| HIS-D1-13 | Runtime owner IDs / generation tags in the checker relation | identity as runtime data | `containers-and-resources/REASSESSMENT.md:384` | REFUSED | "These are compiler relationships, not a request for runtime owner IDs or generation tags." |
| HIS-D1-14 | Slice possible-origin sets (finite static set) | one finite set of ultimate storage origins per direct slice | `design/language/ownership/slice-result-provenance.md:1` | ADOPTED | "a call computes its result's origin set from signatures alone" |
| HIS-D1-15 | Body-derived exact origin summary | infer a returned view's origins from the callee body | `design/language/ownership/slice-result-provenance.md` Rejected | REFUSED | "callable contracts would depend on implementation bodies and recursive call groups would need fixed-point handling" |
| HIS-D1-16 | Explicit return-origin annotation | writer names which input a result borrows from | `design/language/ownership/slice-result-provenance.md` Rejected | REFUSED | "it adds writer syntax plus body validation for a precision a writer can already express" |
| HIS-D1-17 | Per-leaf origin metadata inside generic or stored values | provenance through `Option`, nominals, boxes | `spec/kernel-spec.md:1299`; same node | DEFERRED / REFUSED | "a DEFERRED specification addition; a compiler limitation does not select that boundary" |
| HIS-D1-18 | Cross-pool region branding for per-pool identity | a region brand gives each pool a distinct identity | `research/notes/batch1-spec-deltas.md:648` | REFUSED | "a lexical region holds many pools; the brand buys no per-pool identity" |
| HIS-D1-19 | Anonymous brand syntax | an unnamed brand spelling | `containers-and-resources/FOUNDATION.md:709` | REFUSED | "is not needed to resolve this conflict and would introduce another grammar choice" |
| HIS-D1-20 | Formal brand named even at one occurrence | brands spelled, loan regions elided | `design/language/ownership/region-elision.md:1` | ADOPTED | "omitting a formal brand can select a concrete store rather than abbreviate the same type" |
| HIS-D1-21 | Region axis of nominal identity, erased before lowering | two region instantiations, one representation | `design/language/data-model.md:99` | ADOPTED | "without monomorphizing every function over its regions" |
| HIS-D1-22 | Extent item named by (instance, `region_stmt` NodePath) | identity where the type cannot carry it | `containers-and-resources/RESOURCES.md:92` | ADOPTED | "so monomorphization gives two instances two items" |
| HIS-D1-23 | Symbolic slot shared on the same captured index | dynamic slot identity from a captured index value | `containers-and-resources/FOUNDATION.md:1086` | FLOATED | "a repeated access may share a symbolic slot only when it uses the same captured index value" |
| HIS-D1-24 | Pointer inequality as disjointness | distinct addresses prove non-overlap | `mcts_mem/whitefoot/ownership/no-reborrow.md:19` | REFUSED | "pointer inequality is never sufficient, including for empty or zero-sized referents" |
| HIS-D1-25 | Bare `u64` index instead of a typed handle | untyped integer index into a pool | `research/notes/batch1-spec-deltas.md:645` | REFUSED | "index/len/foreign-handle confusion becomes an in-bounds silent wrong value" |

## D2 Where permission and state live

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D2-01 | Permission in the reference (OWN-2 modes) | `own`, `&`, `&uniq`, each borrow carrying one region | `spec/kernel-spec.md:686` | ADOPTED | "a borrow mode carries one region ... The mode itself is always written." |
| HIS-D2-02 | Permission as checker context facts (Candidate A) | locators name storage; legality proved in the current state | `access-effects/DESIGN.md:24,165` | FLOATED | "The resource context is compiler state, not a heap table or a runtime token." |
| HIS-D2-03 | Validity obligation on the reference (Candidate B) | `Ref<'a,P,T>` = locator plus a validity extent | `access-effects/DESIGN.md:507` | FLOATED | "Destruction or a layout change that would violate the obligation is forbidden until that obligation ends" |
| HIS-D2-04 | Explicit permission tokens (L3, GhostCell style) | aliasable locators plus one threaded linear authority | `access-effects/DESIGN.md:549`, `RESEARCH.md:182` | REFUSED | "demanding the capability through every helper recreates the interface problem motivating this work" |
| HIS-D2-05 | Ghost permission separated from the value (Verus `Tracked<PointsTo>`) | a linear ghost term threaded and erased | `MECHANISM-MAP.md:344` | FLOATED | "met for the linear part" / "permissions threaded by hand through every call" |
| HIS-D2-06 | Mezzo implicit permission flow with adoption/abandon | permissions flow implicitly, with a dynamic escape | `access-effects/DESIGN.md:557` | REFUSED | "Its dynamic adoption/abandon route is not a substitute for the requested erased static solution." |
| HIS-D2-07 | Ghost linear closing key (`FocusKey`) / `Open(...)` slot resource | focus and unfocus tokens over a slot | `containers-and-resources/FOUNDATION.md:245,263` | FLOATED | "Focus generates a fresh linear closing key; unfocus consumes the three pieces and that same key." |
| HIS-D2-08 | Runtime proof table / occupancy bitmap / extra scan | permission carried at run time | `containers-and-resources/FOUNDATION.md:298` | REFUSED | "No runtime proof table, second occupancy bitmap or additional cleanup scan is part of this candidate." |
| HIS-D2-09 | Runtime drop flags or ownership indicators | a hidden bit deciding whether to release | `MECHANISM-MAP.md:333`; LIV-1 | REFUSED | "M2 violated; WF refused it in LIV-1" / "a hidden bit the source could have stated" |
| HIS-D2-10 | Permission per access path (mutable value semantics) | access-path exclusivity, no reference permission | `MECHANISM-MAP.md:396` (Hylo/Swift row) | FLOATED | "M2 partially: Swift falls back to dynamic checks for class storage" |
| HIS-D2-11 | Whole-program permission table computed for the closed world | infer permission globally, no contracts | `MECHANISM-MAP.md:378-388` (R7 family table) | FLOATED | "Not its job: inferring anything from bodies" |
| HIS-D2-12 | Capability held as an ordinary value (no ambient resource) | a store is a value a program must hold | `containers-and-resources/DESIGN.md:396` | ADOPTED | "there is no ambient allocator, thread source, or stack pool" |
| HIS-D2-13 | Writer-declared store type as capability | `struct Heap {}` written in source | `containers-and-resources/DESIGN.md:4425` | REFUSED | "a writer's `struct Heap {}` is constructible" |
| HIS-D2-14 | Linearity read against the scope (PROV-6) | linear only where the release capability is absent | `design/language/ownership/linearity.md:1` | ADOPTED (m) | "forced dozens of explicit dispose statements per hosted function and pushed writers toward single-exit rewrites" |
| HIS-D2-15 | Linearity fixed by the value's type alone | type-level linear classification | same node | REFUSED (m) | "reading linearity from the type alone forced dozens of explicit dispose statements per hosted function" |
| HIS-D2-16 | Effects grant write authority | an effect row as a permission | `access-effects/DESIGN.md:112` | REFUSED | "They neither manufacture an owned resource nor prove that storage exists." |
| HIS-D2-17 | Source modes granting aliasing permission | representation or mode implies alias facts | `design/language/data-model.md:97`; `REASSESSMENT.md:628` | REFUSED | "those modes alone supply no loan origin, lifetime, or aliasing permission" |
| HIS-D2-18 | Optimizer facts supplying missing source proof | backend analysis closes a proof gap | `containers-and-resources/REASSESSMENT.md:231` | REFUSED | "optimizer facts cannot supply missing source proof" |
| HIS-D2-19 | Privacy as proof authority | hidden representation as the unforgeability mechanism | `containers-and-resources/FOUNDATION.md:325` | REFUSED | "privacy could hide representation but is not what makes a proof unforgeable" |
| HIS-D2-20 | No global mutable state, no static region | every root object is passed as a parameter | `design/language/ownership.md:5` | ADOPTED | "a function that reached a global directly could not be pure and every proof that rests on purity ... would collapse" |

## D3 The state vocabulary of a storage

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D3-01 | No per-place state; validity by construction | initialization keyed to binding liveness only | `spec/kernel-spec.md` LIV-1/LIV-2; `design/language/ownership/affine-replacement.md` | ADOPTED | "no program point observes a vacant place" |
| HIS-D3-02 | Typed holes (per-place vacancy type state) | the slot's type flows to an Option-like vacant state | `design/language/ownership/affine-replacement.md` Rejected | REFUSED † [a `&uniq` holder must have a meaning over a vacant referent] | "per-place flow-sensitive type states are exactly what the simplified calculus excludes, and vacancy would leak into every boundary signature" |
| HIS-D3-03 | Closed-scope hole (bare take, refilled before scope end) | vacancy flow with repair on leaving edges | `design/language/ownership/affine-replacement.md` Rejected; `take-replace/DESIGN.md:87-112` | REFUSED † [exclusive borrow over a vacant referent] | "a meaning for an exclusive borrow over a vacant referent, buying only a use-then-refill window neither consumer needs" |
| HIS-D3-04 | Vacancy as an ordinary value (`Option`-shaped element) | occupancy is writer data, checked by `match` | `design/language/ownership/affine-replacement.md:2` | ADOPTED | "per-place flow-sensitive type states are exactly what the simplified ownership calculus excludes" |
| HIS-D3-05 | Per-identity state facts (live / initialized / ended) | `state(identity)` checked at each access | `MECHANISM-MAP.md:522`, `access-effects/DESIGN.md:191` | FLOATED | "every access checks the named storage's state; storage-ending operations are contract-visible" |
| HIS-D3-06 | A fourth "suspended" state on the identity | reproduce OWN-9's parent/child usability as storage state | `EVIDENCE-mechanism-survey-2026-09-16.md:915` | FLOATED | "it needs a fourth state, \"suspended\", or the same information under another name" |
| HIS-D3-07 | State as a term over the captured branch condition | `ite(c, s1, s2)` instead of a lattice value | `MECHANISM-MAP.md:522,583` | FLOATED | "sound but untested for precision" |
| HIS-D3-08 | Lattice-valued join | collapse states to a lattice at merges | `MECHANISM-MAP.md:521` | SUPERSEDED (within the candidate) | "storage state at a join is a term over the captured branch condition rather than a lattice value" |
| HIS-D3-09 | Typestate / property automata per object | finite state per tracked object, finite-state dataflow | `MECHANISM-MAP.md:322` | FLOATED | "precision collapses under aliasing without a permission system underneath" |
| HIS-D3-10 | Stateful views over a linear context (ATS, Cogent) | view assertions per location including uninitialized | `MECHANISM-MAP.md:321` | FLOATED | "M1 to M3 met; M4: a second small language" |
| HIS-D3-11 | Window typestate for a run (`len_of`, `head_of`, `cap_of`) | checker-maintained initialized window carried by the value | `containers-and-resources/DESIGN.md:504` | ADOPTED | "the boundary is checker-maintained typestate carried by the run's own value" |
| HIS-D3-12 | Per-slot tag / occupancy bitmap / runtime discriminant | occupancy as language-visible runtime state | `containers-and-resources/DESIGN.md:505` | REFUSED | "no per-slot tag, occupancy bitmap, or runtime discriminant is language state" |
| HIS-D3-13 | Prefix-only initialized set | one initialized prefix for every sequence | `containers-and-resources/DESIGN.md:509-511` | REFUSED (m) | "\"every other order is arithmetic\" was **false for a queue**, at a measured seven times a hand-written byte ring" |
| HIS-D3-14 | One ring representation for every sequence | a universal circular-window representation | `containers-and-resources/REASSESSMENT.md:568` | REFUSED | "Rejected as the universal semantic/representation requirement." |
| HIS-D3-15 | Three state families: full array / prefix run / circular run | distinct compiler-owned initialization states | `design/language/data-model.md:95`; `REASSESSMENT.md:484` | ADOPTED | "variable length and a head position are not universal array metadata" |
| HIS-D3-16 | Universal bitmap live-set encoding | one bitmap encodes every occupancy state | `mcts_mem/whitefoot/data-model.md:15` | REFUSED | "would tax contracts whose live set is already proved" |
| HIS-D3-17 | Projected sparse slot enum (Empty/Deleted/Occupied) | authoritative discriminant stored on its own plane | `containers-and-resources/FOUNDATION.md:192,1484` | DEFERRED then REFUSED (m) | "the observed gap does not justify selecting compiler-known sparse authority" |
| HIS-D3-18 | Measure terms with descriptor-storage support (MSR-2) | a length is a fact whose support is the descriptor path | `containers-and-resources/CONTAINERS.md:99` | ADOPTED | "so an element write does not kill a length, a sibling-field write does not kill a length" |
| HIS-D3-19 | Refined integer domains (nonzero, bounded) and automatic niches | valid-value sets as types, layout derived | `docs/ideas.md:278-320` | FLOATED | "Any syntax used to express those domains would be a separate language decision" |

## D4 Sequential aliasing policy

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D4-01 | Shared-xor-exclusive, checked unconditionally | no two live usable `&uniq` overlap | `spec/kernel-spec.md:706` (OWN-5) | ADOPTED | "Exclusivity invariant, checked unconditionally" |
| HIS-D4-02 | Several writable aliases allowed in sequential code | aliasing legal; safety from per-storage state | `MECHANISM-MAP.md:404`, `access-effects/DESIGN.md:417` | FLOATED | "met; nothing further needed" / "facts weaken where identities coincide" |
| HIS-D4-03 | Read-only as a type flag rather than a loan | interface restriction with no duration | `MECHANISM-MAP.md:565` | FLOATED | "a type flag, never a loan or a duration" |
| HIS-D4-04 | Two-axis mode vocabulary (exclusivity x write-permission) | adds frozen/exclusive-read and a bounded shared-write form | `spec/kernel-spec.md:174` (LEX-1) | DEFERRED | "DEFERRED with recorded delta: the two-axis mode vocabulary" |
| HIS-D4-05 | `frozen` / unique-read fourth mode | a read-only-unique mode for optimizer payoff | `research/notes/batch1-spec-deltas.md:717` | DEFERRED | "deferred until measured optimizer payoff earns it (R1)" |
| HIS-D4-06 | Interior mutability as a mode | a fourth shared-write borrow mode | `research/notes/batch1-spec-deltas.md:714` | REFUSED | "interior mutability being a gated TYPE not a MODE, §5's writer mode set is COMPLETE at three" |
| HIS-D4-07 | `cell<T>` as a gated capability type | copy-in/copy-out interior mutability, not writer-emittable | `research/notes/batch1-spec-deltas.md:711,647` | ADOPTED (gated) / REFUSED (writer-emittable) | "lets a kernel `let c: cell<i32>` parse — a W3/SCOPE-1 violation" |
| HIS-D4-08 | `RefCell` / `Cell` runtime borrow flags | dynamic borrow counters | `MECHANISM-MAP.md:410`; `docs/why-whitefoot.md:412` | REFUSED | "one shared-mutable hole anywhere in the type system makes every aliasing fact conditional" |
| HIS-D4-09 | Per-scope exclusivity windows through suspension | parent suspended while a child loan lives | `spec/kernel-spec.md:701` (OWN-5/OWN-6) | ADOPTED | "its own read/write allowance is withdrawn: no read, write, move, copy, `set` commit, or call-transfer" |
| HIS-D4-10 | Local non-aliasing (`restrict`/`confine`, Aiken 2003) | check local exclusivity even when aliases exist elsewhere | `access-effects/RESEARCH.md` prior-art table | FLOATED | "Ordinary C `restrict` alone is unchecked and is not an acceptable WF mechanism." |
| HIS-D4-11 | Coincidence declared per pair at a boundary | callers state which arguments may alias | `access-effects/DESIGN.md:259-279` | FLOATED | "P and Q may be equal. The definition is checked once under precisely that possibility." |
| HIS-D4-12 | Overlap by complete resolved paths, conservative | prefix-overlap over fields and literal subscripts | `design/language/ownership.md:2` | ADOPTED | "instead of treating every view of one allocation as a whole-allocation conflict or using general flow-sensitive alias analysis" |
| HIS-D4-13 | Proved half-open view extents establish disjointness | range loans refine the overlap judgment | `design/language/ownership.md:2-3`; `range-loans/DESIGN.md:32` | ADOPTED | "Range extent proofs refine that existing overlap judgment; they do not introduce a second alias model." |
| HIS-D4-14 | Literal-only or equal-size-only ranges | restrict range disjointness to constants | `range-loans/DESIGN.md:120` | REFUSED | "leave runtime rows or recursive uneven subdivisions unexpressible" |
| HIS-D4-15 | Separate storage per row / whole-buffer copies / runtime checks | avoid aliasing by copying or testing | `range-loans/DESIGN.md:122` | REFUSED | "changes the allocation and locality of the algorithm" |
| HIS-D4-16 | Moves through a borrow | move content reached through a reference | `spec/kernel-spec.md:705` (OWN-5) | REFUSED † [the far-side owner holds an unseen hole] | "Content reached through any borrow may never be moved" |
| HIS-D4-17 | SET-2 replace as the sole exception | atomic exchange through a `&uniq` holder | `spec/kernel-spec.md:705`; `take-replace/DESIGN.md:141` | ADOPTED | "sound because the exchange leaves no program point at which the referent place lacks exactly one valid owner" |
| HIS-D4-18 | `&uniq` container parameter refusal (BLK-4 fourth clause) | refuse exclusive parameters of container or generic type | `containers-and-resources/CONTAINERS.md:48` then `FOUNDATION.md:1700` | ADOPTED then REVERSED | "BLK-4 removes the entire recursive exclusive-parameter refusal, including opaque type parameters." |

## D5 Interference facts for parallel permission and the optimizer

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D5-01 | Loans plus effect footprints over resolved places | today's PAR-1/PAR-2 judgment | `spec/kernel-spec.md:2406-2412`; `MECHANISM-MAP.md:361` | ADOPTED | "met today" / "loans restate what rows and identities say" |
| HIS-D5-02 | Window permission over a block of calls and interposed statements | quantify dependencies over the whole window | `design/language/parallelism/permission-judgment.md:1` | ADOPTED (m) | "two programs with byte-identical output differed 1.9 times in wall time" |
| HIS-D5-03 | Adjacent-pair enumeration | only consecutive let-bound calls form a candidate group | same node, Rejected | SUPERSEDED (m) | "permission then turned on statement adjacency rather than semantics, measured as a 1.9 times wall-time difference" |
| HIS-D5-04 | Reliance edges / schedule-parametric ownership / weakening the loan rule | four rivals to reusing the borrow checker's vocabulary | `design/language/parallelism/permission-judgment.md:2` | REFUSED † [a loan is a reference-borne exclusion] | "each either duplicated the borrow checker or weakened it" |
| HIS-D5-05 | Treating exclusive borrows as writes | model a `&uniq` actual as a write footprint | same node | REFUSED † [`&uniq` carries exclusion independent of the row] | "each either duplicated the borrow checker or weakened it" |
| HIS-D5-06 | Row projection alone, no loans | judge permission from effect rows only | `mcts_mem/whitefoot/parallelism/permission-judgment.md:8` | REFUSED † [borrow modes are invisible to a row] | "two read-only-row `&uniq` borrows of one place" |
| HIS-D5-07 | Footprints over storage identities with distinctness facts | DPJ / Regent / CSL parallel rule rooted at identities | `MECHANISM-MAP.md:362` | FLOATED | "met if identity parameters are distinct by default or proved" |
| HIS-D5-08 | Retire the PAR-1 loan clause | permission from footprints alone | `MECHANISM-MAP.md:547,601` | FLOATED † [the loan is the only reference-borne fact left] | "whether a call may claim exclusivity on an identity beyond its row" |
| HIS-D5-09 | Fractional or counting permissions | a permission amount per location | `MECHANISM-MAP.md:363` | FLOATED | "M1 needs a fixed split discipline; the counting form is deterministic" |
| HIS-D5-10 | Region partitions with method effect summaries (DPJ) | hierarchical region names plus per-call effects | `MECHANISM-MAP.md:352` | FLOATED | "array partitioning needs arithmetic disjointness, where solvers usually enter" |
| HIS-D5-11 | Runtime dependency tracking (Legion) | dynamic dependence analysis in the scheduler | `MECHANISM-MAP.md:365` | REFUSED | "M2 and M5 violated for permission" / "scheduler cost" |
| HIS-D5-12 | Compiler-side recovery (LLVM AA, loop versioning) | let the optimizer re-derive aliasing under guards | `MECHANISM-MAP.md:355` | REFUSED (m) | "contradicts M5's premise" / "guards and code bloat" |
| HIS-D5-13 | Purity and uniqueness only (Futhark, Cogent) | data-parallel shapes from purity | `MECHANISM-MAP.md:366` | FLOATED | "no general statement overlap over mutable structures" |
| HIS-D5-14 | Source-order equality as the determinism guarantee | overlap preserves the sequential result exactly | `spec/kernel-spec.md:2404` (CAP-1); `MECHANISM-MAP.md:506` | ADOPTED | "an implementation may not construct a loan state the source checker refuses" |
| HIS-D5-15 | A second, weaker permission level (race freedom plus invariants) | overlap without source-order equality | `MECHANISM-MAP.md:506,601` | FLOATED | "a second permission level: race freedom plus invariant preservation without PAR-1's source-order equality" |
| HIS-D5-16 | Declared parallelism (a writer-marked construct) | a keyword gates or declares overlap | `design/language/parallelism.md:1`; `mcts_mem/whitefoot.md:75` | REFUSED | "instead of a parallel keyword that changes acceptance" |
| HIS-D5-17 | Automatic parallelism discovery as a direction | infer parallelism from ownership and effects | `mcts_mem/whitefoot/parallelism.md:9`; `design/log.md:120` | REFUSED then WITHDRAWN | "the language removes the soundness half of auto-parallelization and none of the decision half" |
| HIS-D5-18 | Claim-free eligibility (no claim in the call closure) | overlap only where no runtime check can fire | `proof-derived-parallelism/DESIGN.md:60` | SUPERSEDED | "declining to overlap a *correct* program to keep a *defective* one's trap identity stable" |
| HIS-D5-19 | Elision-rank join arbitration | ranked lanes arbitrate which trap record survives | `mcts_mem/.../elision-rank-join-arbitration.md:8` | REFUSED | "every byte of the arbitration exists to make a defective program's trap record reproducible" |
| HIS-D5-20 | Counted-loop permission as its own site (PAR-2) | a loop is a permission site, not a synthesized pair | `design/language/parallelism.md:3` | ADOPTED | "its permission must not depend on rewriting the loop as a pair of sibling calls" |
| HIS-D5-21 | Unbounded multi-accumulator reductions / general index search | widen PAR-2's admitted family | `design/language/parallelism/loop-permission.md:1` | REFUSED | "arbitrary scatter is not admitted" |
| HIS-D5-22 | Staged-loop permission by suspension or native classification | pipeline overlap selected by an implementation property | `design/language/parallelism.md` Rejected | REFUSED | "implementation origin cannot authorize additional overlap; the loss of pipeline permission is established" |
| HIS-D5-23 | Per-field alias scopes on loaded data pointers | one `!alias.scope` per identity on loads | `EVIDENCE-mechanism-survey-2026-09-16.md:927`; `research/experiments/scoped-alias-channel/RESULTS.md` | FLOATED (m) | "8 vector ops / 0 guards / 121 asm lines vs Rust obvious 65 / 29 / 2132; short-trip 2.0x at n=8" |
| HIS-D5-24 | Cross-procedural `noalias` on declarations | an identity in the signature carries the fact into the callee | `EVIDENCE-mechanism-survey-2026-09-16.md:940` | FLOATED | "Today's origin substitution happens only at the caller (OWN-12) and the callee's formal-slice origin is opaque" |
| HIS-D5-25 | No alias promises at all (today) | the emitter states nothing to the backend | `MECHANISM-MAP.md:489` | ADOPTED (status quo) | "emits no overflow or alias promises" |
| HIS-D5-26 | Source-order equality as the overlap test | overlap may not invent an outcome sequential execution cannot produce | `io-model/FIRST-PRINCIPLES.md:823` | ADOPTED | "if overlap can invent an outcome the sequential program" cannot produce, the overlap is refused |
| HIS-D5-27 | Race freedom by construction rather than by analysis | the two structural axioms the catalog prices every architecture against | `io-model/CONCURRENCY-CATALOG.md:71,77` | ADOPTED | "No two overlapping executions ever write one place." / "No control flow outlives its fork-join." |
| HIS-D5-28 | Early exit inside a parallel search | stop the other lanes once one finds a hit | `io-model/CONCURRENCY-CATALOG.md:2576,2583` | REFUSED (accepted cost, with a gain) | "The search always scans everything." / "The answer is now the first index, deterministically" |
| HIS-D5-29 | Non-associative accumulators by index-ordered retirement | staged-loop latitude for order-sensitive folds | `io-model/CONCURRENCY-CATALOG.md:2327` | SUPERSEDED (withdrawn) | "One thing the earlier draft claimed here is **withdrawn**" |
| HIS-D5-30 | PAR-3 staged loop (K sessions in flight, cut at the first may-suspend) | pipeline overlap as its own permission rule | `io-model/CONCURRENCY-CATALOG.md:47`; `ordinary-host-values/DECISIONS.md:157` | REFUSED then DELETED | "is set aside and is not used anywhere below" / "Delete PAR-3's current special cut at the first" |

## D6 Distinctness of identity parameters at function boundaries

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D6-01 | May-alias by default (today's OWN-7 for formals) | formal origins never establish disjointness | `MECHANISM-MAP.md:502` | ADOPTED | "formal origins \"never establish that two actual sources are disjoint\"" |
| HIS-D6-02 | Distinct by default with call-site proof | two identity parameters assumed disjoint unless declared | `MECHANISM-MAP.md:555,576` | FLOATED | "which inverts OWN-7's current default; costs a proof where two arguments come from one container" |
| HIS-D6-03 | May-alias by default with a callee-side proof | the callee is checked under possible equality | `access-effects/DESIGN.md:269` | FLOATED | "The definition is checked once under precisely that possibility." |
| HIS-D6-04 | Declared disjointness per pair (`requires disjoint(P,Q)`) | a precondition the caller discharges | `access-effects/DESIGN.md:277` | FLOATED | "Caller knowledge does not justify an undeclared assumption retroactively." |
| HIS-D6-05 | Inferred from the closed world | whole-program alias partitioning | `access-effects/DESIGN.md:286` | REFUSED | "Do not search over alias partitions, higher-order qualifier solutions, or callee bodies." |
| HIS-D6-06 | Distinct caller-supplied regions incomparable, fail closed | any rule needing an order rejects | `spec/kernel-spec.md:694` (OWN-3) | ADOPTED | "any rule requiring an order between them fails closed (reject)" |
| HIS-D6-07 | Provenance candidate: exactly one same-kind, same-region parameter | signature determines a borrow result's root | `spec/kernel-spec.md:743`; `reborrow-extension/SPEC-DELTA.md:74` | ADOPTED | "a `shared` result CAN derive from a `uniq` parameter ... so any same-region parameter of the other kind makes the signature ambiguous" |
| HIS-D6-08 | Possible-provenance set over borrows (non-singleton root) | a borrow holder with a set of roots | `reborrow-extension/SPEC-DELTA.md:60` | REFUSED | "it would be the first non-singleton borrow root and T-A is the load-bearing frontend-scale-checker simplification" |
| HIS-D6-09 | Declaration-site rejection of ambiguous provenance | reject the declaration, not the caller's binding | `declaration-provenance/SPEC-DELTA.md:10,55` | ADOPTED | "a declaration whose result no caller can use is itself the error" |
| HIS-D6-10 | Zero-candidate boundary rooted in const storage | provenance unique by elimination | `spec/kernel-spec.md:744`; `declaration-provenance/SPEC-DELTA.md:43` | ADOPTED | "Provenance is unique by elimination and needs no claim." |
| HIS-D6-11 | Wide claim: result holder takes the whole candidate actual place | coarser than the loan the callee returned | `spec/kernel-spec.md:743`; `reborrow-extension/SPEC-DELTA.md:86` | ADOPTED | "Prefix overlap (OWN-7) makes the wide claim cover every narrower truth" |
| HIS-D6-12 | Formal-free instantiation when actuals have empty origin sets | a precision recovery at substitution | `containers-and-resources/FOUNDATION.md:1034` | REFUSED | "The recovery was rejected without applying it; treating `None` and explicitly unknown actuals conservatively" |

## D7 The contract vocabulary at call boundaries

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D7-01 | One erased contract block with `requires`/`ensures` and `define` | proof-only clauses after the signature | `design/language/checks-and-proofs/requires-entry-contract.md:1` | ADOPTED | "separate executable-looking blocks hid their proof-only role, duplicated shared abbreviations" |
| HIS-D7-02 | A single final requirement block ending in one Boolean | one executable-looking check | same node, Rejected | REFUSED | "it was spelled like executable code so its erased status was invisible" |
| HIS-D7-03 | Callee-entry prologue evaluating requirements | check at entry rather than at the call | same node, Rejected | REFUSED | "it let a helper hide a protected leaf behind a runtime trap and added the prologue's reads to the callee effect row" |
| HIS-D7-04 | Call-site discharge of each clause separately | proved in the caller's state just before the call | same node | ADOPTED | "with no fallback runtime check and no callee prologue" |
| HIS-D7-05 | Goal matching by spelling or algebraic normal form | identify two clauses by syntax or by normalization | same node | REFUSED | "a goal proved once must be recognized wherever it appears again" |
| HIS-D7-06 | Contradictory-requirement instance is legal and uncallable | keep it, lower an unreachable stub | same node | ADOPTED | "rejecting it would require the checker to detect every contradiction, which a deterministic no-search checker cannot promise" |
| HIS-D7-07 | `entry(parameter)` to name a parameter's entry state | relate before and after for a mutating helper | same node | ADOPTED | "without equating both sides of an increment to the same moment" |
| HIS-D7-08 | Effect rows exact in both directions | declared-but-unexhibited is an error too | `design/language/effects.md:1` | ADOPTED | "padding a row would be a place to smuggle effects" |
| HIS-D7-09 | Rows as upper bounds (subsumption) | a row may over-declare | same node | REFUSED | "the later-proof clause keeps acceptance decidable from the signature alone" |
| HIS-D7-10 | Separate `external`, `blocks`, `traps` categories | mechanism labels in the row | `design/language/effects.md` Rejected | REFUSED | "the categories described mechanisms rather than state" |
| HIS-D7-11 | Following owned values through locals or results for effect roots | ownership-routing summaries | same node, Rejected | REFUSED | "ancestry makes that boundary depend on implementation bodies" |
| HIS-D7-12 | `allocates(path)` naming a provider parameter | branded allocation visible in the row | `design/language/effects.md:5`; `spec/kernel-spec.md` EFF-1 | ADOPTED | "different branded stores need distinct caller-visible capabilities" |
| HIS-D7-13 | `allocates(arena 'r)` region spelling | a region rather than a value path in the row | `spec/kernel-spec.md:1905` | ADOPTED, transitional | "That alternative of the production is transitional and retires with `arena<'r, T>`" |
| HIS-D7-14 | `requires`/`ensures` over identity states | entry and exit states per storage identity | `MECHANISM-MAP.md:382`, `access-effects/DESIGN.md:259` | FLOATED | "every callee that changes storage state must say so" |
| HIS-D7-15 | Store-type pre and post in the signature (Alias Types) | store-in / store-out including holes and vanished locations | `MECHANISM-MAP.md:381` | FLOATED | "signatures grow; location polymorphism and existentials are mandatory" |
| HIS-D7-16 | `modifies` / `reads` clauses with SMT discharge | Dafny/Low* notation | `MECHANISM-MAP.md:384` | PARTIAL | "the notation fits M4; the discharge violates M1 and must be replaced by a footprint algebra" |
| HIS-D7-17 | Prophecies / backward functions (RustHorn, Creusot, Aeneas) | the final value of a mutable borrow | `MECHANISM-MAP.md:386` | REFUSED | "M1 poor; unnecessary when access checks state per operation" |
| HIS-D7-18 | Result states routed by variant (CALL-6 reuse) | a relation restricted to a result variant's arm | `spec/kernel-spec.md:3096`; `MECHANISM-MAP.md:527` | ADOPTED | "it is not deferred to the arm, so an [ENT-5] event lying between the call and the arm kills a relation" |
| HIS-D7-19 | Postcondition summaries within a recursive component | assume a callee's postcondition while checking its component | `design/language/checks-and-proofs/obligation-discharge.md:39` | REFUSED | "a recursive component must not bootstrap itself from a summary it has not yet earned" |
| HIS-D7-20 | Formal row authoritative for container behavior | the written boundary does not vary with the selected actual | `design/language/contracts.md:2-3` | ADOPTED | "a generic caller must retain one written boundary while accepting an implementation that touches less state" |
| HIS-D7-21 | Body-derived result-state origin | recover a resource's exit state from the callee body | `design/log.md:90`; `docs/todo.md` | REFUSED (open defect) | "no result-state origin is derived from callee bodies, which the current `result_state_origin.rs` contradicts" |
| HIS-D7-22 | Entry-to-normal-exit transfer as a second FN-1 output | result plus written-back actual | `containers-and-resources/FOUNDATION.md:808` | ADOPTED | "extends the former result-only boundary with that entry-to-normal-exit transfer" |
| HIS-D7-23 | Result-only callable summary | identity of the result only | `containers-and-resources/FOUNDATION.md:928` | REFUSED | "The alternatives considered were result-only summaries (lose the stored output)" |
| HIS-D7-24 | Inspecting callee bodies at every ordinary call | recover identity by looking inside the callee | `containers-and-resources/FOUNDATION.md:929` | REFUSED | "inspecting callee bodies at every ordinary call (breaks the selected callable boundary)" |
| HIS-D7-25 | Retaining every replaced owner indefinitely | keep displaced owners in the origin set forever | `containers-and-resources/FOUNDATION.md:930` | REFUSED | "retaining every replaced owner indefinitely (loses current-state precision)" |

## D8 Storage placement, relocation and moves

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D8-01 | Single-owner affine values with explicit `move` | one owner, explicit transfer, dead-root kill | `spec/kernel-spec.md` OWN-1; `design/language/ownership.md:1` | ADOPTED | "explicit region and endpoint rules give deterministic local checks" |
| HIS-D8-02 | Atomic `replace` as a let-only form | replacement enters and the old value exits in one commit | `design/language/ownership/affine-replacement.md:1` | ADOPTED | "the no-hole property holds by construction when the replacement is required in the same operation" |
| HIS-D8-03 | Bare `take` leaving a hole | move out, refill later | same node, Rejected | REFUSED † [exclusive borrow over a vacant referent] | "needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window" |
| HIS-D8-04 | Two-place `swap p, q` | exchange two places directly | `design/language/ownership/multi-target-commit.md` Rejected | REFUSED | "a third mutation path that breaks initialization-keyed facts, loans, and liveness" |
| HIS-D8-05 | One n-ary `set` commit (swap is an instance) | read every target before writing any | `design/language/ownership/multi-target-commit.md:1` | ADOPTED | "a dedicated exchange operation would duplicate what the general commit already expresses" |
| HIS-D8-06 | Swap of two elements of one run | exchange within one container | `containers-and-resources/CONTAINERS.md:165` | REFUSED | "a swap of two elements of one run is refused by that rule's non-overlap condition" |
| HIS-D8-07 | Whole-binding replace of slice or arena places | rebind a view- or arena-typed place | `design/language/ownership/affine-replacement.md:3` | REFUSED | "it would break the static origin sets and confinement those types carry" |
| HIS-D8-08 | `move` on a copy value | one spelling per meaning, copies used bare | `spec/kernel-spec.md` OWN-1; `move-on-copy/REPORT.md` | ADOPTED (ban) | "The ban has no recorded alternative and no recorded weighing." |
| HIS-D8-09 | Rewrite OWN-1's consumption sentence to cover copy places | legalize `move` on copies, rewire drop obligations | `move-on-copy/REPORT.md` §5 option A | FLOATED (m) | "a legal spelling inside generic bodies, plus a destructive spelling everywhere else" |
| HIS-D8-10 | Narrow relief: `move` only at type-parameter places | positional relief for generic bodies | `move-on-copy/REPORT.md` §5 option A' | REFUSED | "it is exactly the shape `spelling-rule.md:4` forbids" |
| HIS-D8-11 | A `T: Copy` bound instead | extend FN-2's bound vocabulary | `move-on-copy/REPORT.md` §5 option C | FLOATED | "Touches **no** monomorphic code, **no** drop obligations, **no** conformance verdict." |
| HIS-D8-12 | Three-valued class in the symbolic pass | copy / affine / unknown for an unbounded `T` | `move-on-copy/REPORT.md` §6 | FLOATED | "That would close the pincer with **no** spelling change and **no** spec change" |
| HIS-D8-13 | Relocation as a name change (L3 `realloc`) | old identity ends, an existential new one begins | `MECHANISM-MAP.md:394` | FLOATED | "the most explainable form; makes push conditional on `len < cap`" |
| HIS-D8-14 | `own` aggregates passed as handles, storage fixed for life | identity survives the call | `MECHANISM-MAP.md:343,558` | FLOATED | "identity survives the call only if storage does not move" |
| HIS-D8-15 | Take and put on inline storage (Cogent, ATS) | slot state carried in the type | `MECHANISM-MAP.md:395` | FLOATED | "inline-in-container without a hidden tag" / "slot state multiplies with nesting" |
| HIS-D8-16 | Pinning (`Pin`-style flag) | a pinned bit in the type | `MECHANISM-MAP.md:397` | REFUSED | "expresses less than a state event" / "idiom burden" |
| HIS-D8-17 | Handles instead of interior pointers | index plus bounds proof | `MECHANISM-MAP.md:398`; `docs/why-whitefoot.md:589` | ADOPTED (taught) | "met; already taught" / "one indirection" |
| HIS-D8-18 | Every object in a pool, handles as links | big structures live in pools | `docs/why-whitefoot.md:589-591` | ADOPTED (pattern) | "Node links in a tree or graph are handles into the pool, not pointers or references." |
| HIS-D8-19 | Slot recycling / free lists in a pool | reuse slot indices | `mcts_mem/whitefoot/data-model.md:13`; `design/language/data-model.md:91` | REFUSED | "a well-typed slot-recycling use-after-free is unrepresentable when indices never recycle" |
| HIS-D8-20 | Generational per-element free | generation word plus typed stale-lookup outcome | `mcts_mem/whitefoot/data-model.md:12`; `docs/why-whitefoot.md:595` | DEFERRED | "a future recyclable pool would pair each slot with a generation and expose mismatch as a typed lookup outcome" |
| HIS-D8-21 | Ownership relocation for compaction or removal shift | every retained owner reaches exactly one destination | `design/language/data-model.md:93` | ADOPTED | "treating such a move as a metadata update or a copy would duplicate or lose owners" |
| HIS-D8-22 | Guaranteed same-place owned call contract | promise the callee reuses the caller's storage | `containers-and-resources/FOUNDATION.md:1631`; `design/compiler/storage-placement.md` | REFUSED | "instead of allowing reuse only for whole results or guaranteeing same-place behavior in the language" |
| HIS-D8-23 | Marking possibly identical in/out pointers `noalias` | a backend destination as an aliasing grant | `containers-and-resources/REASSESSMENT.md:193` | REFUSED | "Do not mark two potentially identical input/output pointers as independent `noalias` destinations" |
| HIS-D8-24 | Explicit checked initialization destination | writer names where a result is built | `containers-and-resources/FOUNDATION.md:530` | DEFERRED | "it also needs partial-construction and failure ownership rules" |
| HIS-D8-25 | Writer-registered finalizers on release | user code on a scope-exit edge | `design/language/ownership.md:4` | REFUSED | "a user-defined release action would hide code and effects on scope exits" |
| HIS-D8-26 | `dispose` as an explicit early release | release before scope exit over a capability-released graph | `design/language/ownership/linearity.md:2` | ADOPTED | "no ordinary source statement can perform the structural release walk itself" |
| HIS-D8-27 | `on_propagate { ... }` scope section | release statements attached to propagate edges | `containers-and-resources/DESIGN.md:4510` | REFUSED | "an inner and an outer section each pass their own per-point check and **each run on the same edge**" |

## D9 Container elements and backings

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D9-01 | Run of initialized slots as the primitive | one backing primitive under every container | `containers-and-resources/DESIGN.md:310` | ADOPTED | "the lowest common primitive of every container this design ever proposed" |
| HIS-D9-02 | Kernel owns only the unimplementable transitions | growth policies and pools live in source libraries | `design/language/data-model/kernel-minimality.md:1` | ADOPTED | "those policies do not require a second authority over memory safety" |
| HIS-D9-03 | Path identity `'v[i]` for an element | a sub-identity per element path | `MECHANISM-MAP.md:574` | FLOATED (open) | "Interior pointers into containers depend on the answer." |
| HIS-D9-04 | Existential backing unpacked at pointer formation | a backing identity a contract may replace | `MECHANISM-MAP.md:566,574` | FLOATED | "an existential backing that a contract may replace" |
| HIS-D9-05 | Per-element ghost tokens | one capability per slot | `containers-and-resources/REASSESSMENT.md:521` | REFUSED | "This does not mean copying the concrete model's per-slot tokens into the compiler or runtime" |
| HIS-D9-06 | Finite list of concrete tokens / enumerated per-element examples | enumerate the element capabilities | `containers-and-resources/FOUNDATION.md:222` | REFUSED | "insufficient for arbitrary runtime capacities" |
| HIS-D9-07 | Symbolic focus and framing (open one slot, keep the rest) | permission for a runtime-selected slot | `containers-and-resources/FOUNDATION.md:220` | FLOATED | "open the permission for a runtime-selected slot while retaining responsibility for every other live slot" |
| HIS-D9-08 | Finite split/join/open/close certificates with specified induction | explicit resource proof terms | `containers-and-resources/FOUNDATION.md:226,387` | FLOATED | "this is new proof machinery rather than a widening of numeric `ensures` clauses" |
| HIS-D9-09 | Widening numeric `ensures` to dynamic ownership sets | scalar contracts for element ownership | `containers-and-resources/FOUNDATION.md:385` | REFUSED | "Ordinary scalar requires/ensures are insufficient for arbitrary dynamic ownership sets." |
| HIS-D9-10 | Index plus bounds proof at element access | `[OP-4]` against the container's length | `containers-and-resources/CONTAINERS.md:389` | ADOPTED | "[OP-4] against `len(table)`, which is the `requires`" |
| HIS-D9-11 | Iterator / borrowed cursor / multi-index object | a stateful traversal object with invariants | `containers-and-resources/FOUNDATION.md:150`; `mcts_mem/whitefoot.md:65` | FLOATED / named gap | "resumable access tokens (entry tokens, cursors, guards, iterators) are untypeable under the frozen no-reborrow ... rules" |
| HIS-D9-12 | Split-and-join tokens for ranges (a protocol) | a dedicated range ownership protocol | `range-loans/DESIGN.md:124` | REFUSED | "A new split/join resource protocol would duplicate range ownership." |
| HIS-D9-13 | Bounded view formers with captured endpoints | `slice_of` / `mut_slice_of` gain positional endpoints | `range-loans/DESIGN.md:32` | ADOPTED | "assigning to the bindings that supplied them never retargets an existing loan" |
| HIS-D9-14 | Exclusive child views confined to proved subranges | recursive subdivision with parent suspension | `design/language/ownership/no-reborrow.md:2` | ADOPTED | "runtime-width rows and recursive output partitions additionally need view subdivision with explicit suspension" |
| HIS-D9-15 | Shared-only child views through borrowed holders | forbid exclusive subdivision | same node, Rejected | REFUSED † [a child's exclusivity must suspend its parent] | "they prevent recursive subdivision of exclusive output storage even when containment is proved" |
| HIS-D9-16 | Slice-valued `match` or `if` join | join the arms' origin sets | `spec/kernel-spec.md:716`; `docs/todo.md:134` | REFUSED (open) | "the rejection stands without a recorded reason" |
| HIS-D9-17 | A view of two ranges (two-iovec ring) | one descriptor over two spans | `containers-and-resources/CONTAINERS.md:266`; `REASSESSMENT.md:553` | REFUSED / DEFERRED | "this language has no spelling for a view of two ranges" |
| HIS-D9-18 | `buffer_vacant<T>(n)` returning `Option<T>` elements | compiler mints `None`, vacancy is data | `take-replace/DESIGN.md:164` | ADOPTED | "The compiler mints the n `None` values; no source value is duplicated." |
| HIS-D9-19 | Niche layout so `Option<T>` is the size of `T` | `None` occupies an invalid payload representation | `take-replace/DESIGN.md:518` | FLOATED | "Pure representation change below the checked program; sequence it after affine-element buffer lowering lands" |
| HIS-D9-20 | Slot pool as a third kernel store | provider, lease, `PoolSlot`, `PoolVector`, six rows | `containers-and-resources/CONTAINERS.md:310` | REFUSED | "the storage comes from an arena once, and the recycling is ordinary value movement over the outer run" |
| HIS-D9-21 | Linear `Lease` obligation on the handed-out value | the obligation follows the value, not the pool | `containers-and-resources/CONTAINERS.md:319` | ADOPTED | "The obligation belongs on the value that is *handed out*, not on the" container of spares |
| HIS-D9-22 | Proved `pool_release` over `requires room(...) > 0` | no refusal arm on release | `containers-and-resources/CONTAINERS.md:343` | ADOPTED | "there is then no refusal arm and the lease has one route on every path" |
| HIS-D9-23 | Stable identity for every container | a handle/store entry per container | `containers-and-resources/REASSESSMENT.md:569` | REFUSED | "Rejected as a default: the dense workload needs no identity lookup." |
| HIS-D9-24 | Append-only stable identity as its own contract | no generation or recycling cost | `design/language/data-model.md:91` | ADOPTED | "instead of one pool contract that pays for recycling everywhere" |
| HIS-D9-25 | `FixedTable<T,n>` with occupancy as a language typestate | compiler-owned occupancy | `containers-and-resources/CONTAINERS.md:398` | DEFERRED | "whose occupancy is a language typestate remains `DESIGN.md` Q6's question" |
| HIS-D9-26 | `array<T,N>` element formation widened to affine | affine elements in the fixed array | `take-replace/DESIGN.md:186` | REFUSED | "no consumer forces it, and widening it costs a second constructor story for zero demonstrated need" |
| HIS-D9-27 | Generic `Vec<T>` over affine `T` | one body for copy and affine instantiations | `take-replace/DESIGN.md:420` | DEFERRED | "it touches the one-spelling-per-meaning law (FORM-1), so it needs its own owner decision" |

## D10 Shared mutation across threads and the concurrency story

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D10-01 | No thread construct in this version | CAP-1 defines no concurrency permission | `spec/kernel-spec.md:2404` | ADOPTED | "A later thread construct must derive transfer and sharing permission from these same ownership rules" |
| HIS-D10-02 | No writer-visible capability category | modes, overlap and rows are the whole vocabulary | `spec/kernel-spec.md:2403` | ADOPTED | "the complete authority and interference vocabulary available to [PAR-1] and [PAR-2]" |
| HIS-D10-03 | Lexically scoped fork and join over PAR-1 | the window end moves to the join statement | `MECHANISM-MAP.md:497` | FLOATED | "PAR-1 already is the CSL parallel rule with syntactic footprints" |
| HIS-D10-04 | Linear task handle outliving its block | a handle carrying the child's exit contract, consumed by join | `MECHANISM-MAP.md:503` | FLOATED | "an unjoined child's obligations are an open question" |
| HIS-D10-05 | Unjoined child's obligations | a spawned child whose obligations nobody discharges | `MECHANISM-MAP.md:604` | FLOATED (open) | "CAP-1 says nothing about obligation leaks across a thread boundary" |
| HIS-D10-06 | Lock as custodian of identity state | acquire yields the invariant's facts, release requires them | `MECHANISM-MAP.md:406,505` | FLOATED | "met when threads arrive; cost only at acquire" |
| HIS-D10-07 | Two lock-bearing calls actually overlapping | a weaker permission than source-order equality | `MECHANISM-MAP.md:506,601` | FLOATED | "safe and useless" / "a decision, not a derivation; DPJ and Regent decline it to keep determinism" |
| HIS-D10-08 | Ghost protocols over shared state (Iris STS, CAP, sharding) | per-role transitions proved as ghost state | `MECHANISM-MAP.md:407` | REFUSED | "M1 needs a fixed protocol checker" / "proof burden; a second sub-language" |
| HIS-D10-09 | Protocol-governed shared regions with a memory model | several holders, occasional writes, pay at the write | `MECHANISM-MAP.md:507` | REFUSED | "recommended against; the phase and epoch pattern the catalog already measures beats a read-write lock" |
| HIS-D10-10 | Phase / epoch pattern | a measured alternative to a reader-writer lock | `MECHANISM-MAP.md:507` | ADOPTED (pattern) | "at zero reader cost" |
| HIS-D10-11 | Manifest sharing on channels (Balzer, Pfenning) | acquire/release session types | `MECHANISM-MAP.md:408` | FLOATED | "acquire is a typed outcome with real control flow, which matches M2" |
| HIS-D10-12 | GhostCell brand: identity separated from permission | many aliases share a brand; one token grants access | `MECHANISM-MAP.md:405` | FLOATED | "met at zero runtime cost" / "brand-coarse: no concurrency inside one brand" |
| HIS-D10-13 | Sendability by capability (Pony, `Send`/`Sync`) | a per-type capability lattice | `MECHANISM-MAP.md:364` | FLOATED | "met; thread-level only" / "a capability lattice to teach" |
| HIS-D10-14 | Atomic cell as a cross-thread shared-mutable primitive | a gated atomic analogue of `cell<T>` | `research/notes/batch1-spec-deltas.md:713` | DEFERRED | "an atomic-cell (cross-thread shared-mutable) analog is a distinct future gated primitive" |
| HIS-D10-15 | Command-buffer pattern instead of shared mutation | deep code returns write intents; one holder applies them | `docs/why-whitefoot.md:570`; `mcts_mem/.../no-reborrow.md:7` | ADOPTED (pattern) | "deep code cannot manufacture write access it was not handed" |
| HIS-D10-16 | Reference counting (`Rc`/`RefCell`) for sharing | refcounted shared ownership with runtime checks | `research/notes/batch1-spec-deltas.md:708`; `docs/why-whitefoot.md:570` | REFUSED | "shared ownership is a copied `handle<T>` into a single-owner `pool<T>` — zero refcount, no finalizer" |
| HIS-D10-17 | RCU / epoch / shared lifetime from a sequential slot API | claim the slot API covers concurrent reclamation | `containers-and-resources/FOUNDATION.md:112` | REFUSED | "RCU/epoch/shared lifetime is not supplied by a sequential slot API" |
| HIS-D10-18 | A specified publish/read/retire contract with a memory model | the full concurrency contract | `containers-and-resources/FOUNDATION.md:1472` | DEFERRED | "A separately specified publish/read/retire contract with the actual memory model and scheduling behavior" |
| HIS-D10-19 | Runtime queues exposed as language storage | scheduler state visible to source | `design/compiler/parallel-lowering.md:2` | REFUSED | "a runtime structure visible as source storage would need ownership and effects it cannot honestly carry" |
| HIS-D10-20 | Two-worlds lowering (overlapped plus sequential clone) | one lowering per world, selected once per process | `design/compiler/parallel-lowering/two-worlds.md:1` | ADOPTED (m) | "measured as a 2.96 times pool-off tax that fell to 1.00 with the clone world" |
| HIS-D10-21 | Per-task demand switch | a runtime demand signal read per call | same node, Rejected | REFUSED (m) | "the signal cost contended read-modify-writes measured at 0.49 to 0.93 seconds on the fine-grain cell" |
| HIS-D10-22 | Iteration-leaf parallel split | descend the range split to one iteration | `design/compiler/parallel-lowering.md` Rejected | REFUSED (m) | "measured at 3.6 to 7.6 times slower on light bodies" |
| HIS-D10-23 | Connection-level concurrency from source order | independently resumable handlers without a construct | `docs/todo.md:~73` | REFUSED (open defect) | "1024 open connections are not 1024 independently resumable handlers" |
| HIS-D10-24 | Ordinary ownership decides I/O concurrency | loans and owners alone say which I/O calls may overlap | `io-model/FIRST-PRINCIPLES.md:651,678` | ADOPTED | "No `Ordered(OutputBytes)` relation is needed." |
| HIS-D10-29 | `Sendable` / `Shareable` declared per resource family | each family declares its concurrent contract | `system-capability-architecture/DOSSIER.md:875,198` | FLOATED / DEFERRED | "CAP-1 only reserves the names Sendable and Shareable; it defines no thread" construct |
| HIS-D10-30 | Scoped `TaskGroup`, task and join handles | children accounted for before scope exit | `system-capability-architecture/DOSSIER.md:909,1193` | FLOATED / DEFERRED | "A scoped TaskGroup is one future candidate, not a selected runtime architecture." |
| HIS-D10-31 | Arbitrary asynchronous thread kill | cancel a worker from outside | `system-capability-architecture/DOSSIER.md:911` | REFUSED | "Arbitrary asynchronous thread kill is not a baseline" mechanism |
| HIS-D10-32 | Controller plus independent owned ports or lanes | deliberate sharing by explicit split | `system-capability-architecture/DOSSIER.md:46,49` | ADOPTED (in dossier) | "workers do not share a mutable" `&uniq` value |
| HIS-D10-33 | `split` / `reunite` into linked direction owners | one socket becomes two independently owned halves | `system-capability-architecture/DOSSIER.md:453,1119,1124` | ADOPTED (exemplar) / FLOATED | "returns one receive half and one send half" |
| HIS-D10-34 | stdout as a shared global lock | one process-wide lock for output | `system-capability-architecture/DOSSIER.md:887` | REFUSED | "stdout is not turned into a shared global lock." |
| HIS-D10-35 | Message passing, channels, "publish and keep going" | handoff between two concurrent agents | `io-model/CONCURRENCY-CATALOG.md:1238,1243` | REFUSED (not expressible) | "Not expressible as a handoff." / "so \"publish and keep going\" has no" counterpart |
| HIS-D10-36 | Actor mailboxes with `send` writing the destination | per-actor mailbox mutation | `io-model/CONCURRENCY-CATALOG.md:1377,1381` | REFUSED / PARTIAL (restructured to rows) | "cannot write the destination's mailbox." |
| HIS-D10-37 | Lock to phase boundary, thread to phase in the outer loop | recorded transformations replacing locks and long-lived threads | `io-model/CONCURRENCY-CATALOG.md:2696,2704,1855` | ADOPTED (transformations) | "A watchdog is not expressible." |
| HIS-D10-38 | RCU grace period as the lexical region boundary | `synchronize_rcu()` becomes a region, reclamation immediate | `io-model/CONCURRENCY-CATALOG.md:1167,1171` | ADOPTED | "The grace period is the region boundary." / "Reclamation is immediate and exact." |
| HIS-D10-39 | Fibers and user stacks as language constructs | expose the runtime's stack switching to source | `io-model/CONCURRENCY-CATALOG.md:917` | REFUSED | "Fibers exist but are not in the language." |
| HIS-D10-40 | A borrow retained across suspension; a centralized system lock | two of the dossier's rejection gates | `system-capability-architecture/DOSSIER.md:1594,1597` | REFUSED | "a borrow retained across suspension without tracked ownership;" / "a unique global context or centralized system lock on independent work;" |
| HIS-D10-25 | Operation coexistence relations (`Free`, `Ordered`, `Exclusive`) | a second graph deciding which operations may coexist | `io-model/FIRST-PRINCIPLES.md:190-199` | REFUSED | "a second graph deciding which operations may coexist" |
| HIS-D10-26 | Completion as an ownership transfer of a bundle | submission changes the owner of resource, payload and result storage | `io-model/FIRST-PRINCIPLES.md:838-856` | ADOPTED | "An I/O call is not a blocking call disguised by the compiler." |
| HIS-D10-27 | A dedicated I/O thread as part of the model | completion requires its own worker | `io-model/FIRST-PRINCIPLES.md:945-958` | REFUSED | "No dedicated user-space I/O worker is required." |
| HIS-D10-28 | Kernel or target executing a frame as a callback | completion resumes source code directly | `io-model/FIRST-PRINCIPLES.md:963` | REFUSED | "The kernel or target never executes the frame as a callback." |

## D11 External resources

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D11-01 | Host state as ordinary owned values and functions | no source-language exception for the host | `design/language/system-interface.md:1` | ADOPTED | "a special category introduced to repair one resource API would become a permanent second ownership and proof model" |
| HIS-D11-02 | Raw syscall numbers and integer descriptors | host identity as a plain integer | same node, Rejected | REFUSED | "they expose forgeable identity and unchecked host access outside the ordinary ownership boundary" |
| HIS-D11-03 | Ambient mutable host access | host state absent from the parameter list | same node, Rejected | REFUSED | "it hides inter-function state channels that the declared effect row cannot describe" |
| HIS-D11-04 | One retained `Process` object borrowed per operation | a single affine host-state object | same node, Rejected | REFUSED | "it unnecessarily serializes unrelated state" |
| HIS-D11-05 | Compiler qualification tables and target-specific guarantees | acceptance keyed to a native catalog | same node, Rejected; `design/compiler.md` Rejected | REFUSED | "a linked body must implement its ordinary declaration and origin alone grants no additional acceptance" |
| HIS-D11-06 | A literal WASI source contract | import an existing host interface wholesale | same node, Rejected | REFUSED | "it imports a path, buffering and asynchronous interface designed for cross-language components" |
| HIS-D11-07 | Linear owners with explicit consuming close | cleanup obligations visible on every exit | `design/language/system-interface.md:2` | ADOPTED | "ordinary must-consume checking exposes cleanup obligations on every exit without a hidden finalizer" |
| HIS-D11-08 | Opaque drop implicitly closing or returning quota | a hidden release action | same node, Rejected | REFUSED | "an empty ordinary drop cannot conceal a host-state write or a must-consume obligation" |
| HIS-D11-09 | Host types as ordinary prelude declarations | one declaration domain for host and source | `design/language/system-interface/declaration-home.md:1` | ADOPTED | "implementation origin cannot justify a separate acceptance path" |
| HIS-D11-10 | A compiler-owned system declaration domain | a second declaration space for host values | same node, Rejected | REFUSED | "it distinguished otherwise ordinary values and functions solely by implementation origin" |
| HIS-D11-11 | Effect categories for host mechanisms (`external`, `blocks`) | mechanism labels rather than state | `design/language/effects.md` Rejected; `MECHANISM-MAP.md:418` | REFUSED | "system resources are ordinary state under ownership and scheduling belongs to lowering" |
| HIS-D11-12 | Capability tokens for I/O (Effekt, Scala capture checking) | a capability value in scope authorizes the effect | `MECHANISM-MAP.md:417` | FLOATED | "expressible as an identity in scope" / "a second vocabulary" |
| HIS-D11-13 | External state as identities with declared states and opaque contents | one identity per resource, states in contracts | `MECHANISM-MAP.md:416` | FLOATED | "met; no special rule per resource" / "contracts per operation" |
| HIS-D11-14 | A descriptor as three identities | wrapper, open-file description, contents | `MECHANISM-MAP.md:513` | FLOATED | "RESEARCH.md's I/O witness already separates these subjects" |
| HIS-D11-15 | A `foreign` identity qualifier | externally writable storage retains no contents fact | `MECHANISM-MAP.md:508,602` | FLOATED | "passes the constitution only if spelled over interference ... never over origin" |
| HIS-D11-16 | `HandleFactory` borrowed exclusively, credits spent and restored | explicit capacity accounting | `design/language/system-interface/handle-factory.md:1` | ADOPTED | "this exposes accounting without implicit creator links, cancellation effects or early loan release" |
| HIS-D11-17 | Available capacity as a promise that acquisition succeeds | treat a credit as a success guarantee | same node, Rejected | REFUSED | "other host activity and limits can still refuse an otherwise funded attempt" |
| HIS-D11-18 | Region-bearing leases for host strings | `HostString` as a borrow with a lifetime | `design/language/system-interface/host-string-lease.md:1` | REFUSED | "argument-derived values must remain storable and lossless" |
| HIS-D11-19 | Host exhaustion as a source fact or proof fallback | quota failures inside the proof model | `design/language/checks-and-proofs/obligation-discharge.md:47` | REFUSED | "host quota and exhaustion are outside the source-outcome model" |
| HIS-D11-20 | Resource states as ordinary `Result` outcomes | per-operation payload types, no shared union | `design/language/system-interface/outcome-typing.md:1` | ADOPTED | "exhaustive matching should not require outcomes the operation cannot produce" |
| HIS-D11-21 | Exhaustion as a class of an error payload | branch on an error classification | `containers-and-resources/DESIGN.md:4493` | REFUSED | "no route in [CALL-4] is conditioned on a class" |
| HIS-D11-22 | Trusted adapters outside the language | upstream permission models as acceptance authority | `containers-and-resources/FOUNDATION.md:155` | REFUSED | "Their proof/trust models are comparators, not acceptance authority for WF." |
| HIS-D11-23 | Effect-derived sandbox manifests | derive a capability set from checked rows | `docs/ideas.md:203` | FLOATED | "An ordinary effect row alone does not identify every runtime path, endpoint, or foreign call." |
| HIS-D11-24 | Two state domains (`memory` vs `world` rows) | separate row tags for storage and host state | `io-model/FIRST-PRINCIPLES.md:189-191` | REFUSED | "`memory reads(...)` and `world reads(...)` tags" |
| HIS-D11-25 | A system-capability type category | host values in a category of their own | `io-model/FIRST-PRINCIPLES.md:193` | REFUSED | "a system-capability type category" |
| HIS-D11-26 | Capability roots, families, fragments, coexistence tables | a second authority graph over host operations | `io-model/FIRST-PRINCIPLES.md:194` | REFUSED | "capability roots, families, fragments, or coexistence tables" |
| HIS-D11-27 | A hidden global identity repairing an incomplete API | invent an identity where ownership is missing | `io-model/FIRST-PRINCIPLES.md:197` | REFUSED | "a hidden global identity used to repair an ownership-incomplete API" |
| HIS-D11-28 | Removing the syntax but keeping the analysis | drop the tags, retain the second graph internally | `io-model/FIRST-PRINCIPLES.md:200-203` | REFUSED | "would preserve the same contradiction. The implementation must remove the second graph as well." |
| HIS-D11-29 | One opaque state value per resource, `&uniq` plus `writes(path)` | exclusive permission and a declared transition | `io-model/FIRST-PRINCIPLES.md:178-187` | ADOPTED | "That representation access is internal to `Output`; it is not a second state domain." |
| HIS-D11-30 | `external` and `blocks` as effect categories | a call may touch state outside WF memory, or block its thread | `system-capability-architecture/DOSSIER.md:557,560`; `io-model/FIRST-PRINCIPLES.md:196` | ADOPTED-in-dossier then REFUSED | "the call may observe or change state outside ordinary Whitefoot" / "an ordinary call may block its current host thread." |
| HIS-D11-31 | Parameterized effects (`external(cwd)`, `changes(file)`) | resource-origin-typed effect entries | `system-capability-architecture/DOSSIER.md:605` | REFUSED | "the complexity would have no consumer." |
| HIS-D11-32 | Suspend and spawn effect categories | control effects for background I/O and tasks | `system-capability-architecture/DOSSIER.md:624` | DEFERRED | "Starting background I/O, suspending a task, and spawning execution will need" them |
| HIS-D11-33 | Lifetimes / REGIONIDs as effect subjects | `reads('r)`, `writes('r)` naming storage by region | `io-model/FIRST-PRINCIPLES.md:229`; `io-model/DESIGN.md:75` | REFUSED | "being forced to act as duplicate parameter names." |
| HIS-D11-34 | Locals or the result binder as effect roots | non-formal roots in a row | `io-model/FIRST-PRINCIPLES.md:290` | REFUSED | "Only formal-rooted paths appear in a callable boundary." |
| HIS-D11-35 | Attenuation and revocation of a resource capability | consume a broader owner for a narrower one; invalidate handles | `system-capability-architecture/DOSSIER.md:446,464` | ADOPTED-in-dossier / DEFERRED | "Revocation is not part of the first system interface." |
| HIS-D11-36 | Three completion policies (release-complete, abandonable, completion-required) | how much of the obligation the compiler owns | `system-capability-architecture/DOSSIER.md:495,501,504` | ADOPTED (1) / FLOATED (2) / DEFERRED (3) | "compiler-derived release is the complete language" obligation |
| HIS-D11-37 | Compiler-owned semantic operation IDs and a qualification table | host ops identified by a spec-fixed ID per target | `system-capability-architecture/DOSSIER.md:929`; `ordinary-host-values/DESIGN.md:49` | ADOPTED-in-dossier then REFUSED | "Each system operation has one target-independent compiler-owned semantic ID." |
| HIS-D11-38 | Proof-only permit versus a permit backed by real capacity | whether a reservation holds a real unit | `io-model/DESIGN.md:224`; `io-model/FIRST-PRINCIPLES.md:798` | ADOPTED then SUPERSEDED | "A logical permit reserves one real unit of the finite resource, or it reserves" nothing |
| HIS-D11-39 | An `externally_observed` type tag, or a `Published`/`Private` state category | mark host-visible state for the optimizer | `io-model/DESIGN.md:382`; `io-model/FIRST-PRINCIPLES.md:460` | REFUSED | "does not add `Published`, `Private`, or another hidden state category." |
| HIS-D11-40 | An environment-driven-read exception | a read whose result may change with no WF write | `io-model/DESIGN.md:387`; `io-model/FIRST-PRINCIPLES.md:374,377` | REFUSED / PARTIAL | "similar APIs do not require an environment-driven-read exception." |
| HIS-D11-41 | External aliases (hard links, redirected streams) merging owners | let host-level aliasing collapse two owners | `io-model/FIRST-PRINCIPLES.md:720`; `system-capability-architecture/DOSSIER.md:1599` | REFUSED | "Aliases introduced outside the mapped Whitefoot program do not merge ordinary" owners / "handle identity or target metadata used as a disjointness proof;" |
| HIS-D11-42 | Program kinds: command, service, embedded | entry shape as a declared kind | `system-capability-architecture/DOSSIER.md:384,385,388`; `ordinary-host-values/DESIGN.md:156` | ADOPTED-in-dossier then REMOVED | "Remove `command`, `command.x as y`, input ordinals, and the associated entry" machinery |

## D12 Surface form and sugar

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D12-01 | `&uniq` spelling for the exclusive mode | named for uniqueness, not mutation | `design/language/surface-form/borrow-lexicon.md:2`; `spec/kernel-spec.md:172` | ADOPTED | "`mut` conflates exclusivity with write permission in a way that breaks under future interior-mutability capabilities" |
| HIS-D12-02 | `&mut` spelling | Rust's convention | `mcts_mem/.../borrow-lexicon.alt/mut-spelling.md:9` | SUPERSEDED | "the exclusive mode's invariant is uniqueness, not mutation" |
| HIS-D12-03 | `noalias` or other backend vocabulary at the surface | name the mode after its lowering consequence | `design/language/surface-form/borrow-lexicon.md:1` | REFUSED | "a name like noalias names a lowering consequence rather than the invariant the checker enforces" |
| HIS-D12-04 | One spelling per construct, decided by grammar class | legality never depends on inference at a site | `design/language/surface-form.md:1` | ADOPTED | "a rule keyed on whether the checker could infer an element at a site would force the writer to simulate the checker" |
| HIS-D12-05 | Per-position relief | an element optional wherever it is reconstructible | same node, Rejected | REFUSED | "legality must be decidable from the grammar class alone" |
| HIS-D12-06 | Signature positions stay written, body binders derived | modes and types written only at trust boundaries | `design/language/surface-form.md:4`; `spelling-relief/SWEEP.md:58` | ADOPTED | "the interface is the trust boundary; redundancy here is the review story" |
| HIS-D12-07 | Region spelling: brands named, loan regions elided | a formal brand is named even at one occurrence | `design/language/ownership/region-elision.md:1` | ADOPTED | "the writer must determine spelling from the declaration alone" |
| HIS-D12-08 | `gparam += REGIONID` (regions in the generic list) | one spelling for type and region parameters | `research/notes/batch1-spec-deltas.md:643` | REFUSED | "makes region-declaration have two spellings ... the exact irregularity META-1/META-2 name as the enemy" |
| HIS-D12-09 | Region syntax retired, `'s` reused with identity meaning | keep the sigil, change what it names | `MECHANISM-MAP.md:546,585` | FLOATED | "PROV-1 spelling `'s` with identity meaning only" |
| HIS-D12-10 | `&` / `&uniq` as sugar over pointer plus state clause | modes become abbreviations | `MECHANISM-MAP.md:585` | FLOATED (open) | "`&` and `&uniq` as sugar over pointer plus state clause; region syntax retirement; pattern cards" |
| HIS-D12-11 | Explicit `region { }` blocks at every borrow | writer-spelled region scoping | `docs/todo.md:105` | FLOATED (open) | "Compare explicit blocks, function-body regions and implicit regions for non-escaping temporaries" |
| HIS-D12-12 | Every loop body implicitly a region block | no written region at a loop | `spec/kernel-spec.md:771` (OWN-11); `containers-and-resources/CONTAINERS.md:40` | ADOPTED | "an explicit `region { }` as the loop body's only enclosing block is a `[FORM]` rejection" |
| HIS-D12-13 | Last-use endpoints for `let`-bound holders | end a borrow after its last required use | `docs/todo.md:122` | FLOATED (open) | "No change to the ordinary borrow rules is selected." |
| HIS-D12-14 | Confining an argument child's region to one statement | the child's local region holds one statement only | `design/language/ownership/no-reborrow.md` Rejected | REFUSED | "the ordinary temporary endpoint and independent surviving-loan checks already preserve exclusivity" |
| HIS-D12-15 | Flat three-address form with every intermediate named | no nesting, no precedence | `design/language/surface-form.md:3` | ADOPTED | "the form is performance-neutral by measurement" |
| HIS-D12-16 | Comments | free-form text beside code | `design/language/surface-form.md:2` | REFUSED | "a comment is text the checker cannot see and a doc field is a checked part of the declaration" |
| HIS-D12-17 | `replace` as a bare statement without an old-value binder | drop the old value implicitly | `take-replace/DESIGN.md:52` | REFUSED | "a bare `replace p = e;` statement form was rejected exactly because the old value would need an implicit drop" |
| HIS-D12-18 | `seq_` prefix and `X_of` derivation naming | operation naming schemes | `containers-and-resources/DESIGN.md:4700,4735` | ADOPTED / REFUSED | "A non-consuming derivation from a value is spelled `X_of(v)`" |

## D13 Validation and migration strategy

| id | Option | Mechanism | Where recorded | Status | Reason recorded (verbatim) |
|---|---|---|---|---|
| HIS-D13-01 | Bounded small-model enumeration | exhaustive generation over a finite fragment | `research/experiments/access-state/RESULTS.md:33`; `MECHANISM-MAP.md:581` | ADOPTED (for the fragment) | "Report counts and bounds rather than treating bounded enumeration as a soundness proof." |
| HIS-D13-02 | Model check over the pre-reborrow core only | reuse an existing clean run as evidence | `reborrow-investigation/MINIMAL-RULE.md:174` | REFUSED | "The existing 10k-model clean run covered the pre-reborrow core only." |
| HIS-D13-03 | Model check over the widened form space | generate the new forms and the escape vectors | `reborrow-investigation/MINIMAL-RULE.md:171` | ADOPTED (plan) | "assert no accepted program has two live `uniq` borrows with overlapping resolved places unless one derives from the other" |
| HIS-D13-04 | Formal reconciliation against Featherweight Rust | a soundness-proven fragment as the target | `archive/governance/decisions/v0.0-v0.4.md:136`; `MINIMAL-RULE.md:176` | ADOPTED | "Featherweight Rust selected as the section-5 reconciliation target" |
| HIS-D13-05 | Hostile fact-channel review before shipping a fact | re-derive `noalias` under the new premise | `reborrow-investigation/DOSSIER.md:142` | ADOPTED (requirement) | "CLAUDE.md forbids shipping a fact channel on a green gate alone" |
| HIS-D13-06 | A green gate as the review | tests plus `make check` as sufficient evidence | `reborrow-investigation/DOSSIER.md:91` | REFUSED | "167 tests + a green make are the green gate, not that review" |
| HIS-D13-07 | Structural invariant instead of distributed bookkeeping | encode suspension structurally, or assert it | `reborrow-investigation/DOSSIER.md:114` | FLOATED | "A future edit that mints/transfers a child without threading `derived_from` ... silently reopens aliasing with a green gate." |
| HIS-D13-08 | Dynamic-oracle cross-check (Miri-style interpreter) | an interpreter over canonical IR in debug builds | `archive/governance/decisions/v0.0-v0.4.md:74` | ADOPTED | "dynamic-oracle cross-check (Miri-style interpreter over canonical IR in debug builds)" |
| HIS-D13-09 | Differential testing against independent backends | LLVM, C and Wasm as mutual oracles | `docs/ideas.md:146` | FLOATED | "This approach can catch a lowering defect that source conformance misses." |
| HIS-D13-10 | Differential fuzzing / executable disagreement witnesses | generated programs compared across builds | `docs/ideas.md:441` | FLOATED | "Agreement between two executions cannot establish that their shared source checker or lowering is sound." |
| HIS-D13-11 | Independent native oracles per kernel | a C reference computing the same result | `compute-model/DESIGN.md:346` | ADOPTED | "checking every output element, the result length, and unchanged input" |
| HIS-D13-12 | Discriminating programs fixed before measurement | criteria recorded ahead of the result | `range-loans/DESIGN.md:149`; `research/experiments/access-state/RESULTS.md:17` | ADOPTED | "Discriminating criteria, fixed before results" |
| HIS-D13-13 | Compiler's own source as the coverage census | count sites in `compiler/src/*.wf` | `reborrow-investigation/MINIMAL-RULE.md:158` | ADOPTED (m) | "989 `&uniq 'r deref(h)` + 73 shared `&'r deref(h)` = **1,062 sites, 100% admitted**" |
| HIS-D13-14 | The compiler's source as a source of truth for the rule | keep a rule because the compiler needs it | `reborrow-investigation/DOSSIER.md:25` | REFUSED | "wfc is early code and is **not** a source of truth" |
| HIS-D13-15 | One-shot integration switch for a candidate rule | `REBORROW_EXTENSION_ACTIVE` / `DECLARATION_PROVENANCE` | `reborrow-extension/SPEC-DELTA.md:13`; `declaration-provenance/SPEC-DELTA.md:14` | ADOPTED (migration) | "default `false` = v0.30 semantics byte-identical" |
| HIS-D13-16 | Superseded inventory states behind compile-time switches | keep old semantics reachable for differential tests | `design/compiler.md` Rejected | REFUSED | "the compiler implements exactly one specification, the active one" |
| HIS-D13-17 | Amend the specification as one change (retitle plus archive) | vN+1 active, vN bytes archived | `CLAUDE.md` branch-and-main rules | ADOPTED | "There is no candidate state; a branch carrying an amendment is merge-ready when its gate is green." |
| HIS-D13-18 | META-5 delta accounting as a selection instrument | count rules, tokens, spellings, exceptions per amendment | `range-loans/DESIGN.md:140`; `reborrow-extension/SPEC-DELTA.md:180` | ADOPTED | "numbered rules +0/-0 ... lexical tokens +0/-0, operation spellings +0/-0" |
| HIS-D13-19 | Pattern cards migrated with the rule | `docs/patterns.md` updated in the same change | `MECHANISM-MAP.md:585`; `MINIMAL-RULE.md:188` | ADOPTED (plan) | "PATTERNS P4 no-reborrow → bounded reborrow" |
| HIS-D13-20 | Deferring bulk test cleanup until the spelling settles | keep repeated per-call region wrappers for now | `docs/todo.md:119` | ADOPTED | "preserving each case's intended behavior or rejection reason when the selected spelling is applied" |

---

## Decision points the repository's history suggests are missing

| id | Missing decision point | Why the record suggests it | Where recorded | Quote |
|---|---|---|---|
| HIS-DPX-01 | Frame / fact-retention keying as its own choice | R6 exists as a requirement but no D asks how kills are keyed, and the repository has a measured failure of the wrong choice | `design/language/checks-and-proofs/obligation-discharge/loop-fact-retention.md:1` | "the main cause of the DEFLATE decoder proving 5 of its 29 obligation sites where 17 had been predicted" |
| HIS-DPX-02 | Release obligation and the release graph | who holds the obligation, `dispose`, capability-by-scope, cycle refusal — a whole live subtree with no D | `design/language/ownership/linearity.md`; `containers-and-resources/RESOURCES.md:96` | "a cycle through the **release graph** ... is refused at the type in every program" |
| HIS-DPX-03 | Generics and abstraction over ownership classes | the copy/affine/linear bound chain and the generic-body pincer are recorded design work | `containers-and-resources/DESIGN.md:4650`; `move-on-copy/REPORT.md` M2 | "The three classes form a strict chain `copy < affine < linear`" |
| HIS-DPX-04 | Resource bounds and the exhaustion envelope | extent items, stack items, `saturating(d)`, per-activation identity — an axis D0..D13 never names | `containers-and-resources/RESOURCES.md:62,91` | "A reserving occurrence must be a statement of its own region block and of no loop inside it" |
| HIS-DPX-05 | Error and outcome model | Result vs trap vs claim is a settled decision with recorded rivals, not a meta-constraint | `design/language/checks-and-proofs/obligation-discharge.md:45,51` | "An expected failure is a typed outcome or intended control flow" |
| HIS-DPX-06 | Compile-time checking cost of the candidate | a live measured defect that constrains any per-identity state model | `docs/todo.md` (checking cost) | "256 independent inequality pairs with 256 uses still take a median 5.50 s" |
| HIS-DPX-07 | Whether a fact channel to the backend exists at all | R4 has no mechanism today; building the channel is itself a decision with a verification story | `MECHANISM-MAP.md:489`; `docs/ideas.md:447` | "a channel to build, under either model" |
| HIS-DPX-08 | Permission versus actualization and the two worlds | lowering choices (clone worlds, budgets, grain) interact with the permission rule | `design/compiler/parallel-lowering/two-worlds.md:1` | "no single lowering serves both worlds" |
| HIS-DPX-09 | Diagnostics as a first-class output | every rejection must name a rule and a restructuring; M4 treats this only as a cost | `design/compiler.md:4`; `spec/kernel-spec.md:802` | "a hard error citing OWN-14 with the restructuring" |
| HIS-DPX-10 | Specification-size accounting as a selection rule | META-5 deltas are used to choose between candidate rules | `range-loans/DESIGN.md:140` | "The selection ground is evidence-selected for bounded child loans" |
| HIS-DPX-11 | The closed world itself | PROG-1 is listed as a premise, but it is a recorded decision with a reason and a rival | `design/language.md:6` | "a module or separately compiled interface is a fact-loss surface" |
| HIS-DPX-12 | Suspension endpoints as a single principle | four different endpoints exist with no stated principle | `EVIDENCE-mechanism-survey-2026-09-16.md:285` | "The text enumerates them but states no single principle from which they follow." |

## Requirements the repository's own documents state that the map omits or misstates

| id | Requirement | What the record says | Where recorded | Quote |
|---|---|---|---|
| HIS-REQ-01 | R9 "Sequentially this is safe" misstates today's rule | OWN-5's exclusivity invariant is unconditional, so several long-lived writable pointers are not admitted today | `spec/kernel-spec.md:706` | "Exclusivity invariant, checked unconditionally" |
| HIS-REQ-02 | R4 as a language requirement is contested by the tree | the owner removed the optimizer-fact decisions; what reaches the backend is not a language fact | `design/log.md:168`; `design/language.md:3` | "what the compiler hands LLVM ... is an implementation detail the language never sees and needs no rule" |
| HIS-REQ-03 | M2 is a chosen mechanism with recorded rivals, not a given | Result-everywhere and global prove-or-handle were weighed and refused | `design/language/checks-and-proofs/obligation-discharge.md:51-52` | "Rejected as global law: with a deterministic no-search checker the unprovable-but-true residue is large" |
| HIS-REQ-04 | M5's "accepted shapes are the fast shapes" is open, not held | the obvious-shape property was a retired-prototype result | `docs/why-whitefoot.md:410` | "reproducing that property in the current compiler remains open" |
| HIS-REQ-05 | R2 understates the resource requirement | bounded memory is a whole accounting system (envelope `E`, extent items, stacks, lanes), not one clause | `containers-and-resources/DESIGN.md:433` | "`E` is a list of tangible resources ... and never one byte total" |
| HIS-REQ-06 | R7 omits the declaration-side obligation | the boundary constrains declarations, not only calls | `declaration-provenance/SPEC-DELTA.md:10` | "a declaration whose result no caller can use is itself the error" |
| HIS-REQ-07 | The requirement set omits abstraction and generics | monomorphization, bound vocabulary and instantiation termination are live decisions | `design/language/generics.md:1-2` | "a growing recursive instance would not terminate and the syntactic cycle check decides that boundary" |
| HIS-REQ-08 | The requirement set omits typed error outcomes | outcome typing is a live system-interface decision | `design/language/system-interface/outcome-typing.md:1` | "ordinary propagation then needs no conversion from equivalent success/error variants" |
| HIS-REQ-09 | The requirement set omits checking cost and teachability as requirements | both are recorded costs that have already changed decisions | `docs/ideas.md:511-527`; `design/language/pattern-doctrine.md:2` | "Deterministic termination does not establish practical iteration" |
| HIS-REQ-10 | "No global mutable state" is a requirement R9 does not acknowledge | the ban's reason is parallel permission itself | `design/language/ownership.md:5` | "with no shared-memory threads there is nothing a global lock would guard" |
| HIS-REQ-11 | M4's "the specification stays small" is operationalised, not aspirational | META-5 counts are part of every amendment | `reborrow-extension/SPEC-DELTA.md:180` | "Rules edited: OWN-5, OWN-6, OWN-9 (non-normative), OWN-12, OWN-14, ENT-5. Tokens ±0, spellings ±0." |
| HIS-REQ-12 | The map's R1 (d) "not its job: preventing two pointers to one storage" contradicts the live rule | OWN-5 does exactly that today, and OWN-9 depends on it | `spec/kernel-spec.md:763` | "the guarantee is one usable mutable path per place" |

