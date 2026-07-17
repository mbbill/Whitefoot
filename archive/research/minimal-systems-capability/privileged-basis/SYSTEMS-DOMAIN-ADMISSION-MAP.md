# Systems-Domain Admission and Dependency Map

Status: hostile-review-corrected research mapping for owner review,
2026-07-15. This file classifies every domain in `SYSTEMS-DOMAIN-LEDGER.md` by
its likely language, ordinary-library, toolchain, or irreducible gate
dependencies. It is an admission and obligation map, not a derivability proof.
Conditional public-basis routes remain unproved until their exact logic,
cleanup, sealing, and performance obligations close.

## 1. Route vocabulary

| Route | Meaning |
|---|---|
| `LANGUAGE` | The capability is an ordinary checked language semantic, not privileged library authority. |
| `BASIS?` | A conditional hypothesis that ordinary code could use a checked public resource basis. The exact proof logic and complete derivation remain open. |
| `LIBRARY` | Ordinary checked code is the intended policy layer once its `LANGUAGE`, `BASIS?`, or gate dependencies exist. This label does not prove those dependencies sufficient. |
| `GATE(frame)` | An irreducible machine, runtime, allocator, ABI, OS, scheduler, target, or device operation enters through one exact fixed-root or approved-capsule entry for the named frame. |
| `TOOLCHAIN` | The capability belongs to the compiler/build/package system rather than a runtime library. Privileged facts, if any, still use the sealed capability admission verifier. |
| `NON-GOAL` | The exact Rust contract conflicts with an owner-ratified xlang law; any remaining underlying need has another route. |

`GATE` is one admission mechanism, not one semantic operation. Every frame
entry has its own effect, resource, failure, fact, backend, proof, and review
identity.

## 2. Complete 26-domain map

| Domain | Admissible construction route | Why no standard-library privilege is needed | Performance and remaining closure |
|---|---|---|---|
| D01 primitive values, arithmetic, aggregate views | `LANGUAGE`; target-sensitive numeric operations may use `GATE(F-TARGET)` | Checked scalar operations and aggregate views are language semantics. Ordinary numeric conveniences are `LIBRARY`. | Existing checked arithmetic and bounds behavior remains. Complete floating point, math, constant evaluation, and target behavior retain later contracts. |
| D02 core language and callables | `LANGUAGE` | Calls, control flow, static behavior, modules, and callable abstraction cannot be supplied by a privileged collection. They are ordinary language features. | Direct monomorphized calls are the zero-dispatch route. Dynamic dispatch and separate compilation remain later design work. |
| D03 generic behavior, borrowing, conversion, comparison, hashing | `LANGUAGE` plus `LIBRARY`; state/provenance obligations may need `BASIS?` | Generic contracts and borrows are checked language relations. Libraries are intended to implement behaviors in ordinary code. | Direct specialization can avoid mandatory dispatch. Arbitrary behavior remains effectful unless proved; later open/dynamic behavior retains its own cost. |
| D04 layout, raw memory, pointers, provenance | `GATE(F-MEM)` beneath `BASIS?`; ordinary raw authority remains absent | The gate supplies layout and physical memory effects. A future public basis would expose typed roots, footprints, transitions, and borrows rather than raw dereference or unchecked initialization. | Erasable proof state is supported by precedent but not yet shown for xlang. Custom allocation, pinning, volatile access, and arbitrary provenance remain later families. |
| D05 optional values and recoverable errors | `LANGUAGE` plus `LIBRARY` | `Option`/`Result`-class ownership is ordinary checked algebraic data. Error policy is ordinary code. | No exceptions or unwind machinery. Allocation, I/O, callback, and cancellation failures retain domain-specific commitment contracts. |
| D06 traps, assertions, diagnostics | `LANGUAGE` for abort semantics; `GATE(F-TRAP)` for process termination and optional stack capture; presentation is `LIBRARY` | The trap operation is a narrow frame, while assertion structure and messages are ordinary code. | No cleanup-on-trap or unwind tax. Rich diagnostics and backtraces remain separately priced. Catching and resuming are `NON-GOAL` under current law. |
| D07 unique heap ownership and allocation | `GATE(F-ALLOC)` creates/releases a generative root; `BASIS?` would track ownership and typed state; policies are `LIBRARY` | Allocation is irreducible. A named container need not be privileged if sealing, live-state proof, and cleanup close. | Equal allocation count and layout are hypotheses. Failure policy, full/inline adoption, custom allocation, and over-alignment remain open. Current `box<T>` remains builtin. |
| D08 shared ownership and interior mutation | `BASIS?` would need duplicable observations and checked lifecycle/borrow transitions; atomic variants also use `GATE(F-SYNC)` | Ordinary-library counts, weak handles, borrow state, and policies are the objective, not a proven result. | Full soundness, overflow, cycles, weak lifecycle, dynamic borrowing, and concurrent destruction require a later concurrent proof family. |
| D09 sequential collections and topology witnesses | `BASIS?` plus `LIBRARY` | A bounded range grammar plausibly covers dense, interval, ring, and gap forms. Dense/indexed representations may keep complex map or graph semantics outside safety authority. Sealing, cleanup, inline adoption, and any truly safety-relevant sparse liveness remain open. | No universal bitmap has been shown necessary, but equal layout, operations, and protected-path cost are unproved. |
| D10 synchronous iteration and ranges | `LANGUAGE` borrowing plus conditional `BASIS?` footprint partition/transfer; adapters are `LIBRARY` | A cursor may be an ordinary sealed owner or borrow over a remaining footprint if exact close and escape rules exist. | No-intermediate-collection and erased-token routes are hypotheses. Escape, invalidation, short circuit, and cleanup remain exact obligations. |
| D11 bytes, UTF-8, characters, ASCII, strings | Conditional D09 storage through `BASIS?`; validation and refinement need `LANGUAGE` proof/fact relations; algorithms/tables are `LIBRARY` | A string may be an ordinary byte owner plus a checked refinement. No privileged string representation has been shown necessary. | Dense byte layout is plausible; complete validation, mutation, normalization, locale, collation, segmentation, and width remain later families. |
| D12 formatting and textual diagnostics | `LIBRARY`, depending on D03 behavior, conditional D09/D11 storage, and `GATE(F-IO)` only for an OS destination | Formatting policy and integer/text algorithms need no direct privilege. | Zero-allocation and bounded-output implementations remain construction and measurement obligations. |
| D13 runtime type identity and reflection | `LANGUAGE`/`TOOLCHAIN` for any selected type descriptor semantics; layout truth uses `GATE(F-MEM)` | Reflection, if adopted, is compiler metadata with ordinary checked query APIs, not a reason to grant raw memory access. | No route is selected because reflection remains a later family. |
| D14 source, configuration, build, compile-time metadata | `TOOLCHAIN`; immutable privileged facts use exact gate identities | Build configuration and inclusion belong to the build/compiler contract, not a runtime container. | Runtime cost may be zero, but packages, separate compilation, build scripts, stable ABI, and loading remain later contracts. |
| D15 byte and structured I/O | In-memory buffering is conditionally `LIBRARY`; OS progress and handles use `GATE(F-IO)` plus future resource-state proof | Ordinary code is intended to implement buffering, codecs, and retry policy over checked partial-progress operations. | Scatter/gather and buffering equivalence depend on exact frame operations; interruption, deadlines, and cancellation remain open. |
| D16 filesystems and paths | Lexical paths are `LIBRARY`; filesystem effects use `GATE(F-FS)` plus future resource-state proof | Path policy can be ordinary. The frame supplies irreducible calls. | Races, links, permissions, encodings, atomicity, traversal, and platform differences remain open. |
| D17 environment and process control | Parsing/building is `LIBRARY`; runtime process operations use `GATE(F-PROC)` plus future resource-state proof | Command policy and argument storage can be ordinary over narrow operations. | Spawn, wait, termination, pipes, signals, and portable status remain open. |
| D18 FFI, platform strings, OS handles | `GATE(F-ABI)` plus future resource-state proof; pure transformations are `LIBRARY` | An exact frame owns foreign effects and fact attenuation. Opaque foreign behavior remains an explicit trust assumption. | Reentry, callbacks, foreign threads, pinning, ABI breadth, cleanup, and fact preservation remain open. |
| D19 network values and network I/O | Pure values/parsing are `LIBRARY`; effects use `GATE(F-NET)` plus future resource-state proof | Address/protocol policy can be ordinary; the frame supplies socket transitions. | DNS, blocking, shutdown, timeout, TLS, async I/O, and platform behavior remain open. |
| D20 durations, clocks, timers | Duration arithmetic is `LANGUAGE`/`LIBRARY`; observations and waits use `GATE(F-CLOCK)` | Time arithmetic and deadline policy can be ordinary; clocks and waits are irreducible. | Resolution, suspension, errors, wall-clock semantics, and async integration remain open. |
| D21 threads and thread-local state | `GATE(F-THREAD)` plus future `LANGUAGE` send/share rules and concurrent proof logic | Ordinary-library handles and scoped joins are a conditional architecture once a concurrency model exists. | Current v0 is single-threaded; race freedom, join, parking, foreign entry, and TLS destruction remain unproved. |
| D22 atomics, synchronization, channels | Atomic/blocking operations use `GATE(F-SYNC)`; invariants require an unselected concurrent proof logic; policies are intended as `LIBRARY` | Atomics are irreducible, but named mutex/channel containers need not be privileged if their proofs close. | Ordering, publication, reclamation, disconnect, blocking, deadlock, and cleanup remain open. |
| D23 futures, tasks, async, cancellation, pinning | Scheduling/wakeup uses `GATE(F-ASYNC)`; address/lifecycle invariants require future `LANGUAGE` and proof rules | Ordinary-library executors and futures are a goal, not a derivation. | Cancellation, progress, self-reference, wake correctness, address stability, and exact destruction remain open. |
| D24 architecture facilities, hints, target facts, MMIO | `GATE(F-TARGET)` and `GATE(F-MMIO)` with separate exact entries and fact ledgers | Target instructions and device accesses are irreducible. One gate can admit them without exposing arbitrary intrinsic authority. | Multiplicity, width, ordering, tearing, faults, and fact attenuation remain per-entry obligations. |
| D25 prelude, aliases, reexports, packaging | `TOOLCHAIN` plus ordinary module aliases | These are surface organization, not independent runtime capability. A compiler core can expose the public algebra without shipping named containers. | No runtime cost. Any surface with a distinct semantic guarantee returns to its owning domain. |
| D26 compiler/runtime support | Every privileged edge is one exact fixed-root or approved-capsule entry assigned to an existing frame; proof checking and backend parity are `TOOLCHAIN` obligations | The sealed admission verifier is a candidate common route for lang-item, intrinsic, special-type, and frame authority. Runtime helpers receive no ambient authority. | Every semantic branch, fact, target, helper, certificate, and trust assumption remains independently counted. |

## 3. Coverage conclusion

All 26 domains can be classified into four dependency kinds:

1. ordinary checked language semantics;
2. conditional ordinary libraries over a still-unselected public resource basis;
3. irreducible machine or external operations admitted through the one sealed
   gate; or
4. compiler/build/package functionality outside runtime-library privilege.

The map found no evidence that a privileged named container is inherently
required, but it does not prove that one small public basis can replace every
such container. Safety-relevant sparse liveness, recursion, shared ownership,
concurrency, cleanup, and external resources still need exact dispositions.
Abstract algorithm invariants do not become safety-proof obligations unless
they are consumed as checked contracts or facts. No domain is closed by this
map.
