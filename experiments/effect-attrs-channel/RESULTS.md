# Channel 2: effect rows → LLVM function attributes (2026-07-09)

Machine: Apple M4, /usr/bin/clang (Apple 21), rustc 1.91 edition 2021. Kernel: loop of
2e9 iterations calling an opaque-boundary pure fn `mix` with loop-invariant argument,
accumulating with wrapping add. All variants bit-identical semantics.

## Design point
The attribute must be tested on a DECLARATION with the body hidden (separate .o, no
LTO): that isolates *declared+checked source effects* from what LLVM would infer by
reading the body. democ emits `nounwind willreturn memory(none)` on both the define
and the declare — derived from `pure` + the new TOTALITY derivation (loop-free body,
total callees, trap-free ⇒ willreturn; sound, zero writer burden). Finding en route:
without `willreturn`, `memory(none)` alone hoists NOTHING (LICM requires
non-divergence) — EFF-3's honest no-termination stance gated the channel until the
derived-totality tier closed it.

## Results (median of 3)
| variant | boundary | time |
|---|---|---|
| Whitefoot, declared effects on declare | opaque (.o + .o, no LTO) | **0.00 s** (call hoisted; loop strength-reduced to one multiply — O(1)) |
| Whitefoot, no attributes (control) | opaque | 1.47 s (2e9 real calls) |
| Rust `#[inline(never)]`, cross-crate | opaque (no LTO — cargo's default shape) | 1.49 s (1 call site in loop) |
| Rust, `-C lto=fat` | body visible | 0.00 s (0 call sites: inlined/inferred) |

## Honest interpretation
- The channel WORKS and is a complexity-class change (O(n)→O(1)) over both our own
  no-facts control and Rust's default separate-compilation shape.
- Rust with fat LTO ties — with body visibility, LLVM infers the same facts. The delta
  is therefore: **Whitefoot's default = Rust's most expensive configuration**, with the
  guarantee holding where inference cannot reach (opaque/cached objects, future FFI
  frames with declared effects, bodies too complex for the attributor).
- Durable systems story: facts-on-declarations decouple optimization from body
  visibility ⇒ LTO-grade cross-module optimization at per-file -O2 build cost. This is
  the check-loop-latency concern and the P0 delta pulling the SAME direction.
- Pre-registered risk confirmed in the small: same-module, LLVM inference matches us;
  the channel's value is at boundaries. Rust structurally lacks the source channel
  (no way to declare trusted effects on an extern fn).
