# Performance Expressiveness Gap Report

Date: 2026-07-15

Status: finite research input; no language, syntax, checker, compiler, runtime, target, library, fact-channel, experiment, or production selection.

## 1. Question and boundary

This report asks one question: where does current xlang lack the checked expressive power needed to obtain a demanded representation, code shape, or exact machine-event sequence without paying avoidable initialization, clearing, copying, movement, allocation, metadata, indirection, dispatch, checking, or event costs?

The answer is the 15-row companion ledger. It is deliberately organized by performance contract, not by a proposed mechanism. A row may eventually be served by a language rule, ownership rule, proof rule, ordinary checked abstraction, builtin, runtime leaf, target leaf, or a combination. The ledger does not select among them.

The scope is finite in three independent ways:

1. The internal frontier is the 49-row capability-obligation registry. Every row not already fully established or protected is routed to at least one gap row.
2. The workload boundary is the 26-domain systems ledger plus the visible and held-out witness registry. External machine-event gaps are retained even where no capability-registry row exists.
3. The evidence cutoff is 2026-07-15. New workload contracts require a new ledger row or an explicit mapping to an existing row before they can influence a candidate.

This is not a claim that all 15 rows deserve a kernel feature. It is a bounded list of demands that a complete candidate must either satisfy, derive, defer with a reason, or explicitly exclude from the intended systems workload.

Each row is conjunctive. Grouping related contracts keeps the comparison finite but does not permit a candidate to claim the row by covering only its easiest member. For example, a partial-I/O route does not also close foreign-resource lifetime, and a SIMD route does not also close MMIO or DMA ordering.

## 2. Current-language anchors

The strongest current blockers are rules rather than missing APIs:

- TYPE-2 permits only Copy elements in arrays and buffers, and a buffer has one fixed runtime length.
- OWN-1 makes a partial move kill the whole binding; later use, write, or set is rejected, and reinitialization requires a new `let`.
- OWN-7 recognizes unequal literal indices but conservatively overlaps dynamic indices and same-root slices.
- `buffer_new` creates a fully initialized Copy buffer; OP-9 checks allocation-size overflow and leaves OOM at the trusted-computing-base level.
- FN-5 has no function values or dynamic dispatch; monomorphized environment-struct calls are the current direct-call doctrine, while the complete behavior path is deferred.
- The current kernel has no thread construct, memory model, atomics, synchronization, async/pin facility, I/O/FFI resource model, target SIMD/MMIO/DMA model, or executable-memory lifecycle.

Those anchors explain why the ledger contains both near-term representation gaps and later machine-boundary gaps. Omitting the latter would make a small candidate appear complete merely because the current language cannot yet state the workload.

## 3. What the rows establish

The rows separate five kinds of pressure:

| Pressure | Rows | Typical forced cost |
|---|---|---|
| Representation and ownership | PF-G01 through PF-G03 | initialization, tags, bitmap, copying, movement, rebuild, allocation |
| Access, abstraction, behavior, and failure | PF-G04 through PF-G07 | alias guards, borrow tables, materialization, indirect calls, rollback traffic |
| Semantic state and identity | PF-G08 through PF-G11 | repeated validation, generation checks, refcounts, locks, pinning, indirection |
| External machine boundaries | PF-G12 through PF-G14 | staging copies, duplicate events, helper calls, scalarization, loader trust |
| Proof and code shape | PF-G15 | retained checks, versioning guards, scalarization, code growth, dispatch shape |

The grouping is intentionally not one-to-one with the capability registry. For example, partial live storage and relocation share cleanup obligations but have different removal witnesses: a topology rule cannot by itself move an affine value, while a move rule cannot by itself say which slots are live. Likewise, erased sealing and state facts are related but not identical: hiding a field does not prove an optimizer proposition, and a proof system does not by itself prevent clients from constructing an invalid representation.

## 4. Existing evidence and its limits

There is measured evidence that expressive facts can matter materially:

- The scoped-alias experiment removed guard-versioning pressure, with 121 assembly lines versus 2,132 for the obvious Rust shape, a durable 17x code-size difference, and a 2.0x win at trip count 8.
- The checked-law channel produced a 3.3x speedup over the obvious sequential fold while rejecting a false signed saturating-add law.
- The base64 proof tier measured 1.71x over identical facts-off source after bounds-proof discharge.

These measurements support PF-G04 and PF-G15 only in their narrow tested channels. They do not establish that future occupancy, identity, concurrency, refinement, or dispatch facts are sound or profitable.

The witness and held-out budgets are structural evidence, not runtime performance measurements. They make hidden costs observable: W-GAP forbids a per-slot tag and fixes movement accounting; W-PIPE forbids intermediate collections and adapter heap allocation; W-SMALL fixes the inline and spill contract; W-ECS forbids per-entity allocation; the four held-outs keep candidate derivations from being fitted only to visible examples. None is a formal proof of safety or a benchmark result.

The concurrency, address-stability, I/O/FFI, target-operation, and final-image rows currently have domain-boundary evidence only. They remain real completeness obligations, but they must not dominate a near-term mechanism selection without separately authorized census, model, and measurement work.

## 5. Protected baselines and no-tax rules

Two existing routes are controls, not gaps:

- NT-FIXED protects the fixed fully initialized buffer: two-word representation, one allocation, no capacity, occupancy, generation, sharing, or proof metadata.
- NT-P2 protects the append-only SoA pattern: no new metadata, load, branch, indirection, call, or fact requirement.

Every candidate must also preserve narrower no-tax distinctions stated in the rows: dense storage must not inherit sparse occupancy metadata; append-only identity must not inherit recycling checks; unique ownership must not inherit shared refcounts or fences; movable values must not inherit pinning; portable code must not inherit target-dispatch state; programs without generated code must not inherit loader lifecycle.

This prevents a superficially universal ability from winning by taxing weaker program shapes.

## 6. Interpretation rules for candidate construction

A candidate covers a row only when it supplies all of the following:

1. A checked construction route for the target contract.
2. An account of the concrete forced cost named by the row.
3. Exact safety, failure, cleanup, fact-production, and invalidation behavior.
4. Preservation of the protected budget, or explicit evidence for a budget change.
5. A falsifier that could make the claimed need or the candidate's coverage fail.

Paper derivation is not a formal proof. Structural event counting is not measured performance. A capability-registry routing total is not exact Rust-census completeness. A domain routing total is not exact machine-event coverage. Deferred-domain rows may be marked deferred in a candidate, but not silently counted as covered.

The next research step must derive at least three materially different complete capability sets from these rows. It must not translate each row into one keyword or intrinsic, and it must not assume that the previous privileged registry or P/Q basis is the solution.

## 7. Verification

Run:

```sh
python3 tools/verify_performance_gap_ledger.py
```

The verifier checks schema, unique IDs, allowed statuses and evidence strengths, source-reference syntax, capability and domain reference existence, local evidence paths, required cost and falsifier fields, frontier routing, the 15-row closed ID range, and the presence of both protected controls. It validates references into the 26-domain boundary; it does not claim that each domain creates a distinct gap. It checks bookkeeping integrity only; it does not prove the claims in a row.
