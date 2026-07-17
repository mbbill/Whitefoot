# Certified Resource Basis Hostile Review

Status: **ARCHITECTURE-LEVEL PAPER REVIEW; PROPOSED CANDIDATE PENDING OWNER
REVIEW**, 2026-07-15. The review found no architecture-shape blocker in the
paper proposal. It does not authenticate the separately generated exact dense-
outcome ledger and does not grant a combined D14 PASS. Exact dense D-2 fails
closed with 340 unique unresolved obligations across 150 contexts: 168 coarse
Convert route-direction, 24 Convert callable-direction, 136 outer-allocator-
applicability, six IntoOwner ZST/capacity-reshape, and six IntoBoxed no-change
fullness/ZST obligations. Equivalently, 208 are Convert-implicated, 136 are
allocator-implicated, and 12 concern ZST/fullness, with 16 in both the Convert
and allocator sets. Exact P-1 remains pending.

No production language, specification, compiler, capability entry, standard
library, container, experiment, or teaching change is authorized by this
review.

## 1. Reviewed bytes

| Artifact | SHA-256 |
|---|---|
| `CERTIFIED-RESOURCE-BASIS-DECISION.md` | `94fab00f99909f5211d18e11eb86d596523bb94c77a2f0c71af598bd10551e0d` |
| `SYSTEMS-DOMAIN-DERIVABILITY.md` | `f44107e2abadb71959297ceccc47dc90a7c07948410bb874c2e81e851a34cea4` |
| `SYSTEMS-WITNESS-DERIVATIONS.md` | `b82012adc8af1891355e48ce18369bcafeddab0f21bcdb1cfd6df873edbe91b3` |
| `FRESH-HELDOUT-DERIVATIONS.md` | `134dc8a2a61fe987a7ff03cdad6c32266a63f4207728beb00219b2d48df75146` |
| `GATE-AUTHENTICATION-LOCK.md` | `6c104f0cba7a3fe2c3e354be37d4351bd4f9413ab38cdd4bfa35d4cbd6eb4e13` |
| `GATE-AUTHENTICATION-HOSTILE-REVIEW.md` | `54818aa3045824bdb8b3c922829e2d5c6d7882970776786be62b5fe40f31ccc1` |
| `PRIVILEGED-BASIS-COMPLETION-LOCK.md` | `395279c549aa37137d122282eed5c5e4a732521697f41ae406632bd4e2141e4d` |

## 2. Architecture-level reconstruction

Verdicts in this section record support for the paper architecture only. They
do not select the architecture for production, close exact D-2, or grant exact
P-1 credit.

### 2.1 Gate rows

| Rows | Verdict | Reconstruction |
|---|---|---|
| G-1 through G-3 | PASS | Base and successor records use one `PermitUse` predicate under one immutable semantic-root identity. Canonical signed successor snapshots, protected monotonic state, tombstones, and continuous key rotation close the claimed local rollback and replay boundary. Names, paths, flags, packages, and symbols convey no authority. |
| G-4 through G-5 | PASS | Public contract, evidence, exact implementation cone, exact consumer cone, target, frame, review, current membership, and final resolution are independently bound and revalidated through caches, LTO, objects, final links, and future loads. `CERTIFIED` exact bytes are not conflated with an explicitly trusted opaque provider. |
| G-6 through G-7 | PASS | Ordinary proof modules may introduce abstractions but no axioms, machine actions, fact/effect kinds, or foreign assumptions. Plugins, aliases, helpers, package machinery, and alternate backends cannot form a second gate. Replacing the trusted compiler or semantic root is named as TCB replacement rather than in-language admission. |

Primary evidence: `GATE-AUTHENTICATION-LOCK.md` sections 2, 5, 7, 9, 11,
13, and 14.

### 2.2 Public-basis rows

| Rows | Verdict | Reconstruction |
|---|---|---|
| B-1 through B-3 | PASS | Generative byte carriers, checked typed places, padding space, `Vac`, `Full`, `MoveAuth`, `ReleaseAuth`, exact root/epoch/offset identity, and authority-escrowing loans cover heterogeneous layout, affine lifecycle, access, disjointness, and stored provenance. |
| B-4, B-8, B-9 | PASS | An ordinary producer may hide a representation-selected resource algebra and proofs. The fixed verifier checks implication to the public policy; proof search, solvers, AI generation, optimizers, and derived rules remain untrusted. No topology-specific privilege is introduced. |
| B-5 through B-7 | PASS | Exact focus accounts every normal edge. Reserve-first construction separates recoverable preparation from destructive commitment. Abort, recoverable failure, partial progress, `Trivial`, executable `Plan`, and non-abandonable `MustClose` retain exact owner and cleanup obligations without writer-trusted finalizers. |
| B-10 | PASS | Eighteen independent removal witnesses separate typed layout, reshape, full-storage adoption, opaque sealing, move/release authority, focus, disposition, `CoreCopy`, runtime callables, interference, external frames, and final loaded-image binding. Rejected alternatives have explicit dominance or hard-constraint failures. |

Primary evidence: `CERTIFIED-RESOURCE-BASIS-DECISION.md` sections 3 through
5 and 13.

### 2.3 Derivability rows

| Row | Verdict | Reconstruction |
|---|---|---|
| D-1 | PASS | All 26 systems-domain rows terminate in a defined core, ordinary, certified, gate, toolchain, or ratified non-goal route. R-6 final-code binding is an explicit universal dependency. |
| D-2 | FAIL-CLOSED; EXACT ENUMERATION PENDING | The candidate supplies paper route vocabulary for `CoreCopy`, reshape, ZST, full-storage adoption, stored borrows, behavior calls, exact destruction, allocation outcomes, and policy blockers. Exact enumeration retains 340 unique unresolved obligations across 150 contexts: 168 coarse Convert route-direction, 24 Convert callable-direction, 136 outer-allocator-applicability, six IntoOwner ZST/capacity-reshape, and six IntoBoxed no-change fullness/ZST obligations. No D-2 PASS credit is granted. |
| D-3 | PASS | Inline-small, gap, flat-set/heap, ring/deque, dense store, open-addressed sparse storage, and recyclable stable identity have constructive owner and cleanup traces. |
| D-4 | PASS | B-tree/rope, linked topology, multi-block arenas and ordinary custom allocators, pinning, intrusive placement, and self-reference have routes without mandatory per-element allocation. |
| D-5 | PASS | Strong/weak ownership, dynamic borrowing, release/acquire publication, locks, channels, and safe lock-free reclamation separate runtime algorithm state from erased proof authority. |
| D-6 | PASS | Partial I/O, retained callbacks, exact async cancellation, persistence, DMA, and reset separate admitted frame actions from ordinary protocol and policy code. |
| D-7 | PASS | Four independently selected held-outs cover concurrent cuckoo migration, crash-consistent LSM state, hot-swappable JIT code, and sparse GPU residency. None needs a named privileged container or a second gate. |

Primary evidence: `SYSTEMS-DOMAIN-DERIVABILITY.md`,
`SYSTEMS-WITNESS-DERIVATIONS.md`, and `FRESH-HELDOUT-DERIVATIONS.md`.

### 2.4 Structural-performance rows

| Rows | Verdict | Reconstruction |
|---|---|---|
| P-1 | PENDING EXACT EVIDENCE | The candidate can express representation-selected layout and algorithms, but the exact dense ledger has not proved same-route structural parity. Route multiplicity, allocator applicability, Rotate dispatch, stable-sort scheduling, and cached-key sorting remain open or unmeasured and receive no P-1 credit. |
| P-2 through P-4 | ARCHITECTURE-LEVEL PAPER SUPPORT | Prefix/full owners acquire no universal sparse, generation, sharing, atomic, topology, dispatch, pin, or async state. Proof terms and static authority are intended to erase. Required checks remain unless machine proof discharges them. This does not promote any unresolved exact route to a performance result. |
| P-5 | CLAIM BOUNDARY PRESERVED | Dynamic layout and callables, cleanup traversal, atomic helper fallback, pinning constraints, async state, code leases, and `MustClose` retention retain their named costs. Throughput, code size, compile time, proof production, and writer behavior remain unmeasured. |

Primary evidence: `CERTIFIED-RESOURCE-BASIS-DECISION.md` sections 12 and 15.

## 3. Authority-axis attacks

| Axis | Hostile witness | Result |
|---|---|---|
| R-1 spatial resource | Robin Hood occupancy and heterogeneous runtime-tail storage cannot treat ordinary metadata or arbitrary live bytes as typed payload authority. | PASS: checked place formation and full/vacant state are necessary and sufficient in the paper routes. |
| R-2 place identity | Recyclable generations, pinned self-reference, and JIT code leases attack stale reuse, relocation, and premature unload. | PASS: epochs, nonwrapping retirement, stable-place leases, and release escrow close the attacks. |
| R-3 lifecycle/control | Draining repair, a trapping nested disposer, async cancellation, and fallible external close attack affine abandonment. | PASS: exact focus, executable plans, pre-abort prefix invariants, transfer, and `MustClose` retain every obligation. |
| R-4 interference | Relaxed publication and hazard reclamation attack sequential ownership reasoning. | PASS: relaxed reads gain no payload authority; reclamation withholds `ReleaseAuth` until every interfering lease closes. |
| R-5 external frame | Partial I/O, persistence, reentrant callbacks, DMA, and device reset attack fabricated provider facts and cleanup. | PASS: exact admitted frame transitions attenuate facts and return, transfer, or retain every resource obligation. |
| R-6 final code | Relocated JIT code and dynamic providers attack pre-link or deterministic-only certification. | PASS: certification reaches the exact loaded image, resolved dependency cone, immutable code identity, and load receipt. |

## 4. Prior architecture-shape findings

The proposal addresses the earlier architecture-shape findings with:

- explicit positive move and release authority plus lease escrow;
- heterogeneous and runtime-dependent layout, padding, packed access, and
  proof-only zero-payload reshape;
- runtime-selected closures, callbacks, registries, and plugin code leases;
- source-preserving `CoreCopy`, distinct Clone/producer behavior, and paper
  routes for zero-sized-value ownership, while exact dense direction remains
  pending;
- an exact paper weak-memory witness, with production adoption still separate;
- wake/cancel ownership and nonterminal `MustClose`;
- terminal, transfer, and continuation distinctions for external resources;
- exact post-link, post-relocation loaded-image closure;
- corrected axis accounting and nonwrapping stale-handle retirement; and
- removal of universality, global minimality, zero-cost, and throughput claims
  from the architecture verdict, with hidden contract-selected costs named and
  exact P-1 left pending.

## 5. Verification and residual boundary

The reviewer ran both repository gates on the reviewed bytes:

- `make check`: PASS, `ALL VERIFICATION LAYERS GREEN`;
- `make -C compiler check`: PASS; and
- English-only scan of the reviewed artifacts: PASS.

The hostile review found no architecture-shape blocker in the bounded paper
witnesses. Combined research completion does not pass: exact D-2 fails closed,
exact P-1 remains pending, and owner review has not selected the proposal.
Repository durability and final post-edit gates also remain required. This
review does not prove production checker soundness, automatic proof generation,
bounded proof cost, or measured performance.
