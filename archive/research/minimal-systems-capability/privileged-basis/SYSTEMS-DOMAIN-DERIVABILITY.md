# Systems-Domain Derivability

Status: proposed D14 research derivability partition pending owner review,
2026-07-15. This artifact maps all 26 demand categories to candidate routes in
the proposed certified-resource architecture. It is a category-level
constructive design argument, not exact D-2 closure, exact P-1 evidence, or
evidence that the current language or compiler implements any missing route.
Exact dense D-2 fails closed with 340 unique unresolved obligations across 150
contexts: 208 are Convert-implicated, 136 are allocator-implicated, and 12
concern ZST/fullness, with 16 in both the Convert and allocator sets. Exact P-1
remains pending.

## 1. Terminal route vocabulary

| Route | Meaning |
|---|---|
| `CORE(R-n)` | Fixed checked reference semantics. It is public and carries no package or standard-library privilege. |
| `ORDINARY` | Ordinary checked code using current core semantics. |
| `CERTIFIED(R-n...)` | An ordinary opaque proof-carrying resource module checked against the named fixed-policy axes. It requires no approval snapshot. |
| `GATE(F-x)` | One exact irreducible machine, runtime, allocator, ABI, OS, scheduler, target, or device entry admitted by the single sealed gate. |
| `TOOLCHAIN` | Compiler, build, package, proof, or final-code functionality rather than a runtime library. Any semantic edge still follows the same gate. |
| `NON-GOAL` | An exact behavior prohibited by an existing owner law; an underlying systems need, if any, has another route. |

Multiple terms in one cell are dependencies, not alternate unclassified
outcomes. There is no `BASIS?`, unnamed future logic, or container-specific
privilege in this partition.

Every executable route also depends transitively on R-6 final loaded-image
binding. The table names the additional local semantic axes so that repeating
R-6 in all 26 rows cannot hide a missing domain dependency.

## 2. Candidate 26-domain partition

| Domain | Terminal construction route | Constructive derivation and cost disposition |
|---|---|---|
| D01 primitive values, arithmetic, aggregate views | `CORE(R-1,R-2)`; target-specific operations `GATE(F-TARGET)` | Checked scalar and aggregate operations remain core. Target entries expose exact operation/fact contracts. Ordinary numeric algorithms add no privilege or dispatch. |
| D02 core language and callables | `CORE(R-3)` plus `ORDINARY`; indirect machine transfer has exact R-6-bound `GATE(F-TARGET)` lowering | Calls, control flow, algebraic data, modules, and static behavior are language semantics. A known callable specializes to a direct call. Runtime-selected callables are ordinary existential `CodeLease + Environment + InvokeAuth` protocols; only their checked machine transfer is irreducible. |
| D03 generic behavior, borrowing, conversion, comparison, hashing | `CORE(R-1,R-3)` plus `ORDINARY` or `CERTIFIED(R-1,R-3)`; dynamic dispatch additionally uses the D02 target edge | Direct generic specialization supplies static dispatch. Runtime-selected behavior uses the same checked callable protocol. Borrow leaves retain exact roots and regions; stateful behavior effects and returned provenance are proved in ordinary contracts. No vtable is mandatory or forbidden. |
| D04 layout, memory, pointers, provenance | `CORE(R-1,R-2)` over exact `GATE(F-MEM)` entries | Generative byte carriers, checked typed-place/layout/access-profile witnesses, live/dead places, partition reshape, place epochs, and checked access replace raw authority. Packed/unaligned values use checked by-value operations and never mint misaligned references. Allocation, mapping, copying, and target layout edges are exact entries. Static evidence erases; runtime-dependent layouts retain only selected descriptors. |
| D05 optional values and recoverable errors | `CORE(R-3)` plus `ORDINARY` | Sum types and explicit error results carry exact owners. R-3 distinguishes precommit recovery, partial progress, and abort. No exception or unwind runtime is required. |
| D06 traps, assertions, diagnostics | abort is `CORE(R-3)`; termination/backtrace may use `GATE(F-TRAP)`; presentation is `ORDINARY`; catch/resume is `NON-GOAL` | Accepted execution performs no invalid action before trap. Abort owes no unwind cleanup. Rich diagnostics are ordinary formatting over exact optional frame actions. |
| D07 unique heap ownership and allocation | `GATE(F-ALLOC)` plus `CERTIFIED(R-1,R-2,R-3)` | A recoverable allocation entry yields a fresh empty carrier or no owner. Ordinary modules have candidate routes to boxes, vectors, trees, arenas, and custom policies with exact disposition. The architecture is intended to preserve allocation count and layout; exact same-contract P-1 evidence remains pending. |
| D08 shared ownership and interior mutation | `CERTIFIED(R-1,R-3,R-4)` plus atomics when concurrent through `GATE(F-SYNC)` | Strong/weak counts, lifecycle, dynamic borrow state, guards, overflow, and cycle policy are ordinary protocols. Runtime counts or borrow words appear only in contracts that require them. |
| D09 sequential collections and topology | `CERTIFIED(R-1,R-2,R-3)` plus `GATE(F-ALLOC)` when dynamic | Dense, ring, gap, sparse, recursive, stable-index, and arbitrary metadata relations are ordinary invariants over carriers and `take`/`put`. `LiveShape` is derived automation. No named collection is privileged. |
| D10 synchronous iteration and ranges | `CERTIFIED(R-1,R-3)` plus `ORDINARY` adapters | A cursor owns or borrows an exact remaining footprint and source map. Each yield transfers or reborrows exact resources; disposition handles early stop. Composition needs no intermediate collection or dynamic dispatch. |
| D11 bytes, UTF-8, characters, ASCII, strings | `CERTIFIED(R-1,R-3)` plus `ORDINARY` algorithms/tables | A string is an ordinary byte carrier plus a checked refinement predicate. Mutation invalidates or re-establishes the predicate. Raw bytes pay no text metadata or validation tax. |
| D12 formatting and textual diagnostics | `ORDINARY` over D03/D09/D11; destination effects use `GATE(F-IO)` | Formatting algorithms, behavior calls, buffers, and bounded sinks are ordinary. Only the external sink is irreducible. Zero-allocation forms use caller-provided or fixed carriers. |
| D13 runtime type identity and reflection | selected metadata is `TOOLCHAIN` exposed through `CORE`; target layout uses `GATE(F-MEM)` | A compiler may emit exact type descriptors and checked queries. Reflection grants no raw storage authority. If no reflection contract is selected, ordinary code has no ambient descriptor. |
| D14 source, configuration, build, compile-time metadata | `TOOLCHAIN` | Inclusion, configuration, packages, generated metadata, proof dependencies, and build identity belong to the compiler/build contract. Privileged semantic inputs are authenticated by the one gate; ordinary macros are not needed for runtime expressibility. |
| D15 byte and structured I/O | `GATE(F-IO)` plus `CERTIFIED(R-1,R-3,R-5)` and `ORDINARY` policy | An exact read entry returns a live prefix, dead suffix, handle state, and partial-progress outcome. Ordinary loops derive buffering, codecs, retry, scatter/gather policy, and `read_exact` without zero-filling spare bytes. |
| D16 filesystems and paths | lexical policy is `ORDINARY`; effects are `GATE(F-FS)` wrapped by `CERTIFIED(R-3,R-5)` | Path storage and normalization are ordinary. Exact entries state links, races, permissions, encodings, atomicity, partial effects, and handle disposition. Wrappers add no trust. |
| D17 environment and process control | construction/parsing is `ORDINARY`; effects are `GATE(F-PROC)` plus `CERTIFIED(R-3,R-5)` | Arguments, environment maps, pipes, and policy are ordinary structures. Spawn, wait, signal, and status transitions are exact external-resource protocols. |
| D18 FFI, platform strings, OS handles | `GATE(F-ABI)` plus `CERTIFIED(R-1,R-2,R-3,R-5)` and R-4 when callbacks or foreign threads interfere | The capsule names ABI, partial effects, reentry, callbacks, foreign threads, pointer escape, fact attenuation, and cleanup. Ordinary modules prove marshalling and wrapper policy. Retained pointers escrow exact move/release authority through stable-place leases. |
| D19 network values and network I/O | values/parsing are `ORDINARY`; effects are `GATE(F-NET)` plus `CERTIFIED(R-1,R-3,R-5)` | Addresses and protocols are ordinary data. Socket creation, DNS/provider behavior, live/vacant buffer progress, partial I/O, shutdown, and timeouts are exact frame transitions; buffering and protocol stacks remain ordinary. |
| D20 durations, clocks, timers | arithmetic is `CORE`/`ORDINARY`; observations and waits are `GATE(F-CLOCK)` plus `CERTIFIED(R-3,R-5)` | Duration math is ordinary checked arithmetic. Clock identity, resolution, monotonicity, sleep/wake, failure, and timer ownership are exact entries and ordinary wrapper protocols. |
| D21 threads and thread-local state | `GATE(F-THREAD)` plus `CORE(R-4)` and `CERTIFIED(R-1,R-3,R-4)` | Spawn transfers an exact resource package; join returns it. Scoped threads, handles, parking policy, and TLS lifecycle are ordinary protocols. Data-race rejection is fixed semantics, not library trust. |
| D22 atomics, synchronization, channels | atomics/park/wake are `GATE(F-SYNC)`; structures are `CERTIFIED(R-1,R-2,R-3,R-4)` | R-1/R-2 fix typed atomic storage identity and leases; R-4 fixes events and orderings. Ordinary proofs derive mutexes, condition variables, channels, reference counts, lock-free algorithms, and reclamation. Only algorithm-required atomics and metadata remain at runtime. |
| D23 futures, tasks, async, cancellation, pinning | `CERTIFIED(R-1,R-2,R-3,R-4,R-5)` plus `GATE(F-ASYNC)` for exact OS registration/readiness/completion/cancellation observations | Futures, executor queues, scheduling policy, task storage, and waker ownership are ordinary state machines with exact wake and cancellation ownership. Total cancellation permits a verified plan; outstanding or fallible cancellation remains `MustClose`. No async keyword, coroutine runtime, or per-future heap allocation is semantically required. |
| D24 architecture facilities, hints, target facts, MMIO | exact `GATE(F-TARGET)` and `GATE(F-MMIO)` entries plus `CERTIFIED(R-1,R-2,R-3,R-4,R-5)` wrappers as applicable | Every opcode/device action has separate target, width, ordering, tearing, fault, effect, fact, storage-lifecycle, and ownership identity while entering through one gate. Device leases and volatile protocols are ordinary resource wrappers. |
| D25 prelude, aliases, reexports, packaging | `TOOLCHAIN` plus ordinary modules | These organize names and defaults, not runtime authority. The candidate is intended to let user libraries define derived contracts through the public kernel without requiring xlang to ship named collection implementations; exact coverage is not yet established. |
| D26 compiler/runtime support | `TOOLCHAIN`, `CORE(R-6)`, and exact existing-frame `GATE` entries | The canonical IR verifier, proof erasure, and end-to-end refinement or checked per-artifact validation bind proofs through optimization, object emission/assembly, linking, relocation, provider resolution, and loading to the immutable loaded image and receipt. Runtime helpers have no ambient authority. |

## 3. Cross-domain constructive joins

The partition is not merely a list of destinations. The following joins show
that a standard-library privilege is unnecessary:

### 3.1 Memory-backed owner

```text
GATE(F-ALLOC)
  -> fresh vacant carrier under R-1/R-2
  -> ordinary certified invariant and operations under R-3
  -> static Plan or MustClose disposition
  -> final-code binding under R-6
```

This route derives dense and sparse containers, strings, trees, pools, arenas,
boxes, shared owners, and task frames.

### 3.2 Concurrent abstraction

```text
GATE(F-THREAD,F-SYNC)
  -> fixed atomic/thread events under R-4
  -> ordinary user-defined invariant and protocol
  -> checked transfer/reclamation/disposition under R-3
  -> exact target lowering under R-6
```

This route derives locks, channels, atomically shared owners, and lock-free
structures without a privileged named implementation.

### 3.3 External resource

```text
GATE(exact frame action)
  -> exact R-5 outcome and residual assumption
  -> ordinary typestate, retry, buffering, and policy
  -> verified Plan when total, otherwise MustClose
  -> exact provider/final-code binding under R-6
```

This route derives files, sockets, process handles, clocks, devices, FFI
adapters, and asynchronous operations.

## 4. No-standard-library architecture hypothesis

At category granularity, every enumerated runtime domain has a candidate terminal
construction in fixed public semantics, ordinary checked or certified code, and
exact irreducible frame actions, conditional on a certificate and the named
machine/provider profile. This supports the hypothesis that a future compiler
could expose a small public resource kernel and authenticated capability
contracts without shipping named collection, string, synchronization, async, or
resource-wrapper implementations.

If the architecture is later selected and implemented, an AI may generate those
libraries for an application. Accepted proofs would be checked, runtime
representations would remain module-selected, and machine edges would resolve
through the one proposed gate. This category map does not prove that every exact
container or protocol avoids additional public authority: dense D-2 currently
fails closed with 340 unique unresolved obligations across 150 contexts, and
exact P-1 remains pending. No production implementation or language change is
authorized here.
