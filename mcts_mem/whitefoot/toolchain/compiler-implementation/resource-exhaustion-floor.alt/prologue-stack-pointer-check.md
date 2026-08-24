- Every generated function compares the stack pointer against a per-thread limit on entry and calls a runtime handler when it is past it.
- The limit is explicit program data rather than a property of the host's mapping, and the check needs no guard page and no signal disposition.

## Moves

- 2026-08-23 (178d4f69) replaced by [[resource-exhaustion-floor]]: an explicit check in every prologue spends the headroom it guards, where the target's own stack-probing attribute contains the same fault class for free and leaves the reporting to a signal disposition (sourced)
