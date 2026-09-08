- Iteration has two source forms: an ordinary loop with an optional label and break, and an ascending, unit-stride, half-open counted loop over once-captured endpoint values.
- The counted binder is compiler-updated, body-local, and source-immutable; endpoints are captured once before the first iteration.
- Counted iteration supplies structural body-entry bounds. Both loop forms admit checked invariant headers; counted exact exhaustion additionally exports eligible proved outer-value conclusions through the ordinary continuation join.

## Facts

- 2026-09-09 correction: INV-1 and ENT-5 now define header induction and conditional exact-exhaustion publication. The prior statement that the counted form grants no exit postcondition must not erase this later capability; a break still needs its own matching conclusion. (sourced)

## Moves

- 2026-08-09 (3e2e823d) replaced [[sole-loop-iteration]]: the sole-form commitment made every bounded walk spell its own counter, guard, and increment, whose carried facts the loop head then discarded — three real SHA-256 index walks and three of four hostile writer probes independently selected the same ascending half-open counted shape, and adding only that class removed four SHA-256 claims while preserving the unrelated ordinary loop (sourced)
