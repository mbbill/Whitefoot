# L1 is two amendments, not one

Measured 2026-09-05 against the v0.45 compiler, by building the smaller half
and running it. The patch that produced these verdicts is
`admit-exact-clause-rows.patch`; it is recorded here and deliberately not
merged.

## What the smaller half does

`requires.rs`'s clause validator rejects any operation whose `is_exact()`
holds. Replacing that with an allow-list of `AddExact | SubtractExact |
MultiplyExact` — the rows that have a total meaning over the mathematical
integers, which is the carve-out `[INV-1]` already gives an `affine_expr` —
is 22 lines. Division, remainder, the shifts, negation, and absolute value
stay out: each has an input a relation cannot state its way out of.

With that patch the expansion codec's precondition **forms**, where at v0.45
it is `[FN-8] InvalidRequires` at formation:

```wf
define room = len(deref(out));
define count = len(src);
define needed = count * 2_u64;
requires room >= needed;
```

## Where it stops

The clause forms, instantiates at the caller, and is refused there:

```
[FN-8] UndischargedCallRequirement {
  concrete_callee: "hex_encode",
  instantiated_goal: "len(out) >= len(src) * 2_u64",
  disposition: Unproved }
```

The call site is `buffer_new(8_u64, …)` and `buffer_new(4_u64, …)`, so the
goal reads `8 >= 4 * 2` and is true. It is unproved because neither fact
domain can hold it:

- L0 is a two-term difference bound, which cannot carry a coefficient.
- The affine layer has no measure atom. `affine_term_value`
  (`compiler/src/semantic/entailment/flow.rs:6841`) images `Zero`, `Constant`,
  and unprojected binding places; `TermKind::Length` and
  `TermKind::ProjectedLength` fall to the `None` arm. The specification says
  the same thing at `spec/kernel-spec.md:1429` — a measure term "is a clause
  operand and **not yet** an affine atom".

So the smaller half alone ships a surface a writer can state and no caller can
ever discharge. That is the failure mode the contract scoping predicted:
widening a written surface before the fact base can hold what it states turns
a clean rejection into an unsatisfiable obligation. It is not silent — the
instantiated goal is printed and the diagnostic is honest — but the writer is
still handed a precondition that cannot be met.

## What L1 therefore requires

Both halves in one amendment:

1. the exact affine rows admitted in clause position, read mathematically; and
2. `len(P)` admitted as an affine atom, so `len(src) * 2_u64` normalizes and
   the goal reaches a fact domain that can carry it.

The second is the larger of the two and is what `spec:1429`'s "not yet"
defers. It needs a stable affine atom per length term, an `affine_term_value`
image for it, and the `[ENT-5]` support rule it already has — a length fact is
supported by the root binding of its place, and an element write never kills
it — carried into the affine layer unchanged.

Order still holds: the fact base is fed before the surface widens. L0 fed it
one kind of fact; this is the second.
