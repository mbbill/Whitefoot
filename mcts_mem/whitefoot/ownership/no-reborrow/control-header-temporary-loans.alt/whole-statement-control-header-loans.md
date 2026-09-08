- Every call-scoped child reborrow created in a control header remains live through the complete enclosing statement, including every selected arm or branch.
- The child's suspended unique parent cannot be reused within that arm or branch even after the owned header value is complete.

## Moves

- 2026-09-07 (a96be5ee) replaced by [[../control-header-temporary-loans]]: whole-statement retention blocked sequential typed acquisition through a retained unique provider after the owned header value had completed, while non-escape and completed-access checks permit the narrower endpoint without weakening exclusivity (sourced)
