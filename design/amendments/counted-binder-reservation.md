Node: language/name-resolution

Decision: Counted-loop value binders follow the same operation and mode-word reservation as ordinary let bindings, because both introduce ordinary value names and changing their availability with the binding syntax gives the same value domain inconsistent naming rules, instead of admitting reserved runtime names only in counted-loop bindings.

Rejected:
- Permit operation and mode words in counted-loop value binders while reserving them from ordinary let bindings: rejected because the binding syntax does not establish a separate lookup domain, unlike proof-only invariant declarations.
