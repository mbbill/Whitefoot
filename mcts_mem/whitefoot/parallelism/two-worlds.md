- A `--par` module carries two lowerings of the functions on a path from the entry to a handed-out call: the overlapped world and a sequential clone world whose code is the sequential lowering byte for byte.
- One world is selected once per process at the bootstrap, by whether a pool was asked for; neither world calls into the other, and nothing below the branch tests anything again.
- The clone set is derived from the call graph and the permission table — the functions reachable from the entry that can reach a hand-out — never from a name or a source shape.
- The pool-state query reads configuration and starts nothing; the pool is created lazily by the first claim.
- Each emitting world resolves labels against its own overlap set; a clone never names a join block it does not emit.

## Facts

- 2026-08-21 rationale: the hand-out's rejoin phi takes the callee's result out of tail position and forecloses accumulator tail-recursion elimination, so no single lowering serves both worlds — the fib-shaped pool-off tax was 2.96x and fell to 1.00x with the clone world. (sourced)
- 2026-08-21 measurement: answering the bootstrap query by starting the pool eagerly cost 17-18% on the layout demo; lazy creation by the first claim kept it free. (sourced)
- 2026-08-22 (eabefcc8) pitfall: labeling phi predecessors from the unsuppressed overlap table while the clone world suppressed actualization emitted references to join blocks the clone never defines — invalid LLVM on any module whose overlap group sits in a phi-predecessor block; the worlds' overlap sets are one stored slice per world. (code)

## Moves

- 2026-08-21 (d4e674c3) replaced [[per-task-demand-switch]]: a per-task demand signal cost contended read-modify-writes measured at 0.49 s to 0.93 s on the fine-grain cell, while pool-off selection needs one decision per process; the clone world is selected at bootstrap and no per-task signal exists to contend on. (sourced)
