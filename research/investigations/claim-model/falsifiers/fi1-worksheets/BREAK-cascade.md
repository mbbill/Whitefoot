# The repaired `[IND-4]` backward pass has a prover-dependent traversal

Re-verification of 767c4823 on `batch/0106-claim-model-design`.
Verdict: **REFUTED**. `[ENT-1]` monotonicity is still false, by a mechanism the
repair itself introduced.

## The two sentences that collide

DESIGN.md 1843-1847 (`[IND-4]`, the F2 repair):

> replacing, at each `let` or `set` commit whose destination occurs **in the
> polynomial or in any hypothesis this pass has already introduced**, at the
> moment the pass reaches that commit, that destination by the polynomial of the
> commit's right-hand side.

DESIGN.md 1860-1871 (`[IND-4]` clause (b), the F7b repair):

> a **fresh opaque witness term** `o` for the destination, together with ... and
> - **when the state at that commit derives the corresponding exact row's
> `[OP-2]` no-wrap side condition** - the two further hypotheses `o - P <= 0`
> and `P - o <= 0`, where `P` is the exact polynomial of `a op b`.

`P` names the operands. So the operand terms enter "the hypotheses this pass has
already introduced" **only when the prover derives the side condition**. The
visit set of the backward pass is therefore a function of the ambient prover,
not of the path's text.

Every claim built on the opposite is false:

- 2119-2121: "**The cap is on the slot count, and the slot count is syntactic**"
- 2175-2177: "(ii) *the slot list is unchanged, and no slot is emptied.* Slot
  positions are syntactic, so neither cap can be crossed."
- 2169-2173: "(i) ... The only `[IND-4]` decision a strengthening can flip is
  clause (b)'s refusal on a `set` destination, which it flips **from refusing to
  admitting**."
- 2075-2076: "it is the only `[IND-4]` refusal a prover strengthening can lift"
- 4685-4689 (D1): "its bounds are counts of slots and terms fixed by the
  program's own text rather than by what a prover can derive"

## Attack N1 (soundness of the theorem, not of the language): a strengthening
## turns clause (e)'s refusal ON

```whitefoot
let x   = 0_u16;
let acc = 0_u8;
for @l i in 0_u64..n {
  bound @l s: ile(x, 255_u16);
  set acc = load(buf, i);        // C1  set destination, RHS a call -> clause (e)
  let y   = acc +wrap 7_u8;      // C2  let destination, wrap      -> clause (b)
  let z   = cvt<u8, u16>(y);     // C3  widening cvt               -> clause (c)
  set x   = z;                   // C4  copy                       -> clause (c)
}
```

The statement is not a type bound: `x` is `u16`, so `x <= 255` is a real fact,
and `[IND-8]` publishes `x - Z <= 255` on both header edges.

BASE `[IND-5]`: preheader `x = 0`; `p = x - 255`; group 1 empty (only statement);
`RELAX = 0 - 255 = -255`; `floor(-255/1) <= 0`. Verified.

STEP `[IND-6]`, checker v0.40 (weakest conforming prover: type bounds only).
Backward pass, end to entry:
  C4 `set x = z`   : `x` in p, clause (c) copy       -> p = z - 255
  C3 `let z = cvt` : `z` in p, clause (c) widening   -> p = y - 255
  C2 `let y = acc +wrap 7_u8` : `y` in p, clause (b). Destination is a `let`
     binder, so the refusal branch does not apply. Witness `o`; p = o - 255.
     Hypotheses: the two constant-bound slots on `y`, filled from u8 as
     `o - 255 <= 0` and `Z - o <= 0`. The `+wrap` side condition needs
     `acc - Z <= c1`, `7 <= c2`, `c1 + c2 <= 255`; the only bound on `acc` is
     u8's own 255, and 255 + 7 = 262 > 255, so it does NOT derive and the two
     equality slots stay EMPTY.
  C1 `set acc = load(buf, i)` : `acc` occurs neither in p (= `o - 255`) nor in
     any introduced hypothesis (all four mention only `o`). **NOT VISITED.**
p0 = o - 255. Elimination terms: {o}, one.
H slots: (1) H1 = x - 255; (2) 2 constant bounds (filled) + 2 equality slots
(empty); (3) one term, so no ordered pair. Five slots.
Certificate sigma(o) = the slot `o - 255 <= 0`: a = +1, b = +1, a*b > 0,
p := 1*(o - 255) - 1*(o - 255) = 0, s = 1. RELAX(0) = 0, floor(0/1) = 0 <= 0.
**Verified. The program compiles at v0.40.**

Now the strengthening. v0.41 adds any fact source that derives `acc - Z <= 100`
after C1 - a sharper interval domain, or propagating `load`'s existing
`ensures ile(result, 100_u8)` that v0.40 dropped. This is monotone: strictly
more facts derivable, none fewer, and it is exactly the class 2166-2167 defines.

STEP `[IND-6]`, checker v0.41. Same pass down to C2, but now 100 + 7 = 107 <= 255,
so the side condition derives and clause (b) adds
  E1 = `o - acc - 7 <= 0`,  E2 = `acc + 7 - o <= 0`.
`acc` now occurs in a hypothesis this pass has already introduced, so the pass
**reaches C1**. C1's right-hand side is a call, which is clause (e), and C1's
destination is a `set` destination. Clause (e), last sentence:

> If the destination is a `set` destination, the substitution **refuses** the
> statement, with a diagnostic naming that commit.

**HARD ERROR at `[IND-1]`.** A program v0.40 compiled does not compile on v0.41.
That is the exact sentence `[ENT-1]` forbids and the one 2191-2193 asserts:
"**Therefore no fact-source or closure strengthening can refuse a statement an
earlier conforming checker verified**".

No cap is involved; the break is in Monotonicity clause (i), whose case analysis
of "the only `[IND-4]` decision a strengthening can flip" omits the traversal.

## Attack N2 (the cap semantics): the count-crossing scenario the repair asked for

The same mechanism crosses the 32-slot hard error, so the repaired cap is not
syntactic either. Chain the wrap commits on `let` binders so no clause (b)
refusal intervenes:

```whitefoot
let x = 0_u16;
for @l i in 0_u64..n {
  bound @l s: ile(x, 255_u16);
  let a1 = seed +wrap 7_u8;
  let a2 = a1   +wrap 7_u8;
  ...
  let a9 = a8   +wrap 7_u8;      // nine commits
  let z  = cvt<u8, u16>(a9);
  set x  = z;
}
```

v0.40: the pass reaches `a9`'s commit, takes clause (b)'s witness `o9`, fills the
two u8 constant-bound slots, leaves the two equality slots empty, and stops -
`a8` occurs nowhere. Slot count = 1 (statement) + 2 + 2 = **5**. The certificate
`sigma(o9) = (o9 - 255 <= 0)` gives `p := 0`. Verified.

v0.41 derives `a_j - Z <= 100` at each commit. Every side condition now derives
(107 <= 255), each equality pair names the next operand down, and the pass walks
all nine commits. Each contributes 2 constant-bound slots + 2 equality slots.
Slot count = 1 + 9*4 = **37 > 32**.

> `H` has at most **32 slots**, and more is a hard error naming the statement.

Same program, same text, more derivable facts, hard error. Acceptance is **not**
preserved, which is what this attack was asked to show and it does not.

Note what the elimination-term cap does: it survives. The cascaded witnesses
enter only the hypotheses, never `p`, so `p` and its degree-1 monomials really
are a function of the path's text. F7b (the 4-term cap) is closed. F7a (the slot
cap) is not, and it is not closed by counting slots syntactically, because the
set of witness-introducing commits is itself prover-dependent.

## Where the break came from

At 236b837f the pass rewrote only "the polynomial", and clause (b) substituted
the exact polynomial when the side condition derived. That put the operands into
`p`, which is F-I1's F7b: a prover-dependent elimination-term count. The repair
moved the operands out of `p` and into the hypotheses - and simultaneously made
the pass follow the hypotheses (the F2 repair, which I1's midpoint genuinely
needs, since `span` is rewritten only because it stands in the division
witnesses). The two repairs are individually right and jointly re-create the
same prover-dependence one level down.

A third instance of the same mechanism, not needed for the refutation:
`[IND-6]` clause (i) says "a path condition whose substitution refuses is dropped
rather than refusing the statement", and a path condition's substitution also
introduces witness terms. Whether it refuses is prover-dependent through the same
clause (b) route, so the witness-commit count moves with the prover there too.

## What a repair would have to do

Make the equality pair unconditional in *shape*: introduce `o - P <= 0` and
`P - o <= 0` as slots whose terms are always present (empty or filled by the side
condition), so the pass's visit set is fixed by the path's text; or bound the
traversal to the polynomial's own terms plus the witness pairs of clauses whose
introduction is unconditional (d), and accept that clause (b) contributes no
relational hypothesis at all. Either way, the sentence "the slot count is
syntactic" has to become true of the *visit set*, not only of the slot list at a
fixed visit set.
