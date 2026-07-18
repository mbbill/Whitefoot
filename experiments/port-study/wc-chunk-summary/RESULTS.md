# Ordered chunk-summary `wc` experiment

Status: MEASURED 2026-07-10. The candidate is correct and parallelizable,
but it does **not** clear the real-program confidence gate. Do not extend the
compiler's checked-law consumer for this workload on the present evidence.

## Question

Can `wc` be recast as `map byte chunks -> ordered associative reduction` so
that Whitefoot's checked-law channel produces a real-program advantage over
best-effort safe Rust?

The summary carries `(lines, words, bytes, first_space, last_space)`. Its
ordered merge subtracts one word exactly when the left chunk ends in a word
and the right chunk begins in that same word. The empty summary is identity.
The operation is associative but deliberately **not commutative**.

## Correctness

- `check_algebra.py`: identity and every split are checked over all byte
  strings of length <=4 from `{a,b,space,newline}`; associativity holds for
  all **39,651,821** summary triples.
- `verify.py`: Whitefoot and Rust agree with an independent byte oracle on 107
  cases and 1,284 implementation/thread-split verdicts. Cases include empty
  input, every byte value, random binary inputs, empty chunks, and more
  threads than bytes.
- The 421,200,000-byte existing `wc` corpus produces the same result in every
  variant: 5,400,000 lines, 75,600,000 words, 421,200,000 bytes.

This is an experimental proof artifact only. It does not license optimizer
use under FN-4: the current democ prover accepts only single table operations.

## Performance

Apple M4. Kernel timer starts after the file is loaded. Values are medians of
five fresh-process samples; thread creation and ordered merge are included.

| variant | 1 thread | 2 threads | 4 threads | 8 threads |
|---|---:|---:|---:|---:|
| whitefoot-facts | 241.9 ms | 115.9 ms | 57.4 ms | 39.2 ms |
| whitefoot-no-facts | 220.4 ms | 112.9 ms | 56.5 ms | 41.2 ms |
| C control | 133.6 ms | 67.5 ms | 34.8 ms | 24.8 ms |
| safe Rust | 135.0 ms | 67.2 ms | 34.4 ms | **24.1 ms** |

The facts/no-facts `summarize` assembly is byte-identical; their timing spread
is noise rather than a fact-channel delta. Facts only shorten the tiny
`combine` function, which is called once per chunk and is immaterial.

## What the experiment found

1. **The algebra works.** Word counting really can be represented as an
   ordered monoid, and arbitrary chunk boundaries merge exactly.
2. **Parallel headroom is real.** Whitefoot scales about 5.6x from one to eight
   threads. This validates chunk summarization as an implementation pattern.
3. **It is not presently a channel-3 win.** The existing consumer requires
   associative + commutative scalar reductions. This workload needs an
   associative-only, order-preserving tree reduction plus a parallel runtime.
4. **Even the ceiling does not beat expert safe Rust.** The safe Rust shape is
   fully expressible and 1.6-1.8x faster tha Whitefoot here. C and Rust are at
   parity, so this is not a library/I/O artifact.
5. **A separate Whitefoot code-shape gap dominates.** Clang vectorizes the Whitefoot
   loop at width 2 x interleave 4; equivalent C and Rust loops vectorize at
   width 16. Moving `first_space` out of the loop improved Whitefoot from ~311 ms
   to ~220 ms, but forcing width 16 still did not close the gap. The kernel's
   Bool/word-transition lowering needs investigation independently of laws.

## Decision

This candidate fails the pre-registered bar: there is no fact-attributable
win over best-effort safe Rust. Adding associative-only law discharge,
ordered reduction rewriting, and threading would be substantial compiler
investment whose measured performance ceiling is already matched or beaten
by Rust.

Preserve the algebra and harness as evidence, but do not promote this into the
compiler roadmap. The next confidence-gate candidate should exercise a fact
channel that safe Rust cannot recover merely by choosing the same algorithmic
shape; alternatively, first treat the width-2 vectorization as a focused
codegen-quality investigation rather than as a language-feature project.

## Reproduce

```sh
make -C experiments/port-study/wc-chunk-summary check
make -C experiments/port-study/wc-chunk-summary bench
```

## Addendum: Bool-copy amendment closes the gap (2026-07-10)

The width-2 lowering deficiency is FIXED via the OWN-1 amendment (tag-only
enums are copy) + democ i1 lowering (mutable Bool slots; band/bor/bxor/bnot
kept in i1). The scan loop rewritten in i1 dataflow form
(`starts_word = band(prevspace, bnot(issp))`, increments via give-match
selects) now vectorizes at width 16 (29 x16b ops; previously zero).

| variant | 1 thread | 2 threads | 4 threads | 8 threads |
|---|---:|---:|---:|---:|
| whitefoot-facts | ~134-140 ms | 70.3 ms | 35.6 ms | 26.4 ms |
| whitefoot-no-facts | 134.3 ms | 67.2 ms | 34.1 ms | 27.5 ms |
| C control | 132.6 ms | 67.5 ms | 34.8 ms | 27.6 ms |
| Rust safe | 133.9 ms | 67.6 ms | 34.2 ms | 27.9 ms |

Correctness: full harness green (39.65M algebra triples, 107 cases, 1284
split verdicts). facts/no-facts summarize asm byte-identical (label names
only); the initial 153ms facts median was a thermal outlier — interleaved
best-of shows <2% spread.

PARITY ACHIEVED: the 1.6-1.8x gap was entirely the i64-recurrence lowering,
as diagnosed. The experiment's DECISION IS UNCHANGED: this remains a
no-fact-advantage workload (safe Rust expresses the identical algorithm);
what changed is that Whitefoot's scalar codegen no longer owes anyone 1.6x.
