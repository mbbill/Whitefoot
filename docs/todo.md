# Known compiler defects and open costs

Defects, capability gaps, and unresolved costs of the current compiler. None
of them is a decision. Remove an item when its fix and test land.

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
- **Connection-level concurrency through suspended user calls is missing, and
  silently.** A loop that accepts and serves connections in source order
  completes the current handler before entering the next, so a handler waiting
  on a silent peer holds up every later connection, and 1024 open connections
  are not 1024 independently resumable handlers. The source is accepted and
  compiled through ordinary calls with no report. No restoration mechanism has
  been chosen. `WF_STACKS` is inert for the same reason: the runtime has no
  switchable-stack pool for it to size, so it is neither read nor validated.
- **Result-state origins are derived from callee bodies.**
  `semantic/check/result_state_origin.rs` walks every function's checked body
  to a whole-program fixed point to learn which parameter a returned
  resource's state came from, and that feeds the acceptance-bearing
  effect-row check, so a caller's verdict can change when a callee's body
  changes. The system-interface decision rules this out: a resource's state
  is carried by its type at the API boundary. The mechanism is being removed;
  until it is, the contradiction stands.
- **[PAR-3] replicates only iteration-own storage.** Condition 5's
  replicated disposition for a place rooted outside the loop, which the rule
  defines under a byte-coverage proof, is not implemented; every such place
  is denied and the loop stages sequentially. The byte-range coverage
  analysis the case needs consumes the entailment fact state and was
  sequenced after the schedule and storage discipline shipped.
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

- **The unique-parameter container refusal.** [BLK-4] refuses a `&uniq`
  parameter that can reach a container, the fourth disposition of the
  replace-through-unique-borrow defect the containers investigation records,
  after refusal by written type, a conservative fact kill, and doctrine each
  failed. The owner wants to reconsider the rule before it enters the tree.
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
