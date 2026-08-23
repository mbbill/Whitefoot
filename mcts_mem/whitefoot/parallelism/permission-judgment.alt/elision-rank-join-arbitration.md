- The schedule-unobservability guarantee stays unconditional for every execution, erroneous ones included, and is paid for at the join rather than narrowed.
- Each lane carries a rank derived from its position in the source order the sequential elision defines; when several lanes trap, the join arbitrates by that rank, and the record an overlapped execution writes is the record the source-order execution would have written.
- A losing lane is parked and woken by a protocol, and a join is no longer a plain wait-for-all: it is a coordinator that must compute, carry, and compare the rank.
- Weighed and deferred when overlap of claim-bearing regions was first ruled out; refused when that ruling was withdrawn.

## Moves

- 2026-08-23 (f6c55a9d) replaced by [[permission-judgment]]: every byte of the arbitration exists to make a defective program's trap record reproducible and is paid for by every correct program, which cannot reach it; a latch on the trap path is one global and one compare-exchange a correct program never executes, and the reproduction the arbitration would buy is already free at an explicit single-lane setting. (sourced)
