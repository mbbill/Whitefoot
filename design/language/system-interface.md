Decision: System access uses typed entry inputs and ordinary owned resources under ordinary ownership, with no separate capability category, because the alternatives each hid access, forged identity, serialized unrelated resources, or imported another ecosystem's contract, instead of raw descriptors, ambient functions, one process object, or a literal WASI contract.

Decision: State effects on system resources are ordinary parameter paths in the effect row, because system use invisible in a signature cannot be narrowed, tested, or parallelized by ownership, instead of a hidden inter-function channel.

Decision: Resource contracts state outcomes, ownership, cleanup, capacity relations, and target guarantees, while completion and host scheduling are lowering concerns, because a source contract must be checkable without a scheduler in the language, instead of scheduling in the source model.

Decision: Arguments and paths preserve the target host's bytes with explicit conversion, and range-bearing operations expose their exact window and result relations, because a Unicode-only path model cannot represent every host path and an operation whose window is implicit cannot be proved against, instead of string-typed paths and implicit windows.

Rejected:
- Raw syscall numbers and integer file descriptors in source: rejected because they expose forgeable identities, an implicit global descriptor table, manual close, weak effect precision, poor Windows portability, and an unchecked pointer wall; they remain permitted only inside compiler-owned target code.
- Ambient free functions such as args, open, stdin, and spawn callable from any function: rejected because they hide access, create inter-function channels against the no-global rule, and make system use invisible in signatures.
- One permanently retained affine Process object borrowed by every system operation: rejected because every operation then contends for the same unique holder, falsely serializing files, output, networking, clocks, and workers, and sharing it would need a central lock or hidden aliasing.
- A literal WASI source contract: rejected because it imports Unicode-only paths, no guaranteed caller-buffer or zero-copy route, async tied to Component Model costs, and an incomplete threads and process surface chosen for cross-language components; WASI stays a possible target implementation for operations it can supply.
- A ring per scheduler thread, a kernel submission thread, and one armed multishot poll per receive half: rejected because each was built and measured on the same test and left the rate where it was; the ring is not where the time goes.
