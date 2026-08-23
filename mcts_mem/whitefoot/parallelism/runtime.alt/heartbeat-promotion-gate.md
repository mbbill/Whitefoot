- A lane may accept or offer work only after a minimum interval of its own local work since its last promotion; the interval is self-calibrated against measured promotion cost, never a static size constant.
- A deferral variant holds a fired beat until the traversal unwinds to a shallower activation, so promotion prefers large untaken subtrees.
- Designed as a modification of the lane-scan runtime; never landed.

## Moves

- 2026-08-21 (826cea41) replaced by [[runtime]]: rate-limiting promotion bounds refusal cost but cannot lift the coarse ceiling where overhead is already amortized 4.6:1, cannot unblock the skew shape where the caller sleeps on the half it handed out, and promoting the oldest pending fork point requires retaining fork points, which is a deque. (sourced)
