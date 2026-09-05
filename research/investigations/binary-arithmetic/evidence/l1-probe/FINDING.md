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

---

# Second attempt: the measure atom, built and still short

Built the second half against v0.45 rather than reasoning about it further.
The patch is `measure-as-affine-atom.patch`, recorded and not merged. Every
piece below compiles; the motivating case still does not discharge.

## What was built

1. `AffineFlowState.length_values: HashMap<BindingId, AffineForm>` — the
   stable atom for the measure of an unprojected place, keyed by that place's
   root binding rather than by an interned measure term, so a read can resolve
   it without interning anything.
2. Kill: the entry is dropped by exactly the events that drop the binding's
   own image, which is what re-mints the atom. This mirrors the existing
   comment on `apply_affine_kills` — facts name the immutable atom, never the
   map, so a replacement value cannot match an image published before the
   write. A measure survives an element write and dies with a write to the
   root [ENT-5], which is what keying on the root gives.
3. Join: keep the atom where every input agrees, drop it otherwise. A measure
   is not arithmetic-updated, so there is no spread for a join delta to stand
   for; disagreement means a branch replaced the object.
4. Scope exit: dropped with the binding.
5. Mint: `install_measure_atoms` walks the instantiated call goal in the
   `&mut` window before the proof context is built, so the read path stays a
   pure `&self` read.
6. Read: `affine_goal_value` images a `BufferLength`/`SliceLength` over a
   place spelled bare or through one `deref` — `length_term` derives that
   `deref` from `is_holder`, so both spellings key the same measure.
7. Bridge: `affine_l0_candidates` pairs each measure atom with its interned
   length term, so L0 bounds established at creation tighten an atom minted
   as an unknown u64.
8. The clause allow-list from the first attempt, so the contract forms.

## Where it still stops

Unchanged:

```
[FN-8] UndischargedCallRequirement
  instantiated_goal: "len(out) >= len(src) * 2_u64"
  disposition: Unproved
```

The affine route is wired for call goals — `call_goal_disposition` builds an
`affine_goal_ordering_target` and passes it as `ProofGoal::Signed`'s affine
target — and `affine_goal_value`'s `MultiplyExact` arm already scales a form
by a literal on either side. So the shape the goal needs exists on both ends.

What is not established is whether the measure atom is *found* at that point.
The next thing to determine, and the reason this stops here rather than
continuing: `admitted_call_goal_expression` substitutes each formal with the
actual before the goal is judged, and the actual here is a borrow expression.
If substitution leaves the measured operand as something other than a
`GoalDatum::Place` — an `EvaluatedValue`, or a place carrying the borrow's own
projections — then both the mint and the read skip it and every piece above is
correct and unreachable. That is one instrumented run to settle, and it should
be settled before any more of this is written.

## Reading

L1 is not a follow-on to L0. L0 published a fact the checker had already
proved, into a domain that already held that shape. L1 introduces a new kind
of affine atom, and each layer it touches — kill, join, scope, substitution,
the L0 bridge — is a place where the atom's identity has to mean the same
thing. The eight pieces above are a plausible design and they are not
evidence that the design is right; the case that would be evidence is still
refused.
