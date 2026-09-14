# Known compiler defects and open costs

Defects, capability gaps, and unresolved costs of the current compiler. None
of them is a decision. Remove an item when its fix and test land.

- **A field projected after dereferencing a runtime-indexed composite element
  stops as unsupported.** The specification admits ordinary chained element,
  dereference, and field selection, but
  `deref(owners.storage[index]).id` stops in semantic checking as
  `Unsupported(CompositeValues)` with no rule or source diagnostic. The
  growable-vector executable currently borrows `owners.storage[index]` into a
  helper and performs `deref(deref(item)).id` there. Complete the general
  checked-place and lowering path for a subscript followed by dereference and
  field projection, add owning and copy-element tests, then remove that helper.

- **Parallel grain policy needs a dedicated study.** Captured extents are a
  provisional scheduling input, not an established broadly suitable policy.
  The [first same-source trial](../research/investigations/compute-model/DESIGN.md#runtime-extent-trial-result)
  improves prefix, histogram and stencil, but makes chain-pull 51 percent
  slower at two workers and incurs substantial CPU costs in some faster
  cases. Those measurements precede the continuation-accounting correction;
  its performance has not been remeasured. Study whether a robust common
  policy exists or workload, input shape, worker count and hardware require
  different choices, comparing wall time, CPU and scheduling/profile overhead.
  [Runtime profiles and PGO](ideas.md#parallel-grain-policies-and-runtime-profiles)
  are candidate inputs to that later study. The trial's failures remain
  evidence, not proof that no broadly useful strategy exists. Close this item
  when a policy meets explicit representative criteria or its accepted
  tradeoffs are recorded.

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

## Open language questions

Questions the owner has left open on purpose. None of them is a decision;
each is resolved by a discussion and a tree change.

- **Sparse containers over must-consume linear elements need ownership-visible
  slot state.** The behavior-map growth witness previously wrote
  `formal Key<K: linear, ...>` while replacing `progress.held` and disposing
  the returned `Slot<K>`. That depended on a compiler defect which failed to
  apply PROV-6 to a symbolic linear bound. The current checker correctly
  rejects `dispose previous_held`: a numeric phase does not prove that the
  returned enum is `Vacant`, and an `Occupied` value contains a `K` that must
  be consumed. The maintained witness is narrowed to `K: affine`, which still
  covers its scalar and store-branded owning instances. Investigate a state
  encoding or checked variant-state relation that lets rehash move every
  must-consume key without an impossible cleanup branch; do not add a discard
  behavior merely to satisfy the checker.

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
