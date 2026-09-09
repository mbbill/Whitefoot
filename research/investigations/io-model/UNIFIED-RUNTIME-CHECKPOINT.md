# Unified runtime checkpoint

The owner requested this frozen research checkpoint on 2026-09-09 before
comparing the historical main compute runtime with the recovered research
compute runtime. It preserves the current unified implementation for a future
I/O investigation. This is explicit permission to retain its code as research
evidence, not permission to maintain an alternative production implementation.
Do not integrate a selected compute runtime until the owner has read the
comparison report and chosen the direction.

## Exact code and restoration

- Base: `301684269e79467660ffbb9bd00a02c2cb2cc255` (main).
- Checkpoint: `d858008f560b25da896af2a17f8b1d07ac49fd6e` (PR #28).
- [Frozen compiler patch](unified-runtime-d858008f.patch), SHA-256:
  `1cb7c85c56beacb1f47e9e836789ea9d916bea112ff179defd074fa71cdcbd74`.

The patch contains the complete `compiler/` delta, including the shared
scheduler, completion/wait repairs, compiler publication policies, tests and
integration documentation. It deliberately retains the compiler context around
the runtime; it is not a standalone runtime library. Research benchmarks and CI
workflows remain accessible at the checkpoint revision. No normal build or test
loads this patch.

In a clean disposable checkout at the base revision, apply the downloaded patch:

```sh
git apply --check /absolute/path/to/unified-runtime-d858008f.patch
git apply /absolute/path/to/unified-runtime-d858008f.patch
```

Restoration was checked from the base's exported compiler tree: all 307 files
match the checkpoint's Git blob hashes, with no extra or missing files. Do not
apply this patch to a newer main and assume it reconstructs the checkpoint.
Retain it while this unified design remains a candidate; supersede its index
when a later authorized checkpoint replaces it, preserving cited evidence.

## What is preserved

The implementation retains one shared compute/completion scheduler, managed
stacks, READY and free-stack lists, and platform wait/progress interfaces.
Compute gets direct owned-task execution, bounded current-stack helping,
owner-local task-slot allocation/return, atomic foreign returns, and repaired
deque ordering. Empty READY checks avoid the list mutex; host waits rearm their
wake conditions and coalesce redundant signals. Optional ordinary-entry
counter output is available through `WF_SCHED_REPORT=2`.

This is the current optimized checkpoint, **not a demonstrated fastest unified
I/O runtime**. Recent optimization primarily measured compute. I/O compatibility
success does not establish I/O performance superiority over earlier revisions.

## Evidence and unresolved limits

- At `ab6cf582`, which precedes only the ordinary-report addition and its
  documentation/tests, [all 12 gate jobs passed](https://github.com/mbbill/Whitefoot/actions/runs/34400322812).
  [Linux/Windows I/O host qualification](https://github.com/mbbill/Whitefoot/actions/runs/34400322727)
  and [the I/O benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34400322758)
  also passed. These are not fresh comparative I/O rankings.
- [The five-platform compute run](https://github.com/mbbill/Whitefoot/actions/runs/34400322732)
  failed every formal-runtime performance screen and every ordinary-command
  performance screen. The [compute experiment](../../experiments/compute-runtime/README.md#ordinary-cli-mandelbrot)
  owns the detailed ordinary-command results and artifact links. Small or
  frequently synchronized compute remains materially slower than the recovered
  compute baseline; some CI cells have substantial A/A noise.
- The last completed local canonical gate at `444b89b8` passed. The subsequent
  `ab6cf582` attempt stopped on an existing scratch-directory collision. The
  `d858008f` attempt stopped because the sandbox denied loopback sockets to
  seven network tests. Neither failed attempt is an exact-checkpoint gate pass.
- The report-mode tests and ordinary-command correctness checks passed locally
  at `d858008f`. Its [exact CI gate](https://github.com/mbbill/Whitefoot/actions/runs/34402981042)
  passed 11/12 jobs; macOS research failed the quadrature parallel-actualization
  check. [I/O hosts passed](https://github.com/mbbill/Whitefoot/actions/runs/34402980662),
  but [the I/O benchmark](https://github.com/mbbill/Whitefoot/actions/runs/34402979619)
  passed only 3/4 jobs: Windows rejected compute timing as unstable after two
  complete cohorts. The [compute run](https://github.com/mbbill/Whitefoot/actions/runs/34402983009)
  again failed all five formal-runtime and all five ordinary-command jobs.
  These results are preserved limitations, not a qualified checkpoint release.
  No merge or full-goal completion is claimed.

The comparison report must evaluate the old main **pure-compute** implementation
against the research **pure-compute** implementation. Comparing either with this
unified checkpoint answers a different question. Keep runtime mechanism,
compiler publication policy, arithmetic/code generation and diagnostic-counter
settings distinct in that comparison.
