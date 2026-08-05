# Current Plan

Status: ACTIVE — the owner authorized the complete `BOUND-1`
architecture-selection investigation on 2026-08-04; no architecture is
selected yet

Derived from: [Direction Outline revision 6](roadmap.md), items `CAND-8`,
`BOUND-1`, `PAR-4`, and `VERIFY-1`

## Goal

Select one coherent Whitefoot system interface before changing the language or
compiler. It must cover the architectural needs of command-line programs,
filesystems, clocks, randomness, networking, waiting and cancellation, and
future threads or tasks. The first implementation will still unblock `wfgrep`,
but it may not introduce an argv/file/stdout-only API that later capabilities
must replace.

Completeness in this stage means a complete semantic and performance envelope,
not implementing every system operation now. Later implementation slices must
be true subsets of the selected model, use the same resource and effect rules,
and leave no temporary public surface behind.

Working design evidence: [system-capability architecture dossier](../research/investigations/system-capability-architecture/DOSSIER.md).

Progress: the candidate architecture and exact first command slice are drafted,
and the final spec-consistency, OS-semantics, and future-evolution hostile
reviews report no remaining blocker. Owner architecture selection is the
remaining gate.

## Features to design in this stage

- [x] **Entry profile and exact authority imports** — define how a command,
  service, or embedded instance statically declares and receives exactly its
  host-granted capabilities without ambient mutable globals or a runtime bag of
  optional authority. General library/foreign entry remains BOUND-2.
- [x] **Typed resources and rights** — define unforgeable resource identity,
  ownership and borrowing, capability narrowing and delegation, explicit
  completion policy, compiler-derived cleanup, whole-process abort behavior,
  future containment obligations, and the `Sendable` / `Shareable` boundary.
  Every family must provide a protocol descriptor with state, aliases, owner
  disposition, concurrency, cancellation, ordering, and cross-platform
  guarantees.
- [x] **External effects** — keep authority, observable effects, and trusted
  provider identity separate. Define resource-local ordering, cross-resource
  ordering where promised, nondeterministic observations, blocking, independent
  operation progress, language suspension, spawning, cancellation, and cleanup
  effects without using provider metadata as source-semantic proof.
- [x] **Data transport** — select common semantics for stdin/stdout, files,
  pipes, and sockets: partial transfers, EOF, backpressure, errors, caller-owned
  buffers, vectored and positioned I/O, streaming, and an honest route to
  mmap/splice or other zero-copy strategies.
- [x] **System families** — map the coherent interface surface for process
  arguments/environment/status and stdio; filesystem roots, paths,
  directories, files, metadata and durability; clocks/timers and randomness;
  TCP/UDP/DNS and local endpoints; waits, cancellation, threads/tasks and join;
  and the disposition of child processes, signals, memory mapping, local IPC,
  and target/device capabilities.
- [x] **Provider and ABI** — define compiler-gated primitive identities, native
  host providers, target qualification, versioning, deterministic test
  providers, and a direct static lowering path that does not require a dynamic
  component runtime or per-call dispatch tax.
- [x] **Errors and conformance** — define portable outcomes, target-specific
  detail, partial progress, cleanup after recoverable failure, behavior at
  process-aborting traps and any future containment boundary, and the
  independent tests that every provider must pass.

## Binding constraints

- No raw integer fd, syscall number, pointer contract, writer-defined primitive,
  writer-visible `unsafe`, or function-name special case is a source-language
  authority.
- No ambient process API or single permanently unique `Process` handle may hide
  dependencies or serialize otherwise independent files, sockets, output
  streams, or workers.
- No mandatory whole-input materialization, per-byte boundary call, avoidable
  complete copy, centralized provider lock, or runtime-wide I/O serialization
  fence may be
  designed into the ordinary fast path.
- Paths must preserve the filename domain needed for a credible ripgrep
  replacement; a Unicode-only abstraction is not silently treated as full
  native-filesystem fidelity.
- Synchronous convenience may not make composable async, cancellation, or
  borrowed-buffer safety impossible. Async machinery is not selected merely
  because WASI uses it; Whitefoot must state its own ownership and cost model.
- The system interface and general foreign-code FFI are separate problems.
  `BOUND-1` may use a compiler-owned runtime/provider boundary but does not open
  a general import, callback, or dynamic-loading mechanism.

## Work

1. Audit official WASI 0.1, 0.2, and 0.3 plus the relevant native-host cost
   shapes. Preserve the useful lessons—preopened authority, typed resources,
   modular interfaces—and record the failures—flat fd ABI, Unicode-only paths,
   non-composable pollables, missing caller buffers/zero-copy guarantees, and
   unsettled threads/process support.
2. Compare at least raw fd/syscall, ambient process functions, one affine
   `Process` capability, and typed entry interfaces plus runtime resources.
   Reject alternatives by safety, effect precision, concurrency, path fidelity,
   ABI portability, code shape, and implementation cost rather than style.
3. Draft one candidate architecture; instantiate ReadFile, possibly aliased
   stdout/stderr, TCP split plus pending cancellation, and Child protocol
   descriptors; and trace three hostile witnesses through it: the first
   `wfgrep PATTERN FILE...` command path, parallel file search with independent
   workers and ordered publication, and a network service with timeout,
   backpressure, cancellation, and teardown.
4. Inventory the exact v0.17 specification and compiler deltas. Build a small
   executable semantic model only if paper traces cannot settle a resource,
   cancellation, or buffer-lifetime question; do not build a general framework.
5. Present the recommended candidate architecture, rejected alternatives, open
   questions, and the exact first implementation slice for owner review.
   Specification and compiler work require the subsequent approved plan and,
   where applicable, the specification-change workflow.

## Verification

- Every operation family states authority, states and transitions, alias and
  cursor relations, input/output ownership on every outcome, partial progress,
  error and cleanup behavior, effects and ordering, permitted concurrent
  operations, blocking or suspension, cancellation and quiescence,
  thread-transfer rules, ABI mapping, and cost shape.
- The three witnesses use the same core rules; none needs a project-shaped
  primitive, hidden global, central serialization token, or replacement API.
- The design admits initialized caller-owned read buffers, chunked/vectored I/O,
  and a future zero-copy route without exposing uninitialized bytes or extending
  a borrow across an untracked suspension.
- A static native provider can lower ordinary hot operations without allocation
  or dynamic dispatch that the operation itself does not require.
- Hostile review attacks capability forgery, path escape, wrong-resource effect
  reordering, partial I/O, close/error races, cancellation races, cross-thread
  use, provider lies, and portability mismatches.

## Done when

- the owner can choose one architecture from an explicit alternative table;
- the chosen model has a complete capability-family map with deliberate v1,
  later, and unsupported dispositions;
- the v0.17 semantic gaps and provider TCB are explicit;
- the three witnesses pass the paper/model and performance-shape review; and
- the next Current Plan can name one exact implementation slice without making
  another architectural decision.

## Not in this stage

- No numbered-specification, compiler, runtime, or `wfgrep` implementation.
- No promise to implement every mapped system family in the first release.
- No general FFI, dynamic loading, plugin system, artifact replay, or provider
  marketplace.
- No matcher, directory walker, parallel runtime, or ripgrep timing work.

## Parallel research

None. System-capability architecture is the current work.
