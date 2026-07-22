# Owner Ruling Request: STOR-1 pool/handle shape (blocks arena_ast_builder)

Date: 2026-07-09. The last M3 task (`arena_ast_builder`) needs a language shape
for building a tree of heterogeneous nodes without per-node free. STOR-1
already REJECTS untyped arena-index pools: recycling slot indices is
well-typed use-after-free (a stale index silently reads a new node — exactly
the class T1 exists to kill). Three candidate shapes; ruling needed on which
becomes spec surface.

## A. Region arena (`arena<'r, T>`) — RECOMMENDED for the language

Already named in TYPE-2. Typed, allocation-only, bulk-freed at region end;
node references are region-bounded borrows, so staleness is impossible by
construction (borrows cannot outlive `'r`) — no runtime checks, no new UAF
surface, derives from the existing region calculus (D1a). Cost: democ needs
arena_new + allocation codegen + borrow-returning allocation (a fn returning
`&'r T` — return-borrow support exists in checker per x-borrow-return case).
No spec change beyond activating what TYPE-2 declares.

## B. Generational pool (`pool<T>` / `handle<T>`)

Handles carry generation counters; deref checks generation and traps on stale.
Honest (checked, trap-on-violation, OP-4-style) but: new spec surface, a
runtime tax on every node access, and it legitimizes the recycle-idiom STOR-1
rejects — the checked variant of a shape we consider a footgun. Not
recommended unless A proves too weak for real compiler workloads (early-free
of subtrees is the one thing A cannot express).

## C. Append-only struct-of-arrays encoding — the immediate M3 unblock

Expressible TODAY with zero spec surface: nodes-as-indices into append-only
parallel buffers (tag buffer + per-field payload buffers), never freed, bulk
dropped with the owning struct. No recycling => no UAF class (indices only
grow; OP-4 bounds-checks stale-index reads into a trap or a valid old node,
never into a *different type*). This is how column-oriented compilers store
ASTs anyway, and it is exactly the shape channel 1 optimizes. The M3 task can
be written this way now; it measures what the task intends (allocation shape
+ ownership friction), not pool mechanics.

## Requested ruling

1. Approve C as the arena_ast_builder reference shape (unblocks M3 fully).
2. Approve A as the v0.7 direction for real arena support; keep B rejected.
