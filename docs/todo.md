# Known compiler defects and open costs

Defects, capability gaps, and unresolved costs of the current compiler. None
of them is a decision. Remove an item when its fix and test land.

- **Static loop weights underprice runtime helper work.** Prefix and histogram
  with 4,096-word blocks emit no grants at their default four-million-word
  fixture because the outer weight prices an inner loop with a fixed factor.
  Lowering the global work unit from 150,000 to 10,000 speeds those fixtures
  but makes the width-17 stencil 36 percent slower at four workers. The
  [compute-model controls](../research/investigations/compute-model/DESIGN.md#measurements-and-assessment-2026-09-13)
  retain the runtime dimensions, threshold sweep, process CPU and grants.
  A replacement estimate must distinguish actual helper extents and preserve
  the adverse cases; no dynamic estimator has been selected or implemented.

- **Unguarded affine expression nesting depth.** A proof-domain affine
  expression nesting parentheses about 1400 deep aborts the driver with a
  stack overflow and no diagnostic (exit 134); 1200 rejects normally and
  20000 does not finish in twenty seconds. It reaches this from both a `use`
  premise and an `invariant` target, so it is in the shared affine-expression
  handling. The repair is the pattern already used for structural limits, the
  4096-entry `proof_use` capacity and `AffineCheckError::LimitExceeded`,
  applied to nesting depth where the recursion actually is: the checker's
  `check_affine_expression` family in `semantic/check/control/proofs.rs`,
  `AffineExpression`'s derived drop in `semantic/entailment/affine.rs`, and
  the FN-9 scheduler's Tarjan walk in `semantic/entailment.rs`, all three
  native recursions the parser's own iterative machinery hands the input to;
  with a test that pins it.
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
- **Pre-kill L0 closure has an unresolved compilation cost.** Before an
  [ENT-5] invalidation batch, `materialize_before_event_kill` in
  [`semantic/entailment/flow.rs`](../compiler/src/semantic/entailment/flow.rs)
  calls `materialize_closure_before_kill` in
  [`semantic/entailment/state.rs`](../compiler/src/semantic/entailment/state.rs).
  A non-closed state with explicit relations takes the complete closure;
  already-closed, contradictory, and empty-relation states have fast paths.
  The cost on real programs needs stage attribution before changing this
  path. A narrower projection must preserve every surviving consequence,
  including implicit type edges and disequality strengthening; filtering
  explicit edges alone is insufficient. No speedup is established.
- **Connection-level concurrency is not supplied by ordinary source order.**
  A loop that accepts and serves connections in source order
  completes the current handler before entering the next, so a handler waiting
  on a silent peer holds up every later connection, and 1024 open connections
  are not 1024 independently resumable handlers. The source is accepted and
  compiled through ordinary calls. The retained multi-client TCP protocol can
  wait forever when the first handler awaits EOF while clients close only
  after every peer has finished; the
  [C2 measurements](../research/experiments/io-completion-bench/C2-RESULTS.md)
  record that noncompletion without a throughput result. No replacement
  interface has been chosen. `WF_STACKS` is inert: the runtime has no
  switchable-stack pool for it to size, so it is neither read nor validated.
- **Acyclic generic instantiation has no established practical bound.**
  D7's unchanged-argument cycle rule establishes termination while acyclic
  fan-out may still require exponentially many instances relative to written
  source. The owner deferred this question in D7, whereas the current language
  design rules out exponential checking work. The
  [behavior investigation](../research/investigations/containers-and-resources/BEHAVIOR.md#shared-semantic-boundary-and-exact-deltas)
  records the accepted 1343-byte / 2047-instance witness, same-instance controls,
  stage measurements and unresolved correspondence finding. No budget, timeout, new
  source refusal, or measured asymptotic guarantee has been selected.
- **At most eight peers may wait at once on a host without a native ring.**
  On Darwin, and under `WF_IO_NO_NATIVE_RING`, a peer wait beyond the eighth
  concurrent one has no helper and queues with no timeout. The readiness-
  driven adapter that would lift this, one poll over every queued descriptor
  from inside the park, was never built.
- **A `propagate` statement cannot be a [PAR-1] window member.** The rule
  admits only `let`-bound and scrutinee calls, so `let a = f(); let b =
  propagate g();` never overlaps. Allowing a `propagate` second member would
  need the lowering to join the hand-out before the `Err` return; a future
  investigation, taken up when a real program shows the gap.
- **Parallel stencil lowering has a one-worker cost and a grain cliff.**
  The runtime-sized stencil's parallel build is about 15–20% slower than its
  sequential build at one worker on the measured M1 Pro, while granting no
  tasks. At 2046 interior rows, the current estimated row cost affords only
  two chunks even with four workers. The
  [range-loan measurements](../research/investigations/range-loans/DESIGN.md#corrected-native-measurements-2026-09-13)
  retain both that size and the larger four-chunk case. The one-worker cause
  within lowering/code generation is not isolated; the finite range proofs
  add no runtime range checks.

## Open language questions

Questions the owner has left open on purpose. None of them is a decision;
each is resolved by a discussion and a tree change.

- **Local region introduction and explicit region blocks.** Revisit whether
  an ordinary function body should introduce a local region, and which
  borrows need a writer-spelled `region` block. In the
  [weighted-sum example](../tests/snapshot/cases/accumulators/accumulators__adversary-r1__p12_per_byte_widened_checked_sum.wf),
  the four `place_back(vector: &uniq weights, ...)` calls can share one region
  after the `weights` binding, but removing that region rejects under FORM-8.
  The temporary loans already end at their statement boundaries under OWN-6;
  their region's formation and storage-validity extent is a different matter
  under OWN-3 and OWN-10. Compare explicit blocks, function-body regions and
  implicit regions for non-escaping temporaries without conflating those two
  boundaries. Cover locals declared partway through a body, bound holders,
  surviving views, returned borrows, loops and control headers; preserve
  storage validity and exclusivity with deterministic checking. The owner
  requested this investigation during PR #30 review; no alternative is selected.
  Defer bulk cleanup of the repeated per-call region wrappers in migrated
  tests until this question is settled, preserving each case's intended
  behavior or rejection reason when the selected spelling is applied.
- **A view-valued match or if.** [OWN-5] rejects a `match` or `if` expression
  whose value is a view, rather than joining the arms' origin sets, which the
  origin machinery could represent. If the join can be admitted it should be;
  until then the rejection stands without a recorded reason.
- **The automatic-fact menu is a leftover.** [ENT-3] admits a narrow and
  asymmetric set of arithmetic idioms as automatic facts, each added for one
  proof pattern, with no general criterion and no counterpart for rows it
  omits, such as a lower bound from `ior`. The owner wants it made principled;
  nobody has had the time.
- **The two-premise cutoff of automatic affine derivation.** [ENT-6] tries
  zero, one, and two premises and no more without a written certificate. Why
  the line sits at two, against one or three, is not remembered and needs a
  study before it is recorded.
