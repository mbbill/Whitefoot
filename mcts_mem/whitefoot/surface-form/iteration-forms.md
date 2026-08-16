- Iteration has exactly two source forms: the ordinary labelled `loop` with `break`, and one ascending, unit-stride, half-open counted `for` over once-captured endpoint values.
- The counted binder is compiler-updated, body-local, and source-immutable; endpoints are captured once before the first iteration.
- The counted form carries construct-owned structural body-entry bounds; it grants no general loop induction and no exit postcondition.

## Moves

- 2026-08-09 (3e2e823d) replaced [[sole-loop-iteration]]: the sole-form commitment made every bounded walk spell its own counter, guard, and increment, whose carried facts the loop head then discarded — three real SHA-256 index walks and three of four hostile writer probes independently selected the same ascending half-open counted shape, and adding only that class removed four SHA-256 claims while preserving the unrelated ordinary loop (sourced)
