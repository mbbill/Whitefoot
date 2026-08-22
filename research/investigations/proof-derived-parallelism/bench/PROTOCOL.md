# Measurement protocol

## Machine and toolchains

- Apple M4, 10 cores: 4 performance + 6 efficiency. macOS 26.5.2.
- Whitefoot: `compiler/target/release/whitefootc` of this repository, built
  with `cargo build --release` (branch `par/proof-derived-parallelism`, batch
  0074 closed pending owner merge, at the time of the recorded baseline). The
  compiler emits textual LLVM IR and links it with `/usr/bin/clang -O2`; with
  `--par` it also compiles and links its embedded `par_runtime.c` with
  `-pthread`. For the recorded baseline the benchmark lived outside the
  worktree and no file in the worktree was written by it.
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1`, built with
  `cargo build --release` (opt-level 3), `rayon 1.12.0`.

## Grid

The layout family: four tree shapes — balanced depth 8, 10, 12 and one
deterministically skewed depth-16 shape — crossed with three metric-table
lengths, 16, 64, and 192 words per node. Twelve configurations.

The index-split family: one configuration, `grid_d21_w256` — a Mandelbrot
escape count over a 2^21-point index range at an orbit cap of 256, split by
recursive halving. It is one row rather than a sweep because it carries a
different question from the twelve — whether a workload whose parallelism comes
from an index range rather than from a data structure scales the same way — and
a sweep over its own parameters would be a second grid measuring the same
mechanism.

Thirteen configurations in all. The repetition count of each was fitted from a
measured cost model so that its sequential run lands inside the required
0.3-3 s band.

## Implementations, fourteen cells per configuration

| cell | what it is |
|---|---|
| `wf_seq` | `whitefootc` with no `--par`: the default compilation, no outlining, no runtime linked |
| `wf_par/1` | `whitefootc --par`, `WF_WORKERS=1`: outlined and offered, but the pool never starts. The opt-in cost control. |
| `wf_par/2` `wf_par/4` `wf_par/8` | the same binary at 2, 4, 8 lanes |
| `wf_par/default` | the same binary with `WF_WORKERS` genuinely unset: the shipped default, which asks for this machine's logical CPUs, 10 here |
| `rs_seq` | plain recursive Rust |
| `rs_rayon/2` `/4` `/8` | `rayon::join` at every `Branch` (at every split, for `grid`), pool sized 2, 4, 8 |
| `rs_rayon/default` | the same `rayon::join` code with no thread count named anywhere: rayon's own global pool, 10 here |
| `rs_cut/2` `/4` `/8` | `rayon::join` only above depth 5, plain sequential fold below it |

`WF_WORKERS=N` means N threads of execution in total, because the calling
thread is itself a lane. `rayon::join` invoked through `pool.install` from a
non-pool thread blocks the caller and runs the work on the pool, so a pool of
N is also N threads of execution. The two knobs are therefore comparable at the
same number.

### Amendment of 2026-08-22: the `default` cells

The two `default` cells were added after the 2026-08-21 baseline was recorded.
Every cell in that baseline names a worker count, so the number a program
actually gets when it configures nothing appeared in no table — and on both
sides that number is the one most programs will run. The amendment measures it
directly: `wf_par/default` runs the `--par` binary with `WF_WORKERS` unset,
which the runtime answers with `hw.logicalcpu` clamped to its lane ceiling, and
`rs_rayon/default` calls `rayon::join` without building a pool at all, which
rayon answers with its lazily-initialised global pool. Neither cell is a
configured count that happens to equal the core count: the harness clears
`WF_WORKERS` from its own environment before the rotation, and the Rust binary
takes a distinct `default` argument that skips `ThreadPoolBuilder` entirely.

Both languages default to 10 threads of execution on this 4P+6E machine, so
the pair is comparable in the same way the numbered cells are, and the `default`
column answers a question the numbered columns cannot: not how well each
language scales when tuned, but what an untuned program gets.

## Rotation

Every one of the 13 x 14 = 182 (configuration, implementation) cells is visited
exactly once per round, in a fixed rotation, and the rounds repeat N times.
No cell is ever run twice in a row and no configuration is run as a block, so
thermal drift and background activity are spread across all cells rather than
concentrated in one. The authoritative pass is N = 18: 3,276 runs in total.
(The 2026-08-21 baseline predates the `grid` row and the `default` cells, and
is 144 cells at N = 9.)

Each run's stdout goes to its own file in `out/`; its exit status is read
directly from the process, never through a pipeline; its wall time is taken
from zsh's `$EPOCHREALTIME` immediately either side of the process.

## Reporting rule

Per cell: minimum, median, and spread as `(max - min) / min`. Ratios are taken
between minima. **Any ratio between 0.83x and 1.20x is marked `(u)` and is
unresolved** — the instrument does not separate those two numbers, and no
sentence in this file claims otherwise about a cell so marked.

## Byte comparison

`compare_outputs.zsh` runs `cmp` over the real published bytes of every run:
all Whitefoot outputs of a configuration against each other across the default
compilation and every worker count and every round; all Rust outputs likewise;
and then Whitefoot against Rust.
