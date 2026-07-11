# base64 port (safe-direction pilot #2)

Status: MEASURED 2026-07-10. First const-array consumer: byte-identical to the
RFC 4648 alphabet, fuzz-verified, faster than GNU and the Rust rewrite, ~parity
with the platform-tuned BSD tool.

## What it proves
- The const-array feature (implemented this session) works on a real codec:
  the 64-entry alphabet is a `const b64: array<u8, 64>` looked up per sextet.
- Byte-identical to system base64 on all RFC 4648 test vectors and 300/300
  random fuzz cases (newline-normalized; both encode identically).

## Performance (384MB random input, warm, medians, Apple M4)
| implementation | time |
|---|---:|
| BSD base64 (macOS, platform-tuned) | **0.20s** |
| xlang xb64 (kernel + C driver) | 0.23s |
| xlang xb64 (no-facts control) | 0.23s |
| GNU base64 (gbase64) | 0.36s |
| uutils base64 (Rust) | 0.36s |

- xlang is 1.6x faster than GNU AND the Rust rewrite; ~15% behind BSD's
  hand-tuned encoder. A codec is the "fast shape is the obvious shape" case —
  parity-at-C-speed is the honest headline, not a speed win.
- facts vs no-facts is neutral here (single owned src buffer + one out-borrow,
  no cross-buffer aliasing pressure) — the story is codegen quality + safety,
  not the alias channels.

## Language findings surfaced (fed to notes/pattern doctrine)
1. ANF is verbose for bit-twiddling: base64's `(x >> 18) & 63` becomes two
   bound lets. Expected under D2a (AI pays it), but the encode kernel is ~90
   lines for what C does in ~15 — worth a "the obvious xlang shape is verbose
   here" honesty note when advertising.
2. Whole-function no-shadowing forces globally-unique local names across
   sibling blocks (loop body vs the two tail match arms) — had to suffix each
   arm's locals (p*, q*). Mechanical for an AI writer; a human would chafe.
3. Implemented this session to make it compile: `&uniq buffer<u8>` /
   `&buffer<u8>` params (lowered as {ptr,i64} by value — element writes go
   through the shared data pointer, caller-visible; exclusivity stays a
   checker fact). This is the out-buffer idiom for codecs.

## Caveats / next
- Encode only; decode (with input validation — the CVE-relevant direction)
  is the natural follow-up and a stronger safety story.
- Driver slurps; a streaming/chunked driver would confirm the warm numbers.
- The table lookup is scalar; SIMD base64 (which BSD approximates) is a
  blessed-pattern opportunity, not attempted.

## Elision-ceiling experiment (2026-07-10)

`--elide-bounds-experiment` (perfect-prover upper bound; experiment-only
flag, never a shipping mode): encode kernel 2.44 -> 4.2 GB/s (**1.7x**),
hot-loop branches 41 -> 9, outputs byte-identical to system base64 on random
data. Still ZERO auto-vectorization even fully elided — the SIMD base64
algorithm (wide tables + tbl shuffles) is not vectorizer-discoverable, so
elision's honest value here is scalar: shorter dependency chains, no
side-exits. Checks in this kernel divide into two provable classes:
(a) loop-guard-dominated source reads (`rem >= 3` implies i, i+1, i+2 < n) —
a structural prover covers these; (b) output writes bounded by a CALLER
guarantee (out capacity >= 4*ceil(n/3)) — needs a precondition surface;
LLVM cannot know it and the checker can. Design card: gates 2026-07-10.
