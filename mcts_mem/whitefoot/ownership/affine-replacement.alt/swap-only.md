- A two-place `swap p, q` exchanges the contents of two live same-typed places; both bindings survive with changed values.

## Facts

- 2026-08-18 statement: the concrete failure is that after swapping with a local, that binding holds the old value — its value changed without a copy write and without dying, a third mutation path breaking the rule that reinitialization requires a new let and every fact, loan, and liveness judgment keyed to binding initialization; repairing it as consume-plus-rebind is atomic replace spelled worse, with two targets to judge instead of a target and an expression. (sourced)

## Moves

- 2026-08-18 (eb8e8634) replaced by [[affine-replacement]]: swap changes a live affine binding's value without death or a new let — a third mutation path breaking initialization-keyed facts, loans, and liveness — and everything it expresses is one atomic replace with the moved binding as replacement (sourced)
