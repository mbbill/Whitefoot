Node: language/ownership/exchange

Decision: Exchange admits equal captured targets and disjoint storage, but requires proof that neither target may be a proper ancestor of the other, because swapping a recursive owner with its own descendant would manufacture a cycle or lose ownership while equal array slots remain a harmless no-op, instead of extending the equality allowance to every possible overlap of unknown-depth targets.

Rejected:
- Require distinctness for every exchange: rejected because the existing same-slot allowance serves indexed partition and heap adjustment without a branch; the additional condition excludes ancestry, not equality.
- Preserve unrestricted overlap under the existing equality allowance: rejected because equality and ancestry have different ownership consequences, exposed by tests/conformance/cases/op11-neg-wildcard-ancestor-exchange.wf.
