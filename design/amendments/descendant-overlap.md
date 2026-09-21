Node: language/ownership

Decision: Unknown-depth reference targets have subtree interference covers and distinct current-target identities, with ordinary finite suffix comparison only for accesses sharing a captured target, because covers must conservatively include every possible ancestor or descendant conflict while one current node's sibling fields remain disjoint, instead of treating a wildcard as an ordinary exact path step or proving separation between unrelated cursors from suffix spelling.

Rejected:
- Keep the overlap decision limited to exact finite paths: rejected because iterative owned-link descent requires finite covers of paths with unbounded runtime depth; this extends that decision's common overlap relation rather than adding a consumer-private alias relation.
