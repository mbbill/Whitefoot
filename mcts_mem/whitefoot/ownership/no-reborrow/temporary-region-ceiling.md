- An unbound argument child's local region bounds its formation and type validity; the region may contain statements before and after the receiving statement.
- Temporary loans end at the complete statement or the selected completed-header endpoint. Bound results and surviving views retain their own claims, and parent access resumes only when the applicable suspension ends.
- The parent must outlive the child region, and a borrow created inside a loop uses a region inside that loop.

## Facts

- 2026-09-09 (4792abeb) measurement: the executable in tests/conformance/cases/own6-pos-statement-children-use-a-longer-local-region.wf observes old and installed Box values, two provider allocations, and per-statement parent reuse in a loop. The native test passes all three lowering modes with normal and retained helper calls; this establishes those behaviors, not throughput or complete soundness. (code)
- 2026-09-09 (4792abeb) pitfall: checking a later argument only against the previous child's place misses the complete unique parent's suspension, and a bare holder transfer can have no ordinary access entries. Call-local checks must cover both while still permitting disjoint sibling formation. (code)

## Moves

- 2026-09-09 (4792abeb) replaced [[statement-confined-child-regions]]: confining the region to one statement prevented direct observation of displaced owners and sequential provider calls, while the existing statement-loan endpoint and independent surviving-loan checks permit these forms without new lifetime inference (sourced)
