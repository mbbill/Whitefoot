- Every function in an eligible closure carries a sequential clone, and the parallel version switches to the clone per subtree when a runtime demand signal reports no idle capacity.
- The demand signal is a shared word read at fork sites and updated by the scheduler.
- Built and measured inside the deque runtime; never landed.

## Moves

- 2026-08-21 (d4e674c3) replaced by [[two-worlds]]: a per-task demand signal cost contended read-modify-writes measured at 0.49 s to 0.93 s on the fine-grain cell, while pool-off selection needs one decision per process; the clone world is selected at bootstrap and no per-task signal exists to contend on. (sourced)
