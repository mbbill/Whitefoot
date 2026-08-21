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

Four tree shapes — balanced depth 8, 10, 12 and one deterministically skewed
depth-16 shape — crossed with three metric-table lengths, 16, 64, and 192
words per node. Twelve configurations. The repetition count of each
configuration was fitted from a measured cost model so that its sequential run
lands inside the required 0.3-3 s band.

## Implementations, twelve cells per configuration

| cell | what it is |
|---|---|
| `wf_seq` | `whitefootc` with no `--par`: the default compilation, no outlining, no runtime linked |
| `wf_par/1` | `whitefootc --par`, `WF_WORKERS=1`: outlined and offered, but the pool never starts. The opt-in cost control. |
| `wf_par/2` `wf_par/4` `wf_par/8` | the same binary at 2, 4, 8 lanes |
| `rs_seq` | plain recursive Rust |
| `rs_rayon/2` `/4` `/8` | `rayon::join` at every `Branch`, pool sized 2, 4, 8 |
| `rs_cut/2` `/4` `/8` | `rayon::join` only above depth 5, plain sequential fold below it |

`WF_WORKERS=N` means N threads of execution in total, because the calling
thread is itself a lane. `rayon::join` invoked through `pool.install` from a
non-pool thread blocks the caller and runs the work on the pool, so a pool of
N is also N threads of execution. The two knobs are therefore comparable at the
same number.

## Rotation

Every one of the 12 x 12 = 144 (configuration, implementation) cells is visited
exactly once per round, in a fixed rotation, and the rounds repeat N = 9 times.
No cell is ever run twice in a row and no configuration is run as a block, so
thermal drift and background activity are spread across all cells rather than
concentrated in one. 1,296 runs in total.

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
