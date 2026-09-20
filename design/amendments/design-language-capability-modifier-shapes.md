Node: language/ownership/copy-classification

Decision: `nocopy` and `nodrop` may remove capabilities from any source struct or enum, including a fieldless struct and a tag-only enum, because representation shape does not determine whether a value should be duplicable or carry a consumption obligation, instead of preserving the former exception that refused a must-consume modifier on an otherwise copyable declaration.

Rejected:
- Require an owned noncopy field before a declaration may remove drop: rejected because it makes a consumption token depend on a dummy resource field and contradicts treating declaration modifiers as capability removal independent of structural defaults.
