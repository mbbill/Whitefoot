- A view carries a finite set of continuing-loan keys independently of its possible storage origins (SliceLoanKey).
- Each key contains the data region, exact protected place, strength, and optional checked parent holder. Equal keys coalesce their descriptor-use sets.
- Nonconstant-source formation publishes a claim; immutable constant storage carries none. Binding, copying and own-view result substitution attach only the carried keys. An owning shared-view result supplied by a borrow-mode view parameter derives shared keys at the result region from its permitted actual suppliers.
- An incoming formal view carries a symbolic parameter claim without requiring a local formation record.

## Facts

- 2026-09-10 (be12d98d) pitfall: A moved incoming exclusive descriptor can have a live shared child without any local exclusive-formation record. Checking only descriptor associations on those formation records admitted its transfer to a helper; checking the carried keys as well rejects the live-child case and admits transfer after the child's previous-statement last use. (code)
- 2026-09-10 (be12d98d) pitfall: Borrowing a moved view descriptor and projecting only its local place loses the backing write from the enclosing effect row. A callee receiving an already-bound descriptor holder also bypasses a check limited to new unique-borrow formation. Complete origin projection plus the descriptor's projected-write freeze cover these independently tested paths. (code)
- 2026-09-10 (be12d98d) observation: The native borrowed-array control observes copies, helper relays and writes to original array fields with prior scalar snapshots and neighboring values preserved. Its independent moved-formal delegation changes only the middle word from 17 to 109 in all three lowering modes with ordinary and retained helper calls. This establishes those storage and delegation behaviors, not throughput or general affine-element view support. (code)

## Moves

- 2026-09-10 (be12d98d) replaced [[origin-descriptor-registration]]: matching descriptor holders only by storage origin conflated distinct continuing loans and could not carry an ordinary borrowed holder's delegation through copies and returned views (sourced)
