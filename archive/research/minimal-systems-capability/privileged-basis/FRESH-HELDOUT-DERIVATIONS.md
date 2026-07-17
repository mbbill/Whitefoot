# Fresh Held-Out Derivations

Status: independent D14 hostile paper derivation for a proposed architecture
pending owner review, 2026-07-15. This artifact tests the candidate on
structures that were not named in `CERTIFIED-RESOURCE-BASIS-DECISION.md`. It is
not exact D-2 closure, exact P-1 evidence, a mechanized proof, implementation,
performance result, or production decision.

## 1. Independence protocol

The derivation agent read the proposed decision, systems-domain ledger,
capability-obligation registry, onboarding context, and relevant design-memory
nodes. It did not read the earlier sequential or systems witness derivations
before choosing and attacking these cases. It selected four systems with
different stressors:

1. concurrent cuckoo indexing with incremental migration;
2. crash-consistent LSM storage with snapshots and background compaction;
3. a hot-swappable JIT code cache with concurrent execution; and
4. sparse GPU residency with DMA and reset recovery.

A case earns an architecture-level paper-witness disposition only if its data-
structure and policy layer is ordinary checked code, every irreducible machine
or provider action is an exact entry under the proposed gate, all move, release,
code, and external leases close exactly in the trace, and the construction names
no obviously pathological representation. This test does not discharge exact
D-2 or exact P-1.

## 2. Concurrent cuckoo index with incremental resizing

The ordinary invariant is:

```text
Bucket(r,i) =
    Empty(version) * Vac(r,i)
  | Occupied(version,hash) * Full(r,i,(K,V))
  | Migrated(version,new_i) * Vac(r,i)

Index =
  old carrier * optional new carrier
  * migration authority * bucket locks
  * reader epochs * sealed key-placement invariant
```

Lookup opens one shared bucket invariant only after acquiring its lock. An
insertion computes a relocation path, locks buckets in canonical order,
revalidates the path, and moves entries tail-to-head through one exact vacancy.
All allocation, hashing, and recoverable work finishes before the first move.
After commitment, every normal edge completes repair and reseals the index.

Incremental resizing acquires the new carrier first. Migration transfers each
bucket once, publishes `Migrated`, and keeps the old carrier and its
`ReleaseAuth` escrowed until every old-reader epoch ends. Operation guards have
a nonblocking plan that releases locks and epoch leases. The shared index is
`MustClose` while operations or migration workers survive.

R-4 supplies atomic/lock events and invariant opening. Ordered path locking
avoids an apparent need for multiword atomics. The candidate runtime account
contains the cuckoo algorithm's control words, locks, migration state, and reader
epochs. Roots, live/vacant state, and migration proof authority are intended to
erase; exact P-1 parity remains pending.

Attack result: key-placement correctness alone cannot authorize payload access;
the checked occupied state and opened interference invariant are still required.
No cuckoo-specific primitive or general fact escape is needed.

Disposition: architecture-level `PAPER_WITNESS` under R-1 through R-4 and the
proposed R-6 chain. This does not discharge exact D-2 or P-1.

## 3. Crash-consistent LSM storage engine

The ordinary resource protocol is:

```text
Database<S> =
  WAL owner * manifest owner * mutable-table owner
  * authoritative version/run set * cleanup obligations
  * worker owners * Inv(S)

Snapshot(v) = read leases over immutable runs reachable from version v
```

Persistent bytes are decoded and validated before becoming typed live values.
The state progression is `Closed -> Recovering -> Serving -> Quiescing ->
Closed`. A strong commit contract distinguishes at least:

```text
NotCommitted(database, offered input)
Committed(database', sequence)
Indeterminate(database', transaction identity)
```

An `fsync` failure cannot be reported as rollback when the write may already be
durable. Flush and compaction construct and validate a temporary run, perform
the required synchronization, publish a new manifest/version, and only then
retire old runs. Snapshot leases prevent deletion and escrow release authority
until every reader of the old version ends.

The database is `MustClose`: joining workers, synchronization, deletion, and
handle closure may block or fail. A failed close returns a continuation owning
the exact remaining obligations. WAL headers, checksums, run metadata, filters,
version references, compaction queues, and file operations are algorithm costs.
The paper route introduces no ownership header or destructor registry; exact
P-1 parity remains pending.

Attack result: ordinary proofs cannot invent persistence barriers, rename and
directory-sync semantics, torn-write behavior, or power-loss ordering. Memory,
ownership, concurrency, partial-I/O, and cleanup safety derive. A durability
headline requires an exact filesystem persistence frame under the existing
gate, including crash transitions and residual provider assumptions.

Disposition: ordinary protocol `PAPER_WITNESS`; crash durability is
`IRREDUCIBLE_GATE` through an exact proposed `F-FS` persistence frame, not a new
public container authority. This does not discharge exact D-2 or P-1.

## 4. Hot-swappable JIT code cache

The checked states are:

```text
BuildImage =
  writable mapping * canonical IR * relocation state * owned output cells

ExecImage<h,sig,target> =
  immutable executable mapping * exact loaded-image identity h
  * Callable<sig> * no-write lease

Building -> Validated -> Executable -> Published -> Retired -> Unmapped
```

Generation and relocation failure occur while the image is private and
writable. Sealing validates canonical IR against the exact native result,
finishes relocations, removes write authority, performs required instruction-
cache synchronization, maps executable pages, checks the load receipt, and
only then creates `Callable<sig>`.

Callers acquire execution/code leases before entry. Replacement publishes a
new immutable image with release semantics; the old image remains mapped until
every lease ends. A build image may use a total unmap plan only when the platform
contract proves valid-owner unmapping total. Otherwise the cache is
`MustClose`.

Runtime code pages, relocation records, one dispatch/version cell, and selected
epoch or hazard state are inherent to hot replacement. The paper route adds no
per-call structure beyond the chosen dynamic branch; exact P-1 parity remains
pending. Post-validation patching invalidates the certificate and requires a new
immutable version and validation.

Attack result: arbitrary proof-carrying native objects remain rejected. The
candidate route uses the proposed R-6 end-to-end refinement or post-link,
post-relocation validation and load receipt, plus exact W^X, cache-maintenance,
mapping, and indirect-call edges under the proposed gate.

Disposition: architecture-level `PAPER_WITNESS` conditional on the proposed
exact R-5/R-6 target profile. This does not discharge exact D-2 or P-1.

## 5. Sparse GPU residency with DMA and reset

The ordinary protocol owns:

```text
VaLease * PhysicalPages * MappingOwner
* DescriptorOwner * CommandBatch * FenceOwner

HostOwned(buffer)
  -> DeviceInFlight(buffer,queue,fence,footprint)
  -> HostOwned(buffer)
```

Submission transfers the exact buffer footprint and escrows CPU move/release
authority in the device lease. CPU borrows and facts over device-writable memory
end first. Only an authenticated completion event restores host access.
Eviction requires an idle mapping and no outstanding lease before unmap and
page release.

Arbitrary affine host values cannot be bit-copied to device memory. A payload
needs a proved device representation with no untracked host owner/borrow, or an
exact kernel contract transferring those resources. Multiple queues use R-4
for host bookkeeping. The device capsule fixes queue ordering, DMA visibility,
event identity, cache maintenance, tearing, fact attenuation, and reset
outcomes. A CPU Boolean or ordinary atomic flag cannot prove DMA completion.

Reset acknowledgement must state whether DMA quiesced, which mappings survive,
and which resources are lost. Until a terminal outcome, in-flight resources
remain `MustClose`. Page tables, residency metadata, descriptors, fences, and
cache actions are selected algorithm costs; proof resources are intended to
erase. Exact P-1 parity remains pending.

Attack result: mapping, submission, DMA, fences, reset, and MMIO are genuine
R-5/device edges. Hashing, residency choice, eviction, batching, and retirement
remain ordinary policy and may not be absorbed into a privileged capsule.

Disposition: architecture-level `PAPER_WITNESS` under the proposed exact
device/DMA/reset frame; the trace introduces no new language or container
primitive. This does not discharge exact D-2 or P-1.

## 6. Held-out paper-witness result

All four cases use the same ordinary proof mechanism, exact focus,
move/release/code lease accounting, `Plan`/`MustClose` lifecycle, and one
privileged gate. None requires writer-visible unchecked code, a privileged
named data structure, a universal runtime topology tag, or an additional
storage transition.

The cases also expose a candidate boundary: the ordinary proof may derive
algorithms and protocols, but it cannot manufacture crash, loader, DMA, reset,
or weak-memory truth. Those remain exact, reviewable gate entries. The result is
bounded structural expressibility evidence only. Exact dense D-2 still fails
closed with 340 unique unresolved obligations across 150 contexts: 208 are
Convert-implicated, 136 are allocator-implicated, and 12 concern ZST/fullness,
with 16 in both the Convert and allocator sets. Exact P-1 remains pending. Proof
automation, checker soundness, generated-code quality, measured
performance, production implementation, and language changes remain later,
separately authorized gates.
