- The judged unit is a pair of consecutive let-bound call statements; any other statement form ends the candidate group.
- The four conditions are stated over the two calls only; no interposed statement exists to quantify over.

## Moves

- 2026-08-21 (974d5513) replaced by [[permission-judgment]]: one ordinary statement between two calls ended the candidate group, so permission turned on statement adjacency rather than semantics — two byte-identical-output programs differed 1.9x in wall time; the window judges the pair plus every interposed statement with all four conditions quantified over them. (sourced)
