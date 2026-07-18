# base64 port (safe-direction pilot #2)

Status: MEASURED 2026-07-10; PROOF-2 and controlled-adversary update
2026-07-11. First const-array consumer: byte-identical to the RFC 4648
alphabet and fuzz-verified. The complete Whitefoot CLI beats GNU, the uutils
base64 CLI, and the platform BSD tool after proof elision; its scalar kernel
is in practical parity with expert safe Rust.

## What it proves
- The const-array feature (implemented this session) works on a real codec:
  the 64-entry alphabet is a `const b64: array<u8, 64>` looked up per sextet.
- Byte-identical to system base64 on all RFC 4648 test vectors and 300/300
  random fuzz cases (newline-normalized; both encode identically).

## Initial pre-proof performance (384MB random input, warm, medians, Apple M4)
| implementation | time |
|---|---:|
| BSD base64 (macOS, platform-tuned) | **0.20s** |
| Whitefoot xb64 (kernel + C driver) | 0.23s |
| Whitefoot xb64 (no-facts control) | 0.23s |
| GNU base64 (gbase64) | 0.36s |
| uutils base64 (Rust) | 0.36s |

- Whitefoot is 1.6x faster than GNU and the uutils base64 CLI; ~15% behind BSD's
  hand-tuned encoder. A codec is the "fast shape is the obvious shape" case —
  parity-at-C-speed is the honest headline, not a speed win.
- Alias facts vs no alias facts were neutral in this pre-bounds-proof snapshot
  (single owned src buffer + one out-borrow, no cross-buffer aliasing
  pressure). PROOF-1/2 later made bounds facts strongly non-neutral.

## Language findings surfaced (fed to notes/pattern doctrine)
1. ANF is verbose for bit-twiddling: base64's `(x >> 18) & 63` becomes two
   bound lets. Expected under D2a (AI pays it), but the encode kernel is ~90
   lines for what C does in ~15 — worth a "the obvious Whitefoot shape is verbose
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

## PROOF-1 local discharge (2026-07-10)

The shipping facts path now reports every lowered bounds site in `encode`:

- 27 total sites;
- 15 proved locally: 6 source reads from the fixed-stride remainder invariant
  and 9 alphabet reads from masked ranges propagated through unsigned widening;
- 12 retained: every remaining site is an output write whose safety depends on
  the caller-provided capacity.

Five-sample medians on the same 384MB encode-only harness:

| variant | throughput | time/pass |
|---|---:|---:|
| no facts | 2.50 GB/s | 153.9 ms |
| PROOF-1 local facts | **2.93 GB/s** | **131.2 ms** |
| perfect-prover ceiling | 4.23 GB/s | 90.9 ms |

Local proof discharge is a 1.17x throughput gain and recovers about 36% of the
removable time measured by the ceiling. Optimized `encode` shrinks from 127 to
110 instructions (ceiling: 66). The 9 alphabet checks were already removed by
LLVM, so the measured gain comes from the 6 structurally proved source reads.
Output correctness is unchanged: facts vs no-facts and facts vs system base64
both passed 139/139 boundary-biased random cases. The remaining performance
gap is now cleanly a PROOF-2 question: establish
`len(out) >= 4 * ceil(len(src)/3)` at the call boundary and connect it to the
lockstep `i=3k, o=4k` loop invariant.

Reproduce with `python3 proof_benchmark.py [BYTES] [SAMPLES]`; it rebuilds all
three variants in a temporary directory before measuring them.

## PROOF-2 checked capacity + lockstep discharge (2026-07-11)

`encode` now carries one checked callee-entry `requires` clause spelling the
overflow-safe relation `len(src) <= 3 * floor(len(out)/4)`. The check remains
in every build and direct C entry cannot bypass it. The deterministic prover
then connects that passed fact to the exact loop induction `i = 3k, o = 4k`
and the mutually exclusive one-/two-byte tail arms.

Structured accounting on the unchanged 27 lowered index sites is now:

- 27 proved, 0 retained;
- 6 source reads and 9 alphabet reads from PROOF-1;
- 12 output writes from `output-capacity-lockstep`;
- facts-off retains all 27 sites while executing the identical entry check.

Five-sample medians on the 384MB encode-only harness:

| variant | throughput | time/pass |
|---|---:|---:|
| no facts (entry check + all index checks) | 2.480 GB/s | 154.9 ms |
| PROOF-2 | **4.233 GB/s** | **90.7 ms** |
| perfect index-elision ceiling (entry check retained) | 4.215 GB/s | 91.1 ms |

PROOF-2 is 1.71x over the same-source facts-off control and reaches the
measurement-noise envelope of the perfect-prover ceiling: both optimized
`encode` bodies contain 77 instructions and one retained trap path. Correctness
remains pinned by 139/139 deterministic boundary-biased facts/nofacts/Python
reference differentials. A separate direct-C ABI probe confirms exact capacity
succeeds and one-byte-under capacity traps at the callee boundary before the
first body byte store (`verify.py`).

## Post-PROOF-1/2 ladder (2026-07-11, 384MB, byte-identical outputs)

| implementation | time |
|---|---:|
| **Whitefoot, proofs active** | **0.16s** |
| BSD base64 (Apple, wide-table) | 0.21s |
| GNU base64 | 0.36s |
| uutils base64 (Rust) | 0.36s |

Kernel: 4.05-4.12 GB/s with proofs vs 2.45-2.48 no-facts control (1.66x) —
97% of the perfect-prover ceiling (4.2), with full trap-on-violation
semantics and the boundary check intact. History: pre-proof checked build
was 0.23s and LOST to BSD's 0.20; the proof tier flipped the ladder — now
1.3x ahead of BSD and 2.25x ahead of GNU and the uutils base64 CLI.

## Controlled Rust adversary correction (2026-07-11)

The original adversary table mixed two fixed-order harnesses and compared the
complete checked Whitefoot encoder with a Rust `chunks_exact/zip` kernel that
silently truncated short output and discarded one-/two-byte input tails. Its
apparent ~5% Rust lead is superseded by this controlled rerun. The old 384MB
input was divisible by three and its output was ample, so those API mismatches
did not execute in the timed path; fixing them makes the comparison honest,
while isolated balanced timing fixes the unsupported ranking.

`adversary_benchmark.py` builds one executable containing Whitefoot PROOF-2 and
four Rust variants. Each of 30 timing blocks runs in a fresh process; within a
block all five variants share the same source buffer, exact-capacity output
buffer, and clock. Every Rust variant now emits RFC-padded tails. The assert,
chunks, and unsafe candidates enforce the same entry-capacity relation as Whitefoot;
naive remains the deliberate ordinary-bounds-check control. Before timing,
the harness checks all five outputs at exact capacity for every length 0..257;
the independent 139-case differential and short-capacity trap gate also
remains green. The build additionally requires the Whitefoot proof report to stay
at 27 proved, 0 retained, including 12 output-capacity-lockstep sites. Both
toolchains target the native Apple M4: rustc 1.91.1 (LLVM 21.1.2) and Apple
clang 21.0.0 at `-O3`/equivalent, one codegen unit, and aborting Rust panics.

The evidence pass uses three cycles of ten isolated Williams-balanced process
blocks: 30 samples per variant, every variant in every ordinal position six
times, and every ordered first-order pair six times inside isolated blocks.
Block execution order is deterministically shuffled; all samples are retained.
The table reports normalized `MAD / median`. `XL/variant` is the median
within-process throughput ratio with a descriptive deterministic
10,000-resample process-block bootstrap interval, not a general-population
confidence claim. The primary practical-equivalence margin was fixed at plus
or minus 2% before the 384MB evidence run.

| variant | median | MAD/median | throughput | XL/variant process-block ratio (row-bootstrap 95% interval) |
|---|---:|---:|---:|---:|
| Whitefoot obvious shape + requires | 89.625 ms | 0.27% | **4.285 GB/s** | 1.000 |
| Rust naive indexed | 143.670 ms | 0.30% | 2.673 GB/s | 1.602 (1.598..1.613) |
| Rust assert-up-front | 143.427 ms | 0.25% | 2.677 GB/s | 1.604 (1.598..1.610) |
| Rust safe `chunks_exact/zip`, full semantics | **89.355 ms** | 0.29% | **4.297 GB/s** | **0.997 (0.994..0.999)** |
| Rust unsafe indexed | 93.407 ms | 0.90% | 4.111 GB/s | 1.040 (1.033..1.043) |

Verdicts:

1. The assert idiom remains refuted as a recovery path. Assert and naive are
   indistinguishable at about 2.67 GB/s, and assembly inspection shows the same
   inner source and output bounds branches. LLVM still cannot connect the
   entry relation to the coupled `i += 3, o += 4` induction.
2. Expert safe Rust still reaches the fastest measured check-free scalar
   performance class by restructuring. Its
   `chunks_exact/zip` loop computes one group count and contains no inner
   bounds branches, so base64 still does **not** clear D9's strict
   best-effort-safe-Rust bar.
3. The corrected primary comparison is practical parity, not a 5% codegen
   deficit: the observed paired Rust edge is about 0.3%, and the descriptive
   row-bootstrap interval stays between 0.1% and 0.6%, entirely inside the
   predeclared 2% band. Per-cycle medians are 0.997, 0.994, and 0.999;
   XL-first and Rust-first medians are 0.997 and 0.995. The old "~5% residual
   = loop-shape quality" attribution is withdrawn. The assembly shapes still
   differ, but this experiment establishes no practically meaningful
   throughput debt above the 2% margin.
4. The durable W1 result is distributional: Whitefoot's obvious indexed loop plus
   one checked relation reaches expert-safe-Rust performance, whereas Rust's
   obvious indexed loop stays near 2.7 GB/s and the local assert changes
   nothing. The writer must know the iterator restructuring.
5. QOI decode remains the decisive leg-B experiment because variable-size
   token writes cannot be reduced to fixed `chunks_exact` pairs this way.

Reproduce with:

```sh
python3 experiments/port-study/base64/verify.py
python3 experiments/port-study/base64/adversary_benchmark.py 384000000 3
```

The 150 retained timing samples are in
`adversary-rerun-2026-07-11.csv` (SHA-256
`ebea523dda82e7e7d3156da1dbb982a58fa5672a1c2bf751dbfd5a488fca7a20`).
`adversary-rerun-2026-07-11.metadata.json` pins the exact source hashes,
toolchains, flags, host, proof accounting, protocol, base commit, and dirty
state (sidecar SHA-256
`1b83368d7127f96304c97bd65d29aa16406f0ea28387ca2c4a2fa1c651316f84`).
