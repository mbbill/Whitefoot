Decision: Every worker lane runs on a stack the same size as the entry's, that size is one runtime-owned constant exported across the language boundary, and it is not derived from the environment's limit, because sizing a lane from the environment reintroduced on the lanes exactly the environment dependence the entry's own stack had just removed and left a lane two orders of magnitude shorter than the entry, so whether a deep recursion survived was decided by a steal race, instead of inheriting the environment's stack limit.

Rejected:
- A lane stack sized as the larger of the environment's limit and a fixed floor: rejected because it reintroduced environment dependence on the lanes and made a deep recursion's survival depend on which thread stole it.
