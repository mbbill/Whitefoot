Node: compiler/checker-facts

Decision: A window whose element type is linear reaches no scope exit, proved empty or not, and is refused there like any other linear value, because [WIN-3] states the route exactly once -- "the program must take every element out and consume it, and then, with the storage proved empty, call `free_empty` [OP-14]" -- so the zero-length fact is what `free_empty`'s own requirement reads and not a licence for the edge; the v0.59 element-free derived release survived as a branch that fired whenever the capability half of the release graph was satisfiable, which under [STOR-8]'s one heap is always, so every linear window with a `requires run.len == 0_u64` was silently accepted with no release able to carry it, instead of keeping the branch and gating it on the proved length, which would still be a compiler-derived release of a linear value and the compiler never releases one.

Decision: The [PROV-6] `EmptyRunRelease` obligation family and the checked `EmptyRun` release mode stay in place unused rather than being deleted with this change, because nothing selects them now, and a later version that gives a run a derived release short of its elements would want the same shape; removing them is a sweep with its own ripple into lowering and the entailment obligation table.

Rejected:
- Keeping the branch for an affine element type, which is where it was harmless: rejected because the branch is only ever reached when the release is blocked, and for an affine element type it is not blocked, so the branch had no affine case to serve.
