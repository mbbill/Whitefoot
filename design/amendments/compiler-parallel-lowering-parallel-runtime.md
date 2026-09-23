Node: compiler/parallel-lowering/parallel-runtime

Decision: Workers own fixed-capacity offer deques, idle threads steal the oldest task, and a join first reclaims its own offer for inline execution, because local publication avoids a contended lane scan and handshake on every offer, instead of a shared lane array.

Decision: A full deque refuses an offer and runs it inline, making capacity a grain floor on the deepest pending fork, because rate-limited promotion cannot remove coarse-grain overhead or free a caller waiting on its handed-out half, instead of a heartbeat gate.

Decision: A waiting join works instead of sleeping, draining its own deque and then stealing, and parks only when no work exists anywhere, because a sleeping joiner wastes the thread the program already has, instead of a join that sleeps on a condition variable.

Decision: Compute tasks use persistent native threads and ordinary stacks; a stolen-target join helps other compute work before its bounded spin, yield and condition-wait slow path, while a native engine wait stays on its executing call stack without compute helping, because the [runtime comparison](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/investigations/compute-runtime/PRIOR-BUNDLE.md#the-decision-this-bundle-justified) supports current-stack execution without a managed-stack pool or continuation migration, instead of park-on-miss stacks resumed by whichever worker finishes the target. Ordinary published calls may have source or linked bodies.

Decision: Runtime concurrency validation uses native-thread deque stress and ThreadSanitizer with separate startup, join, completion and exhaustion checks, because those exercise the selected runtime while the retired enumerator models a different managed-stack machine, instead of claiming exhaustive coverage from observed executions or retaining an irrelevant schedule model.

Decision: Worker and entry stacks share a runtime-owned one-gibibyte reservation exported across the language boundary, because equal reserved headroom makes deep recursion independent of which worker steals it and commits storage only as used, instead of inheriting host stack limits or combining them with a minimum floor.

Decision: An idle lane uses a 1,000-microsecond monotonic-clock window, sampled every 1,024 spin rounds, only when startup lane count fits the process affinity's CPU count and the host reports one performance level; oversubscribed, asymmetric or unknown-CPU pools use 1,024 rounds and 16 yields, because the [idle-window comparisons](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/investigations/compute-runtime/RESULTS.md#idle-window-comparisons) distinguish wake latency from contention on limited or unequal cores, instead of a larger fixed spin count or a time window on every host.

Decision: The map splitter retains a 150,000 work unit and 16 chunks per lane as build-overridable constants, because the [grain comparison](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/investigations/compute-model/DESIGN.md#block-work-and-the-rejected-global-constant) finds that lowering the global floor helps wide work but penalizes narrow and high-diameter controls, instead of using a universal smaller floor to compensate for missing runtime extents.

Decision: A group's published calls are joined newest-published-first, because the deque is Chase-Lev, so a join taken newest first finds its target at the owner's end or already stolen and never buried under a newer entry, and join order is unobservable since every value is fixed to its source-order result, instead of joining in publish order.

Rejected:
- A global 10,000 work unit as the runtime-block fix: rejected because the same outer static weight prices both wide and narrow helper work, and the measured gains on prefix and histogram transfer a material cost to the narrow stencil and high-diameter pull control.
- A shared lane array with a per-lane compare-and-swap and mutex/condition handshake: rejected because it charges every offer for contention whether granted or refused.
- A heartbeat gate rate-limiting promotion by local work since the last promotion: rejected because it bounds refusal cost but cannot lift the coarse ceiling where overhead is already amortized, cannot unblock the skew shape where the caller sleeps on the half it handed out, and promoting the oldest pending fork requires retaining fork points, which is a deque.
- Parking and migrating a joining stack: rejected because the measured layout comparison lost performance while requiring a managed-stack pool and a separate schedule model.
- A lane stack sized as the larger of the environment's limit and a fixed floor: rejected because it reintroduced environment dependence on the lanes and made a deep recursion's survival depend on which thread stole it.
- Changing wake policy for the sparse-cadence loss on hosted SMT runners: rejected because the [cadence comparisons](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/investigations/compute-runtime/RESULTS.md#sparse-cadence-comparisons) did not reproduce the loss across apparently identical machines, so they did not isolate a runtime change.
