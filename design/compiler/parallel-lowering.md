Decision: The default lowering actualizes eligible finite completion operations while leaving compute-call outlining off, and `--par` additionally actualizes eligible compute groups, maps, and reductions, because permission is a fact about the program while actualization is a cost choice, and the default should not change a program's code shape without being asked, instead of actualizing every permitted overlap by default.

Decision: A parallel range split descends to a leaf that is still a loop over a subrange, never to a leaf of one iteration, because a one-iteration leaf destroys the body's own optimization, measured at 3.6 to 7.6 times slower on light bodies against the 3.1 times a subrange leaf preserves, and a leaf that is still a loop keeps the sequential world's code byte for byte, instead of an iteration-leaf split.

Decision: Native queues, helper lanes, wakeups, and completion ports are target-private protocol state and never Whitefoot shared storage, because the source model has no scheduler and a runtime structure visible as source storage would need ownership and effects it cannot honestly carry, instead of exposing runtime queues as language values.

Decision: An emitted Windows `--par` module requires the compiler-owned compute pool through hard external ABI obligations, so a missing runtime fails to link and an invalid worker configuration fails at the host boundary, because silently selecting sequential execution would hide a broken configuration behind correct output, instead of a silent sequential fallback.

Rejected:
- Splitting a parallel range down to leaves of one iteration with grain left to the runtime: rejected because it destroys the body's own optimization, measured at 3.6 to 7.6 times slower on light bodies.
