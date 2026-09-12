# Triage of audit findings

The owner's rulings on the items the module audits listed under "choices
without a node", recorded as they were given. A ruling of "no decision
needed" means the item is implementation and stays out of the tree; a ruling
of "tree" names the node that now carries it.

## audit/lexer-and-syntax.md, section 3

1. Two-pass, count-then-allocate lexer: no decision needed.
2. Every resource ceiling mandatory with no default: no decision needed. The
   owner's reading: such limits are a concern of the future self-hosted
   compiler, not of this research compiler.
3. Iterative task-stack machines for derivation, the diagnostic re-walk, and
   shape re-verification: no decision needed, with the same reading.
4. The frontend never panics and each stage re-derives what the previous one
   guarantees: the re-verification half is refused. Tree: `compiler` root,
   rejected alternative on hardening and re-verification inside this compiler.
   The no-panic half is ordinary engineering quality and needs no decision.
5. A curated subset of syntax diagnostics carries a mechanical fix: no
   decision needed.
6. Flat postorder parse array finalized by a second pass: no decision needed.

## audit/resolution.md, section 3

1. Historical [SYS-2] inventory states behind compile-time switches: the owner
   ruled them agent self-direction with no use. Deleted from the resolver;
   tree: `compiler` root, rejected alternative on superseded inventory states.
2. `set`-target-declares resolved by rebuilding to a fixpoint: no decision
   needed.
3. Diagnostic wording inside the resolution payload: no decision needed.
4. Contract `define` binder sharing the `let` carrier role: a defect, fixed
   (`ReservedDeclarationRole::ContractDefinition`).
5. `fn_bind` lookup failure citing FN-4: a defect, fixed (cites FN-3).

## main, pull requests #36 to #48 (proposed, awaiting the owner)

Choices carried into the tree for confirmation: the recursion budget clone
family and its runtime-derived default (`compiler/parallel-lowering/two-worlds`),
the idle window in time and the split grain (`compiler/parallel-lowering/parallel-runtime`),
the rejected `N`-layer frontier, alignment flags, and wake-path change.

No decision needed: the compute scoreboard and its harness, CPU column, A/B
twin, Darwin CPU source, and hosted workflows (research instruments); the
`compute-regression` required check and the scratch-root move (infrastructure);
the POSIX spin hint (a missing instruction, not a choice); the io-model
scheduler findings and the chunk-lease measurement (measurements, not
decisions).

Ruled by the owner: the `WF_PAR_TRACE` instrument in `sched/core.c` stays. It
is not the same kind of thing as the deleted inventory switches, which
reproduced historical states of the system API that nothing used; it is a
performance-debugging instrument for the parallel runtime, compiled into no
shipped build, and will be wanted the next time that runtime is tuned.

## Round 2: the eleven module audits (proposed, awaiting the owner)

Every "choice without a node" from the audits under `audit/`, with the
proposed class. Nothing here is decided until the owner rules.

### Drift and defects (a fix or a ruling, not a new decision)

1. Two permanently-on candidate switches, `V031_CANDIDATE_SEMANTICS`
   (`semantic/mod.rs`, five dead arms in `requires.rs`, `types.rs`, `check.rs`)
   and `REBORROW_EXTENSION_ACTIVE` (`check.rs`, two dead arms): the same shape
   the compiler root already rejects for the inventory switches. Proposed:
   delete both with their dead arms. (contracts 1, types 1, expressions 1)
2. The recorded affine-nesting crash in `docs/todo.md` is now attributed: the
   checker's `check_affine_expression` family recurses natively
   (`control/proofs.rs`), `AffineExpression`'s derived drop recurses
   (`entailment/affine.rs`), and the FN-9 Tarjan scheduler recurses
   (`entailment.rs`). Proposed: update the todo entry with the attribution;
   whether to fix now is the owner's call, since ceilings were ruled a
   self-hosted-compiler concern. (contracts 2, ownership 2, entailment 2)
3. `check_float_operation`'s one `.expect()` where every sibling fails closed.
   Proposed: fail closed like its siblings; no decision. (expressions 4)
4. [ENT-3.S10] names five operations; the compiler derives endpoint facts for
   eight, the [SYS-8] family. Proposed: amend the specification to name the
   family, since [SYS-8] already treats them as one; the owner chooses between
   that and narrowing the code. (entailment 1)
5. [PAR-1] admits a `let x = propagate f(...)` as a window's second member,
   which the rule's text does not define. Proposed: follow the specification
   and exclude it; no decision. (permissions 1)
6. [PAR-1] treats a `match`/`value_match` scrutinee call as a window member by
   reasoned analogy the rule's text does not cover. Proposed: the owner chooses
   between amending [PAR-1] to name scrutinee calls, with a decision in
   `language/parallelism/permission-judgment`, and narrowing the code.
   (permissions 2)
7. Compiler-derived release attribution derives each function's result-state
   origin from callee bodies to a whole-program fixed point, and that feeds
   the acceptance-bearing effect-row check, so a caller can flip verdict when
   a callee's body changes. Two ownership nodes reject this shape for
   analogous problems and [EFF-2] says no second provenance system exists.
   Proposed: the owner rules between a signature-derivable rule (which needs
   a way to state opaque result-state provenance), licensing the technique in
   the specification and tree, or accepting it as is. (ownership 1)
8. `TCP_NODELAY` is set on no socket, on any platform, while the io-model
   research measured an 11 to 15 times throughput gain and a 41 ms to 2.4 ms
   p99 from it. Proposed: set it on every socket the runtime creates; a
   measured runtime parameter, so a decision line if the completion runtime
   gets a node. (completion 6)
9. The connection two-count table's `2^20` size mirrors the descriptor
   ceiling in `wf_floor.c` by prose only. Proposed: a shared constant or a
   static assertion; no decision. (completion 7)
10. `design/language/data-model.md` justifies array-of-structs with a
    "copy-struct tier" that neither the specification nor the compiler has.
    Proposed: drop that clause from the decision; the tree is wrong, not the
    code. (types 5)
11. `whitefoot-grammar`'s frontend-contract section set widened once in an
    unrelated commit; the binary is wired to no Makefile target, gate stage,
    or document. Proposed: delete the binary as process residue with no
    consumer, or the owner names its consumer. (driver 2)
12. `CompilerLimits::default()` is the one ceiling profile every caller uses,
    with no override, unchanged since the first scalar slice. Proposed: no
    decision now; note that a real program over 16 MiB of source or 8 million
    parse tasks cannot ask for more. (driver 1)

### Candidate decisions (drafted for the owner to accept, edit, or strike)

13. A new `compiler/completion-runtime` node: the completion record is a
    block of the submitting frame found by address, so no operation is ever
    refused for capacity (completion 2); a ring refused at startup falls back
    silently to the helper adapter while a failure after acceptance is a
    fail-stop, never an `IoError` (completion 3); the record is 160 bytes,
    sized to `open_at`, which keeps `AcceptEx` and positioned stream writes
    off the Windows completion port (completion 4); helper growth is
    demand-driven with the eight-peer bound on ring-less hosts left open
    (completion 5); an open's kind is decided by one `fstat` on the reaping
    thread rather than a linked ring operation, measured (completion 8); the
    completion runtime is a hard link requirement with no weak fallback,
    unlike the compute runtime (emitter 4); and the measured tuning constants
    (reap budget 64, join spin 10 us, window 1024 and 4 MiB, ring depth 64
    and 2,048 completions) if the owner wants constants of that grain in the
    tree as the compute side has (completion 9).
14. Compiler root: the runtime every program links is plain C reviewed by
    hand, outside the crate's safe-Rust guarantee (completion 1).
15. `compiler/parallel-lowering/two-worlds` or `parallel-runtime`: a
    hand-out claims its lane before building the frame, measured at four
    times the stack and a crash with the pool off the other way (emitter 3);
    a group's compute members are joined newest first because the deque is
    Chase-Lev (emitter 2).
16. A stack-ledger decision: numbers come from a completed host compilation
    of the emitted module plus a fixed per-architecture return-address
    correction, because nothing earlier has them and over-promising depth is
    the dangerous direction (emitter 1).
17. A target-qualification decision: the table stays current through one
    hand-checked version tripwire and a per-version review note, never
    generated from the specification (emitter 5).
18. `compiler` or `language/data-model`: a nominal's region parameters are an
    identity axis at check time, and instances differing only by region are
    reconciled to one representation before lowering (types 2).
19. The staged loop I/O pipeline: fixed two slots in flight and no per-slot
    byte accounting, against the investigation's runtime-computed window
    (lowering 1). Provisional if kept.
20. Diagnostics channel policy: a denied [PAR-3] verdict prints without a
    flag, a denied [PAR-2] only when its loop's [PAR-3] also denied, [PAR-1]
    never, and `--no-overlap` silences it, from the 2026-08-28 blind-writer
    trial (permissions 5, driver 3).

### No decision needed (listed for the owner)

21. The entailment closure's dense matrix and interning hasher, a measured
    rewrite verified byte-identical over 623 sources (entailment 3).
22. The throwaway preflight pass that satisfies DIAG-1's interleaved
    selector admission (contracts 3).
23. Retrying a function's check to a bounded fixpoint to intern derived
    nominals (expressions 2), and reporting a missing `main` only after a
    salvage pass finds no more specific violation (expressions 3).
24. Two `ResolvedPlace` representations split by pipeline stage (types 3),
    and three unrelated depth ceilings of 16, 32, and 64 (types 4), which
    could share one named constant.
25. [PAR-3] replication implemented only for iteration-own storage, with the
    enclosing-storage coverage proof deferred; fails closed. A `docs/todo.md`
    gap entry rather than a decision (permissions 3).
26. `StagedDenial` carrying its own wording unlike its two siblings
    (permissions 4); no mechanical-fix text on [OWN-5] borrow conflicts and
    [PROV-6] bound mismatches (ownership 3): diagnostics work.
27. The split weight estimate's constants, 16 per nesting level over three
    rounds (lowering 2), and the aggregate-storage coalescing scope
    (lowering 3): compiler internals.

### From the scheduler audit

28. Drift, fixed in this change: `parallel-runtime` cited the 16.3 us and
    18.9 ns readings of 2026-09-11 for the spin bound, while `sched/core.c`
    re-measured 10.3 us and 11.6 ns on this revision; the decision now points
    at the probe recorded beside the constant instead of one host's numbers.
29. `compiler/Makefile`'s Windows cross-link comment names a retired symbol
    and describes the weak/strong seam backwards. Proposed: fix the comment;
    no decision. (sched 6)
30. `WF_PLACEMENT_PAD` is a second inert measurement instrument of the same
    kind as `WF_PAR_TRACE`. Proposed: the owner rules keep or delete. (sched 4)
31. Three runtime constants with no recorded reason: `WF_FLOOR_STACK_BYTES`
    at 1 GiB per thread, `WF_SCHED_MAX_THREADS` at 64 with a hard failure
    above, `WF_SCHED_FRAME_BYTES` at 256. Proposed: the stack size is the
    "one runtime-owned constant" `parallel-runtime` already names and could
    carry its value; the other two are the owner's to place or leave. (sched
    1 to 3)
32. The descriptor floor's reserve and ceiling in `wf_floor.c` are a third
    resource limit that `resource-exhaustion-floor` does not mention.
    Proposed: a decision line in that node. (sched 5)
