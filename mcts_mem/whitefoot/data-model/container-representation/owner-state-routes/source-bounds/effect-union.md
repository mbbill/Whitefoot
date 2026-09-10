- Keep established effect atoms and uncovered possible atoms separately for
  each source effect category (EffectSet).
- Let local accesses and declared call effects with finite source bounds
  contribute possible atoms when the source has no further nameable struct
  fields. Unknown sources and unresolved field ceilings remain unsupported.
- Union contributions over the complete body and derived releases before
  requiring every possible atom to be independently established. Compare the
  resulting exact row with the declaration; the declaration supplies no fact.
- Keep source routes, selected locations, loan permissions and parallel
  footprints independent of this boundary judgment. Type-directed release
  selection retains its exact-origin requirement.

## Moves

- 2026-09-10 (ebc6d059) replaced [[per-query-exactness]]: Per-query exactness stopped repeated removals even when earlier operations established every possible effect atom; whole-function bounds prove the exact union without refining selected owners. (code)
