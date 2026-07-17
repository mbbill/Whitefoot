# Minimal static public capability basis hostile review

Status: D14 adversarial review, 2026-07-15. Review target:
`MINIMAL-STATIC-PUBLIC-CAPABILITY-BASIS.md`. No production change or experiment
is authorized.

## Verdict

**CONDITIONAL PASS for further derivability research; exact D-2, P-1, and
whole-envelope closure remain FAIL-CLOSED.**

The candidate separates fixed storage semantics, checked ordinary proof and
abstraction mechanisms, and independently counted machine/runtime leaves. It
does not expose writer-accessible unsafe or a generic semantic descriptor. The
candidate may be used as the basis hypothesis for the finite derivability and
structural-cost packet, subject to the blockers below.

## Attacks and dispositions

| Attack | Finding |
|---|---|
| Treat Q1 predicates as trusted contracts | Rejected. Q1 proofs derive only fixed ownership/place propositions; they cannot add effects, facts, proof rules, lowerings, foreign assumptions, or primitive semantics. |
| Encode a container as one Q1 state machine and call it a primitive | Rejected. Its executable operations remain ordinary checked code over P rows and receive no registry identity or P0 credit merely from a proof. |
| Pass a layout that reinterprets live bytes | Rejected. P1 can reshape only vacant footprints or preserve the exact live type/owner mapping; arbitrary live reinterpretation is absent. |
| Use zero-byte/ZST address inequality as disjointness | Rejected. Logical place identity and exact footprint authority, not pointer comparison, establish separation. |
| Abandon a hole-bearing affine token | Rejected by Q2 exact-use/transition closure. Affinity without closure receives no credit. |
| Hide `Copy`, `Clone`, or `Default` inside P2 | Rejected. P2 transfers the offered sole owner and performs no duplication or default construction. Clone is explicit Q4 behavior. |
| Make `resize`, `swap`, `replace`, or `relocate` extra privileged rows | Rejected absent a same-contract removal witness. They derive from P1/P2; bulk lowering is optimization. |
| Give every slot a liveness tag or bitmap | Rejected as a universal basis. Representation-selected metadata is ordinary Q1 state; dense/full paths pay none. |
| Require all storage to be stable or pinned | Rejected. P4 is an explicit lease and affects only selected address-sensitive paths. |
| Use an arbitrary syscall number or FFI signature | Rejected. Every P7 event is a separate compiler-owned identity with a fixed contract. |
| Use an opcode, target feature string, or MMIO address as authority | Rejected. Every P8 event and mapping is a separate reviewed identity; ordinary values are operands only. |
| Turn atomics into unchecked shared access | Rejected. P5 supplies events; Q5 and the fixed memory model control access and race freedom. |
| Claim async, channels, mutexes, or executors as privileged | Rejected. They are ordinary policies over P5/P6 and exact P7 timer/I/O events. |
| Treat a raw address as a code pointer | Rejected. P9 activation returns a code lease tied to a checked callable contract; unload consumes all leases. |
| Let runtime/backend implementation weaken a P row | Rejected by the static gate's single-contract, fail-closed target and exact binding invariants. |
| Mint optimizer facts from ordinary wrapper names | Rejected. Facts arise from checked Q proofs or fixed P contracts and remain separately hostile-reviewed. |
| Claim the nine P classes are nine semantic operations | Rejected. P7/P8 expand to independently counted exact event rows; P5/P6 also contain fixed distinct events. Class count is organization, not authority count. |
| Claim full systems closure from category sketches | Rejected. Exact P7/P8 inventories, 340 dense obligations, and P-1 remain open. |

## Removal challenges

- P1 cannot be replaced by full initialization without dummy values, tags,
  zero-fill, or extra allocations.
- P2 cannot be replaced by whole-owner moves without losing O(1) affine
  insertion/removal and partial construction.
- P3 cannot be replaced by current `buffer_new` while retaining recoverable
  failure, affine vacancy, no initialization traffic, and ordinary allocator
  composition.
- Q1 cannot be deleted without either compiler-known topologies or universal
  runtime liveness metadata.
- Q2 cannot be deleted while permitting early exit and abandonment of
  transition/cursor owners.
- Q3 cannot be deleted while retaining owner-tied returned borrows, arena
  placement under prior borrows, and runtime-distinct mutable access.
- Q4 cannot be deleted while retaining ordinary generic maps, sorts, cloning,
  and lazy pipelines with direct calls.
- P4 through P9 each cover an externally observable machine/runtime capability
  not expressible by pure checked source.

No successful challenge proves that the listed element can be removed under
the same contracts. This supports abstract minimality but not implementation
minimality or proof-system size.

## No-tax review

The candidate imposes no inherent:

- liveness tag or bitmap on full/dense storage;
- capacity field on fixed storage;
- generation check on append-only identity;
- stable-address lease on relocatable values;
- refcount or atomic event on unique owners;
- runtime borrow table on lexical borrows;
- dynamic-dispatch object on static generic calls;
- refinement metadata on raw bytes;
- external ABI transition on direct-lowered P rows; or
- target dispatch on portable scalar code.

These are structural claims from the contracts, not measurements. Concrete
source, IR, machine-code, allocation, and traffic parity remain P-1 work.

## Open blockers

1. Q1/Q2 exact proof judgments and deterministic checking are not designed.
2. Q3 stored/result provenance and dynamic disjointness are not formalized.
3. Q4 callable and disposition interaction remains unimplemented.
4. Q5 needs a selected weak-memory, sharing, and reclamation model.
5. Exact P7 and P8 target inventories are not enumerated.
6. P9 final-image validation and W^X-equivalent target rules are not fixed.
7. Dense D-2 retains 340 unresolved obligations across 150 contexts.
8. No route-level same-contract P-1 structural proof or authorized measurement
   has been completed.

## Review conclusion

The candidate is safe to use as a bounded research hypothesis because the
unresolved items fail closed and no high-level operation or writer-provided
descriptor receives privilege. It is not safe to report as a complete language
design, a production basis, or a whole-systems derivability result.
