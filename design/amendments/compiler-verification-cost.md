Node: design/compiler/verification.md

Decision: Local heavy verification uses one host-wide owner across worktrees, bounded independent build and test parallelism, visible phase/exit accounting and a cancellable process group with an explicit command deadline, because overlapping optimized builds and suites made an ordinary build take minutes without test progress in the [build/test investigation](../../research/investigations/test-economy/build-and-test.md), instead of independently spending the host's whole capacity in each agent or hiding build-lock waits inside a test command. A command deadline reports incomplete verification and never selects source acceptance.

Decision: Tests reuse immutable native objects and dependency-tracked example executables while rerunning their assertions and preserving distinct compiler modes and macro-interposed builds, because repeated compilation of identical support code checks no new behavior and stale compiler-dependent artifacts check the wrong revision, instead of rebuilding every support unit for each case or caching test verdicts.

Decision: The canonical gate covers the compiler, runtime, conformance, current experiment witnesses and independent oracles, while completed research instruments retain explicit reproduction checks and full IO timing matrices run on request, because a historical source miner or model-trajectory harness does not test the current compiler and repeated storage measurements dominate unrelated feedback, instead of treating every runnable historical instrument or exploratory timing matrix as a permanent per-change compiler requirement.

Rejected:
- Shortening repeated schedules or removing normative cases merely because they are slow: rejected because those changes would remove the behavior evidence the gate is meant to preserve.
- A shared cache of compiler verdicts between unit tests and the native conformance adapter: rejected because the adapter must still reach each verdict through its own ordinary compilation path.
