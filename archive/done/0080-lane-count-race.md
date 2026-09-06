# Batch 0080 — the lane-count data race

Branch: `exhaust/floor`, one fix on top of batch 0079's closure.
Authority: owner direction of 2026-08-24, verbatim: "修复", following a
review finding the owner relayed from the concurrency research agent:

> 创建循环结束后,主线程再次写 `wf__par_lane_count = started + 1;` 但此时
> worker 已经可能进入 `wf__par_find` … 一个线程普通写,另一个线程普通读,
> 没有原子,也没有覆盖这两个操作的 mutex。… 所以创建过程不能整体宣称安全。

## The finding, verified

The claim is correct. `wf__par_lane_count` was a plain `int`; the pool
start path rewrites it after worker threads are already running their
steal loops (`par_runtime.c`, the post-loop `started + 1` store and the
three error-path zero stores), and every worker reads it unsynchronized at
the top of `wf__par_find`. Under the C memory model that pair is a data
race in every execution where a worker's read is not ordered after the
write, regardless of the values involved. The pre-loop `requested` store
is not part of the defect: it precedes every `pthread_create` and is
ordered by it.

## The fix

All seven access sites became relaxed atomics
(`__atomic_load_n`/`__atomic_store_n`, `__ATOMIC_RELAXED`), with a comment
at the declaration stating the invariant and why relaxed suffices: the
count is only a scan bound, every lane a stale count can reach was
prepared before any thread started, and lane contents carry their own
ordering through the deque's atomics.

A sweep of both runtime files found no second instance: `wf__par_idle` is
fully atomic, `wf__par_ready` is mutex-protected on both sides,
`wf__par_cached_lanes` was already a commented relaxed atomic,
`wf__par_attached` is thread-local, and `wf_floor.c`'s shared words are
written before any worker exists and ordered by `pthread_create`.

## Why there is deliberately no regression test

ThreadSanitizer does not reproduce this race on this machine: in 33 runs
across three harnesses (the compiled probe program at two optimization
levels and a direct C driver of the real runtime at 16 workers) it never
fired, because the parent reliably reaches the rendezvous lock before any
worker signs in, and every worker's subsequent lock acquisition is then
ordered after the racy store through the parent's `pthread_cond_wait`
release. A minimal structural analog in which one worker signs in first
fires 5/5, which proves the tool would catch the racing schedule if it
occurred; the schedule simply does not occur under this scheduler. A test
that passed before the fix would test nothing, so none is added; the
invariant is stated at the declaration and held by review. Post-fix, the
same three harnesses run clean, and the compiler gate is green with all
program outputs byte-identical at every worker setting.

## Approval classes

No spec bytes, no protected conformance changes, no new root entries.

## Outcome

One commit; the fix, this record, and one dated pitfall fact on the design
tree's runtime node.
