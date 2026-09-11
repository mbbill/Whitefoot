# Known compiler defects and open costs

Defects and unresolved costs of the current compiler that are not decisions
and not unsupported capabilities. Remove an item when its fix and test land.

- **Unguarded affine expression nesting depth.** A proof-domain affine
  expression nesting parentheses about 1400 deep aborts the driver with a
  stack overflow and no diagnostic (exit 134); 1200 rejects normally and
  20000 does not finish in twenty seconds. It reaches this from both a `use`
  premise and an `invariant` target, so it is in the shared affine-expression
  handling. The repair is the pattern already used for structural limits, the
  4096-entry `proof_use` capacity and `AffineCheckError::LimitExceeded`,
  applied to nesting depth in whichever of the parser and the semantic former
  overflows, with a test that pins it.
- **A runtime-sized `buffer_new` fails with no rule and no location.** At an
  unproved runtime capacity the driver stops four stages after semantic
  checking with `TargetLayout(Unrepresentable(RuntimeSizedAllocation))` and no
  rule id or source coordinate; the real defect is an undischarged size
  obligation. The store surface already answers it (`heap_vector` hands back
  an `Option` and [OP-9] refuses at the source with a rule and a line). The
  item is removed with `buffer_new` and `buffer_vacant`, not repaired
  separately.
- **A large `proof_use` block is impractical well below its ceiling.** [PRF-1]
  admits 4096 entries; a 2026-09-05 record reports 389 ms at 64 entries, 3.0 s
  at 128, and 26.8 s at 256, about eight times per doubling, without a pinned
  reproduction bundle or stage attribution. Profile the stages before changing
  the implementation or the accepted proof rules; the
  [selection-ground assessment](../research/investigations/proof-certificate-architecture/SOURCE-CHECKING.md)
  separates the unresolved costs from the safety obligations.
