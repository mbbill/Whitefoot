# xlang: what's different, and why

*For people who already write Rust, C, and C++ — and for people who only
prompt.*

---

## The premise: languages are shaped by who writes them

Every mainstream language is a bargain with a human writer, and the currency
of that bargain is information. Ergonomics is, concretely, a license to leave
facts unstated: don't make me declare types, don't make me name what my
function touches, don't make me say which of these pointers can overlap, let
me keep the patterns I already know, and give me an escape hatch for when I
know better. Each concession makes the language nicer to write — and each one
is a fact the author knew that the source code no longer carries.

The cost compounds down the pipeline. Walk the ladder; you have probably
worked on every rung of it.

**Dynamic languages (JavaScript, Python).** Nearly every fact is deferred to
runtime. The engine must *watch the program run* to learn what the author knew
at their desk: hidden classes and inline caches to rediscover object shapes,
bytecode specialization to rediscover types. Every recovered fact must be
guarded by a runtime check, can be invalidated when an assumption breaks,
degrades one-directionally under polymorphism, and bills warmup time and code
cache. V8 and HotSpot contain some of the most brilliant engineering in our
field, and a large fraction of it exists to conditionally recover information
the language told the author not to write down.

**Managed static languages (Java).** The types are static, but the language
makes nearly everything a heap reference. "This object never leaves this
function" was true in the author's head; the language has no way to keep that
fact, so the JIT runs escape analysis to *maybe* get it back. Oracle's own
documentation is explicit about the result: conditional scalar replacement and
lock elision when the analysis succeeds — never a guaranteed stack allocation.
The fact was free at authoring time. Recovered, it is partial, late, and
revocable.

**C and C++.** Compiled and lean, but the aliasing bargain is inverted: the
compiler must assume any two pointers may overlap unless heroic analysis
proves otherwise. `restrict` exists, but it is unchecked — a wrong one is a
silent miscompile — so in practice it is rarely written. Decades of alias
analysis exist to re-derive non-interference facts the author usually knew.
Part II shows the bill on a single page of assembly: the same reduction runs
roughly twenty times slower for want of one aliasing fact.

**Rust — the rung that proves the thesis.** Make ownership a *checked source
fact*, and one mechanism yields memory safety *and* the optimizer's
no-aliasing facts. Rust is the natural experiment (with C/C++ as the control)
that shows a type system can carry performance facts, not just correctness
facts. But Rust's bargain is still with human writers, so it stops partway:
`unsafe` is writer-accessible everywhere, so every fact is ultimately
defeasible; interior mutability punches deliberate holes in exclusivity; the
aliasing facts ride on function parameters but not on the data pointers loaded
through them (§ 5); there is no way to declare an effect an optimizer can
trust across a call boundary (§ 4); there is no way to state, let alone check,
an algebraic law (§ 6); and a large installed base forbids ever deleting the
slow shapes from the language (§ 1).

The rungs are ordered. Each language further down the ladder keeps more facts
in the source, and each is faster *for that reason*. xlang is designed as the
limit of the ladder.

Two more observations complete the argument.

First, **most of what we call optimization infrastructure is archaeology**:
compile-time and runtime machinery for recovering, conditionally and at real
cost, facts that existed for free at authoring time. Speculative JITs, escape
analysis, alias analysis, link-time optimization, profile-guided optimization
— all of them dig for information the language design discarded.

Second, **the facts that matter most cannot be recovered by observation at
all.** A profile can steer a cost decision, because the worst case of a stale
profile is a slower program. But a correctness-grade fact — these pointers
never alias, this function has no side effects, this operation is associative
— is undefined-behavior-on-violation. It must be *checked at the source*, or
the compiler cannot use it, period. There is no downstream fix for
information thrown away upstream.

So why does every language accept the loss? Because the writer was human.
Humans will not state every fact (that is verbosity), will not tolerate a
single spelling (that is style), demand familiar patterns (that is the
installed base), and demand escape hatches (that is trust). Ergonomics is a
license to omit information, and the optimizer pays the license fee.

**Change the writer, and the bargain flips.** An AI writer pays verbosity in
tokens, which are cheap and getting cheaper. It has no style attachment, no
installed base, no familiarity demands. What it needs instead is regularity
(irregularity, not verbosity, is what makes weak models fail), a specification
compact enough to ride along in its context window, and deterministic
compile-time feedback it can act on. And there is one thing it must *not* be
given: an escape hatch — because a stuck writer will eventually use it.

For the first time, it is rational to design a systems language that demands
the whole truth at authoring time: every fact stated, exactly one way to state
it, everything checked, everything carried to the optimizer. Nothing to
recover downstream, because nothing is discarded upstream.

> **An optimizer is only as fast as the facts it is handed. Human-first
> languages spend their ergonomics budget throwing those facts away, and the
> whole optimization stack labors to get them back. Change the writer, and
> you can finally stop discarding them.**

That is the spine of this document. The goal, in one statement:

> **xlang is built to beat the C baseline, not to reach it. It can express
> what C expresses where performance lives, and it hands the backend
> machine-checked facts C and Rust structurally cannot: guaranteed aliasing
> from ownership, effect declarations the optimizer can trust across opaque
> boundaries, checked algebraic laws, and runtime checks discharged by proof
> instead of by trust. The AI writes under a checker it cannot cheat; the
> human states requirements and verifies results; entire bug classes are
> unrepresentable.**

And the honesty bar the project holds itself to, stated up front: **if a
design decision leaves xlang equivalent to Rust, the decision failed — "just
use Rust" was cheaper.** Every difference described below names what it buys.

---

# Part I — The goal

## 1. Why a new language, when LLMs already write great Rust?

This is the seasoned engineer's first objection, so it goes first: *"LLMs
write good Rust today. Ban `unsafe` in CI and you have safe Rust —
memory-safe, fast, mature, with a decade of tooling. Why invent a language
with no training data, that nobody knows?"*

Concede what is true. For the *ceiling*, expert Rust is genuinely high.
Banning `unsafe` is a real, cheap discipline. If the question is "can a
top-decile engineer, given time, write a fast safe program in Rust," the
answer is yes, and nothing below disputes it.

The answer comes in two moves.

**Move 1 — the ladder.** Rust is the furthest rung a for-human language can
reach, and its remaining information losses are not oversights — they are
load-bearing parts of its contract with human writers. `unsafe` exists
because experts demand final authority. Interior mutability exists because
shared-mutable is sometimes the convenient shape. Five ways to write a loop
exist because expressiveness is an ergonomic good. You cannot fix these by
extending Rust; removing them *is* breaking the contract that made Rust
adoptable. A language whose writer is an AI signs a different contract.

**Move 2 — the floor.** Here is the sentence this document turns on:

> **A ban on `unsafe` bounds what code may *do*; it does not bound what code
> may *be*.**

Nothing in safe Rust prevents the `Rc<RefCell<T>>` object graph, the
pointer-chasing layout, mutation scattered across twenty call sites, the
obvious indexed loop that runs 1.6x slower than the restructured one (§ 3,
measured), or an `assert!` that looks like a contract and optimizes like a
comment (§ 3, also measured). In a large codebase — and AI-written codebases
get large fast — *whatever is representable will eventually be written.* Rust
cannot enforce a floor under program quality, because its human contract
forbids taking shapes away.

xlang's bet is to constrain the writable shape-space itself. One spelling per
construct, to the byte. One loop form, one conditional form. Overflow
behavior chosen in the operation's name, per call site. A closed, taught
catalog of program architectures. Runtime checks that only a machine proof
may remove. And no `unsafe` to ban, because none exists. The worst program
the checker accepts is still memory-safe, race-free, *and on a fast shape* —
that is the floor, and it is enforceable only because the writer has no
installed base to appease.

The floor is not hypothetical. In the project's shipped-library comparisons,
the *first* correctness-green artifact a fixed mid-tier model produced — no
benchmark feedback, no performance hints, no human edits — beat the shipped
Rust `percent-encoding` crate by 1.65x and the shipped `utf8parse` crate by
1.10x on locked workloads, with every bounds check still in place. Not
because the model was clever, but because the shapes the language permits are
the shapes that run fast. (Both results carry their protocols and caveats in
the repository; they are floor evidence for these two libraries and corpora,
not a universal claim.)

For the ceiling, xlang wins where its extra fact channels bite and ties
elsewhere; Part II shows both honestly. But the core answer to "why not
Rust" is the floor, because for the floor Rust has no mechanism at all.

Two follow-up objections, answered briefly:

- *"No training data."* The language is designed to be taught in-context, not
  pretrained: the specification is compact by binding constraint (the current
  working budget, including the taught pattern cards, is on the order of 48k
  tokens — a fraction of a modern context window), with few rules, no special
  cases, and each rule stated once. New model, same spec, same result.
- *"How does an AI even debug it?"* It mostly doesn't have to: the design
  moves failure to the one feedback channel an AI is good at. Every rejection
  cites exactly one rule, the exact tree location, and where possible a
  mechanical fix, deterministically and byte-stably. The compile-check loop
  is the writer's inner loop; runtime debugging is the failure mode the
  whole design works to avoid.

## 2. What safety is actually for here

Safety in xlang is not the mission statement; it is load-bearing
infrastructure, and it is worth being precise about the two jobs it does.

**Job 1: safety is a floor mechanism.** The writer is an AI. Some writers,
when unconstrained, produce subtle memory bugs; and an unattended writer
cannot debug a latent use-after-free three weeks later — runtime corruption
is the worst possible feedback channel. So the language removes the surface:
data races, use-after-free, dangling references, double-free, uninitialized
reads, and silent overflow are not *detected* — they are **unrepresentable**
in accepted programs. There is no `unsafe` block to review, because there is
no `unsafe` block. Rust polices this boundary by convention (CI bans, code
review of `unsafe`); xlang has nothing to police. This is one of several
mechanisms — alongside the single spelling, the taught catalog, and the
checked contracts — that hold the floor: even a bad writer's accepted program
is a good program.

**Job 2: safety machinery is aliasing machinery.** Here is the part that
matters to a performance engineer. The ownership and exclusivity rules that
make the language safe are *the same facts* an optimizer needs:

- An exclusive borrow (`&uniq`) means **no other access path exists** — which
  is exactly `noalias`, held universally, with no interior-mutability holes
  and no `unsafe` that could falsify it.
- Race-freedom is what keeps those facts *sound*: a data race would let
  another thread observe or mutate memory in ways that retroactively falsify
  the compiler's reasoning. A language that guarantees race-freedom is a
  language whose aliasing facts survive concurrency.

So the usual intuition — "safety costs speed, and I buy speed back with
`unsafe`" — inverts. The thing that makes the program safe is a thing that
makes it fast. Sections 3 and 5 show this concretely: the checked capacity
contract that *protects* the output buffer is what deletes the bounds checks,
and the exclusive borrow that *prevents* the aliasing bug is what unlocks
guard-free vectorization.

One honest boundary: this argument says safety and speed share machinery. It
does not say every check is free — it says every check is either proven away
or honestly paid, and Part II shows the ledger.

---

# Part II — Where the speed comes from

Each section below is one mechanism: what the writer states, what the checker
verifies, what the backend receives, and what the machine code looks like on
both sides of the comparison. All listings are excerpts from committed files
(trimmed where marked; full paths in the appendix). Times are medians on an
Apple M4 with the exact protocols in each experiment's RESULTS file.

## 3. Everything is checked — so where does the speed come from?

Start with the objection the previous sentence invites. In xlang, every
risky operation is checked at runtime: every index is bounds-checked, every
`.trap` arithmetic op traps on overflow, and there is no way to write "trust
me" — no `unsafe`, no `get_unchecked`, no assume-intrinsic. A writer cannot
assert a fact into existence.

That sounds like a tax. Here is the deal that removes it:

> **A check is removed only by machine proof — and then it costs nothing
> because it is gone, not because someone promised.**

The mechanism is a checked entry contract. A function may carry one
`requires` block: a predicate on its parameters, *executed on every call* —
including calls entering from foreign C code — and trapping before the first
body effect if false. It is not an assumption, not an optimizer hint, and
not a caller obligation. Only its **success edge** becomes a fact, and a
deterministic prover uses that fact to discharge the checks it dominates
inside the body.

Worked example: a base64 encoder, the exact two-buffer shape every codec
has — one shared input, one exclusive output, and a capacity relation
between them. This is the actual committed source:

```
fn encode ['r] (out: &uniq 'r buffer<u8>, src: own buffer<u8>) -> own u64
    reads('r), writes('r), traps requires {
  let required_out_len: own u64 = len<u8>(deref(out));
  let required_src_len: own u64 = len<u8>(src);
  let required_out_groups: own u64 = ishr.wrap<u64>(required_out_len, 2_u32);
  let required_covered_src: own u64 = imul.wrap<u64>(required_out_groups, 3_u64);
  check ile<u64>(required_src_len, required_covered_src) else trap "base64 output capacity";
} {
  ...
  loop @full {
    ...
    let b0: own u8 = index<u8>(src, i);          // bounds-checked read
    ...
    set index<u8>(deref(out), o) = c0;           // bounds-checked write
    ...
    set i = iadd.wrap<u64>(i, 3_u64);
    set o = iadd.wrap<u64>(o, 4_u64);
  }
  ...
}
```

One line of contract: the output can hold `4 * ceil(len(src)/3)` bytes,
spelled as an overflow-safe comparison. The body is the *obvious* indexed
loop — three reads, four table lookups, four writes per iteration.

**Before the proof tier** (committed `b64.s`, the checked build): every read
and write in the loop carries its compare-and-branch. This is what
"everything is checked" honestly costs:

```
LBB0_9:                          ; hot loop, pre-proof build
	sub	x13, x10, #1
	cmp	x13, x3               ; src bounds check
	b.hs	LBB0_26               ;   -> trap
	add	x14, x13, #1
	add	x13, x13, #2
	cmp	x14, x3               ; src bounds check
	ccmp	x13, x3, #2, lo       ; src bounds check
	b.hs	LBB0_26
	cmp	x12, x1               ; out bounds check
	b.hs	LBB0_26
	...                           ; (each of the 4 stores guarded the same way)
```

**After the proof tier** (compiled from the committed `b64.ll`, the shipping
proof build): the prover connects the passed capacity fact to the loop
induction `i = 3k, o = 4k` and discharges **all 27** bounds sites. The entry
check compiles to one comparison; the hot loop contains *zero* check
branches:

```
_encode:                         ; proof build, from committed b64.ll
	lsr	x8, x1, #2
	add	x8, x8, x8, lsl #1    ; 3 * (out_len / 4)
	cmp	x3, x8                ; the ONE retained entry check
	...
	b.ne	LBB0_11               ;   -> trap (protects even a C caller)
	...
LBB0_6:                          ; hot loop: loads, lookups, stores. No checks.
	ldurb	w13, [x11, #-2]
	ldurb	w14, [x11, #-1]
	ldrb	w15, [x11], #3
	...
	ldrb	w13, [x9, x13]        ; alphabet lookups
	ldrb	w14, [x9, x14]
	...
	sturb	w13, [x12, #-3]
	sturb	w14, [x12, #-2]
	sturb	w15, [x12, #-1]
	strb	w16, [x12], #4
	cmp	x3, #2
	b.hi	LBB0_6
```

Measured on the same 384 MB harness, same source, byte-identical output:
2.48 GB/s with the checks retained, 4.23 GB/s with the proof — **1.71x**,
within noise of a perfect-prover ceiling measurement, with the entry trap
still live (an under-sized buffer traps at the boundary before the first
byte is written; a separate C-ABI probe verifies this).

Now the comparison that makes this a language argument rather than a
compiler trick. A controlled adversary run (all variants full RFC semantics,
all enforcing the same entry relation, same machine, isolated processes):

| variant | throughput | vs xlang |
|---|---:|---:|
| xlang, obvious loop + checked `requires` | 4.285 GB/s | 1.000 |
| Rust, obvious indexed loop | 2.673 GB/s | 1.60x slower |
| Rust, obvious loop + `assert!` up front | 2.677 GB/s | 1.60x slower |
| Rust, expert `chunks_exact/zip` restructure | 4.297 GB/s | tie (0.997) |
| Rust, `unsafe` indexed | 4.111 GB/s | 1.04x slower |

Read the table twice.

- The folk remedy is measured dead: **`assert!` recovers nothing.** LLVM
  cannot connect a top-of-function assert to the coupled `i += 3, o += 4`
  induction, so every interior check stays. The thing that looks like a
  contract optimizes like a comment. xlang's `requires` differs precisely in
  that the *checker* makes the connection, deterministically, and reports
  which fact discharged which site.
- Expert safe Rust ties — say it plainly. A `chunks_exact/zip` restructure
  reaches the same check-free class. It is real skill: the writer must know
  the idiom, the shape does not generalize to variable-size output tokens,
  and nothing verifies the reasoning. The obvious shape stays 1.6x behind.
- Even `unsafe` buys nothing here — slightly worse than both.

That is the floor claim in one experiment: **in xlang the obvious shape plus
one checked line reaches the expert class; in Rust the obvious shape stays
1.6x behind and the honest-looking remedy does not work.** And the ceiling
tie is the honest half of the same story.

## 4. Effect rows the optimizer can trust across an opaque boundary

Every xlang signature declares its effects: `pure`, or a row of
`reads('r)`, `writes('r)`, `allocates(...)`, `traps`. The row is checked in
**both directions** — a function that exhibits an effect it did not declare
is rejected, and a function that declares an effect it does not exhibit is
*also* rejected. So a row is never aspirational; it is a verified fact. The
compiler lowers it to guaranteed attributes on the function *declaration*
(`memory(none)`, `nounwind`, `willreturn` where termination is derived), so
every call site optimizes against it **without seeing the body**.

Rust has no channel for this. An optimizer facing a Rust call must inline it
or prove its purity from the body; across a crate boundary or an `extern fn`,
there is nothing to prove from, and it must assume the worst.

The committed experiment isolates exactly that boundary. A pure mixing
function is compiled into its own object file — the caller's compiler never
sees the body:

```
fn mix (x: own i64) -> own i64 pure {
  let a: own i64 = imul.wrap<i64>(x, 2862933555777941757_i64);
  let b: own i64 = iadd.wrap<i64>(a, 3037000493_i64);
  ...
}

// caller: loop accumulating mix(k) with a loop-invariant argument
```

Without the declared effects (control build), the caller does what every C
and Rust compiler must do — two billion real calls:

```
LBB0_1:                          ; control: the loop survives
	mov	w0, #42
	bl	_mix                  ; a real call, every iteration
	add	x19, x0, x19
	subs	x20, x20, #1
	b.ne	LBB0_1
```

With the declared row on the *declaration*, the call is hoisted and the
accumulation strength-reduces; the loop is gone entirely:

```
_main:                           ; effects build: no loop left
	mov	w0, #42
	bl	_mix                  ; called once
	mov	w8, #49664
	movk	w8, #3051, lsl #16
	mul	x8, x0, x8            ; result * iteration_count
```

Measured: 1.47 s → 0.00 s — a complexity-class change, O(n) to O(1), across
a boundary no inliner can cross. The Rust comparisons frame it honestly:
Rust's ordinary cross-crate build (no LTO) runs the same 1.49 s; Rust with
**fat LTO ties us**, because with the whole program visible LLVM infers the
same facts.

So state the delta precisely: **xlang's per-file default equals Rust's most
expensive build configuration**, and the guarantee holds where inference
cannot reach — genuinely opaque objects, cached artifacts, foreign
boundaries with declared effects, and bodies too complex for the attributor.
This one is a build-economics result as much as a speed result: LTO-grade
cross-module optimization at ordinary `-O2` per-file cost, which is also
what keeps the AI writer's compile-check loop fast on large projects.

There is also an architectural payoff, developed in § 9: because effect rows
are checked, *architecture becomes greppable* — "exactly one function in
this system writes to the world state" is verifiable from signatures alone.

## 5. Ownership is the aliasing fact-base: guard-free vectorization

Two experiments, one mechanism: the borrow mode on a signature becomes
`noalias`-grade facts in the IR, automatically and non-defeasibly.

**First, the C-facing half.** The classic reduction —
`accumulate(acc, addend, n)` where the body folds `*addend` into `*acc` —
compiled from xlang borrows versus plain C pointers. Naive C must reload and
re-store through both pointers every iteration, because `acc` and `addend`
might alias:

```
_accumulate_naive:               ; C, no restrict
LBB0_2:
	ldr	x10, [x1]             ; reload *addend  (might have changed!)
	eor	x8, x10, x8
	mul	x8, x8, x9
	str	x8, [x0]              ; re-store *acc   (might be observed!)
	subs	x2, x2, #1
	b.ne	LBB0_2
```

xlang's `&uniq acc, & addend` emits `noalias`/`readonly` on the parameters
by construction, and the loop keeps everything in registers:

```
_accumulate:                     ; xlang, from the borrow modes
	ldr	x8, [x1]              ; load *addend once
	ldr	x9, [x0]              ; load *acc once
LBB0_2:
	eor	x9, x8, x9            ; register-only loop body
	mul	x9, x9, x10
	subs	x2, x2, #1
	b.ne	LBB0_2
	str	x9, [x0]              ; store once at exit
```

On the additive variant of this kernel the same fact changes the complexity
class: naive C keeps an O(n) loop behind a runtime alias check while xlang
collapses it to a single multiply-add — measured **≈22x** over unannotated C.
Honest label: C with a correct `restrict` and Rust's `&mut` both reach the
same code here — parameter-level no-aliasing is table stakes among the
fact-keeping languages. The difference at this rung is that C's fact is an
unchecked promise the programmer must remember and get right, while xlang's
is a checked consequence of the type.

**Second, the half Rust cannot reach.** Rust's `&mut Cols` gives LLVM
`noalias` on the *reference parameter* — but the `Vec` data pointers loaded
*through* it are fresh pointers the optimizer must treat as possibly
overlapping each other. That is exactly where real numeric kernels live:
multiple columns, loaded from one struct, read and written in one loop.

The committed kernel is a struct-of-arrays update — eight `u64` columns, two
written, six read, the completely obvious loop:

```
struct Cols {
  a: buffer<u64>;  b: buffer<u64>;  c: buffer<u64>;  d: buffer<u64>;
  e: buffer<u64>;  f: buffer<u64>;  g: buffer<u64>;  h: buffer<u64>;
}

fn kernel ['r] (s: &uniq 'r Cols) -> own unit reads('r), writes('r), traps {
  ...
  loop @l {
    ...
    let xa: own u64 = index<u64>(deref(s).a, i);
    let xc: own u64 = index<u64>(deref(s).c, i);
    ...
    set index<u64>(deref(s).a, i) = t4;
    ...
  }
}
```

Because `buffer` values are single-owner and `&uniq` loans are exclusive
with singleton provenance, the checker *knows* all eight columns are
pairwise-disjoint memory — so the compiler emits per-column alias scopes on
the loaded pointers themselves. The identical program in Rust (committed
alongside, same semantics, three shapes) gets vectorized too — but only by
**loop versioning**: LLVM emits a cascade of runtime pointer-overlap guards
and a speculative fast path. From the committed Rust assembly of the obvious
shape:

```
	cmp	x8, x7                ; column pair overlap test
	ccmp	x9, x24, #2, lo
	cmp	x8, x19               ; next pair
	ccmp	x10, x24, #2, lo
	cmp	x8, x20               ; next pair
	ccmp	x11, x24, #2, lo
	...                           ; 29 guards before the first vector op
```

The scoreboard for the same semantics, verified in the committed assembly:

| variant | vector ops | runtime alias guards | asm lines |
|---|---:|---:|---:|
| xlang, obvious shape | 8 | **0** | **121** |
| Rust, obvious shape | 65 | 29 | 2,132 |
| Rust, destructure-and-slice | 73 | 54 | 2,360 |
| Rust, expert inner-fn-with-8-slice-params | 14 | 0 | 499 |

And the honest timing story, all three parts of it:

- **Short trips are the real speed delta**: at trip count 8, xlang runs 2.0x
  Rust's obvious shape and 1.17x even the expert shape (which pays its extra
  call). Short-trip kernels called in outer loops — per-row updates, small
  fixed-width blocks — are common.
- **Long trips tie.** At trip counts ≥ 32 Rust's guards amortize and
  everything converges. Say so.
- **Code size is the durable win**: 121 lines versus 2,132 for the same
  large-n speed — 17x. Versioned-loop bloat is invisible in a
  microbenchmark and is instruction-cache pressure in a real program. At 16
  columns the Rust guards grow to 111 and the code to 2,836 lines; the xlang
  kernel stays at 183 lines with zero guards. The fact is static and O(1);
  the recovery is a runtime mechanism that scales with pointer count.

The safe-Rust escape (the committed `inner-fn` shape) is instructive: pass
all eight columns as separate slice arguments to an inner function, because
parameter-level `noalias` is the only aliasing channel Rust has. It works —
at the cost of being the kind of idiom you must already know, applied at
every such loop. In xlang the obvious shape *is* the fast shape at every
trip count, which is the property that matters when the writer is not
hand-tuning every loop.

One more consequence worth naming: `RefCell`, `Cell`, and every other
interior-mutability device do not exist here. That is not puritanism — a
single shared-mutable hole anywhere in the type system means every aliasing
fact becomes conditional. Because shared-xor-exclusive is absolute, the
facts hold universally. What replaces those idioms is an architecture
pattern, § 9.

## 6. Checked algebraic laws

A compiler is not allowed to reassociate your reduction. Floating-point and
saturating operations are not associative in general, so LLVM must preserve
your evaluation order, and your beautiful serial fold stays serial — one
`add` chained to the next, bounded by dependency latency, on every language's
output.

Rust experts know the workaround: hand-write the multi-accumulator loop. And
that workaround contains a hidden assertion — *the human is claiming the
operation is associative*, and nothing checks the claim.

xlang makes the law a checked, declared fact. The committed kernel:

```
contract SatMonoid {
  fn combine (x: own u64, y: own u64) -> own u64 pure;
  law associative(combine);
  law commutative(combine);
  law identity(combine, 0_u64);
}

fn satadd (x: own u64, y: own u64) -> own u64 pure {
  return iadd.sat<u64>(x, y);
}

conform u64 : SatMonoid {
  combine = satadd;
}

fn reduce (b: own buffer<u64>) -> own u64 traps {
  doc "The obvious reduction shape: sequential fold with the user op.";
  ...
  loop @l {
    ...
    set acc = satadd(x: acc, y: x);
    ...
  }
}
```

The writer states the laws; the checker discharges them against the
operation table's semantics (unsigned saturating add *is* associative, and
that is table data, not writer folklore); and only then does the optimizer
use them — reassociating the obvious fold into four independent,
block-interleaved accumulators seeded with the proved identity. From the
committed assembly, the four parallel saturating chains:

```
LBB1_3:                          ; reassociated: 4 independent chains
	adds	x8, x8, x15
	csinv	x8, x8, xzr, lo       ; saturate
	adds	x10, x10, x16
	csinv	x10, x10, xzr, lo
	adds	x9, x9, x15
	csinv	x9, x9, xzr, lo
	adds	x11, x11, x16
	csinv	x11, x11, xzr, lo
	...
	b.lo	LBB1_3
```

Measured at n = 65,536: 0.512 ns/element for the obvious fold in *either*
language (LLVM correctly refuses to reassociate what it cannot prove),
0.156 ns/element for xlang — **3.3x from stating a fact** — tying Rust's
hand-written four-accumulator expert shape (0.159). At short arrays the
per-call guard overhead shows and the expert shape leads (0.210 vs 0.155 at
n = 4,096); honesty in the small print.

Now the part that earns this section its place. The committed Rust
adversary says it in its own comment: the expert shape works only while the
human's algebra is right. Swap the operation for one that is *not*
associative — signed saturating add, where
`(MAX ⊕ 1) ⊕ -1 = MAX - 1` but `MAX ⊕ (1 ⊕ -1) = MAX` — and:

- **Rust compiles it silently.** The four-accumulator loop now computes a
  different function than the fold it replaced. No warning exists or can
  exist; the language has no idea the shape encodes an algebraic claim.
- **xlang rejects it at compile time.** A declared `associative` law on the
  signed op is *refuted* against the operation table with a rule-citing
  diagnostic (there is a conformance test pinning exactly this case), and an
  undischargeable law is a hard error — stated-but-unchecked never reaches
  the optimizer.

The transform every performance engineer does on faith is, here, a checked
fact. The honest mistake is unrepresentable, and the fast shape is the
obvious fold.

## 7. Keep boolean state boolean, and the vectorizer widens it

A smaller mechanism with an outsized codegen swing, and an honest framing:
this one is about *guaranteeing* a shape C and Rust also have — not beating
them.

Scanner kernels (word counting, token classification, run detection) carry
per-byte boolean state across loop iterations. Express that state as an
integer flag (`0`/`1` in a `u64`) and LLVM's vectorizer must carry a
full-width integer recurrence: it vectorizes at width 2. Express it as a
genuine 1-bit boolean combined with boolean operations, and the recurrence
stays `i1`: the same loop vectorizes at width 16.

The committed word-count kernel keeps every predicate in `Bool`:

```
    let c: own u8 = index<u8>(b, i);
    let ge9: own Bool = ige<u8>(c, 9_u8);
    let le13: own Bool = ile<u8>(c, 13_u8);
    let inrange: own Bool = band<Bool>(ge9, le13);
    let sp32: own Bool = ieq<u8>(c, 32_u8);
    let issp: own Bool = bor<Bool>(inrange, sp32);
    let notsp: own Bool = bnot<Bool>(issp);
    let starts: own Bool = band<Bool>(prevspace, notsp);
```

and the committed assembly shows the result — full 16-byte vector lanes
(`movi.16b`, `cmeq.16b`, `and.16b` …) where the integer-flag version of the
same logic capped at width 2×4.

Measured effect on the whole kernel: the integer-recurrence form ran
1.6–1.8x behind C and Rust; the `i1` form is at parity with both (the full
before/after tables are in the experiment record). Safe Rust can express
the same wide shape — this workload gives it no fact advantage — which is
exactly why the result is filed under *floor*, not *ceiling*: in xlang the
boolean-dataflow form is the taught, blessed pattern for scanner state, so
the writer lands on the 16-wide shape by default instead of discovering the
1.6x cliff in production.

---

# Part III — Making the fast shape the only shape

The sections above are fact channels: things the writer states and the
optimizer uses. This part is the complementary bet: **remove the shapes that
waste the facts.** The framing for an expert reader is not "you write bad
code" — it is that nobody hand-tunes the average line under a deadline, and
in a million-line codebase the average line is where the time goes. xlang
makes the slow shapes unrepresentable or unreachable, so the floor rises for
every line, not just the profiled ones.

## 8. One spelling, to the byte; overflow chosen in the name

There is exactly one legal spelling per construct and one legal byte-level
formatting — indentation, spacing, blank lines, all of it. The toolchain
**rejects** non-canonical input; it never reformats. There are no infix
operators and no precedence table. There are no comments (documentation is a
structured `doc` field). `match` is the only conditional; `loop` + `break`
the only iteration. And arithmetic overflow behavior is part of the
operation's name:

```
let s1: own u64 = iadd.wrap<u64>(a, b);     // wraps mod 2^64, by declaration
let s2: own u64 = iadd.trap<u64>(a, b);     // traps on overflow, always
let s3: own Result<u64, Overflow> = iadd.checked<u64>(a, b);
```

Three different operations, chosen per call site, with no default to forget.

Why an expert should care rather than wince:

- **Irregularity is the weak-writer failure surface.** What breaks smaller
  models is not verbosity — it is choice: alternate spellings, precedence,
  context-dependent elision, special cases. One spelling costs tokens
  (which the writer pays gladly) and buys zero ambiguity. The rule survives
  model scaling: it is about error surface, not context size.
- **Canonical bytes leave nowhere to hide.** Two programs differ if and only
  if their bytes differ. A "formatting-only" diff cannot exist; a sneaky
  edit cannot hide in one. Review, diff, and caching all sharpen (§ 11).
- **The overflow-mode split kills a real semantic divergence.** C and Rust
  ship two different programs from one source: Rust panics on overflow in
  debug and wraps in release; C invokes undefined behavior on signed
  overflow, and the optimizer *uses* that. In xlang, debug and release
  optimize the *same program* — every mode is spelled at the site, so there
  is nothing for the build type to reinterpret. `iadd.trap` also traps in
  release; `iadd.wrap` also wraps in debug. What you test is what ships.

The verbosity price is real and stated in Part VI. The purchase is zero
irregularity, and every mechanism in Part II leans on it.

## 9. A closed, taught catalog of architectures; data-oriented layout by default

At statement level, xlang forces one loop form and one conditional. The same
policy is applied at *architecture* level: the set of blessed program-scale
patterns is **closed and taught up front** — and the catalog is held to two
tests: it must be **complete** (every task is modelable inside it; a gap is
a documentation defect to fix, not a writer error) and **efficient** (each
pattern names the machine property or fact channel that makes it fast).

A human language could never do this — designers must let users carry their
familiar architectures in, or the language is rejected. An AI writer has no
architectural nostalgia. So the catalog can consist of exactly the shapes
the fact channels light up.

The flagship example: **the command-buffer pattern**, which replaces
scattered mutation. Deep code — the parser, the rules engine, whatever
lives at call depth ten — is declared `pure` or `reads('world)`: it
computes and *returns* write-intents as plain values. Exactly one shallow
function holds the single exclusive borrow of the world state and applies
the intents. In Rust this is a discipline you may adopt; here it is what
the checker's rules leave you: the deep code *cannot* manufacture write
access it was not handed, and the `Rc<RefCell<World>>` alternative — hand
out shared handles, mutate from anywhere, hope the panics find the overlap
— is unrepresentable.

Notice what enforcing it buys back:

- The aliasing facts of § 5 stay sound *because* mutation cannot scatter.
- Effect rows make the architecture checkable at a glance: grep the
  signatures — there is exactly one `writes('world)` in the system, and the
  checker verified every row (§ 4). "Where can state change?" becomes a
  query, not an investigation.
- Read-only deep code is precisely what makes future parallel fan-out safe.

Second example: **struct-of-arrays is the default layout** for bulk data
(§ 5's `Cols` is the shape as it looks in source). Contiguous per-field
columns are what the cache and the vectorizer want; per-node heap objects
with headers and refcounts are what object-oriented habit wants. The AI has
no such habit, so the language teaches SoA as the norm. And the catalog is
empirical, not ideological: where measurement shows the paired layout wins —
as it did for hash-table key/value slots, where splitting keys from values
doubled the cache misses per insert — the catalog pins *that* instead.
Patterns encode measured layout decisions.

The one-line justification for the whole part: **whatever is representable
will eventually be written.** A language that leaves the slow architecture
representable has scheduled its own performance regressions; one that
removes it has made the fast shape the path of least resistance — in fact,
the only path.

## 10. Handles and copies instead of references

The final floor mechanism is where long-lived data lives. The center of
gravity moves off borrows entirely: big structures live in **pools**;
ordinary code holds either the value itself (for small copyable data), or
ownership (moved in and out), or a **handle** — an index, plain copyable
data, a claim ticket rather than the coat.

Node links in a tree or graph are handles into the pool, not pointers or
references. Consequences, in both directions this document cares about:

- **Performance:** contiguous storage, no per-node allocation, no headers,
  no refcount traffic, indices that survive relocation. This is the layout
  discipline high-performance C eventually adopts by hand (entity systems,
  arena-indexed ASTs); here it is the default, and it composes with § 5's
  disjoint-column facts.
- **The writer's burden:** most code holds no loans at all, so the borrow
  rules bite only at the few sites that genuinely point into something.
  The self-referential-struct wall that pushes real Rust projects through
  `Pin`, `unsafe`, or index-arena workarounds does not exist — structs
  store values, not borrows, by construction, so the problematic program is
  not painful to write; it is impossible to state.
- **Safety of stale handles:** pool slots recycle with generation counters;
  a stale ticket presented after its slot was reused is a deterministic
  trap, never a silent read of the new occupant. (Honest note: the
  per-access generation check has a cost; check-free schemes — loans that
  freeze reuse for a scope, affine owned handles, proof-discharged repeat
  checks — are an active research track with the same rule as always:
  the check is removed by proof or not at all.)

A Rust engineer will recognize this as "just use indices into a `Vec`" —
the arena idiom Rust folklore already recommends *to escape its own borrow
checker* at exactly these sites. The difference is status: in Rust it is a
workaround with a footgun (any stale index silently reads whatever occupies
the slot now — a well-typed use-after-free); in xlang it is the blessed
pattern, taught up front, with the staleness hole closed by construction.

---

# Part IV — One program, one tree, one byte-form

## 11. The canonical-form dividend

Section 8's "one spelling to the byte" has consequences beyond writer
reliability, and they deserve their own section because they compound.

The facts, as the specification pins them: one spelling per construct and
one byte-level formatting (rejected, never reformatted); a one-to-one,
machine-enforced mapping between grammar productions and syntax-tree nodes;
accepted programs elaborate to a canonical artifact in which every derived
operation — drops, releases, instantiations, retained checks — is explicit,
and acceptance is decidable from the artifact alone; and diagnostics that
are deterministic and byte-stable, each rejection citing exactly one rule
and one tree path.

What that buys, labeled by evidence status:

- **Reproducibility (by construction, exercised).** Same source, same
  artifact, byte-identical binaries. The project's own gate demonstrates it
  at scale: the self-hosted compiler's optimized object is pinned by
  SHA-256 in continuous integration — a 264 KB binary reproduced to the
  byte.
- **Semantic diff and merge (by construction).** No formatting variance
  exists, so every diff is a semantic diff. There is no "reformatted, 2,000
  lines changed" commit, no style debate, and nowhere for an unintended
  edit to hide in noise. For AI-written code this is the review story:
  what changed is exactly what the diff says changed.
- **Caching and build speed (projected — labeled as such).** Source-to-tree
  is a bijection; the canonical artifact is deterministic; effect rows
  already decouple optimization from body visibility (§ 4 — the measured
  half of this claim). Those are precisely the preconditions
  content-addressed build caching wants, and they were designed in
  deliberately. The honest status: the design removes the classical
  obstacles to very fast incremental compilation; a build-speed headline
  number has not been measured yet.
- **The repair loop (by construction, exercised daily).** Deterministic,
  rule-citing, byte-stable diagnostics are an API, not prose. The writer
  that consumes them is a machine: same mistake, same message, same fix,
  every time. This is what "the AI can act on failure" concretely means.

The through-line: in most toolchains, formatting, diffing, caching, and
diagnostics are all *heuristic*, because the language admits many texts for
one program. Collapse that to one text, one tree, one artifact, and
everything downstream stops guessing.

---

# Part V — The trust story

## 12. Ten sealed building blocks — and a proof lane in

An expert's next question is the right one: *"Some structures cannot be
written in checked code in any language. A production hash table's reality
— 'one thousand slots, thirty-seven live, liveness tracked in side-band
control bytes' — is not expressible in your type system. Where's the escape
hatch?"*

There isn't one. There is a short list and a proof lane, and the
distinction is the point.

**The short list.** A small, fixed catalog — currently ten kernels: the
growable sequence, the hash table, the object pool, the arena, the
single-producer/single-consumer queue, file I/O, and peers — is implemented
with **trusted internals** by the toolchain itself. Inexpressibility is the
admission criterion, applied ruthlessly: a kernel enters the catalog only
if ordinary users *cannot* implement its capability at par performance from
the language and the existing checked libraries. Everything user-writable
ships as taught, checked source instead. The catalog is the language's
entire trust surface.

Contrast the shape of the risk. Rust's `unsafe` is the same admission —
"some invariants live outside the type system" — distributed as a keyword
to every writer, in every crate, forever; the ecosystem's soundness is the
union of every author's care. xlang concentrates the identical risk into
ten artifacts it owns, audits, and — critically — can *fix once* so the
world recompiles. No writer-reachable syntax escalates into the privileged
layer, so there is nothing to ban, lint, or review for.

**What "trusted" costs before shipping.** Each kernel must pass a five-part
acceptance battery — differential testing against the reference
implementation it replaces, exhaustive small-bound model checking,
sanitizer and fuzz soak, hostile pre-ship review, and a CI-pinned
performance and assembly shape — plus complete failure semantics and
teardown protocols with fault-injection evidence. Not "we were careful":
a ledger, green or the kernel does not ship.

The queue kernel shows the standard concretely. Its protocol was
exhaustively model-checked over all reachable states — and, as importantly,
the model *catches every mutant*: weaken any one of the four
acquire/release orderings and the checker exhibits a concrete violation
(a read of an unpublished slot, or an overwrite of an unconsumed one). The
implementation's steady-state hot path contains **zero read-modify-write
atomics** — verified in the committed disassembly: a plain load, a cached
compare, and one release store per operation — and it beats a mature Rust
SPSC crate on round-trip latency on the same machine (the same record shows
that crate ahead on batched throughput; both numbers are in Part VI).

**The proof lane.** The elegant part, and the project's long-term answer to
extensibility:

> **Trust is a temporary loan; proof buys permanent privilege.**

Any kernel whose representation invariants are later machine-proved leaves
the trusted list — same privileged representation, now *checked* code. The
trusted list only ever holds kernels not yet proved. And the same lane is
open, long-term, to users: performance never *requires* proof (the default
path is composing the sealed catalog), but a user structure that
machine-proves its invariants earns the same privileged representation
rights — uninitialized storage, elided internal checks — as checked code,
via deterministic proof checking, never via human review of the code.

This is one doctrine applied at every scale. A bounds check is removed by a
proof (§ 3) or it stays. A reassociation is licensed by a checked law (§ 6)
or it does not happen. A privileged representation is earned by a proved
invariant or it stays inside ten audited walls. Nowhere in the language
does "I promise" compile.

---

# Part VI — What it does not beat, and what is not yet known

Claims earn belief by naming their edges. The current honest ledger:

- **Expert safe Rust ties on the base64 kernel at full semantics** (0.997,
  controlled rerun, § 3). The durable delta there is the floor — obvious
  shape 1.6x behind in Rust, `assert!` recovering nothing — not the
  ceiling. Where a fixed-ratio restructure like `chunks_exact` exists, an
  expert who knows it reaches the same class.
- **Hand-tuned SIMD is still ahead of everything shown here.** Specialist
  kernels (`memchr`, bytecount-class scanning) beat the naive
  autovectorized shapes by 2–3x. Proof elision made the base64 kernel's
  *scalar* code perfect; it did not synthesize the wide-table SIMD
  algorithm. Vectorized blessed patterns are future work, not a present
  claim.
- **The language is verbose by design.** The base64 encode kernel is ~90
  lines where C writes ~15. The writer pays in tokens; humans read the
  contracts and the docs. If human writability is what you are optimizing,
  this is the wrong language, on purpose.
- **The frequency question is open.** The fact channels demonstrably win on
  kernels that exercise them. How often those patterns dominate real
  medium-to-large codebases is not yet established; an early survey attempt
  was directional at best. This is the biggest honest unknown in the
  performance story.
- **The sealed-kernel numbers are shape validations, not shipped-product
  benchmarks.** The catalog dry runs were C implementations of the
  specified kernel shapes against mature Rust baselines on one Apple
  development machine: sequence push-then-sum ~1.5x over `Vec`, table
  iteration ~1.4x over hashbrown, steady-state insert modestly behind,
  4 of 5 workloads inside the preregistered band; the queue beats `rtrb` by
  ~20% on round-trip latency while `rtrb` is ~25% ahead on batched-32
  throughput (both far above the band's floor). None of this is
  xlang-emitted code yet; magnitudes will be re-established on the deploy
  target.
- **Single-shot writability is not solved.** The current clean baseline for
  one research kernel is roughly a quarter of programs correct on the first
  attempt, with the failure modes catalogued and fixes staged; the design
  answer has always been the diagnostic feedback loop (§ 11), and the
  loop's measured effect is the next experiment, not a completed one.
- **Several announced mechanisms are selected designs under validation, not
  shipped features** — the ten-kernel catalog and the user proof lane among
  them. This document describes the design and the evidence gathered so
  far; the project's own gates block production claims until each
  validation closes.

---

# Part VII — If you don't write systems code at all

Everything above was for the reader who argues with compilers. This page is
for the reader who prompts.

When you ask an AI for software today, you inherit a bargain you never
signed: the language it writes was designed for human hands. If the AI
holds memory wrong, the program corrupts silently. If it picks a slow
architecture, nothing objects; you find out when your users do. Your only
defenses are testing what you can see and trusting what you cannot.

xlang changes what you are trusting. The AI writes; a checker — a program,
deterministic, with no goodwill to appeal to — accepts or rejects. The
categories of failure you cannot see are not found by the checker; they are
*unwritable in the first place*. Memory corruption, data races, silent
overflow: there is no sequence of tokens the AI could emit that means those
things and compiles. And the shapes that run slowly are, to the greatest
extent the design can force, also not on the menu — the fast layout and the
fast architecture are the only ones the language teaches and permits.

You still verify what the program *should do* — no checker knows your
intent. But "it quietly does something horrifying under load" comes off
your list. The slow version and the corrupt version won't compile.

---

# Appendix — evidence map

Every number above, with its committed record. Protocols, machines, and
caveats live in each RESULTS file; nothing here is quotable without them.

| claim | record |
|---|---|
| Proof tier: 27/27 bounds sites discharged, 2.48 → 4.23 GB/s (1.71x), entry trap retained | `experiments/port-study/base64/RESULTS.md` |
| Pre-proof checked loop (asm) | `experiments/port-study/base64/b64.s` |
| Proof-build IR (asm regenerable with `clang -O2 -S b64.ll`) | `experiments/port-study/base64/b64.ll` |
| Adversary table: xlang 4.285 GB/s; Rust obvious 1.60x; assert 1.604x; chunks tie 0.997; unsafe 1.040 | `experiments/port-study/base64/RESULTS.md`, `adversary_benchmark.py`, pinned CSV + metadata |
| Effect rows: 1.47 s → 0.00 s across opaque boundary; Rust no-LTO 1.49 s; fat LTO ties | `experiments/effect-attrs-channel/RESULTS.md`, `main_attr.s`, `main_plain.s`, `kernel.xl` |
| Naive C vs borrow-derived noalias; ≈22x additive variant; Rust parity | `experiments/codegen-vs-rust-c/SUMMARY.md`, `asm/kernelB_c.s`, `asm/kernelB_xl_acc.s` |
| Vectorization: 0 guards/121 lines vs 29 guards/2,132 lines; short-trip 2.0x; long-trip tie; 16-column scaling | `experiments/scoped-alias-channel/RESULTS.md`, `kernel.xl`, `kernel_facts.s`, `rust_kernels.rs`, `rust_kernels.s` |
| Checked laws: 3.3x over obvious fold; ties expert; signed-law refuted at compile time | `experiments/checked-law-channel/RESULTS.md`, `kernel.xl`, `kernel.s`, `rust_reduce.rs`; conformance case `fn4-neg-law-refuted-signedness` |
| Boolean dataflow: width 16 vs 2×4; 1.6–1.8x closed to parity | `experiments/port-study/wc-chunk-summary/RESULTS.md`, `chunk_wc.xl`, `chunk_wc.s` |
| Shipped-library floor: 1.653x [1.631, 1.667] vs `percent-encoding` 2.3.2; 1.098x [1.085, 1.145] vs `utf8parse` 0.2.2; bounds retained | `experiments/default-floor/RESULTS.md` |
| Kernel-shape dry runs (C mockups vs `Vec`/hashbrown; bands) | `optimizer-language-research/implementation/systems-performance-coverage/m3a-kernel-dryrun/RESULTS.md` |
| Queue: exhaustive model check, all 4 weakened-ordering mutants caught; zero-RMW hot path; latency vs throughput vs `rtrb` | `.../systems-performance-coverage/m6a-spsc-dryrun/RESULTS.md` |
| Reproducibility: whole-compiler object SHA-256-pinned | `THE-PLAN.md` (build-track record) |
| Language rules cited (one spelling; reject-not-reformat; two-way effect checking; `requires` semantics; overflow op names; trap = abort) | `spec/kernel-spec-v0.6.md` — FORM-1/2/3, EFF-1/2/4, FN-8, OP-1 |
| Pattern doctrine (command buffer, SoA pool, boolean classifier, traps-to-boundary) | `PATTERNS.md` |
| Founding evidence for the premise (escape analysis conditionality, JIT recovery machinery, non-interference as the central enabler, IR semantics preservation) | `optimizer-language-research/notes/verified-findings.md`, `notes/phase2-jit-findings.jsonl`, `archive/research/debates/round1-static-vs-profile.md` |
