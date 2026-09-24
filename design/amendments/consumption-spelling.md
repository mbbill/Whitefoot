Node: language/ownership

Decision: A consuming use of an existing owned place writes `move` when its written type or generic bound does not grant copy, including in `match` and `propagate`, because [the consumption comparison](../../research/investigations/contract-surface/OWNERSHIP.md#consumption-spelling) identifies whole-owner death and residual cleanup as events worth marking under one rule, instead of context-specific implicit consumption; temporaries need no marker and reference matches remain non-consuming.

Rejected:
- Implicit consumption of every bare noncopy place: rejected because selecting one field can consume the entire owner and release other fields without a written consumption boundary, even when no later use exposes that change.
