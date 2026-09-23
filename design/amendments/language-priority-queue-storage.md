Node: language/data-model/priority-queue-storage

Decision: Use a growable binary heap over an ordinary boxed prefix as the reusable priority-queue baseline, because the [complete owning comparison](../../research/experiments/container-representation/priority-library/RESULTS.md) establishes growth, replacement, heapify and consumption for arbitrary owners while separating wide-element sift movement from result-boundary costs, instead of treating the earlier fixed-capacity witness as the complete library or moving priority policy into the kernel. This accepts the measured costs as an improvement baseline, not native parity or an optimum over heap fanouts.

Decision: Keep priority-ordered draining distinct from physical-order final consumption, because ordered consumption requires repeated heap repair while a caller needing only complete owner cleanup can consume the physical window in linear time, instead of imposing comparison and logarithmic repair on every final release.
