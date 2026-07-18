# why Whitefoot — document outline (round 2, for owner review)

Status: OUTLINE / PLAN, not the document itself. Round 2: every inline
comment from round 1 is resolved below (see the changelog), and every
evidence claim was re-verified against the committed experiment records.
Drop comments anywhere (a `>>` line, a `TODO`) and I will revise again.

Working title: **"Whitefoot: what's different, and why — for people who already
write Rust, C, and C++ (and for people who only prompt)."**

---

## Round-2 changelog

What changed and why, one line per round-1 comment:

1. *Goal line ("not just C-class")* — rewritten: the claim is now "everything
   C can do, plus optimizer facts C cannot hand the backend," with the
   honest scoping kept (hand-tuned SIMD still wins today; see Honesty).
2. *"The four points are weak for answering why-not-Rust"* — § 0 is rebuilt
   around your floor argument: Rust+unsafe-ban bounds *memory safety* but
   not *how the program may be written*; Whitefoot constrains the writable
   shape-space itself, which is what raises the floor. The old four points
   are demoted to supporting mechanisms.
3. *Safety's role* — § 1 rewritten to your framing: safety is one of the
   methods that holds the floor (remove the bad-code surface entirely,
   don't police it), and its machinery happens to be the optimizer's alias
   fact-base. Reasoning first, then data; flagged as the section needing
   the most careful logic review in draft.
4. *"Make the examples simple, and test them before use"* — adopted as
   framing decision #6 and a concrete build plan (end of file): small
   purpose-built kernels, compiled and verified in-repo before any line of
   them is quoted; the existing experiments stay as deep-evidence links.
5. *"Address checks-vs-speed immediately, as a Part II opener"* — done:
   new § 2 opens Part II with exactly your two-buffer example (`&uniq`
   arguments + one `requires` relation -> checked entry, check-free loop).
6. *§ 4 confusing digression (node links / self-referential structs)* —
   removed from the vectorization section; that material belongs with
   handles/pools and now lives only in § 10.
7. *"Don't title every part 'differences that…'"* — all parts retitled as
   plain technical-article headings.
8. *Missing points (one AST per program, fast compilation, …)* — new
   Part IV section on the one-program/one-tree/one-byte-form property and
   its toolchain payoffs (diff, merge, caching, reproducibility, build
   speed), with measured vs projected claims separated.
9. *(Round 2.1) "The spine is weak — the author shift changed a lot more
   than who writes the facts."* — The spine is rebuilt as the
   author-bargain / information-loss ladder, mined from the founding
   research corpus: dynamic languages recover facts by watching the
   program run, guarded and revocable (V8/PEP 659/HotSpot, J001-J005);
   Java makes everything a reference and escape analysis only
   conditionally recovers what the author knew (F7); C/C++ invert the
   aliasing burden and `restrict` is an unchecked footgun (F1-F3); Rust
   proves checked-source-facts work, then stops at its human bargain.
   The general law: optimization infrastructure is largely archaeology on
   discarded authoring-time information, and correctness-grade facts
   cannot be recovered by observation at all (static-vs-profile ruling).
   The author shift is what makes stopping the loss rational.

Factual corrections from re-verifying the previous session's draft against
the committed records (these were wrong or stale in round 1):

- **base64 vs expert Rust**: round 1 claimed expert Rust ties "only by
  silently abandoning the output-capacity contract." That described the
  *superseded* adversary run. The corrected 2026-07-11 rerun gives every
  Rust variant full RFC semantics and the same entry contract; safe
  `chunks_exact/zip` Rust then **ties Whitefoot at full semantics** (0.997,
  inside the ±2% band). The durable, measured delta is different and
  better: Rust's *obvious* indexed loop is 1.60x slower and adding
  `assert!` up front recovers **nothing** (measured 1.604x — LLVM cannot
  connect the assert to the coupled induction), while Whitefoot's obvious loop
  plus one *checked* `requires` reaches the expert class. Sections 3 and
  13 now tell that story.
- **"CI-banned-unsafe Rust cannot do this"** — overclaim, removed. Safe
  Rust *can* reach the check-free class here by iterator restructuring;
  what it cannot do is get there from the obvious shape, or state the
  contract as a checked fact. (And the restructuring does not generalize
  to variable-size writes — recorded in the experiment as the QOI note.)
- **SPSC dry run**: "~20% over rtrb" is the **round-trip latency** result
  (~31 vs ~39 ns/way). On batched-32 throughput rtrb is ~25% *ahead*
  (1050 vs 1320 M/s; both far above the band). § 13 now scopes the claim.
- **Sealed kernel count**: ten (post-re-cut catalog), not "a dozen."
- **kernelB "complexity-class" asm pair**: that is a *vs-unannotated-C*
  result (~22x); Rust's parameter-level `&mut` reaches the same O(1).
  Relabeled and moved to § 2/§ 5 support, where the honest boundary is
  drawn: parameter noalias is table stakes; *loaded-pointer* facts (§ 5)
  are where Rust runs out of channel.
- **i1/wc experiment**: it ended at C/Rust *parity* (it closed a 1.6-1.8x
  deficit that was ours). § 7 reframed as a taught-form/regularity story,
  not a win-over-Rust story.

---

## 0. Locked framing decisions

1. **Audience: expert-first.** Lead for the seasoned Rust/C/C++ engineer;
   the vibe-coder payoff is one short closing section.
2. **"Can't write bad code" is an expert benefit.** The expert's pain is
   safe/fast/maintainable-pick-two; floor-raising is aimed at that pain.
3. **Length is free; points are the budget.** Every kept point gets a full
   worked example.
4. **Real Whitefoot snippets, sparingly; Rust/C/C++ side-by-side only where the
   comparison earns it.**
5. **Structure-first, mechanism over numbers.** Lead with the mechanism and
   the emitted code, not the ratio; numbers appear as support with caveats
   attached.
6. **Examples are purpose-built, minimal, and tested before use.** The doc
   is teaching *how and why*, not re-running benchmarks. Each example is a
   small kernel built and verified in-repo (correctness + captured asm)
   before it is quoted; the full experiments under `experiments/` are
   linked as the deeper evidence, not pasted wholesale.

### Sourcing rule for evidence
Every codegen claim traces to a committed file. New doc examples get their
own committed mini-experiment (build plan at the end of this file). Any
trimmed listing is labeled as an excerpt with a pointer to the full file.

---

## 1. The spine

*(Rewritten in round 2.1. Sources: the founding research corpus —
`optimizer-language-research/notes/verified-findings.md` F1-F9,
`phase2-jit-findings.jsonl` J001-J005, and the archived static-vs-profile
debate, `archive/research/debates/round1-static-vs-profile.md`.)*

The story the whole document hangs on is **the author bargain**: languages
are shaped by who writes them, and the author shift from human to AI
changes the whole bargain — not just who states the aliasing facts.

1. **Every mainstream language is a bargain with a human writer, and the
   currency is information.** Ergonomics is, concretely, a license to
   leave facts unstated: don't make me declare types, don't make me name
   what my function touches, don't make me say which pointers can
   overlap, let me keep my familiar patterns, give me an escape hatch
   when I know better. Each concession is information the author had and
   the source no longer carries.

2. **The cost compounds down the pipeline** — a ladder the target reader
   already knows first-hand, each rung worked in the doc:
   - **Dynamic languages (JS, Python):** nearly every fact is deferred to
     runtime, so the engine must *watch the program run* to learn what
     the author knew at their desk — hidden classes, inline caches,
     bytecode specialization — and every recovered fact must be guarded,
     can be invalidated, degrades one-directionally, and bills warmup and
     code-cache (V8 maps/deopt, CPython PEP 659, HotSpot tiering —
     J001-J005). Some of the most brilliant engineering in the field
     exists to conditionally recover information the language told the
     author not to write down.
   - **Managed static languages (Java):** types are static, but the
     language makes nearly everything a heap reference. "This object
     never leaves this function" was true in the author's head; the
     language has no way to keep it, so the JIT runs escape analysis to
     *maybe* get it back — and Oracle's own documentation is explicit
     that the result is conditional scalar replacement, never a
     guaranteed stack allocation (F7). The fact was free at authoring
     time; recovered, it is partial, late, and revocable.
   - **C/C++:** compiled and lean, but the aliasing bargain is inverted —
     the compiler must assume pointers may overlap unless heroic analysis
     proves otherwise; `restrict` exists but is unchecked (a wrong one is
     a silent miscompile) and therefore rarely used (F1-F3). The kernelB
     pair shows the bill on one page: the same reduction is ~22x slower
     for want of one aliasing fact the author knew.
   - **Rust — the step that proves the thesis:** make ownership a
     *checked source fact* and one mechanism yields memory safety AND the
     optimizer's noalias facts (the constitution's natural experiment:
     Rust as treatment, C/C++ as control). But Rust's bargain is still
     with human writers, so it stops partway: `unsafe` is
     writer-accessible everywhere (facts are defeasible), interior
     mutability punches holes in exclusivity, the aliasing facts ride on
     parameters but not on loaded data pointers (§ 5), there is no effect
     declaration an optimizer can trust across a boundary (§ 4), no
     checked algebra (§ 6), and the installed base forbids deleting the
     slow shapes (§ 0's floor).
   - The rungs are ordered: each language down the ladder keeps more
     facts in the source, and each is faster *for that reason*. Whitefoot is
     designed as the limit of the ladder.

3. **The general law** (the doc's "aha"): most of what we call
   optimization infrastructure is archaeology — compile-time and runtime
   machinery for recovering, conditionally and at real cost, facts that
   existed for free at authoring time. And the correctness-grade facts
   can never be recovered by observation at all: a profile can steer a
   cost decision (worst case: slower), but a noalias/purity/law fact is
   UB-on-violation — it must be *checked at the source* or it cannot be
   used, period (the founding static-vs-profile ruling). There is no
   downstream fix for information the language threw away upstream; the
   IR lesson agrees (Swift built SIL precisely to stop losing its own
   semantics before LLVM — F9).

4. **The author shift breaks the bargain.** For an AI writer every clause
   flips: verbosity costs tokens, not patience; it has no style
   attachment, no installed base, no familiarity demands; what it needs
   instead is regularity, an in-context spec, and machine-actionable
   compile-time feedback — and what it must *not* be given is an escape
   hatch, because a stuck writer will use it (W3). So for the first time
   it is rational to design a language that demands the whole truth at
   authoring time: every fact stated, exactly one way to state it,
   everything checked, everything carried to the optimizer. Nothing to
   recover downstream, because nothing is discarded upstream.

5. **Then, and only then, the one-liner:**

> **An optimizer is only as fast as the facts it is handed. Human-first
> languages spend their ergonomics budget throwing those facts away, and
> the whole optimization stack labors to get them back. Change the
> writer, and you can finally stop discarding them.**

Candidate lines: *"ergonomics is a license to omit information — and the
optimizer pays the fee"*; *"fifty years of compiler heroics are
archaeology on facts the language forced the writer to discard."*

The goal statement for the top of the doc (revised per round 1):

> **Whitefoot is built to beat the C baseline, not to reach it. It can express
> what C expresses where performance lives — and it hands the backend
> machine-checked facts C and Rust structurally cannot: guaranteed
> aliasing from ownership, effect rows on declarations, checked algebraic
> laws, and runtime checks discharged by proof instead of by trust. The
> AI writes under a checker it cannot cheat; the human states requirements
> and verifies results; entire bug classes are unrepresentable.**

Scoping kept honest in Part VI: today, hand-tuned SIMD kernels still beat
our autovectorized shapes; the beat-C claim is per-mechanism and measured,
never universal.

The honesty bar, stated up front: **R0, the Rust Test — "if a decision
leaves us equal to Rust, it failed; 'just use Rust' was cheaper." Every
section names what it buys over Rust.**

---

## PART I — THE GOAL

### § 0. Why a new language when LLMs already write great Rust?

Open on the reader's own objection: "LLMs write good Rust today. Ban
`unsafe` in CI and you have safe Rust — memory-safe, fast, mature. Why
invent a language with no training data that nobody knows?"

**Concede what's true, then answer in two moves: the ladder, then the
floor.**

- Concede: for the *ceiling*, expert Rust is genuinely high. Banning
  `unsafe` is a real, cheap discipline. If your question is "can a
  top-decile engineer, given time, write a fast safe program in Rust," the
  answer is yes and this document does not dispute it.
- **Move 1 — the ladder (from the spine):** Rust is the furthest rung of
  the keep-the-facts ladder a for-human language can reach; its remaining
  information losses (defeasible facts, interior-mutability holes, no
  effect or law channel, no floor) are not oversights — they are the
  price of its bargain with human writers. A language for an AI writer
  can pay a different price and keep everything. This is why "extend
  Rust" is not the move: the losses are load-bearing parts of Rust's
  human contract.
- **Move 2 — the floor: a ban on `unsafe` bounds memory safety; it does
  not bound how the program may be written.** Nothing in safe Rust prevents the
  `Rc<RefCell>` object graph, the pointer-chasing layout, scattered
  mutation, the 1.6x-slower obvious indexed loop, five spellings of the
  same function, or an assert that looks like a contract and optimizes
  like a comment. In a million-line AI-written codebase, *whatever is
  representable will eventually be written.* Rust cannot enforce a floor
  because its contract with human writers forbids taking those shapes
  away.
- **Whitefoot's bet is to constrain the writable shape-space itself.** One
  spelling per construct, to the byte. One loop form, one conditional.
  Overflow behavior chosen in the operation name. A closed catalog of
  taught architectures. Checks that only a machine proof can remove.
  No `unsafe` to ban because none exists. The worst program the checker
  accepts is still safe *and still on a fast shape* — that is the floor,
  and it is enforceable only because the writer is an AI with no installed
  base to appease.
- Ceiling deltas exist too, and Part II is about them (facts Rust has no
  channel for). But the honest core of "why not Rust" is the floor: for
  the ceiling, xl wins in some cases and Rust is already high; for the
  floor, Rust has no mechanism at all.
- Close the objection loop: "no training data" is answered by design —
  the spec is compact enough to teach in-context (the whole language
  rides along in the prompt; spec-compactness is a binding constraint,
  D2), and diagnostics are deterministic, rule-citing, and machine-
  actionable, so the writer's repair loop runs at compile time, the one
  feedback channel an AI is good at.
- Supporting mechanisms (each gets its full section): cheating made
  unrepresentable, not punishable (§ 8, § 12); regularity over familiarity
  (§ 8); explicitness as the optimizer's food (§ 2-6).
- Candidate line: *"a ban bounds what code may do; Whitefoot bounds what code
  may be."*

### § 1. What safety is actually for here.

**This is the novel-reasoning section — draft it with the logic airtight
first, data second (owner instruction).** The argument, in order:

1. Premise: the writer is an AI; some AIs write wrong or slow code when
   unconstrained, and runtime failure is the worst feedback channel an
   unattended writer can have.
2. Therefore the language's job is to make bad programs *inexpressible*,
   not to detect them: memory corruption, races, silent overflow — removed
   from the language rather than policed in it. Rust bans `unsafe` by
   policy; Whitefoot has nothing to ban. This is safety as a *floor
   mechanism*: one of the ways the language guarantees that even a bad
   writer produces good programs (safe, checked, and shaped onto the
   patterns that perform).
3. The same machinery is the speed machinery: ownership/exclusivity *is*
   the optimizer's noalias fact-base, and race-freedom is what keeps those
   facts sound (a data race would falsify the compiler's reasoning
   retroactively). The `noalias` fact happens to serve safety and
   performance at once — so the usual safety-tax intuition inverts: the
   thing that makes it safe is a thing that makes it fast.
4. Data to support each step (drafted from committed results): step 2 —
   the writability rounds (defect families are caught at check time, with
   rule-citing diagnostics); step 3 — § 5's guard-free vectorization from
   ownership facts, and the kernelB pair (ownership emits the `restrict`
   that C programmers forget, automatically and non-defeasibly, ~22x on
   the reduction vs unannotated C — with the honest note that Rust's
   parameter-level noalias gets this case too).
- **Serves the goal:** dissolves "safety versus speed" for the expert
  reader and sets up every Part II section.
- Candidate line: *"safety is not the goal; it is the floor's enforcement
  mechanism — and it pays for itself in aliasing facts."*

---

## PART II — WHERE THE SPEED COMES FROM
*(the expert core; each section = mechanism + real emitted code)*

Per framing decision #6: every section below gets a NEW minimal worked
example, built and tested before drafting (build plan at end of file).
The existing experiments are cited as the deeper evidence behind each
mechanism, with their caveats.

### § 2. Opener: everything is checked — so where does the speed come from?

The reader's immediate objection to "checks are always on" is that it
contradicts the performance claim. Answer it before any detail, with one
complete example (the owner's two-buffer case):

- A function takes two caller-provided buffers — `src` shared, `out`
  exclusive (`&uniq`) — plus one `requires` line stating the capacity
  relation, and transforms `src` into `out` in a loop.
- What the reader sees, side by side:
  - **C**: no checks, and the compiler must assume the pointers may
    alias unless someone remembers `restrict`; one missed qualifier and
    the loop keeps reloads or a runtime disambiguation guard.
  - **Rust (safe, obvious shape)**: parameter noalias is emitted, but
    every indexed access keeps its bounds branch; `assert!` up front does
    not remove them (measured: changes nothing).
  - **Whitefoot**: the entry check runs once, every access inside is
    discharged by proof from that checked fact plus the loop induction;
    the borrow modes emit the aliasing facts automatically. Inner loop:
    no bounds branches, no alias guards — at or above the C codegen,
    *with* checked arguments and checked boundaries.
- The rule the example teaches: **checks are never traded away at the
  source level; the compiler earns their removal by machine proof, and
  the entry contract itself always remains** (it protects even a C caller
  entering through the FFI boundary).
- Evidence behind the example: the base64 PROOF-2 record (27/27 sites
  discharged, 2.48 -> 4.23 GB/s, 1.71x, entry trap retained —
  `experiments/port-study/base64/RESULTS.md`), and the kernelB noalias
  pair for the aliasing half (`experiments/codegen-vs-rust-c/`).
- **Serves the goal:** P0 and W3 in one picture; frames § 3-6 as "four
  independent fact channels feeding this same machine."
- Candidate line: *"speed is earned by proof, never bought by weakening a
  check."*

### § 3. A check is removed only by machine proof (`requires`).

- **The mechanism:** `requires` is a checked callee-entry prologue, not an
  assumption and not an optimizer hint. It executes on every invocation;
  a false condition traps before the first body effect; only its *success
  edge* becomes a fact the deterministic prover may use to discharge
  dominated checks. A writer cannot assert a fact into existence — the
  contract cannot be weakened to make a failing body pass.
- **Example + codegen:** base64. The obvious indexed loop with its
  retained bounds branches; then the one checked capacity relation
  (`len(src) <= 3 * floor(len(out)/4)`) connects to the `i=3k, o=4k`
  induction and all 27 sites discharge; branches drop; output stays
  byte-identical; the entry trap remains.
- **The honest Rust comparison (corrected adversary rerun, all variants
  full RFC semantics, same entry relation):**
  - Rust obvious indexed loop: 1.60x slower; the checks stay.
  - Rust obvious + `assert!` up front: 1.604x — statistically nothing.
    The optimizer cannot connect the assert to the coupled induction.
    The folk remedy is measured dead.
  - Rust expert `chunks_exact/zip`: ties Whitefoot (0.997) — say so plainly.
    The restructuring is real skill, it does not generalize to
    variable-size writes, and nothing checks it: it is a shape you must
    know, not a contract the compiler verified.
  - Rust `unsafe` indexed: 1.040 — slightly *slower* than both.
- **The delta stated precisely:** in Whitefoot the *obvious* shape plus a
  *checked* one-line contract reaches the expert class; in Rust the
  obvious shape stays 1.6x behind and the honest-looking remedy does not
  work. That is a floor claim, backed by the ceiling tie.
- Evidence: `experiments/port-study/base64/RESULTS.md` (PROOF-1/2 and the
  controlled adversary), `b64.s`/`b64.ll`.
- **Serves the goal:** P0 earned by proof + W3 (no writer-side cheat) +
  W1 (the obvious shape is the fast shape).

### § 4. Effect rows the optimizer can trust across an opaque boundary.

- **The mechanism:** every signature declares its effects (`pure`,
  `reads('r)`, `writes('r)`, `allocates`, `traps`), checked *both ways* —
  undeclared-but-exhibited and declared-but-unexhibited are both errors.
  They lower to guaranteed LLVM attributes on the *declaration*, so calls
  optimize without body visibility. Rust has no channel to declare
  trusted effects on an `extern fn`; its optimizer must see the body.
- **Example + codegen:** the opaque-boundary kernel: a pure call inside a
  2e9-iteration loop, callee body hidden in a separate object file. With
  declared effects the call hoists and the loop strength-reduces to O(1)
  (0.00s); without them, 2e9 real calls (1.47s). Rust's default
  cross-crate shape: 1.49s; Rust with fat LTO: ties — with the body
  visible, LLVM infers the same facts.
- **The honest boundary:** same-module, LLVM inference matches us; the
  channel's value is at boundaries. The durable claim is build-economics:
  **Whitefoot's per-file default equals Rust's most expensive configuration
  (fat LTO)** — and the guarantee holds where inference cannot reach
  (opaque objects, cached artifacts, future FFI frames).
- Evidence: `experiments/effect-attrs-channel/RESULTS.md`, `main_attr.s`
  vs `main_plain.s`.
- **Serves the goal:** P0 with no Rust source channel at all, and it
  scales with project size (LTO-grade results at -O2 build cost).

### § 5. Ownership as the aliasing fact-base: guard-free vectorization.

- **The mechanism:** `&uniq` is exclusive and loans have singleton
  provenance, so inside a function taking `s: &uniq Cols`, every buffer
  column of `s` is pairwise-disjoint memory — and the compiler emits
  per-field alias scopes on the *loaded data pointers*. This is exactly
  where Rust's channel ends: `&mut Cols` gets parameter noalias, but the
  Vec data pointers loaded from it are fresh provenance roots LLVM must
  treat as may-alias. No `RefCell`, no interior mutability, so the fact
  holds universally.
- **Example + codegen:** multi-column struct-of-arrays update. Whitefoot:
  8 vector adds, **zero runtime guards, 121 asm lines**. Rust obvious
  shape: vectorized only behind loop-versioning — 29 runtime alias
  guards, **2132 lines** (17x). At 16 columns the guards grow to 111 and
  the code to 2836 lines while Whitefoot stays at 183 with zero.
- **Honest scoping:** at long trip counts Rust's guards amortize and
  times tie; the durable deltas are short trips (2.0x at n=8 — the
  per-row-update case), code size (i-cache pressure in real programs,
  invisible in microbenchmarks), and guarantee-vs-heuristic (static O(1)
  fact vs runtime disambiguation that grows with pointer count). Say all
  three.
- Evidence: `experiments/scoped-alias-channel/RESULTS.md`,
  `kernel_facts.s` vs `rust_kernels.s`.
- **Serves the goal:** P0 from a fact Rust cannot express for its own
  soundness reasons; W1 — the obvious Whitefoot shape is the fast shape at
  every n.

### § 6. Checked algebraic laws (the cheat-proofness jewel).

- **The mechanism:** declare `associative`/`commutative`/`identity` on an
  operation; the compiler *checks* the law against its op-table semantics
  and only then reassociates the obvious serial fold into parallel
  accumulators. A stated law it cannot discharge is a hard reject; a
  *false* law is refuted with a rule-cited diagnostic.
- **Example + codegen:** saturating-add reduction. The serial dependency
  chain is the bottleneck; the checked law licenses breaking it: 3.3x
  over the obvious fold in both languages, tying Rust's hand-written
  4-accumulator expert shape.
- **The kicker:** the expert Rust shape *asserts* associativity on faith.
  Swap in signed saturating add — not associative — and Rust silently
  computes garbage; Whitefoot refutes the declared law at compile time
  (`(MAX sat+ 1) sat+ -1 != MAX sat+ (1 sat+ -1)`).
- Evidence: `experiments/checked-law-channel/RESULTS.md`, `kernel.s`.
- **Serves the goal:** P0 fused with W3 — the transform Rust takes on
  folklore is here a checked fact; the honest mistake is structurally
  unavailable. W1: the fast shape is the obvious fold.

### § 7. Boolean dataflow stays `i1`, so the vectorizer widens it.

- **Reframed (round-2 honesty fix):** this is a *floor/regularity* story,
  not a win-over-Rust story — the experiment ended at C/Rust parity after
  closing a 1.6-1.8x deficit that was ours.
- **The mechanism:** loop-carried classification state kept in `Bool`
  (i1) and combined with boolean ops vectorizes at width 16; the same
  logic routed through integer flags caps at width 2x4 and loses 1.6-1.8x
  on scanner-class kernels. The taught pattern (P7) *is* the guarantee
  that the writer lands on the wide shape.
- **Example + codegen:** wc word counting — the i64-recurrence loop vs the
  i1-recurrence loop, same semantics, 16-wide vs narrow vectors.
- Evidence: `experiments/port-study/wc-chunk-summary/RESULTS.md` (incl.
  the Bool-copy addendum), `chunk_wc.s`.
- **Possible merge:** into § 9 (the catalog is what makes this shape the
  default) if Part II runs long.

---

## PART III — MAKING THE FAST SHAPE THE ONLY SHAPE

Framing for the whole part: even the expert does not hand-tune the average
line under a deadline, and in a million-line codebase the average line is
not optimal. Whitefoot makes the slow and the unsafe shapes unrepresentable
(or unreachable-by-teaching), so the floor rises for everyone, expert
included.

### § 8. One spelling to the byte; overflow chosen in the operation name.

- Exactly one legal spelling and one legal byte-level formatting; the
  toolchain **rejects** non-canonical input rather than reformatting it.
  No operators, no precedence. `iadd.wrap` / `iadd.trap` / `iadd.checked`
  are different operations and there is no default — the writer chooses
  overflow semantics per call site, visibly.
- The why: irregularity is weak-writer error surface; canonical bytes
  leave nowhere to hide an edit; and naming the overflow mode kills the
  C/Rust debug/release split where two build modes literally optimize
  different programs (Rust: panic in debug, wrap in release).
- Small example: the same arithmetic in Whitefoot vs Rust's debug/release
  divergence.
- **Serves the goal:** W1 (regularity) + W3 (no hidden edits) + P0 (no
  semantic divergence between builds).
- Candidate line: *"pay unlimited verbosity to buy zero irregularity."*

### § 9. A closed, taught pattern catalog; struct-of-arrays as the default.

- The set of allowed program-scale architectures is closed and taught up
  front, exactly as one loop form is forced at statement level. A human
  language must admit familiar patterns or be rejected; an AI writer has
  no installed base, so the catalog can be exactly the shapes the fact
  channels light up — provided it stays *complete* (every task modelable;
  a gap is a documentation defect, not a writer error) and *efficient*.
- Example: the command-buffer pattern — deep code is `pure`/`reads` and
  returns write-intents as values; exactly one shallow function holds the
  single `&uniq` and applies them. The architecture is checkable by
  grepping signatures: one `writes('p)` in the system. Contrast the
  `Rc<RefCell>` scattered-mutation shape, unrepresentable here by design —
  and that restriction is what keeps § 5's alias facts sound.
- SoA as the blessed layout: contiguous columns are what the vectorizer
  and the cache want; "SoA feels like bad design" is an object-oriented
  prior the AI does not have. (Honest note from the dry runs: where a
  paired key/value layout is what the access pattern wants, the catalog
  pins that instead — AoS for the hash-table slots. The catalog encodes
  measured layout decisions, not ideology.)
- **Serves the goal:** this is where "you can't write the slow
  architecture" lives.
- Candidate line: *"whatever is representable will eventually be
  written."*

### § 10. Handles and copies instead of references (the coat-check model).

- The center of gravity moves off borrows: big things live in pools; code
  holds either the value (copy), ownership (move), or a *handle* — a
  plain-data claim ticket, not a pointer. Node links in a tree/graph are
  indices; the self-referential-struct problem that pushes real Rust
  projects to index-arenas-with-hacks simply does not arise, because
  structs store values, not borrows, by construction.
- Generational handles (owner-ratified): slot reuse is allowed and a
  stale ticket is a deterministic trap, never a silent read of the new
  occupant. (Honest note: the per-access version check has a cost;
  check-free alternatives — session loans, affine owned handles,
  proof-elided repeat checks — are under research.)
- Example: an AST/graph as an index pool vs Rust holding references and
  fighting the borrow checker over `&mut self`.
- **Serves the goal:** P0 (contiguous, cache-friendly, no per-node
  allocation) + W1 (most code needs no loan reasoning at all) + T1 (no
  use-after-free by construction).
- Candidate line: *"you're holding a claim ticket, not the coat."*

---

## PART IV — ONE PROGRAM, ONE TREE, ONE BYTE-FORM  *(new in round 2)*

### § 11. The canonical-form dividend: the toolchain side.

- The facts: one spelling per construct and one byte-level formatting
  (FORM-1/2); a 1:1 mapping between grammar productions and core-tree
  nodes, machine-enforced (GRAM-1/META-1); accepted programs elaborate to
  a canonical artifact from which acceptance is decidable, with every
  derived operation explicit (DIAG-2); diagnostics deterministic and
  byte-stable (DIAG-1); no comments — docs are structured fields.
- Consequences to draft, each labeled measured vs by-construction vs
  projected:
  - **Reproducibility (by construction):** same source, same artifact,
    byte-identical builds — already exercised by the self-hosting gate
    (the whole-compiler object is SHA-pinned).
  - **Diff/merge/review (by construction):** no formatting diffs exist;
    a diff is always semantic; canonical bytes leave nowhere to hide an
    edit (this is also W3).
  - **Caching and build speed (projected — label it):** source-to-tree is
    a bijection, effect rows already decouple optimization from body
    visibility (§ 4's per-file-cost result is the measured half), and a
    deterministic canonical artifact is exactly what content-addressed
    caching wants. The honest claim: the *design* removes the classical
    obstacles to very fast incremental compilation; we have not measured
    a build-speed headline yet.
  - **The AI repair loop (by construction, exercised):** every rejection
    cites exactly one rule ID, the node path, and where applicable a
    mechanical fix — deterministic and byte-stable, i.e. the diagnostics
    are an API for the writer, not prose for a human.
- **Serves the goal:** W3 (nowhere to hide), W1 (machine-actionable
  repair), build economics for the AI's compile-check loop (D0: the
  compile loop is the writer's inner loop, so its latency is a
  first-class cost).
- Candidate line: *"the program has one body; everything downstream stops
  being heuristic."*

---

## PART V — THE TRUST STORY (what an expert will poke at)

### § 12. Ten sealed building blocks — and you can prove your way in.

- A small, fixed set of high-performance kernels (growable sequence, hash
  table, pool, arena, SPSC queue, io-file, …) have trusted internals,
  because some invariants — "1000 slots, 37 live, tracked by side-band
  control bytes" — are unexpressible in checked source *in any language*.
  Rust's answer is `unsafe` scattered across thousands of crates; Whitefoot's
  is ten kernels it owns.
- Why it is still safe: there is no `unsafe` keyword to reach for, so the
  privileged layer is structurally unreachable from ordinary source; harm
  concentrates in ten audited kernels (fix once, the world recompiles).
  Each ships only through the five-part acceptance ledger: differential
  testing against the reference implementation, exhaustive small-bound
  model checking, sanitizer/fuzz soak, hostile pre-ship review, CI-pinned
  performance and assembly shape — plus complete failure semantics and
  teardown protocols with fault-injection evidence.
- Worked credibility example: the SPSC queue — exhaustively model-checked
  SAFE, and the model catches every single weakened memory-order mutant
  (all four acquire/release halves load-bearing); the emitted steady-state
  path has zero read-modify-write atomics.
- **The escalation story:** *trust is a temporary loan; proof buys
  permanent privilege.* Any kernel that machine-proves its invariants
  leaves the trusted list and becomes checked code with the same
  privileged representation rights. The same lane is open, long-term, to
  users: performance never requires proof, but a user structure that
  proves its invariants gets the privileged representation as *checked*
  code. The trusted list only ever holds project kernels not yet proved.
- **Serves the goal:** P0 (native representations, no tax) + W3/T1 (tiny
  audited base, no writer-reachable escape) + the real answer to "so
  where's the escape hatch?" — there isn't one; there's a proof lane.
- Candidate line: *"trust is a temporary loan; proof buys permanent
  privilege."*

---

## PART VI — HONESTY (this is what earns the expert's trust)

### § 13. What it does not beat, and what is still unproven.

All corrected to the current committed records:

- **Expert safe Rust ties on base64 at full semantics** (0.997, corrected
  controlled rerun). The durable delta is the floor, not the ceiling:
  Rust's obvious loop is 1.60x behind and `assert!` recovers nothing,
  while Whitefoot's obvious loop + one checked relation reaches the tie.
- Hand-tuned SIMD kernels (bytecount, memchr) remain 2-3x ahead of our
  naive autovectorized shapes. The SIMD-shape story is future work, not a
  present claim.
- The language is verbose by design: the base64 encode kernel is ~90
  lines where C is ~15. (D2a: the AI pays that cost; humans read the doc
  fields.)
- The frequency question is open: how often the winning patterns occur in
  real medium/large codebases is not yet established (the leg-A pilot was
  directional only).
- Dry-run status stated precisely: the sealed-kernel *shapes* were
  validated as C mockups against mature Rust baselines on an Apple dev
  machine — seq push-then-sum ~1.5x over `Vec`, table iterate ~1.4x over
  hashbrown, 4/5 in band with one edge case; the SPSC queue beats rtrb by
  ~20% on round-trip latency while rtrb is ~25% ahead on batched
  throughput (both far above the band). These validate shapes, not final
  magnitudes, and none of it is whitefoot-emitted code yet.
- Single-shot writability of the research kernel is 26.3% at the current
  clean baseline — the honest current answer to "can a model just write
  it?" is "with a diagnostic feedback loop, and that loop is the design."
  (Include only if the doc ships after round 4; else state the round-4
  status.)
- **Serves the goal:** for this audience the caveats are the credibility;
  a doc that names its losses gets believed on its wins.

---

## PART VII — THE OTHER READER (short)

### § 14. And if you don't write systems code at all.

One page, no jargon: you describe intent and verify results; you never see
the checker. The slow version and the corrupt version won't compile.
Possible cut to two closing paragraphs.

---

## Worked-example build plan (before drafting; per framing decision #6)

New minimal kernels, each committed with source, build script, correctness
check, and captured asm — created at draft time, quoted only after green:

1. **two-buffer transform** (§ 2): `&uniq out` + `& src` + one `requires`
   capacity relation; C / Rust-obvious / Rust-assert / Whitefoot side by side.
2. **column update, 3 columns** (§ 5): trimmed from the 8-column
   scoped-alias kernel so the guard-versioning contrast fits on a page.
3. **opaque pure call in a loop** (§ 4): already minimal in the effect-attrs
   experiment; re-emit and excerpt.
4. **saturating-add fold** (§ 6): already minimal; add the signed-satadd
   compile-time refutation transcript.
5. **overflow-mode trio** (§ 8): one arithmetic line under `.wrap` /
   `.trap` / `.checked` + the Rust debug/release pair.
6. **i1 scanner** (§ 7): minimal word-boundary loop, i64 form vs i1 form.

Location: a new `experiments/doc-kernels/` (or similar) with one README
and per-kernel RESULTS stubs, so the doc's quotes stay reproducible under
the standing sourcing discipline.

---

## Open questions for owner review (round 2)

- **A. (resolved round 1):** § 1 stays separate, rebuilt on the
  safety-as-floor-mechanism argument per your comment.
- **B. (resolved by your comment #4):** purpose-built simple tested
  examples, real output excerpted, full files linked.
- **C. Load-bearing sections:** my round-2 read — § 2 (the opener) plus
  § 3/§ 5/§ 6 are the load-bearing four; § 4 is the build-economics story;
  § 7 is mergeable. Agree?
- **D. Part IV scope:** is the canonical-form/toolchain section (§ 11) at
  the right depth, and should the projected build-speed claim appear at
  all before it is measured? My leaning: keep it, clearly labeled
  projected — experts respect a labeled projection more than a hole.
- **E. Still-omitted candidates:** `try` propagation with auto context;
  closed-world compilation; no-destructors / trap = process abort;
  channel-4 musttail interpreter dispatch (structurally inexpressible in
  Rust — could be a strong extra Part II section; its measured experiment
  is still pending, which is why I left it out). Say the word on any of
  these.
