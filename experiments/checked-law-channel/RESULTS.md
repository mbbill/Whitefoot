# Channel 3: Checked-Law Reassociation (FN-4)

Status: BUILT + MEASURED 2026-07-09. Third channel differentiated: 3.3x over
Rust's obvious shape, ties expert Rust, and REFUTES the false law expert Rust
silently miscompiles with.

## The fact channel

FN-4: a law (`associative`/`commutative`/`identity`, closed table) becomes an
optimizer-usable fact only when stated AND checked. democ's static prover
accepts exactly one shape: the conform-bound fn's body is a single table op
whose law is OP-8 table data (facts by construction, no clever analysis). The
table is signedness-aware: unsigned `iadd.sat` is associative; signed is NOT
((MAX sat+ 1) sat+ -1 != MAX sat+ (1 sat+ -1)) — a stated signed law is
REFUTED with a rule-cited FN-4 diagnostic (conformance:
fn4-neg-law-refuted-signedness). Undischargeable laws are hard rejects
(fn4-neg-law-undischarged): stated-but-unchecked never reaches the optimizer.

The consumer: reduction loops over a proved associative+commutative+identity
op are reassociated into 4 independent block-interleaved accumulators seeded
with the proved identity (interleaving is licensed by assoc+comm; the seed by
identity), original loop kept as the scalar tail. LLVM cannot do this on ANY
language's output — saturating adds have no reassociation axiom and no
vectorizer recurrence kind; Rust has no channel to even state the law.

## Benchmark

`kernel.wf` reduce: fold of `satadd` (u64 saturating add) over a buffer.
Opaque boundary (C driver, no LTO). Rust adversaries (`#[inline(never)]`,
rustc -O3): `obvious` = `iter().fold(0, saturating_add)`; `expert` =
hand-written 4-accumulator chunks_exact(4) — the human ASSERTS associativity,
nothing checks it. Apple M4, ns/element:

| n | whitefoot-facts | whitefoot-control | rust-obvious | rust-expert |
|---:|---:|---:|---:|---:|
| 4096 | 0.210 | 0.530 | 0.521 | 0.155 |
| 65536 | **0.156** | 0.511 | 0.512 | 0.159 |

All variants produce identical sinks (semantics preserved).

## Interpretation

1. **3.3x over the obvious shape in both languages** at n=65536. The serial
   sat-add dependency chain (add + conditional-invert, ~3 cycles) is the
   bottleneck; the law licenses breaking it. LLVM leaves both obvious shapes
   serial — correctly, since it cannot prove associativity.
2. **Ties expert Rust** — but the expert shape is an UNVERIFIED assertion.
   Swap in a non-associative op (signed sat-add, saturating_sub) and Rust
   compiles it silently to garbage; Whitefoot refutes the law at compile time.
   This is the W3 delta in its purest form: the cheat (or the honest mistake)
   is structurally unavailable.
3. **W1 delta**: the fast shape in Whitefoot IS the obvious fold; in Rust the fast
   shape requires knowing the 4-accumulator idiom and being right about the
   algebra with no checker behind you.
4. Headroom: the 4 chains are scalar (adds+csinv x4, paired loads); SLP does
   not pack them into NEON uqadd.2d. A wider rewrite (8 accs) or SLP hints
   could roughly double the delta on this port.
5. At n=4096 facts trails rust-expert (0.210 vs 0.155): per-call guard
   overhead (rem/lim computation) is visible at short arrays; amortized away
   by 65536.

## Regression pin

`make check` pin #3: facts IR must contain >= 8 sat-add sites (reassociated),
control <= 3 (serial).
