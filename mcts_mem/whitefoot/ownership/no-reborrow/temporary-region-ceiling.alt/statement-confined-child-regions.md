- A non-candidate argument child's local region block is confined to its receiving statement. A longer region cannot form that child even when its ordinary loan would end at statement completion.

## Moves

- 2026-09-09 (4792abeb) replaced by [[../temporary-region-ceiling]]: confining the region to one statement prevented direct observation of displaced owners and sequential provider calls, while the existing statement-loan endpoint and independent surviving-loan checks permit these forms without new lifetime inference (sourced)
