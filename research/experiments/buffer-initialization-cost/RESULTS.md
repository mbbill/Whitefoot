# Buffer initialization cost — results

Run id `buffer-init-cost-1`, 2026-08-06. Preregistration: `PROTOCOL.md`,
written before any number below was observed. Raw samples:
`raw/buffer-init-cost-1.jsonl`.

Host: Apple M4, macOS, `/usr/bin/clang -O2`, `rustc -C opt-level=2`. Corpus
256 MiB, buffer 4096 bytes, 30 paired rounds, whole-process timing.

## Result

**The one-time buffer fill is immaterial. The dossier §11 stop condition did
not fire.** The §9.1 initialization-cost row is discharged for this target.

## Observable 1 (primary) — Whitefoot against the uninitialized native control

Ratio is `uninit elapsed / whitefoot elapsed`; above 1.0 means Whitefoot is
faster.

| Comparison | Median | 95% percentile interval | Relative half-width | Band |
|---|---|---|---|---|
| whitefoot / uninitialized C | **1.0014** | [0.9982, 1.0083] | 0.51% | practical parity |

The whole interval lies inside the preregistered `[0.98, 1.02]` parity band and
the half-width clears the 2% precision requirement. A Whitefoot drain whose
reusable buffer the language initializes at allocation measures at parity with
the same drain in C over uninitialized storage, with the one-time fill counted
inside the timed region.

## Observable 2 (secondary) — the same-source ablation

One C source, one binary, one changed allocation call.

| Comparison | Median | 95% percentile interval | Relative half-width | Band |
|---|---|---|---|---|
| uninitialized C / initialized C | **0.9985** | [0.9935, 1.0071] | 0.68% | practical parity |

Within one language, one binary, and one loop, replacing `malloc` with
`calloc` changes steady-state throughput by nothing measurable. The point
estimate is nominally *below* 1.0 — that is, the initialized arm was nominally
the faster of the two — which is a direct sign that the effect is inside the
noise rather than merely small.

A third ratio was recorded for description only and was **not** preregistered
as an observable: whitefoot / initialized C, median 0.9981, interval
[0.9870, 1.0070], half-width 1.00%. Its interval does not lie wholly inside the
parity band, so it is reported unclassified rather than being read as a result.

## Observable 3 (decisive) — the cost itself

2,000,000 allocations per invocation, median of nine invocations per arm.

| Arm | Per allocation |
|---|---|
| `malloc(4096)` | 15.3485 ns |
| `calloc(1, 4096)` | 44.1065 ns |
| **initialization of one 4096-byte page** | **28.7580 ns** |

That is the entire quantity §9.1's row is about, measured directly rather than
inferred from a ratio. Against the runs it would have to be material to:

| Reference | Elapsed | Fill as a share | Distance from the 1% materiality line |
|---|---|---|---|
| 256 MiB drain (median of 30) | 30.517 ms | 0.000094% | 10,612x below |
| the same program on an **empty** input | 1.76 ms | 0.00163% | 612x below |

The second row is the one that settles it. 1.76 ms is the whole-process floor
of this program — the cost of a run that reads nothing at all — measured over
500 invocations of the same binary on an empty file (`malloc` arm 1.80 ms,
`calloc` arm 1.76 ms, Whitefoot drain 1.78 ms; the three are indistinguishable,
which is itself further evidence that the fill is unmeasurable at process
granularity). Because the floor already exceeds the fill by more than four
orders of magnitude, **there is no input size at which the fill reaches 1% of a
run**: the run would have to complete in 2.9 µs, which is 600x faster than the
program can start.

Stated against transfer work alone, excluding process startup: the drain moves
256 MiB in 28.74 ms of post-startup time, or 9.34 GB/s, so the fill equals 1%
of the *transfer work* at roughly 26 KB of input. That framing is the least
favourable one available and is recorded for completeness; it does not describe
a real run, because no run of this program costs less than its 1.76 ms floor.

## Interpretation and limits

- The §9.1 row is discharged: the one-time fill at allocation is not material
  to steady-state throughput on this target. Dossier §11's condition — "if
  initialized-buffer cost is material, work stops for a separately proved
  initialization model" — did not fire, and no such model is proposed here.
- Observables 1 and 2 could only ever fail to refute the hypothesis, since the
  effect is orders of magnitude below what whole-process timing resolves. That
  was stated in the preregistration, and it is why observable 3 exists. Do not
  read the parity bands as the evidence; observable 3 is the evidence.
- The `fill` figure is an **upper bound** on what `wfgrep` pays. It allocates
  and frees repeatedly, so the allocator recycles one hot block and `calloc`
  must zero it every time; a single startup allocation may instead come from
  fresh already-zero pages and cost less. Conservative in the direction that
  matters for a stop condition.
- The corpus is served from the page cache after the first round. This measures
  the drain and the transfer path, not a storage device.
- One host, one architecture, one allocator, one buffer size, one chunk size.
  This is evidence about this target and this shape, not a portable law and not
  a claim about any buffer larger than one page.
- This bundle measures nothing about `wfgrep`'s matching, its output batching,
  or any comparison against another program. It authorizes no language,
  compiler, specification, or optimizer change.

## Reproducing

```sh
make -C research/experiments/buffer-initialization-cost check   # oracle only
make -C research/experiments/buffer-initialization-cost bench   # full run
```

`bench` refuses to overwrite a recorded sample: the fixed run id's raw log must
not already exist in the work root, so a rerun uses a fresh `WORK_ROOT`.
