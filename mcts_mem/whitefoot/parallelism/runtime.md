- Each worker thread owns a fixed-capacity deque of offer slots (`wf__par_claim`).
- An offer is a push onto the offering thread's own deque; idle threads steal the oldest entry.
- A join first tries to reclaim its own offer and runs it inline on success; only a stolen offer is waited on.
- A full deque refuses the offer and the call runs inline; refusal falls on the deepest pending fork and the bound doubles as a grain floor with no tuned constant.
- Synchronization cost attaches to steals, whose count follows imbalance, never to offers; the owner's common-path push and pop carry no read-modify-write.
- A waiting join works instead of sleeping: it drains its own deque, then steals, and parks only when no work exists anywhere; publish wakes at most one registered sleeper.
- The grant counter counts frames executed by a thread other than the offerer; counting pushes would pass a runtime that never overlaps.
- An unset worker setting defaults to one lane per logical CPU; zero, one, or an unparsable setting starts no pool.

## Facts

- 2026-08-21 measurement: the deque replacement moved the pathological fine-grain probe from 48.6x slower than one lane to 1.99x faster, and per-fork excess from 48.8 ns to 4.80 ns against rayon's 5.80 ns in the same pass. (sourced)
- 2026-08-22 measurement: a batch steal claiming a range of entries under one CAS on top hangs — the owner pops from bottom and the ends overlap whenever the deque is short — so draining more than one entry per steal is unsound under this protocol, not merely slow. (sourced)
- 2026-08-22 measurement: sleep/wake is not the fine-grain residual — a whole fine-grain run parks 10 times with zero join-waits and zero system time; backoff on the steal scan is monotonically worse because pickup latency dominates probe traffic. (sourced)
- 2026-08-22 measurement: worker QoS classes move nothing on the 4P+6E machine, and a self-set background QoS is defeated by priority inheritance while the main thread joins; no QoS class creates a fifth performance core. (sourced)
- 2026-08-22 measurement: the coarse cells improve monotonically past the performance-core count to W=10 (5.16x against 4.79x at 8), while the finest cells peak at 4; no single width fits both, and the default takes the coarse win while holding every cell at or above its sequential build. (sourced)

- 2026-08-24 pitfall: the lane count was a plain int rewritten by the start path after workers were already reading it in their steal loops, a formal data race hidden from ThreadSanitizer by a parent-first rendezvous schedule; every access is now a relaxed atomic, and the invariant is held by review because no sanitizer run reproduces the racing schedule on this machine. (code)

## Moves

- 2026-08-21 (826cea41) replaced [[lane-scan-runtime]]: an offer paid an O(lanes) contended scan and a mutex-condvar handshake whether granted or refused — 48.8 ns per fork at the coarsest cell and up to 48.6x slower than sequential at fine grain — while a per-thread deque makes an offer two local stores and moves all synchronization cost to steals, which follow imbalance instead of offer rate. (sourced)
- 2026-08-21 (826cea41) replaced [[heartbeat-promotion-gate]]: rate-limiting promotion bounds refusal cost but cannot lift the coarse ceiling where overhead is already amortized 4.6:1, cannot unblock the skew shape where the caller sleeps on the half it handed out, and promoting the oldest pending fork point requires retaining fork points, which is a deque. (sourced)
