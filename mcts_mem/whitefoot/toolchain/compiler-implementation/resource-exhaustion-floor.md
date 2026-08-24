- Exhausting a resource the trusted computing base supplies ends the process by a defined abort that first writes one fixed record naming only the exhausted resource class (`wf_floor.c`).
- That record names no source construct, rule identifier, function, node path, worker, host thread, dynamic call stack, address, depth, or size, and the absence of those fields is what distinguishes it from a language trap record.
- One latch serves every record any thread of the process can write, across both the runtime's writer and the emitted module's, and no execution writes a second record.
- The entry runs on a stack the runtime sizes rather than on the one the environment supplies, and a program's depth ceiling travels with the program rather than with the shell that started it.
- Every generated definition carries the target's stack-probing attribute, and a frame larger than the guard region walks its pages on the way down rather than stepping over them.
- A fault outside the reach of that probe below a thread's stack keeps the host's own disposition, and the process is not allowed to outlive the restoring of it.
- The record is a quality obligation of the implementation, not a language output: neither its bytes nor an exit status for the death is fixed by the language.

## Facts

- 2026-08-23 (178d4f69) measurement: before this floor, a stack that ran out produced zero diagnostic bytes and a bare host signal in every build, while a false claim — which a reviewed program cannot reach at all — produced a byte-exact record. (sourced)
- 2026-08-23 measurement: a stack-pointer check in every prologue consumes 9 to 23 percent and halves the depth a minimum-frame recursion reaches, where the target's own stack-probing attribute contains the same fault class at no measurable cost on this target. (sourced)
- 2026-08-23 (453f40e8) pitfall: discriminating "this thread ran out" from "this pointer is wild" by a generous window below the stack converts real corruption faults into exhaustion records, which misdirects worse than reporting nothing. The window has to be the probe's own geometry — one page-walk stride plus the ABI red zone — because that is the whole set of addresses a descent can fault at. (code)
- 2026-08-23 (453f40e8) pitfall: restoring the host's default disposition for a fault this mechanism does not own is per-signal and process-wide, while the classification above it is per-thread. A signal with no faulting instruction behind it is then swallowed and the mechanism is disarmed for every later thread, so the handler must make the process die under the restored disposition rather than return into it. (code)
- 2026-08-23 (5b460bdd) pitfall: a latch per record writer serializes each writer against itself and against nothing else, so two threads exhausting different resources at once each win a latch and interleave two records on one channel. (code)
- 2026-08-23 measurement: the mechanism costs one extra thread — a page-granular footprint delta and roughly a tenth of a millisecond per process, with the batch-to-batch spread on a shared host several times the effect. The reservation itself costs nothing: an 8 MiB and a 4 GiB entry thread measure the same. (sourced)

## Moves

- 2026-08-23 (178d4f69) replaced [[prologue-stack-pointer-check]]: an explicit check in every prologue spends the headroom it guards, where the target's own stack-probing attribute contains the same fault class for free and leaves the reporting to a signal disposition (sourced)
