# Loop-shaped permission — the batch 0078 design

Lead synthesis over five research dossiers (spec rule, checker mechanism,
lowering, adversarial soundness, prior art and the decision record) and one
value falsifier, all executed against `27e02b1f..4a005b1f`. The mortal
research corpus lives in the lead's scratch; every load-bearing claim below
is restated with its evidence, and the probes this design depends on are
promoted beside this file.

> **Superseded in one place by a later landing on the same branch, and
> corrected in place.** This ruling was written against the v1 doctrine in
> which eligibility required a claim-free call closure. **That condition was
> withdrawn on 2026-08-23** by the owner's chartering direction of that day,
> the same direction the parent `../DESIGN.md` records at its §1 bullets, and
> `f6c55a9d` deleted it from both judgments. A `claim` in a loop body is now
> an ordinary statement: `compiler/src/semantic/loop_permission.rs` carries
> `PermittedEligible | Denied` and nothing else, and
> `a_claim_in_the_body_is_permitted` is a landed case. Every place below that
> stated the withdrawn condition is struck and dated where it stands; the rest
> reads as the design ruling it is.

## The charter and the ruling it forces

The owner's direction: the default form must be the optimal form — a
counted loop whose iterations are independent must itself be judged, not
survive only as advice to rewrite into a recursion. The research program
asked what a sound loop rule can actually grant, and three measured facts
force the scope:

1. **Element-granular disjointness does not exist in the language today.**
   A resolved place carries no index segment, [OWN-7]'s subscript clause
   requires literal offsets, element borrows are unimplemented, and no
   sub-slice operation exists. `dst[i]` and `dst[j]` are one place,
   fail-closed. The only outer writes a counted body can perform are
   whole-binding writes — the accumulator — or element writes the overlap
   relation cannot split.
2. **Map permission is legal-but-worthless at today's granularity.** The
   falsifier measured the most favorable proxy available (K disjoint
   destination buffers, judged eligible today, an upper bound on any
   [PAR-2] map lowering): memory-shaped bodies gain 1.21-1.29x wall time
   for ~5x the CPU, inside the auto-parallelism record's 1.1-1.5x
   bandwidth prior; the four real map loops in the corpus are 16 words to
   a few KiB, where the split LOSES 1.70x. The discriminator is arithmetic
   intensity: the split leaves the unresolved band as soon as the body
   does about a nanosecond of real arithmetic per element, and reaches 5x
   by ~50 ns.
3. **The reduction is where the measured demand lives.** The grid family's
   escape-count fold — a compute-heavy counted loop reducing under
   `+wrap` — is the shape that delivered 6.5x when hand-split, and wrap
   arithmetic recombines value-exactly: a 2^20-term `+wrap` fold and its
   recursive tree publish one byte sequence at every worker count
   (promoted probe), while the same shape under `fadd.strict` publishes
   two different sequences — which is precisely why the admitted set must
   be normative.

Ruling: **v1 loop permission is the reduction, not the map.** Scope:
full-range counted `for`; body ~~claim-free,~~ (**struck 2026-08-23**)
external-free, blocks-free,
exit-free (no `break`, `return`, `propagate`, or `give` leaving the loop);
cross-iteration state exactly one accumulator combined under a normatively
enumerated exactly-associative integer/boolean operation set; per-iteration
`own` data unrestricted. Element writes (the map) are DEFERRED with a named
re-entry condition: a real program with a compute-heavy single-destination
map, or places gaining index granularity. The counted-loop hint keeps
covering the refused shapes and names the refusing condition.

## Why regrouping is admissible (the doctrinal boundary, priced)

[PAR-1] never regroups — the combination tree is written in the source,
and that is why it is byte-stable even over floats. A loop rule lets the
implementation choose the tree, which is a genuinely new normative object.
Two facts make it sound here and nowhere wider:

- Wrap-family and boolean/min/max operations are value-exact under any
  association (probed, not asserted); the admitted set is stated in the
  rule text, so a conforming implementation may regroup exactly these and
  nothing else — the float exclusion is a rule, not a hedge.
- No entailment fact established inside a counted iteration survives to a
  later head or the continuation (the spec's counted recurrence subtracts
  facts supported by continuing writes), so a regrouped accumulator can
  falsify no surviving proof. Probed: a provably-in-range accumulator
  still carries no fact into a subscript after the loop.

## The judgment (batch A — zero emitted bytes)

A new CANDIDATE rule [PAR-2] (not an amendment inside [PAR-1]: the pair
conditions and the quantified conditions read badly interleaved, and a new
rule keeps the reviewed byte surface minimal). Conditions, the loop analogs
of the four:

1. No cross-iteration dataflow other than the single admitted accumulator.
2. No shared writable footprint: every written place is per-iteration
   `own` data or the accumulator; anything else denies (fail closed,
   including every unclassified body form — the exhaustive-match
   discipline of the window judgment).
3. No `external` or `blocks` row anywhere in the body's call closure.
4. No exit edge leaves the loop; the range is the whole range.

~~Eligibility: the body's transitive call closure is claim-free — unchanged
v1 doctrine.~~ **Withdrawn 2026-08-23**, with the parent dossier's §1 bullets:
a permitted loop is eligible, and a `claim` in the body or in a callee is an
ordinary statement whose predicate is read like any other expression. The
judgment consults no entailment state; the quantification
over iterations is structural (one binder, unit-increment recurrence), so
facts-on/off produce one permission table, as today. The diagnostic
channel gains a loop verdict line naming the judging condition on denial;
the split hint survives only for refused loops.

## The lowering (batch B — after A, separately measured)

The permitted loop lowers, in the overlapped world only, to a synthesized
recursive range split over the existing machinery — the two halves are an
ordinary permitted pair, so claim/publish/join/release, the deques, the
two-world selection, and the thunk outliner apply unchanged:

- Leaf = the loop over a subrange (never one iteration): preserves the
  body's own optimization, measured 3.1x against a 3.6-7.6x penalty for
  iteration-leaf on light bodies.
- One new IR operation carrying {splitter, chunk, arguments, seed,
  combine}; the splitter and chunk are ordinary synthesized IrFunctions;
  the sequential world renders the operation as a call to the chunk's
  clone — the original loop, byte-for-byte, zero-cost.
- Identity-element seeding per admitted operation; chunks combined in
  index order; the incoming accumulator folds into the leftmost chunk.
- Split allowance: compiler-estimated body weight x a one-query-per-entry
  runtime answer (span/lanes/deque state). No per-iteration checks, no
  static grain constant in the compiler; the runtime constants live beside
  the existing spin/park constants and are flagged in the packet.
- Empty and inverted ranges: the splitter tests hi <= lo before any width
  arithmetic (probed: a wrapping width splits a bogus 2^63 range).
- Frame-width refusal is compile-time with a diagnostic line, never a
  silent runtime sequentialization.
- v1 splits single-accumulator loops only; multiple accumulators keep the
  hint and a named widening path (returned aggregate).

Prerequisites already landed on this branch: the two-world phi-label
defect (invalid LLVM for any join in a phi-predecessor block) and the
hint's `give` unsoundness, both found by this batch's adversarial probe
and fixed at `eabefcc8`/`04d591af`.

## What the owner decides at merge

- The [PAR-2] reduction clause itself: the first rule that lets an
  implementation choose a combination tree, with the admitted operation
  set as spec text. This is the doctrinal step; everything else follows.
- The runtime allowance constants (measured, beside the existing park
  thresholds) — or their rejection, which forces the strictly worse
  static-gate fallback.
- The T2/map deferral with its recorded re-entry condition.

## Deferred register (each with its reason)

- Element-write (map) permission: legal-but-worthless at current
  granularity (falsifier table beside this file); re-entry named above.
- Multi-accumulator loops: no measured demand; widening path named.
- `loop`-form (unbounded) parallelism: no index to split; a range read
  from a hand-written counter is source-shape keying.
- Derived-index writes (`dst[perm[i]]`): PAL's structural runtime guard is
  the recorded candidate; parked with the map.
- The two-worker anti-scaling notch on memory-shaped bodies (2x slower at
  W=2 than W=1, reproducible, mechanism unchased): recorded as an open
  runtime question for the next performance batch.
