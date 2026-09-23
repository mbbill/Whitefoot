Node: compiler/parallel-lowering/parallel-runtime

Decision: Workers own fixed-capacity offer deques, idle threads steal the oldest task, and a join first reclaims its own offer for inline execution, because local publication avoids a contended lane scan and handshake on every offer, instead of a shared lane array.

Rejected:
- Adopting DONE-before-help join entry from this trial: rejected because the deterministic completed-target benefit does not establish workload suitability and the [identical-image cost control](../../research/investigations/compute-model/DESIGN.md#amended-cost-result-identical-image-control-failure) failed before either combined or runtime-only timing; defer adoption until a new prospectively qualified cost comparison, without inferring a runtime regression.
