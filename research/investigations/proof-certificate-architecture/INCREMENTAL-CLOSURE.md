# Incremental L0 closure

## Question

After the [flow-analysis follow-up](CHECKING-COST.md#flow-analysis-closure-follow-up),
`tests/programs/wfgrep.wf` and `tests/programs/fixed_run_library.wf` each still
take about 24 s to check. A whole-run sample of wfgrep puts about 66% of
semantic samples in the complete closure and 11% in its proof-free
contradiction twin. The remainder is mostly rebuilding relation maps around
those closures.

Every kill, predecessor join and query that follows a relation change
recomputes the complete cubic fixed point. It does so even when nearly the
whole state is already a closed difference-bound matrix from the previous
materialization or join. This investigation asks how much of that work
disappears when the fixed point starts from the part of the state that is
already closed.

## Rule boundary

The [ENT-4 rule](../../../spec/kernel-spec.md) fixes the least closure and
allows incremental computation, provided every derivability and disposition
answer equals the least-closure answer. For L0 bounds it does not fix which of
several equal-bound derivations is retained. The minimum-depth sentence
governs the opaque-goal reconstruction.

So this work may change:

- which equal-bound L0 derivation a cell retains;
- the ledger node inventory and its identities;
- diagnostic parent choice.

It must not change:

- any L0 bound value, disequality or contradiction;
- any opaque-goal answer or obligation disposition;
- acceptance, diagnostics' rules and locations, or emitted LLVM.

Every retained derivation must still pass the existing derivation validation.

The live [closure-row-dominance](../../../design/compiler/closure-row-dominance.md)
decision promises unchanged selected derivations. A selected implementation
therefore needs an owner-approved revision of that promise before merge. The
owner has stated the direction: relax it for a large speedup. That statement
is not approval of a specific revision.

## Mechanism

A fact state records how much of its bound matrix is known closed:

- **unknown**: no claim;
- **closed** over a term count;
- **closed core**: closed over a base term count, except for listed fresh
  cells and fresh terms.

Operations update the record:

- A materialization or join produces a closed state.
- Adding a bound makes its cell fresh.
- Adding a disequality makes both orientations fresh.
- An endpoint kill removes the killed terms' cells and marks those terms
  fresh, because their implicit facts hold again immediately.
- Terms registered after the base count are fresh.
- Removing an S12 proof candidate can weaken a selected bound without a
  matching projection, so it resets the record to unknown.
- Removing signed goals leaves the L0 matrix unchanged.

Removing whole rows and columns from a closed matrix leaves the survivors
closed among themselves. The removed terms' implicit edges composed with
closed survivor paths cannot improve a survivor-to-survivor bound. Each such
path enters and leaves a removed term only through edges the closed matrix
already accounted for.

The closure engine is unchanged except for its seed:

- Stale cells start not fresh, so round one visits only triples touching a
  fresh cell.
- In seeded mode a candidate replaces a cell only with a strictly smaller
  bound. The disequality rule adds only absent pairs. A strictly smaller
  bound can reach a stale cell only through a fresh one, which the semi-naive
  rounds follow.
- Equal-bound, shallower candidates are no longer accepted. Accepting them
  would rewrite stale proofs across the whole matrix and restore cubic work.

The unseeded closure keeps its current acceptance, so states with an unknown
record retain today's derivations. The proof-free contradiction probe uses
the same seed.

## Alternatives

- A separate incremental difference-bound implementation with explicit
  per-edge O(n²) updates duplicates the closure rules. It also needs its own
  kill, strengthening and contradiction logic. Reusing the engine with a seed
  keeps one rule implementation.
- A cache keyed by complete state content only helps exact repeats. After
  generic validation scope, repeats are about 1–5% of closure time.
- Persisting a dense matrix inside the fact state, replacing the relation
  maps, would also remove map rebuilds. Measure first whether the seed alone
  leaves map work as the dominant cost.
- Changing ENT-5 so a kill does not materialize would change language rules
  and is outside this investigation.

## Selection criteria

These are recorded before implementation timing.

**Correctness.**
- A test-only verification switch recomputes every seeded closure unseeded on
  a cloned ledger. It must then find identical bound values, disequality
  sets, contradiction flags and goal-contradiction results. The switch runs
  over the real-program bundles `utf8parse.wf`, the raw DEFLATE chain,
  `wfgrep.wf` and `fixed_run_library.wf`.
- The ordinary compiler, program and conformance suites pass. Any changed
  test expectation must be a ledger-shape or derivation-choice expectation,
  listed with its reason, never a verdict, rule, location or runtime result.
- Base and candidate emit byte-identical LLVM for the paired sources.

**Performance.** Use five warmed alternating pairs against the PR #68 head
compiler, on the paired-source set of the flow follow-up. Select only if
all of these hold:
- wfgrep and fixed-run each improve by at least 3x in median time;
- no protected fixture or real program regresses by both more than 10% and
  more than 1 ms.

Sub-second checking is the owner's target and is reported whether or not it
is reached.

If the seed misses 3x, attribute the remainder before choosing a persistent
dense representation.
