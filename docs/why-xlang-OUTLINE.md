# why xlang — document outline (for owner review)

Status: OUTLINE / PLAN, not the document itself. Purpose: agree the point
list, the order, the framing, and the example/codegen evidence for each
section *before* writing the full prose. Drop inline comments anywhere (a
`>>` line, a `TODO`, whatever is easiest) and I will revise the plan, then
draft the sections we keep.

Working title of the eventual document: **"xlang: what's different, and why —
for people who already write Rust, C, and C++ (and for people who only prompt)."**

---

## 0. Locked framing decisions (from the discussion)

These are settled unless you change them here.

1. **Audience: expert-first.** Lead for the seasoned Rust/C/C++ engineer. The
   vibe-coder payoff is real but secondary — one short section near the end.
2. **"Can't write bad code" is reframed as an EXPERT benefit, not a beginner
   one.** The expert's world is *safe, fast, maintainable — pick two*. The
   whole floor-raising argument is aimed at that pain, not at novices.
3. **Length is free; points are the budget.** Every point kept gets a full
   section with a worked example. We decide scope by choosing points, not by
   trimming explanations.
4. **Real xlang snippets, used sparingly.** Rust/C/C++ shown side by side only
   where the comparison earns the explanation.
5. **Structure-first, mechanism over numbers.** Do NOT lead with "1.6x faster."
   Lead with *the mechanism*, and prove it by showing the actual compiled
   assembly / IR: "here is what xlang emits, here is what the normal safe
   C/Rust emits, and here is the single fact in the IR that caused the gap."
   Numbers appear as support, always with their caveat attached.

### Sourcing rule for evidence
Every codegen claim is pulled from committed files under `experiments/`, never
invented. Exact numbers and asm are read from the relevant `RESULTS.md` at
draft time. Illustrative-but-trimmed snippets, if any, are labeled as such.

---

## 1. The spine (one story the whole document hangs on)

> **An optimizer is only as fast as the facts it is handed. Humans won't write
> those facts — an AI will. So build the language for the writer we actually
> have now.**

Everything downstream is a consequence of the writer changing from a human to
a machine. Existing languages cannot adopt these consequences because they all
have humans to keep happy (installed base, ergonomics, familiarity, semver).

The goal in one line, for the top of the doc:

> **The code an AI writes — under a checker it cannot cheat — runs at C-class
> speed, with entire bug classes made impossible, while a human only states the
> requirements and verifies the result.**
// in face with all we have here, it's not just c-class, it should be much faster than c. it can do all c can, and it can use the compiler backend's optimization chances better than c - proof, aliasing annotations, etc.

The honesty bar we hold ourselves to, stated up front: **R0, the Rust Test —
"if a decision leaves us equal to Rust, it failed; 'just use Rust' was
cheaper." Every difference below names what it buys over Rust.**

---

## PART I — THE GOAL (short; sets the lens; little/no code)

### § 0. Why a new language when LLMs already write great Rust?
- **Open on the reader's own objection** (this is the seasoned engineer's first
  reaction): "LLMs write good Rust today. Ban `unsafe` in CI and you basically
  have safe Rust. So why invent a language with no training data, that eats
  context, that nobody knows?"
- **Answer with the four-point reframe** (each fully explained):
  1. The expensive part of AI code is *trusting* it, not typing it — so move
     failure from runtime (an AI's worst feedback channel) to a compile-time
     repair loop it can act on. // I don't quite understand the logic here, how can this answer the above question?
  2. Models cheat when stuck — so make cheating *unrepresentable*, not
     punishable. There is no hatch to reach for. // yeah this is a good point, but as mentioned, Rust can also ban unsafe, which makes it equally qualified for this point.
  3. What breaks weak models is *irregularity*, not verbosity — so one spelling,
     verbose everywhere, zero special cases. // verbose everywhere is one of the benefits from "programming language for AI not for human". but this is also not quite supporting the answer.
  4. Explicitness is what the optimizer wants anyway — so writability and speed
     *reinforce* each other instead of trading off. (This is the hinge of the
     whole document.)
- **Serves the goal:** frames the reader for "this is not another safe Rust; it
  is a different bet about who writes the code."
- Candidate line: *"make cheating unrepresentable, not punishable."*

### § 1. Safety is a consequence, not a goal.
- The expert hook: in xlang, **safety, the optimizer's aliasing facts, and speed
  are the same type-system fact seen from three sides.** Ownership is not there
  to protect you; it is there because it *is* the `noalias` fact-base the
  optimizer needs — and race-freedom is what keeps those facts sound.
- Therefore the usual tradeoff dissolves: you are not paying for safety with
  speed, because the thing that makes it safe is the thing that makes it fast.
- **Serves the goal:** sets up every Part II section ("you stop choosing between
  safe and fast").
- **OWNER DECISION (open question A):** does this land as its own opening
  section for experts, or is it too abstract to lead with — merge it into § 0?

---

## PART II — DIFFERENCES THAT BUY RAW SPEED
*(the expert core; each section = the mechanism + the real codegen that proves it)*

### § 2. Checks are always on; a check is removed ONLY by machine proof.
- **The difference:** every risky operation (bounds, overflow) is checked at
  runtime. The *only* way to remove a check is a machine-verified proof at
  compile time. A writer can never assert a fact into existence. "Speed is
  earned by proof, never bought by weakening a check."
- **The mechanism:** the `requires` entry contract — a predicate checked once at
  the function boundary; only its *success edge* becomes an optimizer fact the
  prover uses to discharge the dominated checks inside. The entry check itself
  is never removed (it protects even an invisible C/FFI caller).
- **Example + codegen:** base64. Show the hot loop with retained bounds branches,
  then after the capacity proof discharges 27/27 sites — branches drop, the
  kernel speeds up, output is byte-identical.
  Evidence: `experiments/port-study/base64/b64.s`, `b64.ll`, and the numbers +
  caveats in the port-study / proof-tier `RESULTS.md`.
- **The expert kicker:** CI-banned-`unsafe` Rust *cannot* do this — it forfeits
  `get_unchecked`, so it keeps the checks and eats the cost. xlang gets "peak
  performance without unsafe, provably."
- **Serves the goal:** P0 (speed earned by proof) + W3 (no writer can cheat the
  check away).
- Candidate line: *"the compiler earns the speed through proof."*

### § 3. Effect rows the optimizer can trust across an opaque boundary.
- **The difference:** every signature declares its effects (`pure`, `reads('r)`,
  `writes('r)`, `traps`), checked both ways (under- and over-declaring are both
  errors). These lower to guaranteed LLVM attributes *on the declaration*.
- **The mechanism:** Rust has no way to declare trusted effects on an `extern fn`;
  its optimizer must inline to prove purity and usually gives up across a call or
  a cycle. xlang states the fact and checks it, so calls optimize like values
  without body visibility.
- **Example + codegen:** the opaque-boundary kernel — declared facts let the call
  hoist out of a 2e9-iteration loop (assembly: the loop body collapses), tying
  Rust's *fat-LTO* build at ordinary per-file build cost.
  Evidence: `experiments/effect-attrs-channel/main_attr.s` vs `main_plain.s` /
  `main_wr.s`, and `experiments/effect-attrs-channel/RESULTS.md`.
- **Tie-in (great asm story):** the naive-C-vs-`noalias` pair where the *only*
  difference in the emitted IR is a single `noalias` attribute, and the machine
  code diverges by a complexity class.
  Evidence: `experiments/codegen-vs-rust-c/asm/kernelA_*.s`, `kernelB_*.s`.
- **Serves the goal:** P0 delta with *no Rust source channel at all* — LTO-grade
  optimization that also scales with project size.

### § 4. Ownership as the aliasing fact-base → guard-free vectorization.
- **The difference:** exclusive vs shared borrows (`&uniq 'r` / `&'r`) with
  explicit regions and no inference; the region lives on the *access*, not on
  the data type. So node links are plain `u64` values with no lifetime, and the
  self-referential-struct problem is *unrepresentable* rather than painful.
- **The mechanism:** because shared-xor-mutable is absolute (no `RefCell`, no
  interior mutability punching holes), the compiler's alias facts hold
  *universally*, so it vectorizes without inserting runtime disambiguation
  guards.
- **Example + codegen:** the multi-column struct-of-arrays kernel — xlang emits
  clean, compact vector assembly where the ordinary Rust shape needs
  loop-versioning with many runtime guards and far more code.
  Evidence: `experiments/scoped-alias-channel/kernel_facts.s` vs
  `kernel_nofacts.s` vs `rust_kernels.s` (and the width-16 pair
  `kernel16_facts.s` / `rust16.s`), with `RESULTS.md` for the exact line counts
  and trip-length caveats (ties at long trips — say so).
- **Serves the goal:** P0 (a fact Rust omits for its own soundness reasons),
  and the self-referential wall that pushes real Rust projects to index-arenas
  simply does not exist here.

### § 5. Checked algebraic laws (the cheat-proofness jewel).
- **The difference:** you *declare* a law (e.g. `associative`) on an operation;
  the compiler only uses it after *checking* it, and reassociates your obvious
  serial fold into parallel accumulators for you.
- **The mechanism vs Rust:** an expert's hand-written multi-accumulator
  reduction asserts associativity *on faith*. Swap in an operation that isn't
  actually associative (signed saturating add) and Rust silently computes
  garbage — while xlang **refuses the declared law at compile time.**
- **Example + codegen:** the saturating-add reduction — serial fold vs the
  compiler-reassociated form; then the signed-saturating-add case as the
  correctness kicker.
  Evidence: `experiments/checked-law-channel/kernel.s` and
  `experiments/checked-law-channel/RESULTS.md`.
- **Serves the goal:** P0 (a real speedup) fused with W3 (the transform Rust
  takes on faith is here a *checked* fact; a false law can't ship).

### § 6. Boolean dataflow stays `i1`, so the vectorizer widens it.
- **The difference:** loop-carried classification state (word boundaries, token
  starts) is kept in `Bool`/`i1` and combined with boolean ops, never routed
  through integer flags or match-arm control flow.
- **The mechanism:** an `i1` recurrence lets LLVM widen to full-width byte
  vectors; the same logic expressed as an integer recurrence caps the vector
  width and loses ~1.6–1.8x on scanner-class kernels.
- **Example + codegen:** wc word counting — show the `i64`-recurrence loop
  (narrow vectors) vs the `i1`-recurrence loop (16-wide).
  Evidence: `experiments/port-study/wc-chunk-summary/chunk_wc.s` and its
  `RESULTS.md`.
- **Serves the goal:** a tiny, regular language rule producing a large codegen
  swing — and it is a *taught pattern* (leads into § 8).
- **Possible merge:** if the doc runs long, fold § 6 into § 4 as a second
  vectorization example.

---

## PART III — DIFFERENCES THAT MAKE THE FAST SHAPE THE ONLY SHAPE
*(floor-raising, framed as the expert's "safe/fast/maintainable — pick two" pain)*

Framing note for this whole part: the point for an expert is that **even you do
not hand-tune the average line under a deadline, and in a million-line codebase
the average line is not optimal.** xlang makes the slow and the unsafe shapes
*unrepresentable*, so the floor is raised for everyone, expert included.

### § 7. One canonical spelling; no operators; overflow mode in the op name.
- **The difference:** exactly one legal spelling to the byte; the toolchain
  *rejects* non-canonical code (it does not auto-format); no infix operators, no
  precedence; overflow behavior is part of the operation name
  (`iadd.wrap` / `iadd.trap` / `iadd.checked` are different ops, no default).
- **The why:** irregularity is weak-model error surface; canonical bytes leave
  nowhere to hide an edit (cheat-proof, diffable, reproducible); and putting the
  overflow mode in the name kills the C/Rust debug-panic/release-wrap split,
  where the two builds literally optimize different programs.
- **Small code example:** the same arithmetic in xlang vs the Rust debug/release
  divergence.
- **Serves the goal:** W1 (regularity) + W3 (no hidden edits, no hatch) + P0
  (no debug/release semantic divergence).
- Candidate line: *"pay unlimited verbosity to buy zero irregularity."*

### § 8. A closed, taught pattern catalog + struct-of-arrays as the default layout.
- **The difference:** the set of allowed program-scale architectures is closed
  and taught up front, exactly as one loop form and one conditional are forced
  at the statement level. The blessed default data layout is struct-of-arrays.
- **The why:** a human language must let people bring their familiar design
  patterns in, or it gets rejected. An AI writer has no installed base to
  appease, so the language can *force* the fast subset — provided the catalog is
  *complete* (every task modelable) and *efficient* (the blessed patterns are
  exactly the shapes the fast paths light up). "SoA feels like bad design"
  is just an object-oriented prior the AI does not have.
- **Example:** the command-buffer / write-intent pattern (deep code stays
  read-only and returns intents; one shallow function applies them) contrasted
  with the Rust `Rc<RefCell>` / scattered-mutation shape that is *unrepresentable*
  here — and why that restriction is what keeps the alias facts sound.
- **Serves the goal:** this is where "you can't write the slow architecture"
  lives; the patterns are the fact-channel feeders.
- Candidate line: *"whatever is representable will eventually be written."*

### § 9. Handles and copies instead of references (the coat-check model).
- **The difference:** the center of gravity moves off borrows. Big things live in
  a pool; you hold the value (copy), take ownership out, or store a copyable
  *handle* — a claim-ticket number that is plain data, not a pointer.
- **The why:** a `u64` copy is cheaper than a borrow and cannot dangle; most code
  then needs no loans at all, so the borrow rules only bite at the few places
  that genuinely point into something. Generational handles make a stale ticket
  a clean trap, never a silent read of the wrong occupant.
- **Example:** store a tree/graph as an index pool; compare with Rust holding
  references to dodge copies and fighting the borrow checker over `&mut self`.
- **Serves the goal:** P0 (cheap, cache-friendly, no pointer-chasing) + W1
  (fewer places the writer must reason about lifetimes) + T1 (no use-after-free).
- Candidate line (analogy): *"you're holding the coat, not a claim ticket."*

---

## PART IV — THE TRUST STORY (what an expert will poke at)

### § 10. A dozen sealed building blocks with trusted internals — and you can prove your way in.
- **The difference:** a small, fixed set of high-performance primitives (growable
  array, hash table, object pool, arena, queue, etc.) have trusted internals,
  because some things — a hash table's "1000 slots, 37 live, tracked by side-band
  control bytes" — are *unexpressible* in any checked language. That
  inexpressibility *is* the safety theorem.
- **The why it is still safe:** there is no `unsafe` keyword anywhere, so the
  privileged layer is unreachable from ordinary source — nothing to escalate
  into, nothing to forget to audit. The trusted base stays tiny: harm
  concentrates in a dozen kernels (fix once, the world recompiles), versus
  Rust's `unsafe` scattered across thousands of crates. Each kernel ships only
  after a five-part acceptance battery (differential test vs the real reference,
  small-scale exhaustive model check, sanitizer/fuzz soak, hostile review,
  CI-pinned performance + assembly shape).
- **The escalation story (the elegant part):** *trust is a temporary loan; proof
  is how anything buys its way in permanently.* Any kernel that later
  machine-proves its invariants leaves the trusted list. And a user who proves
  their own novel structure's invariants gets the same privileged, check-free
  representation — as *checked* code, not trusted code. "Performance never
  requires proof; but if you prove it, it earns the privileged benefits."
- **Serves the goal:** P0 (native representations, no tax) + W3/T1 (tiny audited
  base, no writer-reachable unsafe) + a real answer to "so where's the escape
  hatch?" (there isn't one; there's a proof lane instead).
- Candidate line: *"trust is a temporary loan; proof buys permanent privilege."*

---

## PART V — HONESTY (this is what earns the expert's trust)

### § 11. What it does not beat, and what is still unproven.
- Expert Rust *ties* on base64 after restructuring into iterators — but it does
  so by silently abandoning the output-capacity contract (it would truncate on a
  short buffer; xlang traps at entry). State this plainly.
- Hand-tuned SIMD kernels (GNU memchr, bytecount) stay 2–3x ahead of the naive
  autovectorized shapes.
- The language is verbose by design; a bit-twiddling kernel is ~90 lines where C
  is ~15.
- The open question the project keeps pressing: *how often* do the winning
  patterns actually occur in real medium/large codebases? Still unproven.
- The newest dry-run numbers (growable array ~1.5x over `Vec`; SPSC queue ~20%
  over the mature `rtrb`, mechanically proven race-free) are on an Apple dev
  machine and validate *shapes, not final magnitudes*.
- **Serves the goal:** for this audience, the caveats are the credibility. A doc
  that names its own losses gets believed on its wins.

---

## PART VI — THE OTHER READER (short)

### § 12. And if you don't write systems code at all.
- One page, no jargon. You describe intent and verify results; you never see the
  checker. The story: **the slow version and the corrupt version won't compile.**
  Whatever comes out is memory-safe, race-free, and on a fast shape by
  construction — because whatever is representable eventually gets written, so
  only good shapes are representable.
- **Possible cut:** could be a closing two paragraphs rather than a section,
  since we are expert-first.

---

## Open questions for owner review
- **A. § 1 ("safety is a consequence"):** keep as its own opening section, or
  merge into § 0? (Raised inline above.)
- **B. Codegen snippets:** use the *actual* committed asm/IR from `experiments/`
  (real and verifiable, but noisy), or hand-trimmed representative snippets that
  make the one-fact-that-matters obvious (cleaner, labeled as illustrative)?
  My leaning: real asm, but excerpted to the ~10 lines that carry the point,
  with a link to the full file.
- **C. Load-bearing three:** my read is that § 2, § 4, § 5 are the strongest
  (fact channels Rust structurally can't express, each with killer asm), and
  § 7–9 carry the floor-raising-for-experts argument. Agree, or weight differently?
- **D. Anything missing from the point list?** Candidates I left out as likely
  too-in-the-weeds: `try` propagation with auto context (vs Rust `?`+`From`);
  the closed-world compilation model; no-destructors / trap-abort-no-unwinding;
  the channel-4 musttail interpreter dispatch (structurally inexpressible in
  Rust — could be a strong § if you want a fourth "Rust can't express this" case).
