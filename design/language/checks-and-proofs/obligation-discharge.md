Decision: Acceptance of a partial operation is defined by the specification's entailment rules alone, because one authority keeps optimizer results and runtime origin from becoming independent sources of proof, instead of optimizer-derived or runtime-derived facts.

Decision: Postcondition summaries publish in call-graph component order and are unavailable while their own component is being checked, because a recursive component must not bootstrap itself from a summary it has not yet earned, instead of assuming a callee's postcondition while checking its component.

Decision: Every step of a local certificate reads the same entering context and publishes nothing; only the checked outer invariant becomes a fact, because one flat weighted combination is checked without any search over step order, instead of cumulative steps that build on each other.

Decision: An unproved required operation rejects at compile time with a diagnostic naming the obligation and the missing evidence, because an unproved domain must never lower, instead of an implicit retained runtime check.

Decision: An integer-typed named const is an affine atom folded at formation to the one value it declares, because a limit declared once must mean that number in the relations written about it or a stale inline digit silently diverges code from proof, instead of excluding named consts from the affine fragment.

Rejected:
- Implicit retained checks on every unproved obligation: rejected because the trap surface was neither stated nor enumerable, a caller could not tell from a signature when a callee would trap, and the saturating traps bit hid every unproved obligation until it fired.
- Result-everywhere, where every fallible operation returns a value: rejected because each checker-incompleteness site would force an error arm for a condition that is impossible when the code is correct, and a catchable internal error lets a stuck writer swallow a violated invariant.
- Global prove-or-handle as language law: rejected because a deterministic no-search checker leaves a true-but-unprovable residue that would be camouflaged inside genuine fallibility.
- Assume-without-check, a writer-stated fact reaching the prover with no proof: rejected because no construct may introduce a fact without proof.
- A total-access operation beside the ordinary index: rejected because under caller-side discharge the value branch already is the total access, so the second form served nothing.
