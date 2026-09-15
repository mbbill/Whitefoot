# Access-state checking experiment

## Question and prior criteria

Recorded before implementing the model, 2026-09-15. The consumer is the
[ownership-system investigation](../../investigations/access-effects/RESEARCH.md).
Keep this executable evidence while that investigation relies on it; replace
or remove the model when a stronger experiment supersedes these observations.

This experiment tests a finite first-order fragment, not WF source programs.
Its common objects have data and a reference field. The same operations must
cover retained graph edges, retargeting, shared control state, partial
initialization, ownership transfer, and individual reclamation with physical
slot reuse. No resource or function name may select a checking rule.

Discriminating criteria, fixed before results:

1. A reference's target is stable information; current live/initialized state
   is separate. Multiple aliases allow sequential writes. Reclaiming a target
   removes access through every alias, including aliases stored in objects.
2. An owner moves without renaming its backing allocation. Retargeting a
   reference field does not retarget an already loaded alias. A fresh logical
   allocation can use a freed physical slot without reviving old locators.
3. Generic bodies admit possibly equal reference arguments. State changes
   conservatively affect possible aliases; callers substitute relationships
   into checked signatures. They do not inspect or clone the body.
4. Effects on separate payloads can be independent despite a common stored
   link. Effects through that common link conflict. Partial initialization
   blocks a nested call that would read the affected field.
5. For exhaustive bounded instruction sequences, every accepted sequence must
   execute without stale or uninitialized access in an independent physical
   storage oracle. An address-only execution must have the same observed
   values. Deliberately faulty checkers must expose counterexamples, showing
   that the oracle can discriminate the intended failures.
6. Report counts and bounds rather than treating bounded enumeration as a
   soundness proof. Measure checker operation counts for increasing symbolic
   parameter sets; do not infer a bound for a complete language from this
   fragment. No runtime-cost claim follows from interpreter timings.

The oracle may carry allocation generations to detect errors. Those are test
instrumentation, not a proposed runtime mechanism. The address-only machine
has allocator occupancy bookkeeping but no generation or borrow check.
Neither interpreter establishes native-pointer lowering or LLVM validity.

## Results

No result is recorded before the experiment runs.
