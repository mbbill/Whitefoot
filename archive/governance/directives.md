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
rejected; Whitefoot has no such constraint (D0a: AI writes, human approves). So
Whitefoot may make radical restrictions: force a curated SUBSET of design patterns
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
headline is already taken (uutils -> Ubuntu), so Whitefoot's differentiator is the
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
already the Whitefoot op table), AI-written, RFC-8439-vector-verified, with
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

## D9a (2026-07-12, owner re-ruling): Default-path Whitefoot vs shipped Rust is the primary confidence gate

The primary question is no longer whether benchmark-specialized expert safe
Rust can match Whitefoot. Rust is expressive enough that expert restructuring can
often reach the ceiling, while shipped libraries commonly choose maintainable
shapes. The product claim to test is whether Whitefoot raises the default
performance floor for its intended AI writer.

Before any Whitefoot generation, pre-register: (a) one unmodified existing Rust
library at an exact version/commit, with default features and its ordinary
release build; (b) one fixed low-tier writer, with an exact 5.6 Luna or Terra
model identifier and settings; and (c) the behavior contract, corpus,
correctness harness, repair budget, measurement protocol, and success band.
The scoring workload must be fresh: it cannot be one previously used to
develop or tune Whitefoot's compiler, proof rules, or benchmark implementation.
“Default Whitefoot” is the first artifact in one model trajectory to pass the
frozen correctness gate. The model may respond to compiler/checker diagnostics
and failing correctness cases, but receives no Rust implementation source,
benchmark numbers, profiler output, IR or assembly, performance hints, or human
source edits. Multiple candidates may not be sampled and ranked by speed.
Freeze the source hash and complete generation/repair trace at first green.

Benchmark that frozen source both with facts on and with facts off for causal
attribution. A primary win may come from proof elision, ownership/effect
information, or a language-taught or language-forced algorithm, layout, or
control-flow shape; fact attribution is informative, not the only admissible
source of value. Expert safe Rust and stronger Whitefoot models may be run only
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

## D11 (2026-07-13): General-purpose data-structure capability is an eventual target; closure is staged by family

Whitefoot is intended to become a general-purpose systems language. For data
structures, representative standard collection contracts must have one taught,
efficient route, and ordinary no-unsafe libraries must be able to implement
representative held-out structures through public checked mechanisms without
asymptotic regression, unavoidable pathological storage, or
standard-library-only raw privilege. This does not mean that every named
structure is a kernel or standard-library type, that one representation serves
all contracts, or that every unforeseen structure is guaranteed globally
fastest. R3 selects one canonical writer-facing mechanism per frozen
caller-observable semantic and performance contract. Append-only and recyclable
stable identity are separate contracts; any substrate sharing remains
experimental, and the protected append-only path pays no generation or
recycling tax.

Closure is staged. The immediate step is a bounded G0-Core, timeboxed to one
owner-review cycle. G0-Core freezes only the full capability registry and
B/M/W/H/O roles; coarse observable contracts and asymptotic requirements;
forbidden pathological simulations; global ownership, drop, failure,
invalidation, and fact-channel rules; R3 contract dimensions; protected
baseline and no-tax rules; family order and reopening rules; held-out witness
identities and dependency budgets; and the schemas for later benchmark,
soundness, and META-5 derivability records. It does not freeze every family's
reference algorithms, workload matrix, numeric thresholds, soundness corpus, or
complete derivability ledger. Scope that does not fit is deferred to the
relevant family lock rather than expanding G0-Core.

Before candidate implementation for a family, that family receives its own Lock
A freezing exact operation semantics; ownership, drop, allocation-failure, and
invalidation behavior; candidate and reference algorithms; payloads, traces,
targets, and allocator; endpoints, thresholds, and selection rule; soundness
corpus; and META-5 derivability ledger. A candidate claiming a cross-family
substrate must first freeze every implicated family contract. Dense affine
sequence is first because it has the broadest dependency surface. The existing
E0.1 pause may be lifted only after G0-Core and the dense-family Lock A close;
lifting the pause does not automatically restart the previous fixed-record
paired protocol, which may instead be superseded by the dense result.

A family that passes its complete gate may seek separate owner approval for
production adoption without waiting for unrelated families. The project may
claim the complete general-purpose data-structure floor only after all B
baselines remain non-regressed and every M, W, and H obligation is closed;
optional O obligations block only if promoted or required by a mandatory
contract. This ruling authorizes correction of the research boundary only. It
does not authorize G0-Core work, a family lock, candidate or production
implementation, specification changes, wfc migration, scored runs, or default
teaching; the next step remains a separate owner discussion.

## D12 (2026-07-14): Complete the minimal systems-capability research before selecting mechanisms

Whitefoot's target is a general-purpose systems language. It may have no standard
library, and it need not reproduce Rust types or APIs one for one, but ordinary
checked libraries must be able to express the capabilities needed by everyday
systems programs with competitive asymptotic and structural performance. Named
containers may be efficiently derived from one canonical substrate when their
observable contracts permit it.

Use the stable Rust `core`, `alloc`, and `std` surface as a finite external
completeness anchor. Extract caller-observable contracts rather than copying
Rust mechanisms: results and order, ownership and invalidation, failure and
destruction, complexity, contiguity and stable identity or address, iteration,
resource ceilings, behavior parameters, concurrency, and platform boundaries.
Rust is not a design oracle, and its traits, unsafe internals, destructors, and
representations receive no default presumption. Stable unsafe and nightly APIs
are implementation evidence, not acceptable Whitefoot surface authority.

Derive a Pareto-small checked capability basis below the named-container layer.
Do not optimize primitive count in isolation: normative rules, checker and TCB
state, trusted facts and paths, writer spellings, runtime metadata and branches,
code size, and tax on protected baselines are all costs. A capability counts
only when an ordinary no-unsafe library can derive its assigned contract with a
complete ownership/destruction argument and without an asymptotic regression,
pathological storage or element traffic, hidden Copy/Clone requirement,
container-specific compiler recognition, or standard-library-only raw
privilege.

The Rust anchor is necessary but not sufficient. Preserve visible
cross-ecosystem topology witnesses and training-excluded held-out structures to
test ordinary-library generativity. Account for the entire systems-library
envelope, while keeping the first detailed closure named honestly as the
sequential, unique-owner data-structure floor; concurrency, resources and FFI,
custom allocation, async cancellation, pin/address-sensitive values, shared
ownership, complete text semantics, and target intrinsics remain separate
blocked claims until their own families close.

The owner now authorizes completion of this research program and the bounded
G0-Core artifacts. This authorization permits source inspection, reproducible
census tooling, contract normalization, derivation and cost ledgers, proof
sketches, cross-ecosystem review, E0.1 traceability, and hostile review. It does
not authorize a Family Lock A, candidate or production mechanism, language or
specification change, compiler implementation, wfc migration, scored candidate
run, E0.1 restart, or default teaching. The next production-relevant action
after the research remains an owner review.

## D13 (2026-07-14): Draft the dense unique-owner Family Lock A

The owner authorizes drafting a complete dense unique-owner Family Lock A for
review. The draft may expand the exact G0 audit domain, refine exact member and
outcome contracts, describe a finite candidate and reference-mechanism set,
freeze soundness fixtures and generators, preregister structural and measured
performance protocols, instantiate protected no-tax controls, record META-5
and E0.1 dispositions, build fail-closed lock validators, and receive hostile
review. These are research and protocol artifacts; they may not contain or
select a production implementation.

This authorization does not permit candidate construction, Candidate Freeze
B, scored or held-out execution, a language or specification decision,
compiler implementation, production adoption, E0.1 restart, wfc migration, or
default teaching. Completing and reviewing the draft returns it to the owner.
Any permission to construct candidates or make a language change requires a
separate explicit decision after review of the frozen lock.

## D14 (2026-07-15): Select the privileged capability basis before container mechanisms

The prior capability research was operating one layer too high when it treated
dense-container mechanisms as the next design choice. Preserve G0-Core as the
finite completeness ledger and preserve the dense Family Lock A as detailed
obligation and adversarial evidence, but do not select or construct a dense
candidate until the common privileged foundation has been researched and
reviewed. OD-0 through OD-5 are therefore not the current owner action.

Research one elegant, minimal, nonforgeable privileged admission route and the
smallest public checked capability basis that it can establish. Distinguish one
gate from one semantic operation: allocation, atomics, OS calls, target
operations, and typed storage transitions may be irreducible actions even when
they share one admission mechanism and proof ledger. Do not minimize by hiding
independent effects, facts, cleanup obligations, or TCB branches behind one
name.

Whitefoot need not ship a standard library. Ordinary unprivileged libraries must be
able to implement the required container and systems contracts through the
public checked basis without a privileged named container, writer-visible
unsafe, a hidden Copy/Clone/Default requirement, pathological simulation,
asymptotic regression, or unavoidable runtime tax on weaker protected shapes.
The prior registries and held-out witnesses are tests of this basis, not a list
of language primitives to add one by one.

This ruling authorizes literature and production-language research, proof and
derivability arguments, cost analysis, and hostile review. It does not
authorize a language or specification change, compiler implementation,
standard-library or container implementation, candidate construction or
execution, E0.1 restart, wfc migration, production fact channel, or default
teaching. Any production design or implementation requires a later explicit
owner decision.

### D14 scope clarification (2026-07-15)

For D14, "nonforgeable privileged admission route" means one static compiler,
runtime, or official-core privilege-definition boundary that ordinary source
cannot enter. The active research must first study how production languages
reserve semantic-definition authority to their toolchains, then recommend one
such mechanism for Whitefoot, and only afterward derive the minimum safe public
capability basis ordinary checked libraries require.

This clarification excludes cryptographic authorization and independently
distributed privileged extensions. F/C/S, signed grants, authorization
releases, successor snapshots, replay, revocation, key rotation, and identity
graphs are historical research for a different problem, not pending D14 design
choices. It authorizes no production change, implementation, candidate, or
experiment beyond D14's existing research-only envelope.

### D14 performance-first correction (2026-07-15)

The preceding mechanism-first clarification used the wrong objective ordering.
The project is not trying to draw a user/system authority line as its primary
result. The primary problem is performance: current language expressiveness can
force initialization, zeroing, copying, relocation, tags, metadata,
allocations, indirection, dynamic dispatch, retained checks, extra machine
events, or an unavailable native representation. Safety, no writer-accessible
`unsafe`, proof-only check elision, checker feasibility, specification
regularity, AI use, and ordinary-library systems coverage remain binding design
constraints.

Research must first freeze the finite performance-relevant expressiveness gaps,
then derive their common semantics, and then compare at least three materially
different complete capability sets. Each set must state its exact members, why
they are needed, how they remove the recorded performance barriers, what
ordinary checked libraries derive, its checker/compiler/backend/runtime shape,
its safety and cleanup rules, costs on weaker shapes, open problems, and
falsifiers. The final comparison must explain each candidate's pros, cons, and
trade-offs before recommending one, recommending an exact combination, or
finding that current evidence selects no winner.

Keywords, language rules, type states, ownership/checker mechanisms, checked
proof systems, ordinary libraries, builtins, and exact runtime or target leaves
are all admissible candidate components. Ordinary safe writer access is not a
defect. Isolation or a sealed registry is only a conditional implementation
constraint for a surviving capability that cannot be expressed as an ordinary
safe language or checker rule. The prior static-gate and P1-P9/Q1-Q6 packet is
historical candidate evidence, not an owner-selected direction or a starting
assumption.

The existing research-only authorization and all production prohibitions remain
unchanged. Exact D-2 and P-1 stay fail-closed; category routing and paper cost
arguments do not close exact derivability, formal safety, or measured
performance.

### D14 Candidate C bounded-validation ruling (2026-07-15)

The owner accepts Candidate C as the first bounded validation hypothesis.
Candidate B is retained as a later compression challenge over evidenced
families, and Candidate A as the generality fallback if a finite family set
cannot express a required efficient mechanism. These research roles select no
production design.

The controlling operational contract is
`optimizer-language-research/implementation/minimal-systems-capability/
CANDIDATE-C-BOUNDED-VALIDATION-PLAN.md`. The current authorization covers only
making that plan durable, Stage 0's frozen Candidate C v0 audit baseline, and
Stage 1's five-operation Hashbrown paper calibration. Work must stop at Gate 1.
Every allocator, SQLite, Crossbeam, Tokio, Wasmtime, safety-model, prototype,
machine-event, benchmark, AI, language, specification, checker, compiler,
runtime, standard-library, and production step requires separate owner
authorization. Time-bounded uncertainty remains `UNKNOWN`; it does not permit
scope expansion or a pass claim.

### D14 Candidate C sparse-repair ruling (2026-07-15)

After Gate 1 returned `C-REVISE`, the owner selected route 1: design and compare
paper repairs for the six already identified sparse-definition gaps before any
allocator audit. The controlling contract is
`optimizer-language-research/implementation/minimal-systems-capability/
CANDIDATE-C-SPARSE-REPAIR-PLAN.md`.

The authorization covers exactly three frozen repair alternatives, the two
shared exact C0 row proposals, a fifteen-row comparison over the five existing
Hashbrown operations, hostile paper review, and a mandatory stop at the Sparse
Repair Gate. It does not authorize applying a repair, changing Candidate C v0,
admitting a family, widening source inspection, entering Stage 2, auditing
another project, formal safety work, implementation, execution, generated-code
inspection, benchmarking, measurement, AI trials, or production changes.

### D14 Candidate B cross-project design ruling (2026-07-15)

After comparing Candidate B's stated liabilities with Candidate C's extension
pressure, the owner authorized applying the same bounded comparative method to
Candidate B and explicitly rejected a Hashbrown-only evidence set. The active
question is whether a small project-independent closed algebra can cover
materially different high-performance structures more elegantly than a family
catalog while remaining distinct from arbitrary proof authority.

The controlling contract is
`optimizer-language-research/implementation/minimal-systems-capability/
CANDIDATE-B-ELEGANT-DESIGN-PLAN.md`. It freezes exactly fourteen operations over
Hashbrown, mimalloc, SQLite, and Crossbeam Epoch; exactly three alternatives;
exactly 42 candidate-operation rows; a readable capability, removal-witness,
performance-role, ordinary-library derivation, residual-risk, and pros/cons
report; and a mandatory Candidate B Design Gate. A proposed B rule must serve
at least two independent projects rather than one operation or project.

The authorization covers only pinned primary-source inspection, paper design,
comparison, hostile review, deterministic verification, and status durability.
It does not authorize another project or operation, a formal safety model,
prototype, candidate construction or execution, generated-code inspection,
benchmarking, measurement, AI trials, language/specification/checker/compiler/
runtime changes, standard-library or container work, or a production choice.

### D14 B-Strata decisive ruling (2026-07-15)

After the `B-REVISE` gate, the owner selected B-Strata as the sole architecture
to pursue. Do not pivot to Candidate C and do not develop B-Graphs as a
competing design. The required outcome is forced: make B-Strata coherent and
show that it works under the standing performance and safety constraints, or
return a concrete explanation of the irreducible reason it cannot work. An
open-ended recommendation to continue research is not a final result. The
owner prioritizes reaching a landable answer soon over expanding the candidate
or project search.

The controlling plan is `optimizer-language-research/implementation/minimal-
systems-capability/CANDIDATE-B-STRATA-DECISIVE-PLAN.md`. It retains the exact
four projects and fourteen complete operations, treats the existing eight
strata as analytical jobs rather than preselected primitives, and requires one
normalized minimal core, exact full-route derivations, hostile safety and
erasure analysis, implementable deterministic checking and lowering, and
bounded cross-project structural and performance evidence. The final verdict
is exactly `STRATA-YES` or `STRATA-NO`.

This ruling authorizes the plan, semantic normalization, exact-route work, a
general operational safety model and preservation/resource/race/release
arguments supported—but not replaced—by bounded executable counterexample
search, hostile review, and—only after the paper and model gates pass—the
smallest preregistered prototypes, generated-code inspection, and measurements
needed to decide the B-Strata claims. It
does not authorize another candidate, another project or operation, arbitrary
writer predicates or proofs, project/container/reclamation-policy language
forms, hidden runtime state, weakened checks, or production specification,
checker, compiler, runtime, standard-library, container, wfc, migration,
default-teaching, or shipping changes. A final `STRATA-YES` returns an exact
production landing proposal for owner review; it does not silently ship the
design.

### D14 B-Strata demand-substitution amendment (2026-07-15)

The owner clarified that the minimum capability set does not need to reproduce
every data structure used by every audited project. The real target is the
systems-performance demand. A different data structure, reclamation strategy,
or algorithm is acceptable when it preserves the relevant consumer contract
and safety requirements and provides comparable or better efficiency. For
example, a Crossbeam demand may be met by container B even when the reference
uses container A, provided B serves the same demand without a material
performance loss.

The four projects and fourteen audited operations therefore remain fixed as
demand cases, reference baselines, and stress evidence, not as mandatory final
implementations. Exact-route failure alone is not a B-Strata failure. The
research must use substitutes to shrink the capability set when possible and
must judge the final minimum by the rules required across selected successful
demand routes, not by the rules required to mimic every source topology. This
amends the earlier phrase "exact full-route derivations"; it does not reopen
Candidate C, add a project or demand, weaken safety, or authorize production
changes.

## D15 (2026-07-16): Fresh autonomous derivation of the systems-performance capability set

The owner reviewed the capability track's history and redirected the research.
The prior convergence-failure analysis was accepted, but the owner instructed
that the new research must not be anchored to that analysis or to any prior
candidate framing. The research restarts as a fresh, fully autonomous
derivation.

The objective, in the owner's terms:

- Whitefoot must be usable for general systems programming with top-tier
  performance.
- The project does not need to provide the ability to rewrite existing
  software line by line. It must provide, for most systems-programming
  scenarios, at least one blessed way of writing whose performance reaches or
  exceeds the best existing implementations.
- Calibration example given by the owner: if one container, forced everywhere,
  were optimal in every scenario, it would be a correct solution. Since that
  is impossible, the language provides n forms, and n must stay small, because
  the second binding requirement is that the specification remain compact
  enough to fit in a model context window. The goal is an elegant, concise
  method for expressing most systems-programming demands at high performance.
- All original constraints are unchanged, with performance first: the safety
  theorems, no writer-accessible unsafe, proof-only check elision, checker
  feasibility, and AI writability remain binding.

Recorded consequences for the active research:

- The owner's container example readmits compiler-known forms with disciplined
  trusted internals into the answer space. The earlier counting rule that
  excluded container-specific compiler recognition no longer binds this
  research. Checked-source forms remain preferable to trusted forms all else
  equal, because they stay inside the safety proof.
- The D14 B-Strata-only lock no longer constrains the active derivation. The
  B-Strata plan, core, and all earlier candidate artifacts remain historical
  evidence and falsifiers; the fresh derivation may converge to, reuse, or
  discard them on merit.
- Coverage is judged at the scenario level ("most systems-programming
  scenarios") against best-existing-implementation performance, not at the
  level of the fourteen frozen reference operations, which remain stress
  evidence and workload baselines.

This ruling authorizes scenario-demand mapping, independent design derivation,
hostile review, cross-design comparison, and durable recording of the results.
Production language, specification, checker, compiler, runtime,
standard-library, container, wfc, migration, fact-channel, teaching, and
shipping changes remain separately gated on later explicit owner decisions.

## D16 (2026-07-16): Copy declaration, catalog minimality, acceptance ledger, delegated execution

Four owner rulings from the same session, recorded verbatim in intent:

1. **Records copy semantics (decision 5): explicit declaration selected.** A
   record type is copyable only when declared so (`copy struct` spelling,
   R3-provisional); the compiler verifies every field is copyable. Automatic
   structural Copy and nominally-affine-only storage are rejected. This
   settles the E0.1 three-way choice at the ruling level.

2. **Catalog minimality criterion (decision 6, with owner amendment).** The
   built-in component set must be lean. Do not mirror the Rust standard
   library. A sealed built-in is admitted ONLY when ordinary users cannot
   implement the capability themselves at par performance from the language
   primitives and checked libraries. Everything user-implementable ships as
   taught checked source or composition cards instead of new forms. The
   owner's words: provide the most basic and critical methods; let users
   implement the rest; only include what users cannot implement.

3. **Built-in acceptance ledger.** Every sealed component must prove
   performance AND safety AND reliability before shipping: the five-part
   battery (differential testing against the reference implementation,
   exhaustive small-bound model checking, sanitizer/fuzz soak, hostile
   pre-ship review, CI-pinned performance and assembly shape), plus complete
   failure semantics (programmer error traps; environmental failure returns
   Result; no unspecified behavior), resource ceilings, and teardown
   protocols with fault-injection evidence. A component without a complete
   green ledger cannot ship.

4. **Delegated execution.** The owner authorized continuing the program
   autonomously ("start working; make appropriate decisions yourself; do not
   stop until done"). Research-scope prototypes, measurements, checker
   experiments, and writability trials are authorized within the research
   track. Production language, specification, checker, compiler, runtime,
   and teaching changes remain separately gated. Under the delegation, three
   recommendations are adopted provisionally pending explicit morning
   ratification: pool generational slot recycling (supersedes the P2
   never-recycle posture; stale handles trap deterministically), trap =
   process abort for the concurrency layer, and the named v1 non-goals
   (concurrent ordered map, writable shared-memory IPC, inbound FFI
   callbacks, user-authored novel lock-free structures).

## D17 (2026-07-16): Proof-gated admission to the privileged tier

The owner ruled on the sealed-tier endgame during review of the built-in
architecture:

1. The verification track (machine-checking container-style representation
   invariants for specific implementations — "phase 3") is a committed
   long-term goal, not merely a research option: any sealed kernel whose
   implementation is machine-proved leaves the trusted list and becomes
   checked code with privileged representation rights.
2. The same lane is, in the long term, open to USERS: performance never
   requires proof (the default path is composing the sealed catalog), but
   any user implementation that machine-proves its invariants may be
   admitted to the privileged tier and enjoy the same representation rights
   (uninitialized storage, elided checks). Proof substitutes for trust;
   the trusted list only ever holds project kernels that have not yet been
   proved. The owner's words: "performance wise we don't require proof of
   everything, but if anything is proved then it can move into the
   privileged area and enjoy all the benefits."

Recorded consequences:

- The v1 non-goal "user-authored novel lock-free structures" acquires its
  principled long-term resolution: the proof-gated lane, alongside the
  existing catalog-escalation lane. It remains a non-goal for v1.
- Soundness constraints on the lane: proofs are machine-checked
  deterministically (proof-carrying style — the writer constructs the
  proof, the checker verifies it without search, preserving the
  frontend-simple checker law); the invariant language is versioned and
  each extension passes hostile soundness review; admission never rests on
  human review of user code.
- Difficulty ladder acknowledged: sequential representation invariants
  (tables, sequences, pools) are the realistic near-term candidates;
  concurrent lock-free protocols remain research-grade per artifact and
  stay in the sealed tier until the proof technology matures.
- This is the constitutional principle "speed is earned by proof, never by
  weakening a check" applied to representation rights; the existing checked
  requires-prologue fact system (PROOF-1/2) is the same doctrine at the
  bounds-check scale.

No production change is authorized by this ruling; it fixes the direction of
the verification track and the extensibility story.

## D18 (2026-07-16): Five rulings on the parked decision list

1. **Pool slot reuse: APPROVED** — generational handles supersede the
   never-recycle posture; stale handles trap deterministically. Rider: the
   owner asks for a bounded look at alternatives that avoid the per-access
   version-check cost where possible ("if we can avoid the performance
   penalty that's better") — candidate directions recorded: session loans
   (a shared loan freezing free/reuse makes in-session derefs check-free via
   the ratified loan machinery), affine owned-handle mode (staleness
   impossible by construction), and fact-based elision of repeated checks on
   one handle (requires-prologue style). Research-scope only.
2. **Trap scope: option A ratified** — a runtime safety trap terminates the
   whole process; availability is a supervision concern outside the process;
   expected failures remain Result values.
3. **v1 non-goals: all four ratified** — concurrent ordered map, writable
   shared-memory IPC, inbound FFI callbacks, user-authored novel lock-free
   structures; each keeps its written re-entry trigger; the fourth carries
   the D17 proof-lane as its long-term resolution.
4. **Loan-rule amendments: all ratified** — the four repair amendments
   (statement-local mint disjointness; mint mode-capability; explicit
   issues-on-source tie; R12 wording reconciliation) plus the optional
   declaration-time tightening. The consolidated 15-rule text becomes the
   normative research draft.
5. **Spec budget: option 2 ratified** — eight load-bearing pattern cards
   remain in the always-loaded manual with full worked examples; the
   remaining seven ship as taught, non-normative example files; light trim
   of the operation long-tail; target ~48k tokens total manual. Card
   selection is provisional pending round-4 writability data.

No production change authorized; these rulings bind the research drafts and
the validation protocol.

## D19 (2026-07-17): The five concurrency-delta decisions

The owner ruled on the five decisions surfaced by the concurrency kernel delta
(dossier `systems-performance-coverage/DESIGN-DOSSIER.md` §5):

1. **Land the concurrency memory model (CONC-0) as kernel text — YES.** The
   MM-0..MM-10 happens-before model becomes the reduction target that makes D1
   (data-race impossibility) checkable rather than asserted. Production landing
   remains gated on the separate landing review; this ratifies the direction.
2. **Loop-spawn fan-out — OPTION (a) for v1, with (b) scheduled as a tracked
   follow-up.** v1 ships fixed straight-line spawn plus the sealed
   `par.for_chunks` combinator (which already gives runtime-N chunk parallelism
   internally). The OWN-11 capture carve-out that would allow user-written
   dynamic spawn loops capturing shared outer state is deferred and MUST be
   tracked so it is not lost (owner's explicit instruction); the re-entry
   trigger now lives in root `THE-PLAN.md`.
3. **Ratify AMD-5-carve-out / AMD-7 / AMD-8 — YES, all three.** The minimal
   amendments that make the concurrency mechanisms sound (par-slot capability
   premise; R15 concurrency schema; mutex guard interior-view carve-out).
4. **Clone-row re-mode to `&uniq` — YES (option a).** `cq_tx_clone`/`cq_rx_clone`
   take exclusive receivers, making a concurrent clone unspellable by
   construction (closes the race without adding a trusted proof obligation). The
   ergonomic cost is nil: endpoints are cloned during setup while owned, never
   on the hot path. The re-mode was already applied in the CONC v3 draft; this
   ratifies it.
5. **Concurrency spec-mass cut — YES (normative-only).** Keep rule text and
   operation tables in the always-loaded manual; move rationale, attack
   walkthroughs, and review history to the separate review record; then count
   tokens against the ~48k target. Governing principle stated by the owner:
   the always-loaded spec gives a new AI agent exactly what it needs to write
   correct and efficient code, and nothing more — rationale is excluded unless
   its absence would cause wrong or slow code.

These rulings settle the concurrency delta's design at the research level. No
production language, specification, checker, compiler, runtime, or teaching
change is authorized; the CONC-0 kernel landing, the amendment ratifications,
and all production spec drafting remain gated on the separate landing review.

## D20 (2026-07-17): Single execution plan and authorization through `seq`

The owner approved consolidation of the project's current plans and authorized
the resulting development sequence through phase 7:

1. **One current plan.** Root `THE-PLAN.md` is the sole source for current
   status, execution order, phase gates, stop conditions, and next work. Other
   plan and handover files move to the archive as historical evidence. Design
   dossiers, research registers, owner directives, and the decision log retain
   technical evidence and rulings but do not define a competing work queue.
2. **Authorization boundary.** Phases 1 through 7 in `THE-PLAN.md` are
   authorized in their recorded order, ending when `seq` passes its complete
   acceptance ledger. This authorization covers the production specification,
   checker, compiler, runtime, conformance, testing, and review changes those
   phases require. Each phase still obeys its entry gate, stop conditions,
   hostile-review requirements, and both repository checks. This ruling
   supersedes the no-production authorization boundaries in D16 through D19
   only for those seven phases; their technical decisions remain binding.
3. **Stage-0 role.** `prototype/democ` bootstraps wfc through the facts-off
   byte-identical self-hosting fixpoint. The project freezes democ at that gate
   and keeps it as an independent differential oracle. Post-fixpoint
   accepted-language growth belongs in wfc, with conformance artifacts and
   purpose-built reference checkers providing independent evidence. The project
   will not grow democ as a second production compiler.
4. **Work beyond `seq`.** Later sealed components and concurrency remain outside
   this authorization. In particular, the concurrency memory model, sync
   effect, sharing rules, runtime forms, and per-form safety models require a
   new owner directive after phase 7 closes. Phase 6 therefore lands the
   sequential projection of the loan/freeze judgment: R1 through R13 and the
   sequential part of R15. R14 and R15's concurrent-invocation clause stay with
   the later concurrency landing, along with their nine parallel research
   cases.

## D21 (2026-07-20): Production architecture over coverage-counter clearing

The owner stopped the Phase 2 exact-profile sequence after reviewing the
guarded-span slice and approved the following correction after hostile review:

1. **The objective is the production compiler.** The exact compiler unit must
   ultimately be certified for self-hosting, but its `Unsupported` census is an
   observation, not a work selector, pass criterion, product, or optimization
   target. Work that improves the counter while making the compiler more
   source-shaped, larger, or less general is a regression, not progress.
2. **Function, family, and whole-subtree admission or emission dispatch are
   rejected.** Production semantics use one syntax-directed, spec-derived
   checker applied to every function, followed by atomic whole-unit acceptance
   and one unit-bound elaborated artifact. Lowering is one generic consumer of
   that artifact. A capability family may describe a delivery slice or test
   group only; it does not survive as production admission or emission
   architecture. No conjunction
   of exact signature, resolved nominal identity, literal, statement order,
   callee, loop, source name, ordinal, or body properties may act as a source or
   profile fingerprint or select an alternative admission or lowering path. A
   whole-body profile or preselected source-shaped subtree validator may not
   produce an admission verdict, authoritative typed transition, or emission
   selection. Syntax-directed handlers may recurse compositionally over
   arbitrary resolved children and state where the cited rule requires it,
   including block flow, exhaustive match, and GRAM-7/GIVE-1 judgments; they
   return only that local rule's typed or flow fact, never a source-profile
   status. Each exact property remains checked compositionally wherever a
   numbered language rule requires it.
3. **Checked acceptance and optimizer proof are separate.** During facts-off
   bootstrap, legal checked operations retain their traps. Source-specific
   proofs of bounds, overflow freedom, termination, or literal properties must
   not become prerequisites for compiling those operations. Proof-based check
   removal remains a later, separately reviewed fact-channel concern.
4. **Stop, plan, then clean up.** Further compiler implementation is frozen
   until `THE-PLAN.md` records explicit prohibited routes, a general semantic
   architecture gate, a read-only debt inventory, and a cleanup-first recovery
   sequence. The next uncommitted exact probe must not be merged. Existing work
   is not blindly reverted: reusable general machinery is separated from
   source-specific authority, and the owner reviews the inventory and first
   cleanup proposal before compiler implementation, compiler tests, or compiler
   source-unit files change.

This ruling refines the authorized Phase 2 implementation method without
changing the language specification, the phases-through-`seq`
authorization, or any protected semantic expectation. Approval of this packet
does not authorize compiler cleanup: the separate implementation entrance gate
in `THE-PLAN.md` still requires a read-only debt inventory, complete architecture
packet, preregistered first tranche, hostile review, and explicit written owner
approval.

## D22 (2026-07-20): One safe-Rust production compiler for exact v0.8

The owner replaced the self-host-first ladder and its wfc recovery path with the
following direction:

1. **Exact target.** The first production compiler implements the exact
   normative content of `spec/kernel-spec-v0.8.md`, SHA-256
   `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.
   Its provisional and deferred markings do not silently change. Every later
   specification delta remains separately owner-gated and creates a new
   immutable numbered version.
2. **One permanent implementation.** The production compiler starts fresh in
   safe Rust and proceeds directly through the permanent checked-artifact
   architecture. There is no disposable Rust compiler and no parallel compiler
   ladder. Self-hosting is a later product decision, not a prerequisite.
3. **Retired predecessors.** wfc and democ move intact to an inert historical
   archive. No active build, source import, semantic decision, lowering path, or
   release claim may depend on them. General defects may be re-derived as
   additive tests against permanent interfaces; retired implementation code is
   not transplanted.
4. **Durable authority.** The exact specification, compiler-independent
   conformance corpus, focused independent models, facet catalog, checked-unit
   contract, independent verifier, generic facts-off lowering, and later
   separately verified fact overlays form the durable implementation program.
5. **Authorization.** Phases 1 through 5 of the rewritten `THE-PLAN.md` are
   authorized in order through the exact-v0.8 production baseline. Phase 6
   research and additive dogfood evidence are authorized, but cannot alter the
   specification. Phase 7 qualification and any self-hosting work require a new
   explicit owner decision.

The owner explicitly authorized retiring `prototype/democ/test_codegen.py`
with democ. Its bytes remain preserved under
`archive/toolchains/self-hosting-2026-07-20/democ/test_codegen.py`. This
authorization changes no kernel-specification byte, conformance verdict, frozen
oracle digest, or active reference-checker expectation.

## D23 (2026-07-21): Four narrow compiler-architecture rulings

After three hostile scope reviews, the owner approved items 1 through 4 in the
compiler-architecture dossier exactly as scoped:

1. **Syntax construction direction.** Use one typed postorder derivation tree,
   one linear topology finalizer, no copied second syntax tree, and a
   tree-driven audit that reconstructs expected source bytes and compares them
   with the input without normalizing or rewriting it. This selects a design
   direction only; it does not authorize implementation or any blocked parser,
   node, location, compilation-unit, formatting, identity, schema, or
   `CanonicalSyntaxUnit` contract.
2. **Future semantic trust topology.** Select as the proposed future replacement
   for D22's independent production semantic certificate verifier one trusted
   semantic kernel plus mandatory complete artifact-only same-kernel replay
   before an originating invocation may construct lowering authority. Later,
   cached, third-party, and hand-authored bytes can never construct lowering
   authority. Replay is shared trusted authority, not independent evidence;
   independent grammar, source/tree, conformance, model, target, backend,
   guard, publication, and optional-fact evidence remain mandatory. D22 and
   `THE-PLAN.md` remain current until a separate roadmap redecision. No concrete
   replay type, record, order, schema, API, profile, crate, migration, or
   implementation is approved.
3. **Possible successor-specification evidence plan.** Prepare the reviewed
   grammar-verifier design, discrepancy evidence, protected-surface impact
   census, and exact non-authoritative candidate bytes. Exact v0.8 remains the
   active target. Verifier implementation, normative bytes, parser work,
   protected-expectation changes, oracle-digest changes, a successor version,
   and an active-target change remain separately gated. The leading evidence
   candidate subtracts the complete mechanically extracted set of 47 fixed
   lowercase grammar terminals from the lowercase-word `IDENT` language; this
   ruling does not select normative wording or approve that compatibility
   restriction.
4. **Read-only foundation audit.** Bind an exact manifest of the current dirty
   workspace and inventory every crate, module, authority-bearing API,
   dependency/feature, reachable allocation path, and publication path. The
   audit may recommend a disposition but cannot edit, rename, refactor, delete,
   create a crate, change a wire contract or A-10, alter specification/protected
   content, make a capability or profile claim, revise roadmap/design-memory
   authority, or perform an implementation migration. Every later mutation
   requires separate review and approval.

These rulings change no numbered specification, protected expectation,
approved guard baseline, roadmap, design-tree node, or compiler implementation.

## D24 (2026-07-21): Closed-unit top-level function visibility

The owner selected A-01's language semantics:

- All top-level function signatures are visible throughout the closed
  compilation unit.
- Locals, regions, labels, and named constants remain declaration-before-use.

This resolves the semantic choice between TYPE-6 declaration-before-use and
FN-1/FN-6 mutual recursion. Inventorying a declaration still does not grant
visibility outside this rule. The decision must be encoded in the exact
version-bumped successor-specification proposal before resolver implementation.
It does not authorize editing v0.8 in place, creating the successor bytes,
changing a protected expectation or approved guard baseline, switching the
active target, or implementing the resolver.

## D25 (2026-07-21): Prepare the repository for architecture handoff

The owner stopped the section-by-section walkthrough and directed the
repository to be prepared as a clean handoff for implementation under the new
production-compiler architecture. This authorizes the following transition:

1. Adopt the compiler-architecture dossier as the design record, with every
   listed language, protected-surface, schema, profile, and entrance blocker
   still enforced.
2. Rewrite `THE-PLAN.md` as the sole implementation roadmap around one trusted
   semantic kernel, mandatory complete artifact-only same-kernel replay before
   originating-invocation lowering authority, and all assigned independent
   grammar, source/tree, conformance, model, target, backend, guard,
   publication, and optional-fact evidence lanes.
3. Reconcile the live design tree, repository instructions, READMEs, status
   text, and dependency/authority descriptions with that roadmap while
   preserving superseded designs and dated records.
4. Preserve the completed lexical-observer checkpoint separately, record the
   exact Decision 18 audit, and apply the audit-selected foundation preparation:
   truthful source-audit/lexer responsibilities and explicit fallible owned
   source/source-binding boundaries. No placeholder parser, semantic, artifact,
   backend, or publication schema is created.
5. Leave one exact first implementation tranche: the standalone grammar-change
   verifier and non-authoritative successor-specification evidence preparation.
   Parser implementation remains stopped until the approved successor bytes and
   complete grammar gate exist.

Exact v0.8 remains the active numbered specification until a separate exact
successor-specification approval and target switch. This directive does not
authorize an in-place specification edit, a protected expectation or oracle
digest change, baseline regeneration without approval, or implementation past
an unresolved entrance gate.

## D26 (2026-07-21): Install the exact v0.9 owner-review packet

The owner approved the exact nine-item Phase-3 packet committed at `7fbb018`:

1. Install the 98,044-byte reviewed candidate as the new immutable
   `spec/kernel-spec-v0.9.md`, SHA-256
   `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
2. Approve the exact three-path protected syntax-repair layer, SHA-256
   `724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479`.
3. Apply the combined 274-path FORM-2 migration patch, SHA-256
   `4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2`,
   exactly once.
4. Apply the post-FORM-2 case-intent patch, SHA-256
   `62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a`,
   second.
5. Apply the manifest-metadata patch, SHA-256
   `ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6`,
   third, yielding the exact 99,869-byte manifest with SHA-256
   `0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c`.
6. Append the exact v0.9 derivation-ledger amendment bound by SHA-256
   `f29b326f446aa9e5f512d079f1dbd14e641e6d840f18b69faab0ea39950e52a0`.
7. Create or regenerate v0.9-bound evidence and discrepancy assets and update
   exact live references while preserving all v0.8 historical material.
8. Switch the active compiler and roadmap target to the installed exact v0.9
   bytes only after the preceding installation work is complete.
9. Regenerate the protected-surface baseline and append the governance entry
   only through `make approve-spec REASON="..."` after the full worktree has
   been verified.

This approval changes no expected verdict, runnable status, frozen oracle
digest, or existing reference-semantics test. Their exact expectation/status
projection remains SHA-256
`5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132`.
It authorizes only the reviewed sequence and its consistent active-target
installation; later language deltas and protected changes remain separately
owner-gated.

## D27 (2026-07-21): Approve the exact Phase-5 successor proposal

The owner approved the exact Phase-5 successor proposal SHA-256
`7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d`
and its generated 118,314-byte v0.10 candidate SHA-256
`71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9`.
The approval covers the exact ten-item boundary in `PROPOSAL.md`: the numbered
language delta, three TYPEID domains, OP-1 reservation inventory, complete
role/scope matrices, diagnostic event and payload order, the schema-only R-04
invocation-wide resolver resource contract, architecture consequences C-01
through C-06, and the explicitly limited census and abstract evidence claims.

This approval selects no numerical hard maximum. It does not install v0.10,
change an active target or protected surface, regenerate the guarded baseline,
reproduce the canonical frontend against successor identity, or authorize
resolver implementation. Those remain the three separate stops recorded by
the approved packet.
