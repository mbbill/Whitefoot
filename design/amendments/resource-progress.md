Node: language/checks-and-proofs/resource-bounds

Decision: A written scalar affine rank denotes an erased immutable value captured at function entry or the current loop header, with its range and every recursive actual or continuing backedge proved through the ordinary fact context, because the [rank probes and implementation contract](../../research/investigations/fixed-resource-execution/DESIGN.md) distinguish entry-relative progress from resetting a mutable counter before decrementing it, instead of trusting a declared depth, inferring a rank by search, or comparing only with the value immediately before a recursive call.

Decision: Completion and finite source resource bounds compose over the complete concrete call and loop closure, with ranked direct recursion admitted when its bound fits and linked definitions requiring their own progress evidence, because a bounded outer loop or small stack does not make an unknown callee finish and two recursive children multiply work without doubling simultaneous depth, instead of equating no-heap, tail transfer or a runtime exhaustion limit with completion.

Rejected:
- Banning all recursion in the fixed-resource domain: rejected because a checked finite activation bound can fit the same supplied stack budget as an iterative algorithm.
- Runtime fuel as authority for normal completion: rejected because exhaustion stops an execution without proving that it reaches its declared result.
