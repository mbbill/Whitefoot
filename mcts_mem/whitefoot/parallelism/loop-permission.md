- A counted loop is judged in its own right: the permission attaches to the loop, not to a pair of calls written inside it.
- v1 grants the reduction only: a full-range counted range whose body carries exactly one accumulator combined under a normatively enumerated exactly-associative operation set, with per-iteration owned data unrestricted.
- The admitted combination set is fixed by the rule text rather than left to the implementation, and admits no inexact operation.
- Element writes — the map — are refused, deferred with a named re-entry condition rather than left open.
- The lowering is a synthesized recursive range split whose leaf is the loop over a subrange, never a single iteration; a split's two halves are an ordinary permitted pair of the same window machinery, and the sequential world renders the site as a call to the leaf, which is the original loop.
- The split allowance is one runtime query per loop entry, taking a compiler-estimated body weight; no check is made per iteration, and the descent depth is bounded by the allowance rather than by the range.
- A permitted loop carrying a `claim` is eligible like any other region ([[permission-judgment]]).
- Condition 2 carries a loans half beside the written half: an exclusive [OWN-5] loan on storage the iteration does not introduce denies through its own slot and message, shared loans are unconstrained, and the written half's contract stays uses-only (`record_writes`).

## Facts

- 2026-08-24 (db543775) pitfall: the loop side was the actualized half of the borrow-mode blindness — a body taking `&uniq` of one outer cell with a reads-only row was permitted and emitted as a split, N workers each holding what the source spells as an exclusive borrow of one place. (code)
- 2026-08-24 (db543775) pitfall: routing the loan denial through the shared-write slot printed a denial calling a loan a write on a callee declared reads-only; the loans half owns its slot and message for that reason. (code)

- 2026-08-23 measurement: map permission does not pay at today's place granularity and the reduction shape does. On the most favorable legal proxy — K distinct destination buffers, K adjacent calls, judged eligible by the compiler, an upper bound on any map lowering — a 64 MiB byte copy gains 1.29x, a 4 MiB copy 1.21x, a 4 KiB copy *loses* 1.70x, and a 64 MiB u64 copy gains 1.26x, while a 256-round-per-element mix gains 6.12x. The grain sweep puts the discriminator at arithmetic intensity, not size: twin/loop ratios run 0.662 to 0.166 across 0.18 to 303 ns per element, and the band is left below about 1 ns of real work per element. Table and physics anchors in `research/investigations/proof-derived-parallelism/loop/VALUE.md`. (sourced)
- 2026-08-23 measurement: the counted-loop form reaches the hand-written recursive split it was meant to replace. On the oracle grid workload, minimum of 18 interleaved rounds at load 4.04-4.29 with the 0.83x-1.20x unresolved band applying to every ratio: loop/twin is 0.98 sequential, 0.98 at one lane, 0.95 at two, 0.98 at four, 0.99 at eight, and 1.00 at the shipped default; loop/rayon is 0.95-0.99 across the same. The loop form scales 6.54x from its own sequential build, and every cell publishes one byte sequence. (sourced)
- 2026-08-23 measurement: the rule fires on none of the twelve counted loops the corpus already contains — seven are element writes into an enclosing buffer, four are expression statements, one is a sequential recurrence. The justification is the principle that the default form must be the optimal form, the grid measurement above, and programs not yet written; it is not corpus payoff. (sourced)
- 2026-08-23 (13ffab4c) rationale: the map's re-entry condition is named rather than left open — a real program with a compute-heavy single-destination map, or places gaining index granularity. At today's granularity a resolved place carries no index segment, so two element writes through distinct indices are one place and deny. (sourced)

## Moves

- 2026-08-23 (ddf1d139) replaced [[iteration-leaf-split]]: a leaf of one iteration destroys the body's own optimization, measured at a 3.6-7.6x penalty on light bodies against the 3.1x a subrange leaf preserves; a leaf that is still a loop keeps the sequential world's code byte-for-byte and costs the split nothing to decline. (sourced)
- 2026-08-23 (501966ad) replaced [[par1-amendment]]: the pair conditions and the conditions quantified over iterations read badly interleaved in one rule, and a separate rule keeps the byte surface an owner reviews at activation minimal. (sourced)
