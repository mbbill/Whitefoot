- A fixed array of worker lanes is shared by all offerers; an offer scans the array with an acquire compare-and-swap per lane and takes the first idle lane.
- A granted offer hands its frame to the lane through a mutex-and-condvar handshake; the join sleeps on the lane's condition variable until the lane publishes completion.
- Work that found no idle lane at offer time leaves no record; a lane that becomes idle later cannot discover it.

## Moves

- 2026-08-21 (826cea41) replaced by [[runtime]]: an offer paid an O(lanes) contended scan and a mutex-condvar handshake whether granted or refused — 48.8 ns per fork at the coarsest cell and up to 48.6x slower than sequential at fine grain — while a per-thread deque makes an offer two local stores and moves all synchronization cost to steals, which follow imbalance instead of offer rate. (sourced)
