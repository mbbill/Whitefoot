# The paired benchmark

This is the standing oracle for the proof-derived parallelism investigation.
One algorithm is written twice, once in Whitefoot and once in Rust, statement
for statement, and every Whitefoot binary is compiled both with and without
`--par`. Both sides publish their result as sixteen hexadecimal digits, so
`cmp` on the bytes, not a tolerance, decides whether the two languages and
every worker count computed the same thing.

Two workload families share the grid, the harness, and the reporting rule:

- **the layout fold** (`bal`, `skew`) — a bottom-up browser-layout fold over a
  heap-boxed binary tree, where the eligible pair is the two child calls. This
  is the original workload and everything called "the baseline" below is it.
- **the index split** (`grid`) — a Mandelbrot escape count over a linear index
  range, halved recursively, where the eligible pair is the two halves of the
  split and no data structure is involved at all. It is here because the
  layout fold answers only for parallelism that falls out of a tree, and the
  ordinary data-parallel shape — a counted loop over a range with an
  accumulator — is the one the permission judgment gives nothing to. Written
  as a recursion instead of a loop, that same computation is eligible with no
  change to the compiler; this row is what that is worth.

What it is for: an optimization dig in this investigation states its before and
after against this grid, at this protocol. It measures four things at once —
Whitefoot sequential codegen against Rust sequential codegen, the `--par`
opt-in cost at one worker, `--par` lane scaling against Whitefoot sequential,
and Whitefoot's best against `rayon`'s best.

`WORKLOAD.md` describes both families, the builders, the passes, and the
Whitefoot-to-Rust operation mapping including its one inexact cell.
`PROTOCOL.md` describes the machine, the grid, the twelve implementation cells,
the rotation, the reporting rule, and the byte comparison. Read both before
quoting a number from here.

## What is checked in

The generator (`gen_wf.sh`, `configs.txt`) and the Rust twin (`rust/`) are the
sources. The `.wf` files are not checked in: `build_wf.sh` regenerates them
deterministically from the generator on every build, and a second checked-in
copy would go stale beside it. `baseline-20260822/` holds the current reference
measurement — N = 18 over all thirteen configurations including the two
`default` cells — and `baseline/` is kept unchanged as the earlier 2026-08-21
snapshot, whose tables are the ones quoted under "Baseline, 2026-08-21" below.
`wf/`, `bin/`, `out/`, `logs/`, and `rust/target/` are generated and ignored.

## Rerun

From this directory, with the release compiler already built
(`cargo build --release --manifest-path ../../../../compiler/Cargo.toml`):

```
./build_wf.sh                                        # generate + compile every config, seq and --par
cargo build --release --manifest-path rust/Cargo.toml # the Rust twin
./rerun.zsh 9                                        # measure, byte-compare, rebuild the tables
```

`rerun.zsh N` runs N rounds of the full rotation, byte-compares every output,
and writes `logs/t_*.md`. The protocol wants N >= 9; the recorded baseline is
two passes of 9, N = 18 per cell. `build_wf.sh` honours a `WFC=` override if
the compiler you want to measure is not this worktree's release build.

Nothing else in the loop is optional: a dig that changes the compiler rebuilds
both `build_wf.sh` products and reruns the whole rotation, because a partial
grid measured against a different thermal state is not comparable.

`timeit.zsh N label=WORKERS:binary ...` is the same protocol for a smaller
question: an interleaved min-of-N timer over an arbitrary set of binaries,
reporting min, max, spread, failure count, and an output digest per cell. It is
what a probe outside the configuration grid — a depth sweep, a single shape at
one worker count — is measured with, so a probe number and a grid number are
produced by the same rotation and minimum rule.

## Reading rule

Ratios are min-of-N. **Any ratio between 0.83x and 1.20x is unresolved** and
the tables mark it `unres.`; this instrument does not separate those two
numbers, and no claim may be made about a cell so marked. Per-cell spreads are
large — mean 112%, worst 411%, over the current reference snapshot's 182 cells
— because the machine is a shared laptop with efficiency cores; that is exactly
why the minimum, not the mean, is the statistic, and why sub-20% differences are
not conclusions.

## Baseline, 2026-08-21

N = 18 repetitions per cell, 144 cells, Apple M4 (4P + 6E). Byte comparison:
every run of every configuration published identical bytes, in both languages
and across them (`baseline/byte_comparison.txt`). Full snapshot in
`baseline/results.tsv` and `baseline/cells.tsv`.

This snapshot is the twelve layout configurations only; the `grid` family was
added afterwards and is not in it. **This section is superseded and kept as the
dated 2026-08-21 record.** It once promised that the next authoritative rotation
would replace it with all thirteen; that rotation landed as
`baseline-20260822/`, which ships its own README and ten generated tables, and
this section was left in place instead of being rewritten. Read the tables below
as history — their headline WF/Rust column reports 1.56x-3.29x with `wf_par/1`
as the best Whitefoot cell, where the current snapshot reports 0.83x-1.05x,
unresolved on twelve of thirteen. The `baseline/` pointers in this section are
correct for this section.

### Configuration inventory

| config | shape | tree nodes | leaves | words/node | reps |
|---|---|---:|---:|---:|---:|
| `bal_d8_w16` | bal, depth 8 | 511 | 256 | 16 | 100000 |
| `bal_d8_w64` | bal, depth 8 | 511 | 256 | 64 | 32000 |
| `bal_d8_w192` | bal, depth 8 | 511 | 256 | 192 | 11500 |
| `bal_d10_w16` | bal, depth 10 | 2047 | 1024 | 16 | 25000 |
| `bal_d10_w64` | bal, depth 10 | 2047 | 1024 | 64 | 8000 |
| `bal_d10_w192` | bal, depth 10 | 2047 | 1024 | 192 | 2850 |
| `bal_d12_w16` | bal, depth 12 | 8191 | 4096 | 16 | 6300 |
| `bal_d12_w64` | bal, depth 12 | 8191 | 4096 | 64 | 2000 |
| `bal_d12_w192` | bal, depth 12 | 8191 | 4096 | 192 | 700 |
| `skew_d16_w16` | skew, depth 16 | 3663 | 1832 | 16 | 14000 |
| `skew_d16_w64` | skew, depth 16 | 3663 | 1832 | 64 | 4500 |
| `skew_d16_w192` | skew, depth 16 | 3663 | 1832 | 192 | 1600 |

### Sequential parity — the codegen axis

Whitefoot with no `--par` against plain recursive Rust. Ten of twelve are
unresolved; the two resolved cells are both the skewed deep tree, and the loss
grows with per-node work. That is the skew sequential-gap dig of batch 0076.

| config | WF-seq min | med | spread | Rust-seq min | med | spread | WF/Rust (min-of-N) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `bal_d8_w16` | 0.569 | 0.780 | 95% | 0.578 | 0.744 | 70% | 0.98x unres. |
| `bal_d8_w64` | 0.488 | 0.678 | 77% | 0.491 | 0.639 | 63% | 1.00x unres. |
| `bal_d8_w192` | 0.602 | 0.809 | 80% | 0.603 | 0.787 | 85% | 1.00x unres. |
| `bal_d10_w16` | 0.569 | 0.784 | 68% | 0.569 | 0.752 | 110% | 1.00x unres. |
| `bal_d10_w64` | 0.498 | 0.705 | 66% | 0.552 | 0.640 | 62% | 0.90x unres. |
| `bal_d10_w192` | 0.716 | 0.812 | 40% | 0.675 | 0.778 | 58% | 1.06x unres. |
| `bal_d12_w16` | 0.710 | 0.787 | 30% | 0.639 | 0.778 | 52% | 1.11x unres. |
| `bal_d12_w64` | 0.555 | 0.714 | 58% | 0.571 | 0.681 | 43% | 0.97x unres. |
| `bal_d12_w192` | 0.651 | 0.825 | 65% | 0.604 | 0.790 | 67% | 1.08x unres. |
| `skew_d16_w16` | 0.739 | 0.879 | 55% | 0.696 | 0.760 | 60% | 1.06x unres. |
| `skew_d16_w64` | 0.684 | 0.852 | 73% | 0.560 | 0.700 | 98% | 1.22x |
| `skew_d16_w192` | 0.868 | 1.063 | 76% | 0.618 | 0.778 | 84% | 1.41x |

### Whitefoot `--par` scaling, against Whitefoot sequential

Speedup tops out at 2.23x, the optimum is never past 4 lanes, and at 16
words/node 8 lanes run up to 7x SLOWER than sequential (0.13x). That is the
scheduler dig of batch 0076. The last column is the opt-in cost of compiling
with `--par` and never starting the pool.

| config | WF-seq | --par w=1 | w=2 | w=4 | w=8 | seq/w2 | seq/w4 | seq/w8 | opt-in cost w1/seq |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `bal_d8_w16` | 0.569 | 0.550 | 1.051 | 2.167 | 4.324 | 0.54x | 0.26x | 0.13x | 0.97x unres. |
| `bal_d8_w64` | 0.488 | 0.500 | 0.486 | 0.823 | 1.586 | 1.01x unres. | 0.59x | 0.31x | 1.02x unres. |
| `bal_d8_w192` | 0.602 | 0.613 | 0.405 | 0.561 | 0.784 | 1.49x | 1.07x unres. | 0.77x | 1.02x unres. |
| `bal_d10_w16` | 0.569 | 0.554 | 0.671 | 1.444 | 2.375 | 0.85x unres. | 0.39x | 0.24x | 0.97x unres. |
| `bal_d10_w64` | 0.498 | 0.518 | 0.310 | 0.474 | 0.805 | 1.61x | 1.05x unres. | 0.62x | 1.04x unres. |
| `bal_d10_w192` | 0.716 | 0.629 | 0.332 | 0.378 | 0.493 | 2.16x | 1.89x | 1.45x | 0.88x unres. |
| `bal_d12_w16` | 0.710 | 0.601 | 0.622 | 1.042 | 1.619 | 1.14x unres. | 0.68x | 0.44x | 0.85x unres. |
| `bal_d12_w64` | 0.555 | 0.550 | 0.290 | 0.335 | 0.529 | 1.92x | 1.66x | 1.05x unres. | 0.99x unres. |
| `bal_d12_w192` | 0.651 | 0.608 | 0.314 | 0.292 | 0.305 | 2.08x | 2.23x | 2.13x | 0.93x unres. |
| `skew_d16_w16` | 0.739 | 0.575 | 0.984 | 1.923 | 3.099 | 0.75x | 0.38x | 0.24x | 0.78x |
| `skew_d16_w64` | 0.684 | 0.535 | 0.454 | 0.698 | 0.853 | 1.51x | 0.98x unres. | 0.80x | 0.78x |
| `skew_d16_w192` | 0.868 | 0.625 | 0.511 | 0.514 | 0.499 | 1.70x | 1.69x | 1.74x | 0.72x |

The skewed rows' opt-in column is below 1.00x in all three: `--par` outlining
makes that shape 22-28% FASTER single-threaded. That is the outlining-paradox
dig of batch 0076.

### `rayon::join` at every branch, against Rust sequential

The comparison target: 4.46x at its best, and it degrades at fine grain far
more gracefully than Whitefoot's lane budget does.

| config | Rust-seq | rayon t=2 | t=4 | t=8 | seq/t2 | seq/t4 | seq/t8 |
|---|---:|---:|---:|---:|---:|---:|---:|
| `bal_d8_w16` | 0.578 | 0.409 | 0.360 | 0.981 | 1.41x | 1.61x | 0.59x |
| `bal_d8_w64` | 0.491 | 0.290 | 0.257 | 0.443 | 1.69x | 1.91x | 1.11x unres. |
| `bal_d8_w192` | 0.603 | 0.325 | 0.189 | 0.237 | 1.86x | 3.20x | 2.55x |
| `bal_d10_w16` | 0.569 | 0.376 | 0.274 | 0.503 | 1.51x | 2.08x | 1.13x unres. |
| `bal_d10_w64` | 0.552 | 0.301 | 0.199 | 0.278 | 1.84x | 2.77x | 1.99x |
| `bal_d10_w192` | 0.675 | 0.343 | 0.193 | 0.190 | 1.97x | 3.50x | 3.54x |
| `bal_d12_w16` | 0.639 | 0.427 | 0.307 | 0.291 | 1.49x | 2.08x | 2.19x |
| `bal_d12_w64` | 0.571 | 0.305 | 0.185 | 0.201 | 1.87x | 3.08x | 2.84x |
| `bal_d12_w192` | 0.604 | 0.316 | 0.164 | 0.135 | 1.91x | 3.69x | 4.46x |
| `skew_d16_w16` | 0.696 | 0.443 | 0.342 | 0.470 | 1.57x | 2.04x | 1.48x |
| `skew_d16_w64` | 0.560 | 0.320 | 0.202 | 0.212 | 1.75x | 2.77x | 2.64x |
| `skew_d16_w192` | 0.618 | 0.322 | 0.172 | 0.152 | 1.92x | 3.58x | 4.07x |

A depth-cutoff variant (`rs_cut`, `rayon::join` only above depth 5) is measured
too and is often the Rust winner at fine grain; see `baseline/t_rayoncut.md`.

### Cross-language headline

Best Whitefoot cell against best Rust cell, and against Rust sequential.

| config | best WF | which | best Rust | which | WF/Rust | best WF vs Rust-seq |
|---|---:|---|---:|---|---:|---:|
| `bal_d8_w16` | 0.550 | wf_par/1 | 0.253 | rs_cut/4 | 2.17x | 0.95x unres. |
| `bal_d8_w64` | 0.486 | wf_par/2 | 0.185 | rs_cut/4 | 2.62x | 0.99x unres. |
| `bal_d8_w192` | 0.405 | wf_par/2 | 0.189 | rs_rayon/4 | 2.14x | 0.67x |
| `bal_d10_w16` | 0.554 | wf_par/1 | 0.191 | rs_cut/4 | 2.91x | 0.97x unres. |
| `bal_d10_w64` | 0.310 | wf_par/2 | 0.199 | rs_rayon/4 | 1.56x | 0.56x |
| `bal_d10_w192` | 0.332 | wf_par/2 | 0.187 | rs_cut/8 | 1.78x | 0.49x |
| `bal_d12_w16` | 0.601 | wf_par/1 | 0.223 | rs_cut/8 | 2.70x | 0.94x unres. |
| `bal_d12_w64` | 0.290 | wf_par/2 | 0.182 | rs_cut/8 | 1.59x | 0.51x |
| `bal_d12_w192` | 0.292 | wf_par/4 | 0.135 | rs_rayon/8 | 2.16x | 0.48x |
| `skew_d16_w16` | 0.575 | wf_par/1 | 0.287 | rs_cut/4 | 2.00x | 0.83x |
| `skew_d16_w64` | 0.454 | wf_par/2 | 0.202 | rs_rayon/4 | 2.24x | 0.81x |
| `skew_d16_w192` | 0.499 | wf_par/8 | 0.152 | rs_rayon/8 | 3.29x | 0.81x |

Read the ratio column as the gap to close, not as a verdict on the language:
sequential parity is unresolved on ten of twelve configurations, so the gap
here is almost entirely the scheduler.

## Removal condition

This directory exists to support optimization digs against the parallel path.
It goes away when the investigation closes, together with the rest of
`research/investigations/proof-derived-parallelism/`.
