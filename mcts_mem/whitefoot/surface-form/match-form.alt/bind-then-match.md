- The match scrutinee was restricted to a place; matching on a computed value required binding it to a temporary first, one invented name per conditional use.

## Facts

- 2026-07-07 statement: this was one of exactly two recorded candidates for resolving the GRAM-4/EX-1 contradiction, and the prototype compiler already implemented expr scrutinees when the decision was taken — the restriction existed only in the spec text. (sourced)

## Moves

- 2026-07-07 (7c1d7641) replaced by [[match-only-conditional]]: widening the scrutinee to expr resolved the GRAM-4/EX-1 contradiction; bind-then-match was rejected under R3/W1 because it taxes the sole conditional idiom with a mechanical temporary at every use and adds weak-writer naming burden (sourced)
