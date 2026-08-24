> Lead synthesis over the six-dossier resource-exhaustion research of 2026-08-23 (e1-e6); promoted verbatim as the design authority for batch 0079.

# Resource exhaustion — lead synthesis over six dossiers (2026-08-23 night)

Charter: owner 2026-08-23 verbatim (in whitefoot-exhaustion-charter memory
and to be quoted in the batch record): a reachable segfault is a WF design
failure; exhaustion must become a controlled, designed event.

The owner's asymmetry, measured: the only abnormal death a CORRECT program
can reach (exhaustion) is the only one with zero diagnostic bytes; a false
claim — impossible in a correct program — gets a byte-exact record. Heap
exhaustion today is a bare abort() with zero bytes, and at -O2 the OOM edge
is optimizer-erasable entirely (legal, but no test may assume it exists).

## The v1 batch — "the exhaustion floor" (post-rebase, one batch)

Ranked by consensus across all six dossiers; every item prototyped and
measured in scratch (probes inventoried in the dossiers):

F1. `probe-stack` attribute on every generated function. Measured
    byte-identical on Darwin (chkstk is ABI-mandated there) — free
    insurance; on ELF it is what keeps a large unprobed frame from
    SKIPPING the guard page into an adjacent thread's stack (silent
    cross-thread corruption — a soundness hole, not a diagnostics gap).
    Open sub-task: an end-to-end .wf-level repro attempt on Linux; the
    stack-clash question must be answered before any packet claims
    memory-safety under exhaustion.
F2. Runtime-owned main stack of a stated size (~40 lines; the same
    pthread_attr machinery the pool uses; reserve a deliberate constant,
    e.g. 1 GiB — 0 ms, 0 RSS on Darwin). The ceiling becomes one
    compiler-chosen, environment-independent number instead of "whatever
    RLIMIT_STACK left over".
F3. sigaltstack + sigaction(SIGSEGV AND SIGBUS, SA_SIGINFO|SA_ONSTACK) on
    every thread (bootstrap + per-lane attach). Measured: main-thread
    overflow presents as SIGSEGV/SEGV_ACCERR, worker overflow as SIGBUS —
    a SIGSEGV-only handler misses every worker overflow. Guard-hit
    discrimination by per-thread stack bounds; NON-guard faults restore
    SIG_DFL and re-raise (wild faults keep exit 139 + core dump — the
    mechanism must not eat real corruption evidence).
    [2026-08-23, batch 0079 audit] Two corrections to this line, both
    measured. (a) The parenthesis is only true if the band is the probe's
    geometry. As first shipped it was 1 MiB and converted every fault in a
    megabyte below any thread's stack, so the band is now one page-walk
    stride plus the ABI red zone. (b) "restore SIG_DFL and return" swallows a
    signal that has no faulting instruction to re-execute — an externally
    delivered SIGBUS — and leaves the disposition restored process-wide while
    the classification is per-thread, disarming the floor for the rest of the
    run. The handler re-raises after restoring. Record: a fixed
    constant naming ONLY the resource class ({"resource":"stack"}) —
    async-signal-safety and [PAR-1] byte-identity independently force
    fixed bytes; the ABSENCE of rule_id/function/node_path is what proves
    it is not a [DIAG-3] record. Install-failure falls back to today's
    behavior (diagnostic quality, not semantics). SHIP F2+F3 TOGETHER
    (a handler without per-thread altstack made the worker case worse).
F4. Heap twin: the four allocation-refusal sites route through one
    wf_resource_abort -> the first-trap-wins latch -> {"resource":"heap"},
    distinct exit status from traps. Stated coverage limit: this catches
    the allocator's null return, which overcommit makes the rare case.
F5. Iterative drop glue for recursive nominals. The compiler-generated
    drop glue is ITSELF recursive today; 3 of 5 recursive corpus programs
    drag it; the writer cannot see, avoid, or instrument it. One helper
    generator; removes an unbounded-recursion class in every build mode.
    Best charter-satisfaction per line on the list.
F6. The stack ledger behind --stack-ledger: -fstack-usage on the clang run
    that already happens; per-function frame bytes, Tarjan SCC per-level
    cost, "one stack of S bytes ~ N levels" derivations; par clones
    reported separately; a regression test asserting predicted-vs-measured
    ceiling agreement ((stack-6144)/frame model, accurate to 25 frames) —
    fails loudly the day an optimizer moves a frame. Would have caught:
    the 0076 depth regression, wfgrep's 300x-conservative hand cap, the
    invisible drop-glue recursion.
F7. Deterministic pool depth: raise/settle the worker-stack floor and fix
    the schedule-dependence — bt_skew's ceiling at the default is a COIN
    FLIP today (7/20 passes; whether the deep spine lands on a fresh stack
    depends on a steal race). Liveness must not depend on a race; the fix
    rides the already-expensive steal path.
F8. Spec: ONE clause family replacing the SCOPE-3 exhaustion carve-out —
    fail-stop, containment, exactly one latched record, distinctness from
    [DIAG-3] (its exclusivity sentence and [ERR-4] need the class named:
    protected-adjacent, flag it), distinct exit status, stated coverage
    limit; plus one [PAR-1] observables sentence (the record carries no
    worker/thread data). Prepared as owner-application recipes against the
    POST-REBASE spec text (main's v0.34 lineage).
F9. Record/packet truthfulness riding along: the stderr-byte change is a
    disclosed default-visible difference; the parallel heap multiplier
    (peak = lanes x sequential peak, byte-stable) is the heap twin of the
    0076 depth flag and the packet is incomplete without it; the
    -O2-deletes-allocations note so nobody writes a green OOM test that
    tests nothing; the 0076 "roughly a third" depth claim corrected to
    structural 1.5x with the IPCP artifact carrying the rest.

## Deferred, each with a named re-entry condition

- Static acyclic stack bound (proves 15/20 corpus programs can never
  exhaust; forces closing the spec-asserted-but-empty qualification gap):
  a batch of its own, after the floor. The doctrinally right end-state.
- Proof-derived stack segmentation (prototype: 77x sequential ceiling,
  identical bytes, zero happy-path cost, no hot-split — WF-unique and
  REAL): deferred because today's reach is 2/7 writer recursions, --par
  only, and it converts a stack ceiling into an RSS ceiling that still
  needs the floor beneath it. RE-ENTRY CONDITION PARTIALLY MET TONIGHT:
  the claim-in-closure exclusion — which removes exactly the deep
  algorithms — is being deleted by the claim-redirect landing. Re-evaluate
  reach after the rebase.
- Prologue SP checks: measured to consume the headroom they guard
  (+9-23%, halves the min_stack depth) — only if a target without guard
  pages appears, and only gated on a per-function ledger check.
- Typed allocation failure (try_reserve shapes): not v1; reopen on a
  target where allocation failure is truthful (no overcommit).
- F6/buffer_fits routed to bounded-arena facts (a derivable-fact problem,
  not a failure-value problem): adjacent to Dig 9 territory; the only
  mechanism here that PREVENTS heap death rather than reporting it.
- Depth-bound proofs past the acyclic tier: measured mirage; the loop
  splitter's owned ten-frame theorem gets printed as a bound (free), the
  general question waits for a program it blocks.

## The acceptance falsifier (from the segmentation dossier, adopted)

tests/programs/wfgrep.wf caps directory recursion at 16 levels and
documents that the truncation is indistinguishable from a complete search.
When the floor ships: if that cap can be deleted and wfgrep still cannot
corrupt or crash on a deep tree — clean resource record instead — the
charter is satisfied on real code. If it cannot be deleted, we are not
done.

## Sequencing

After: claim-redirect landing -> 0077 audit/close -> rebase onto main.
The exhaustion floor is the first post-rebase batch; its spec recipes are
drafted against the rebased text once, not twice.
