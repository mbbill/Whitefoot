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

## D5 (2026-07-09): Phase C model-tier sprint deprioritized

Owner: model capability is improving fast enough (GPT-5.6, Fable-class models
usable) that "can a weak AI write it" is a shrinking constraint; do not rush
the W1 model-tier sprint. Priority low, possibly none. CAVEAT kept by owner:
context limits still exist — so D2 (spec compactness; the whole spec must fit
in a prompt) remains binding even as W1's weak-writer bar softens.
Consequences: R0's criterion needs rewording (it was "self-host iff >=1 robust
B delta OR C W1 win"); the decision now rests on the B deltas plus the
frequency/distributional question (how often the channel patterns occur in
real projects), plus the qualitative W3/determinism case. The trial harness
and writer's excerpt stay (already built; can run anytime as validation).

## D6 (2026-07-09): Pattern doctrine — the language ships a closed, taught pattern catalog

Owner: design patterns are part of a language's contract with its writers.
Human languages must let users carry familiar patterns in, or they are
rejected; xlang has no such constraint (D0a: AI writes, human approves). So
xlang may make radical restrictions: force a curated SUBSET of design patterns
for modeling tasks, exactly as it forces one loop form and one conditional
form at the micro level. Two-fold acceptance test for the catalog:
(1) COMPLETE — every task must be modelable inside the blessed patterns
(gaps are findings, to be closed by a new blessed pattern or a recorded
rejection); (2) EFFICIENT — tasks modeled with the blessed patterns must hit
the fast paths (the patterns should be exactly the shapes the fact channels
light up). Corollary: patterns are TAUGHT, not discovered — the teaching pack
must present the catalog up front so no writer (AI or human) hits a wall
mid-design discovering that a familiar architecture (e.g. scattered deep
writes) is unrepresentable. Catalog artifact: PATTERNS.md (normative,
spec-adjacent). Trigger: owner review of the deep-write discussion — the
command-buffer idiom must be doctrine, not folklore.

## D7 (2026-07-10): Impact target — swap-in everyday artifact, not niche VM

Owner (who has personally built high-performance wasm interpreters/JITs —
Silverfir-rs, Silverfir-nano, mitey-jit in ~/Dev; mitey-jit hand-implements
the musttail register-pinning trick channel 4 would automate): "a fast wasm
interpreter" is not attractive to most people — they don't know what it means.
The advertising-grade demonstration is REBUILDING AN EVERYDAY APP OR LIBRARY
that users can swap into their existing workflow/toolchain and FEEL the
performance difference. Consequences for the port-study ladder: candidates are
ranked by (subset fit) x (felt-by-users) x (swap-in-ability), not by benchmark
canon. Swap-in credibility protocol: bit-identical outputs vs the incumbent +
fuzz-diff harness (fits the conformance culture). Channel-4's eventual real
experiment: port the dispatch core of the owner's own engine (Silverfir
in-place tier), owner referees equivalence.

## D7a (2026-07-10): Headline target — hot-path coreutils, AI-authored

Owner: the QOI-class artifact is technically right but lacks "big news"
feeling; the target class is "re-implemented X: xx% faster, feature parity,
no CVE" headlines. Ruling folded with strategy discussion: the Rust-rewrite
headline is already taken (uutils -> Ubuntu), so xlang's differentiator is the
AI-authorship story stacked on the safety+speed story: "a language designed so
AI cannot write the bug classes; the AI wrote your coreutils; they are faster
than GNU and the memory-corruption CVE class is unrepresentable." Scope
discipline (the parity trap): full coreutils parity is a multi-year human
project even for uutils — the honest unit is PER-UTILITY FULL PARITY on the
hot pipeline utilities first (wc, base64, cksum/sum, tr, cut, uniq, head/tail
— the byte-compute speed demons), each verified by (a) GNU's own test suite
for that utility, (b) fuzz-diff vs GNU byte-identical output, (c) LC_ALL=C
byte-mode parity FIRST with locale parity staged later, honestly labeled.
Claim discipline: never "no CVE" — "the memory-corruption CVE class is
unrepresentable"; logic bugs remain possible and the fuzz-diff harness is the
answer. Language gap list this pins (priority order): (1) const arrays
(codegen exists in spec, unimplemented) — tables; (2) an I/O frame (D4
FFI-narrow: read/write/open shims) — the first FFI-frame instance; (3)
growable-buffer story or chunked-buffer pattern; (4) argv/byte-string
ergonomics. The existing ladder (fannkuch -> QOI) is re-aimed as engineering
rungs toward this target, not ends in themselves. AI-authorship angle revives
the shelved M3/W1 trial apparatus as the PRODUCTION method (the harness
becomes the factory, not the experiment).

## D7b (2026-07-10): Headline strategy — fuse an uncontested property onto a felt artifact

Five-lens brainstorm (39 ideas, archived: notes/headline-brainstorm-39-ideas.json)
+ owner-recovered synthesis. Key finding: the skeptic lens splits everything
into Pile A ("felt swap-in" artifacts — CRC32/base64/inflate/PNG — whose
"rewritten, safe, fast" story the Rust wave already owns via
zlib-rs/uutils/rustls) and Pile B (properties Rust structurally cannot express
— checked constant-time, guaranteed musttail dispatch, checked laws). Strategy:
FUSION — put a Pile-B property on a Pile-A artifact. The chosen fusion:
constant-time crypto primitive (ChaCha20, pure ARX = add/rotate/xor on u32 —
already the xlang op table), AI-written, RFC-8439-vector-verified, with
timing-safety as a CHECKED TYPE PROPERTY (secret-typed data; branching or
indexing on a secret is a compile error). Framing requirement (owner-risk
noted): "AI-written crypto" alone reads as scary to security people — the
checker-proves-the-properties story must lead (W3/D0a: you don't trust the AI,
you trust the checker). Claim discipline: research/DSL precedents EXIST (FaCT,
CT-Wasm, Jasmin — Jasmin is used for real crypto); the honest claim is "first
general-purpose systems language where timing-safety is a checked type
property", not "first ever".

Ladder: (1) NOW — CRC32 single-symbol LD_PRELOAD rehearsal ("this AI-written
function is running inside your git right now"): runtime-generated tables
avoid the const-array gap; claim is bit-identical + AI-written + parity speed,
NOT faster (beating zlib-ng needs hw crc32/PMULL ops we don't have — honest).
(2) NEAR — the ChaCha20 fusion; requires CHANNEL 5 carding: the secret
effect/type (no secret-dependent branches, no secret-indexed memory — a
taint-tracking lattice in the checker, derivable from W3+T1; card before
speccing). (3) BIG — inflate/zlib drop-in, contested by zlib-rs, only worth it
with the fusion story attached (AI-written + faster-than-the-Rust-rewrite).

## D7c (2026-07-10): Drop crypto/checksum artifacts; re-aim on compute + correctness framing

Owner: the constant-time-crypto (ChaCha20) and CRC32/symbol-interposition
directions repeatedly tripped an automated content filter (false positive —
the work is entirely legitimate defensive/performance engineering, but the
topics pattern-match to sensitive categories). RULING: shelve channel-5
(secret/constant-time) and all crypto/checksum/interposition artifacts for
now. Retarget the headline on PERFORMANCE + CORRECTNESS + AI-AUTHORSHIP, not
on security-bug-class framing. Framing rules going forward: lead with speed
and "the compiler proves the code correct by construction; the AI wrote it";
verify with fuzz-differential testing against the incumbent; AVOID the
vocabulary that trips filters. The differentiators that are NOT
security-flavored still stand: guaranteed threaded/musttail interpreter
dispatch (channel 4), checked-law reassociation / safe parallel reductions
(channel 3), reproducible-to-the-byte builds, AI-authored + machine-checkable
trust. Fusion strategy (D7b) survives with a non-security Pile-B property:
put channel 3/4 (a performance guarantee) onto a felt compute artifact.

Re-aimed ladder (all avoid the filter): (1) NOW — fannkuch-redux (pure index
compute, validates the compute path, zero new features); (2) NEAR — QOI image
codec (byte format conversion, graphics audience, bit-identical verifiable,
needs const arrays or runtime table); (3) NEAR/MID — a data-pipeline compute
tool people feel (byte counting / column cut / format conversion class),
gated on the thin I/O frame + growable buffers. Headline shape: "AI-written,
compiler-proven-safe, faster than the C tool, produces byte-identical output."
Durable-log note: decision-gates.md + user-directives.md are the recovery
record; replay from last git commit after any rewind.

## D9a (2026-07-12, owner re-ruling): Default-path xlang vs shipped Rust is the primary confidence gate

The primary question is no longer whether benchmark-specialized expert safe
Rust can match xlang. Rust is expressive enough that expert restructuring can
often reach the ceiling, while shipped libraries commonly choose maintainable
shapes. The product claim to test is whether xlang raises the default
performance floor for its intended AI writer.

Before any xlang generation, pre-register: (a) one unmodified existing Rust
library at an exact version/commit, with default features and its ordinary
release build; (b) one fixed low-tier writer, with an exact 5.6 Luna or Terra
model identifier and settings; and (c) the behavior contract, corpus,
correctness harness, repair budget, measurement protocol, and success band.
The scoring workload must be fresh: it cannot be one previously used to
develop or tune xlang's compiler, proof rules, or benchmark implementation.
“Default xlang” is the first artifact in one model trajectory to pass the
frozen correctness gate. The model may respond to compiler/checker diagnostics
and failing correctness cases, but receives no Rust implementation source,
benchmark numbers, profiler output, IR or assembly, performance hints, or human
source edits. Multiple candidates may not be sampled and ranked by speed.
Freeze the source hash and complete generation/repair trace at first green.

Benchmark that frozen source both with facts on and with facts off for causal
attribution. A primary win may come from proof elision, ownership/effect
information, or a language-taught or language-forced algorithm, layout, or
control-flow shape; fact attribution is informative, not the only admissible
source of value. Expert safe Rust and stronger xlang models may be run only
after the primary result is frozen, as ceiling or model-gradient evidence;
neither may rescore the primary shipped-Rust comparison.

This is a narrow override of D5: the broad model-tier sprint remains
deprioritized, while one fixed low-tier run is required to measure the
production default path. The completed leg-A pilot remains context for
generalization, not the primary score. A first win is followed by a
pre-registered second shipped-library replication before a broad distributional
claim.

## D9a model selection amendment (2026-07-12, owner-selected)

For the first `percent_decode` trajectory, use exact model slug
`gpt-5.6-terra`, not Luna, at medium reasoning.  The one-trajectory,
first-correctness-green, benchmark-blind rules above are unchanged.

## D10 (2026-07-13): All repository content is English-only

Every repository-resident artifact must use English: source and test comments,
documentation, reports, plans, prompts, datasets authored for this project,
natural-language labels, and filenames.  Do not add translated duplicates or
language-suffixed report variants.  Mathematical notation, programming-language
tokens, and proper names are not alternate prose languages, but all surrounding
explanation remains English.  Repository-wide language scans are part of the
validation for documentation and report changes.
