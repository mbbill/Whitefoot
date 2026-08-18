# O11 signed Boolean decomposition — exact candidate delta text

Candidate text for the single v0.31 candidate; the lead integrates it,
and it lands only under the owner's exact-byte approval carrying the
ruling-(2) conformance rewrite in the same sitting. Anchors quote the
active v0.30 bytes (SHA
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`).
No grammar production changes; the grammar verifier has nothing new to
verify. Design and soundness: `DESIGN.md` beside this file.

## D1 — [ENT-3]: the rule, stated once (new paragraph)

Insert after the goal-origin paragraph ending "Clause-local expansion in
FN-8 is unconditional because the admitted block contains no mutation."
and before "The sources are:":

> Signed Boolean decomposition applies at every establishment of a signed
> goal fact by the sources below.
> The decomposition set of `+G` whose complete root is `band(A, B)` is
> `+A` and `+B` together with each member's own decomposition set; the
> decomposition set of `-G` whose complete root is `bor(A, B)` is `-A`
> and `-B` together with each member's own decomposition set; the
> decomposition set of `+G` or `-G` whose complete root is `bnot(A)` is
> respectively `-A` or `+A` together with that member's own decomposition
> set; a `bxor`, `eeq`, `ene`, comparison, datum, or non-Boolean root has
> the empty decomposition set — in particular `-band` and `+bor` carry
> only genuinely disjunctive content and establish nothing about a child.
> When a source establishes `+G` or `-G`, it establishes every member of
> that signed decomposition set at the same point; each member is one
> concrete goal under [FN-8]'s structural identity, and each member whose
> complete root is one comparison call admitted by comparison-origin
> shape (a), whose operands are each an admitted term, constant, or
> `len(P)` length term, independently establishes that exact relation
> under `+` and the relation's exact L0 negation under `-`.
> A member's support is the ordinary [ENT-5] signed-goal support of its
> own complete typed expression; kill events, scope exits, joins, and the
> loop rule apply to each member independently of its parent.
> Decomposition is a finite structural walk of the established goal's
> tree: it performs no algebraic rewrite and no children ever establish
> or derive a parent.

## D2 — [ENT-3.S4]: replace the no-child sentence

Replace:

> No child of any other goal is established.

with:

> Beyond that projection, only the members of G's signed decomposition
> set and their projections are established; no other child of any goal
> is established.

## D3 — [FN-8], the indivisibility pair (v0.30 L1184–L1185)

Replace:

> In particular a complete `band`, `bor`, `bxor`, or `bnot` tree is one
> indivisible goal: evidence for its children establishes nothing about
> the whole, and evidence for the whole establishes nothing about a
> child.
> When the complete root is exactly one [ENT-3] comparison relation over
> admitted [ENT-2] terms or constants, [ENT-4] may additionally derive
> that one goal through its exact L0 projection; no Boolean subtree
> receives such a projection.

with:

> In particular a complete `band`, `bor`, `bxor`, or `bnot` tree is one
> goal that no evidence for its children ever composes: discharging the
> whole requires the exact whole tree, while an established whole
> additionally establishes exactly its [ENT-3] signed decomposition set.
> When the complete root is exactly one [ENT-3] comparison relation over
> admitted [ENT-2] terms or constants, [ENT-4] may additionally derive
> that one goal through its exact L0 projection; a Boolean subtree
> projects only as an established member of a signed decomposition set,
> never toward its parent.

## D4 — [FN-8], the body-entry sentence (v0.30 L1199)

Replace:

> The function body is checked with its one complete requirement goal
> established true as [ENT-3] source S4, together with the exact L0
> relation only when that complete root has the projection above.

with:

> The function body is checked with its one complete requirement goal
> established true as [ENT-3] source S4, together with the members of its
> signed decomposition set, and with the exact L0 relation of the
> complete root or of a member only where that root has the projection
> above.

## D5 — [ENT-4], the opaque-component pair (v0.30 L2758, L2760)

Replace:

> The opaque component retains exactly the established signed facts and
> receives no closure, decomposition, composition, or implication rule.

with:

> The opaque component retains exactly the established signed facts —
> Boolean decomposition happens at [ENT-3] establishment, never here —
> and receives no closure, composition, or implication rule.

Replace:

> Deriving the two children of a Boolean operation never derives its
> parent, and deriving the parent never derives either child.

with:

> Deriving the two children of a Boolean operation never derives its
> parent, and derivability never decomposes: only an established parent
> establishes its members, at its establishment point.

## D6 — [ENT-3], the origin exclusion sentence (v0.30 L2679)

Replace:

> No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`,
> user-function results, and deeper indirection chains contribute no L0
> comparison origin in this version.

with:

> No other shape has one: `band`, `bor`, `bxor`, `bnot`, `eeq`, `ene`,
> user-function results, and deeper indirection chains contribute no L0
> comparison origin in this version; an established Boolean goal
> contributes relations only through the members of its signed
> decomposition set.

## D7 — [CLM-2], the worked example (v0.30 L2578)

Replace:

> Redundancy and refutation are judged only for a predicate with
> comparison origin [ENT-3]; a conforming claim whose predicate has none
> — a constructed `True()`, a `band` result — is neither redundant nor
> refutable, is accepted, and traps whenever it evaluates false at
> runtime, exactly as today's `check` on the same expression.

with:

> Redundancy and refutation are judged only for a predicate with
> comparison origin [ENT-3]; a conforming claim whose predicate has none
> — a constructed `True()`, a `band` result — is neither redundant nor
> refutable, is accepted, and traps whenever it evaluates false at
> runtime, exactly as today's `check` on the same expression, even though
> the passed claim establishes the predicate's signed decomposition
> members [ENT-3].

## Impact inventory pointer

Acceptance widens (Boolean-guard discharge) and narrows (claim
refutation and redundancy against member projections); the v0.30 corpus
flip set is exactly `ent3-neg-bor-no-comparison-origin.wf`, disposed by
the 2026-08-09 ruling (2). Details and the corpus sweep: `DESIGN.md`.
