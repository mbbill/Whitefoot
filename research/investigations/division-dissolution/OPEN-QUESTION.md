# Open question — should an out-of-class division site demand a claim?

Raised by the owner, 2026-08-18, reviewing the v0.32 explanation page.
Status: open design question for a v0.33 candidate. No specification byte
changes on this record.

## What v0.32 does

A bare `/` or `%` over a signed selected type whose two operand atoms are
both non-constant is **outside** the divisor class. No obligation attaches
and the site keeps its complete runtime trap.

The delta's stated ground is that the retained safe condition
`dividend != iK::MIN or divisor != -1` is a disjunction, which [ENT-4]'s
closure can neither state nor derive, and [ENT-6] additionally requires
each conjunct to be one atomic relation over terms — so the obligation is
not merely undischargeable, it is unstatable in the obligation language as
that rule currently defines it.

## The gap in that reasoning

The argument establishes that the *closure* cannot prove the condition. It
does not establish that nothing may be *demanded of the writer*, and the
delta never argues the second step — it is skipped rather than reasoned.

Discharge has two routes, not one: closure derivation, and exact goal
matching. [ENT-3.S3] establishes each goal in a claim predicate's
goal-origin set with positive sign. If the division obligation's goal were
the whole `bor(ine(n, min(T)), ine(d, -1))` expression, a claim written
over exactly that expression would establish exactly that goal, and exact
goal identity would discharge it — with no disjunction ever entering the
conjunctive fact state, and with [ENT-3]'s refusal to decompose a positive
`bor` remaining correct and irrelevant (the goal is used whole).

So the real choice was:

- **(a), taken by v0.32:** no obligation, silent trap, zero writer burden.
- **(b), not considered:** a goal-matched obligation, dischargeable only by
  an exactly-matching claim or a dominating branch on the same expression.
  The retained runtime test moves from the operation's anonymous internal
  check to a named, justified, accountable, refutable claim.

## Evidence bearing on the choice

- The whole live tree holds **three** division sites (two constant-divisor
  decimal-formatting sites in `tests/programs/byte_string.wf`, one signed
  two-variable conformance case). Under (b) the migration cost is near
  zero, which removes the usual "writer tax" objection.
- The recorded measurement that real programs choose wrapping arithmetic
  228:30 argues the same way: the trapping sites are few.
- The one substantive objection is consistency with the constant-operand
  overflow family, whose out-of-class sites (two-variable `a + b`) are the
  same shape. But their safe condition, written as a predicate, itself
  requires trapping arithmetic — the shapes are not symmetric, which is an
  argument for judging them separately rather than for copying (a).

## What would settle it

An owner ruling on whether the trap thesis means "the writer is asked for a
justification wherever proof does not reach" or "the writer is asked only
where the obligation language can express the goal". The first reading
selects (b) for division; the second keeps (a) and should be stated in
[OP-2] as a deliberate boundary rather than left implicit.
