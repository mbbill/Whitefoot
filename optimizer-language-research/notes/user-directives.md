# User directives (fixed design constraints)

## THE GOAL — see /Users/bytedance/Dev/xlang/CONSTITUTION.md

The constitution (floors, P0 with the Rust test R0, P1 = W1/W2, balance rule, R1–R6) is the authoritative top-level document at the repository root: `CONSTITUTION.md`. It supersedes and grounds every directive below; the text was relocated there on 2026-07-07 for discoverability (stated once, referenced here).

Subordinate directives D0-D3 below are instances of this constitution.

These are decisions made by the project owner. They are not debate topics; debates argue mechanisms and envelopes under them.

## D1 (2026-07-02): Memory- and thread-safety bugs must be impossible
Data races, use-after-free, dangling references, double-free, and reads of uninitialized memory must be impossible in programs accepted by the language as shipped. Rust-class static enforcement (ownership/borrowing, Send/Sync-class capabilities) is the reference bar.

Rationale recorded: these are the nastiest bug classes in other languages; the project owner does not want them representable. Corpus support: F001, C004 (zero-runtime-cost static enforcement demonstrated); round-1 finding that non-interference proofs quantify over race-free executions, making race freedom load-bearing for optimizer soundness.

Open (delegated to round-3 safety-envelope debate): existence/form of any unsafe escape hatch; runtime-check policy for bounds/overflow; FFI fact-attenuation; proof burden placed on AI writers.

## D0 (2026-07-01): Writer is an AI; extreme performance over ergonomics
All code is AI-written. Verbosity/repetition acceptable. Machine-writer costs (tokens, drift, compile-loop latency, machine diagnostics) are admissible; human ergonomics are not.

## D2 (2026-07-02, CORRECTED same day by owner): The language SPEC must be compact; program verbosity is acceptable
The language is new — models learn it from a spec + examples in context. The constraint is the SPEC budget only: the language's description (grammar, rules, annotation vocabulary, examples) must be small — few rules, uniform, no special cases, each stated once. Program-text verbosity is explicitly ACCEPTABLE: a uniform verbose-everywhere rule is spec-cheap (stated once, never repeated), and the owner rules that the benefits of explicit facts in generated code far outweigh generation-token usage.

What eats the spec budget is IRREGULARITY: syntactic sugar, alternate spellings, inference rules, context-dependent elision, special cases, exceptions to rules. Each adds description mass and model-error surface.

Consequence: round-2's "verbosity is free" premise is RESTORED for program text. Spec description-length/regularity becomes the first-class metric. The interpretation in the superseded round-4 run (program-token budget as first-class) is void; see debates/round4-superseded-journal-wf_868fd57c-1c6.jsonl.

## D1a (2026-07-02, owner ruling): D1 quantifier = Rust-class conditional, gated on checker feasibility
Safe code is unconditionally sound; a small, gated, audited trusted base (primitive base impls, FFI frames) carries named ledger obligations. CONDITION attached by owner: this stands only if the ownership/borrow checker is implementable at reasonable effort — "it is pointless if the effort of implementing such a checker is an impossible task"; a normal compiler frontend is acceptable effort; replicating rustc's borrow checker is not, unless simplified. Consequence: checker simplification is a design requirement, and a checker-core feasibility prototype is a blocking gate for D1a.

## D0a (2026-07-02, owner ruling): trusted-base authorship = AI-authored, human-approved
Gated-channel content (toolchain primitives, unsafe-family members, non-conservative FFI frames) is AI-authored; a human approves each gate request, at least initially; revisit after gate-efficacy experiments run. The steady-state writer still never authors trust.

## D3 (2026-07-03, owner ruling): Rust-influence guard + lexicon rulings
We are not building another Rust; we are avoiding Rust's downsides while borrowing deliberately. Every Rust-shaped choice must trace to non-Rust evidence or carry an explicit justification; the lexicon divergence census treats the Rust prior as a risk, not a default.

Lexicon rulings of record:
- The exclusive borrow mode is `uniq` (uniqueness-type lineage: Clean/Futhark; cf. Pony iso), NOT `mut` (Rust's overload conflates exclusivity with mutation and breaks under interior mutability) and NOT `noalias` (backend-IR vocabulary names a lowering consequence, not the source invariant; layer separation; rustc noalias on/off history shows backend coupling ages badly).
- Surface names label checked invariants, self-containedly defined (kernel-spec LEX-1). Flagged for the same census: Ok/Err/Some/None, box, fn/let, angle-bracket generics.

## D4 (2026-07-06, owner ruling): Rewrite-first, FFI-narrow
FFI exists for exactly two situations: (1) opaque binaries (source unavailable: OS/syscalls, drivers, vendor/certified blobs) — foreign data races are out of scope; the wall's only job is that foreign behavior cannot falsify internal proofs (quarantine + pre-declared foreign_shared, per round 3); (2) source-available foreign code — the default is REWRITE into this language (AI rewrite cost falling; per-module gain = W3 + P0 + proofs), FFI only where rewriting is genuinely blocked (validation-certified numerics, licensing). Consequences: the ffi_abi_runtime dossier narrows to the C-ABI binary boundary (calls out, buffer pinning, unwind-abort); rich cross-language object interop (JNI-style object graphs, extension-API emulation) is OUT OF SCOPE; FFI-IN (exports/callbacks/foreign threads) is deferred behind a single-threaded-entry contract for v1; rewrite-equivalence verification (differential testing tooling) is registered as future work.

## D2a (2026-07-07, owner ruling): W2 deprioritized
Token/context budget remains a consideration but NOT a top priority: model context windows are growing rapidly, moving this constraint down the list. Consequences: (a) spec token counts remain MEASURED (spec-delta discipline, calibration audits) but no token cap acquires gating authority for the foreseeable future; (b) the round-4 regularity invariants (one spelling, zero context-dependence, empty exception lists, stated-once) are RETAINED — re-grounded on W1 (irregularity is weak-writer error surface, unaffected by window growth) rather than on window size; (c) the resident-vs-total open ruling and spec-size-ablation experiment drop in priority; (d) residual W2 weight: long-context recall degradation and per-token cost/latency, tracked not gating.
