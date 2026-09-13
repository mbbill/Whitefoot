Decision: One set statement commits any number of named targets from one value list or one multi-result call, reading every target's previous value before writing any, so a swap or a rotation is an ordinary instance of it, because a dedicated exchange operation would duplicate what the general commit already expresses, instead of a separate swap or exchange operation.

Rejected:
- A two-place `swap`: rejected because it changes a live affine binding's value without death or a new binding, a third mutation path that breaks initialization-keyed facts, loans, and liveness, and everything it expresses is one multi-target `set` commit with the moved bindings as the values.
