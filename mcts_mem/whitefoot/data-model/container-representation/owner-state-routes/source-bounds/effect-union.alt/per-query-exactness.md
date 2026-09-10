- Require exact selected origins at each local effect query and complete
  supplier coverage at each declared whole-value call-effect query.
- Stop at the first query containing a surviving imported bound, independently
  of established contributions elsewhere in the function.
- Keep unknown sources unsupported and type-directed releases exact.

## Moves

- 2026-09-10 (ebc6d059) replaced by [[effect-union]]: Per-query exactness stopped repeated removals even when earlier operations established every possible effect atom; whole-function bounds prove the exact union without refining selected owners. (code)
