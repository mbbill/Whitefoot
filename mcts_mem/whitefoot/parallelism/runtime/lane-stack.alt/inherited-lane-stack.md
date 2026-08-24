- A worker lane's stack is the larger of the environment's stack limit and a fixed floor the runtime names.

## Moves

- 2026-08-23 (1a9b4c45) replaced by [[lane-stack]]: sizing a lane from the environment reintroduced on the lanes exactly the environment dependence the entry's own stack had just removed, and left a lane two orders of magnitude shorter than the entry, so whether a deep recursion survived was decided by a steal race (sourced)
