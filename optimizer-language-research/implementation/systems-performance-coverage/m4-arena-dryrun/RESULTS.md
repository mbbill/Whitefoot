# M4 arena branded-id deref dry run — results

INDICATIVE (Apple M4, macOS arm64). Deploy target Linux x86-64; validates SHAPE.
cc = `/usr/bin/cc` (Apple clang 21). Sources: `kernels.c` (the three walks),
`arena_walk.c` (build/bench/differential). M4 cache: L1d 64KB, L2 4MB.

## The question

The arena part's story (dossier §2 item 4): a branded id deref is CHECK-FREE —
BRAND-1 (Delta 5, closed) fixes which-arena at compile time via the nominal
brand, and arena ids are affine (one live per allocation, unlike pool's
generational handles), so `arena_get(ar, id)` lowers to a bare `base + idx*size`
load with NO which-arena check and NO generation check. The dossier's stated
fallback IF brands proved infeasible is an "always-on elidable checked deref
within 1.25x of a C pointer arena." This measures both.

## Three walks (`kernels.c`), 16-byte nodes, random single-cycle (Sattolo) chain

- **(a) check-free** — `slab[idx]`, the branded-affine-id lowering (idx is a valid
  slot by construction; no check).
- **(b) checked fallback** — `if (idx >= count) trap;` per deref, on a
  DATA-DEPENDENT idx loaded from the node. Unlike M8's loop-induction index, DOM-1
  cannot discharge this — it is the realistic pointer-chase check.
- **(c) C pointer baseline** — a native pointer-linked arena (`Node* next`), the
  raw-pointer comparison the 1.25x bar is stated against.
(No Rust slotmap baseline: slotmap's `get` is a generational-handle check — the
pool model, not the affine-arena model — so it measures a different thing;
skipped as not the apples-to-apples baseline. Affine arena ids AVOID that
generation check by construction, which is the point.)

## Result 1 (ASM): (a) is a bare scaled-index load; (b) adds cmp+branch

`-O3 -mcpu=native`, `otool -tvV`. The (a) main loop:
```
add  x9, x8, x9, lsl #4     ; addr = base + idx*16  (scaled index; the ONLY extra ALU)
ldr  x10, [x9]              ; load val
add  x0, x10, x0            ; sum += val
ldr  w9, [x9, #0x8]         ; load next_idx -> idx
subs x2, x2, #0x1
b.ne <loop>
```
NO compare, NO bounds branch — the check-free branded-id lowering exactly. The (b)
main loop is identical PLUS, per step:
```
cmp  x1, x9                 ; count vs idx
b.ls <brk #1>              ; -> trap if out of range
```
So the fallback adds exactly 2 instructions per deref (`cmp` + predicted-not-taken
`b.ls`), data-dependent on the just-loaded idx, uneliminable.

## Result 2 (timing): ns/node, median of 21, each run ~4M node-visits (looped
cycle for steady-state residency). Stable representative values across 3 runs:

| residency | (a) free | (b) chk | (c) ptr | a/c | b/c | b/a |
|---|---:|---:|---:|---:|---:|---:|
| 4K  (64KB, L1)      | 1.49 ns | 1.45 ns | 0.91 ns | 1.63x | 1.59x | 0.97x |
| 64K (1MB, L2)       | 5.06 ns | 5.03 ns | 5.10 ns | ~1.0x | ~1.0x | ~1.0x |
| 1M  (16MB, L2/DRAM) | 10.6 ns | 10.5 ns | 10.1 ns | ~1.05x| ~1.05x| ~1.0x |
| 4M  (64MB, DRAM)    | 71.0 ns | 70.4 ns | 70.1 ns | 1.01x | 1.00x | 0.99x |

(64K and 1M have run-to-run noise of +/-0.1x; 4K and 4M are stable.)

Two clean, monotone findings:
- **The CHECK is free at every residency: b/a in [0.96, 1.03].** A pointer chase
  is LATENCY-BOUND — each step waits for the load of `next_idx` — so the
  compare+branch executes in the shadow of that load-use latency and costs ~0.
  This is the load-bearing answer for the fallback: its overhead is ~0%, far
  inside 1.25x.
- **Index-vs-pointer (a vs c) is exposed only at L1: 1.63x, converging to par at
  memory latency (1.01x at true DRAM).** The `base + idx*16` address add is a
  fixed ~2-cycle cost — a big fraction of the ~0.9 ns L1 latency, negligible
  against the ~71 ns DRAM latency.

## Result 3: correctness

Differential over one full cycle at every size: (a), (b), (c) sums IDENTICAL
(DIFFERENTIAL OK). The three traversals visit the same nodes in the same order.

## Consequence

- **Branded-id check-free deref: VALIDATED.** The asm confirms (a) is a bare
  scaled-index load with no compare or branch — BRAND-1's affine-id design removes
  both the which-arena and the generation check. And the retained-check fallback
  is essentially free in a traversal (b/a ~ 1.0): the bounds compare hides under
  pointer-chase latency. So the fallback, if ever needed, costs ~0% here — well
  inside the 1.25x bar interpreted as the check's overhead.
- **Honest caveat on "at par with C pointer."** Against a RAW-pointer C arena, the
  index-based branded deref is ~1.63x at L1 (the address-arithmetic add), and at
  par (~1.0x) only at memory latency. This gap is index-vs-pointer, ORTHOGONAL to
  checking (the check-free (a) already pays it), and it vanishes for
  memory-resident structures — the common case for arenas large enough to matter
  (trees/graphs that don't fit L1). A real INDEX-based C arena (common for
  compactness/relocatability) matches (a). If the M4 bar is read literally as
  `b/c <= 1.25x` on an L1-resident walk, it is MISSED at L1 (b/c ~ 1.59x) — but
  entirely because of index-vs-raw-pointer, not the check.
- **Cross-cut with M8:** data-dependent checks live in latency-bound pointer
  chases (hidden, this dry run); loop-induction checks live in throughput-bound
  loops (DOM-1-discharged to nothing, M8). Either way the safety check is cheap —
  brands' value is removing the which-arena/generation checks (asm-confirmed),
  not the bounds check, which was never the bottleneck.
