- Every worker lane runs on a stack the same size as the entry's, and no thread of a program has less room than the thread the program started on.
- That size is one constant the runtime owns, exported across the language boundary rather than restated on each side, and it is not derived from the environment's limit.

## Facts

- 2026-08-23 (1a9b4c45) measurement: with lanes sized from the environment's limit, a 2,000,000-level recursion under a permitted overlap completed 11 of 30 runs at two workers, 3 of 30 at eight, and 2 of 30 at sixteen, while completing 30 of 30 with no pool; with lanes sized like the entry it completes 30 of 30 at every worker count. Every failure wrote the resource record rather than a bare signal. (sourced)
- 2026-08-23 measurement: sixteen lanes reserve sixteen gibibytes of address space between them and cost roughly fifty kilobytes of resident memory each — thread bookkeeping and the pages a signal stack touches, not the reservation. (sourced)
- 2026-08-23 pitfall: equal lane stacks do not make a permitted overlap depth-safe. The overlapped and sequential clones of one call cost different amounts of stack per activation, so a band of depths exists where the same binary completes with no pool and exhausts the stack with one. (code)

## Moves

- 2026-08-23 (1a9b4c45) replaced [[inherited-lane-stack]]: sizing a lane from the environment reintroduced on the lanes exactly the environment dependence the entry's own stack had just removed, and left a lane two orders of magnitude shorter than the entry, so whether a deep recursion survived was decided by a steal race (sourced)
