<!-- Serves research/experiments/compute-bench: the durable record of the
     tables that matter. Job logs and artifacts expire, so a table a decision
     rests on is copied here by hand with its run id and the host that
     produced it. Nothing in this file is generated, and nothing here is a
     gate: no number in it passes or fails anything. -->

# Compute scoreboard results

Status: **four tables recorded**, in the dated sections at the end of this
file: a baseline at compiler `33ed2c00` on the local four-logical-CPU Linux
host, then the first hosted run, `34574271919` at `5dd1eb7b`, one section per
leg of the bundle's workflow — `macos-14` (three CPUs, recorded block W=2) and
`ubuntu-24.04` (four CPUs, recorded block W=4) — and last the merged tree
re-emitted at `11d1e4a2` on the local host, which replaced a merged-tree table
that had timed one stale module. Each run that matters is added the same way,
newest last.

The question every table here answers is the bundle's: for each kernel, at each
width, is the Whitefoot program built by this tree's `whitefootc` with plain
`--par` the fastest thing in the row? Read
[`../../experiments/compute-bench/README.md`](../../experiments/compute-bench/README.md)
for what the columns mean, what each reference's grain policy is and is not, and
the admitted asymmetry between the two flag sets.

## Reproduce

```sh
export WHITEFOOT_SCRATCH_ROOT=$HOME/do_not_scan
make -C research/experiments/compute-bench deps     # once, with a network
make -C research/experiments/compute-bench build
make -C research/experiments/compute-bench verify
make -C research/experiments/compute-bench compare PASSES=5 CALLS=5
```

`compare` refuses a `RESULTS` directory that already exists, so a re-run by hand
needs a fresh `RESULTS=<path>`. `make clean` also clears the way and is the
blunter instrument: it removes the whole work directory under the scratch root,
every earlier table in it included. The hosted tables come from
`.github/workflows/compute-bench.yml`, which runs the same four commands on both
legs and uploads `manifest.txt`, `raw.tsv`, `table.txt` and `logs/` as a job
artifact besides printing the table to the job summary.

## Two rules for reading anything below

1. **Never pool two hosts.** The recorded block for a host is its highest width
   that is not oversubscribed, and hosts differ in that width — a four-CPU
   runner records `W=4`, a three-CPU one records `W=2`. Two hosts' blocks answer
   the question at different widths and about different machines; a median over
   them is a number about neither.
2. **Never pool runs taken with different `BENCH_ARCH`.** It reaches every
   reference translation unit and no Whitefoot one, so changing it changes the
   references' code generation and placement without touching the WF row. Runs
   on either side of that change are not samples of the same comparison.

Both rules are about pooling, not about comparing: two sections may be read
against each other as long as what differs between them is stated.

## How to add a section

Copy `$(RESULTS)/table.txt` verbatim into a fenced block under a new
`## <date> — <host>` heading, and record above it, from that run's
`manifest.txt`: the host (`uname -a`), the logical CPU count and the inherited
CPU mask as observed, the compiler revision, the clang, clang++, rustc, cargo
and cmake versions, the three dependency pins, `BENCH_ARCH`, the sizes and
emitted chunk counts, and the workflow run id (or `local`). State which width
block is the recorded one and confirm the sizing window held: the `wf-seq`
median inside [5 ms, 60 ms], the `wf` median above 1 ms at the recorded width,
and `steals > 0` on the `wf` row at every parallel width. Say what moved and why
if a size constant had to change. Newest section last.

<!-- Sections begin here. Template, kept for the next run and never filled in
     with numbers that were not measured:

## YYYY-MM-DD — <ubuntu-24.04 | macos-14 | local host>

- host: <uname -a>
- logical CPUs: <n>   inherited mask: <as recorded, or "unqualified">
- recorded block: W=<n>   oversubscribed blocks emitted: <list>
- compiler revision: <git rev>
- clang: <v>   clang++: <v>   rustc: <v>   cargo: <v>   cmake: <v>
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: <-march=x86-64-v3 | empty>
- sizes and emitted chunk counts: <as printed in the table header and note>
- workflow run: <run id or "local">
- sizing window: wf-seq <n> ms at W=1; wf <n> ms at W=<recorded>; steals <n>

```text
<table.txt, verbatim>
```

What it says: <one paragraph, and nothing that the table does not show>
-->

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), baseline at `33ed2c00`

**The before side.** This run was taken on `main` at `33ed2c00`, before PR #33
and PR #34, and therefore before `--par`'s scalar-leaf default of 16 reached the
compiled programs. It is kept for exactly that comparison; the merged tree's
table is the last section of this file. Its table is in the earlier reducer's layout,
with each form's grain policy inline as a column instead of the per-block legend
the reducer prints now; that is presentation only, and no number in it moved.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded; `cgroup_cpu_max` and
  `cpuset.cpus.effective` were both absent and are recorded as unqualified, not
  as "no limit")
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `33ed2c000f7ae2f097bb9c3deea9f8d90267be2a`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`
- sizing window, confirmed on this host before the table was recorded: every
  `wf-seq` median at W=1 inside [5 ms, 60 ms] — mandelbrot 26.674 ms,
  quadrature 16.399 ms, records 36.113 ms, fir 26.803 ms; every `wf` median at
  the recorded W=4 above 1 ms — mandelbrot 7.154 ms, quadrature 24.328 ms,
  records 9.508 ms, fir 7.398 ms; and `steals > 0` on the `wf` row at every
  parallel width — mandelbrot 3/6/7, quadrature 858/3,270/3,575, records
  1/5/19, fir 1/7/21 at W=2/4/8. No size constant had to change: the shipped
  sizes reach the chunk counts the specification derives, and nothing moved.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 14999's current affinity list: 0-3  date=2026-09-11T06:29:10Z
run=local  compiler=33ed2c000f7ae2f097bb9c3deea9f8d90267be2a  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form             grain                                                                                                                             median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                      13411.0   1.6 13198.0..13742.7                                      
mandelbrot   2 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                          13461.6   1.4 13278.1..13761.8                                      
mandelbrot   2 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                            13502.5   0.8 13392.4..13812.7                                      
mandelbrot   2 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                         13558.6   0.5 13484.3..14112.3                                      
mandelbrot   2 wf               compiler-chosen                                                                                                                     13816.5   1.4 13558.4..14743.1       1.032 [1.02-1.12]   0/5       3 16 chunks
mandelbrot   2 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    26466.4   0.6 26315.8..26953.6                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no

mandelbrot   4 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       6833.7   1.4 6733.3..6995.4                                        
mandelbrot   4 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             6990.7   1.2 6878.7..7205.9                                        
mandelbrot   4 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           7077.7   0.5 7042.2..7628.6                                        
mandelbrot   4 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          7146.5   1.6 6948.7..7322.4                                        
mandelbrot   4 wf               compiler-chosen                                                                                                                      7154.4   1.1 6868.7..7482.6         1.053 [1.02-1.07]   0/5       6 16 chunks
mandelbrot   4 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    26543.2   0.2 26284.7..26593.7                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no

mandelbrot   8 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       6895.3   0.7 6807.2..7355.2                                        
mandelbrot   8 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          7039.8   1.5 6933.9..9489.8                                        
mandelbrot   8 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             7151.7   1.6 6940.5..7347.6                                        
mandelbrot   8 wf               compiler-chosen                                                                                                                      7404.7   3.7 6956.9..7683.9         1.046 [1.01-1.09]   0/5       7 16 chunks
mandelbrot   8 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           7529.2   2.6 7155.1..8117.9                                        
mandelbrot   8 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    31368.4   0.6 29598.9..31568.9                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)

mandelbrot   1 serial           none: one thread, a loop over all callbacks                                                                                         26513.1   0.2 26473.3..27561.0                                      
mandelbrot   1 wf-seq           control                                                                                                                             26674.2   0.6 26502.9..26957.6                                      
mandelbrot   1 wf               compiler-chosen                                                                                                                     27400.4   1.6 26688.6..27832.7                                      

quadrature   2 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          8639.9   2.6 8243.7..9926.4                                        
quadrature   2 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew     8687.1   1.0 8545.5..9397.4                                        excursions retained
quadrature   2 rayon-join-left  rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork                                                           9148.2   4.8 8625.3..9642.5                                        
quadrature   2 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             9764.8   2.5 9391.4..10072.9                                       
quadrature   2 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                      10348.4   1.1 9988.5..10470.9                                       
quadrature   2 parlay-left      ParlayLib native scheduler, parallel_for granularity 1, left-offer fork                                                             12440.2   3.8 9520.9..14291.7                                       
quadrature   2 wf               compiler-chosen                                                                                                                     45543.1   1.4 44098.2..46198.1       5.243 [5.11-5.60]   0/5     858 
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no

quadrature   4 rayon-join-left  rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork                                                           5032.0   3.4 4596.9..5665.1                                        
quadrature   4 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          5104.1   2.3 4911.6..5338.9                                        
quadrature   4 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew     5543.4   2.6 5104.2..6039.6                                        excursions retained
quadrature   4 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       6188.5   1.5 6097.7..7098.6                                        
quadrature   4 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             7622.7   7.3 6835.7..8273.2                                        
quadrature   4 parlay-left      ParlayLib native scheduler, parallel_for granularity 1, left-offer fork                                                             10672.1   9.4 9294.8..11672.6                                       
quadrature   4 wf               compiler-chosen                                                                                                                     24327.9   1.2 23155.2..24630.3       4.962 [4.68-5.04]   0/5    3270 
quadrature   4 BEST REFERENCE = rayon-join-left FASTEST = rayon-join-left WF fastest: no

quadrature   8 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          5248.5   6.3 4917.9..6626.6                                        
quadrature   8 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             5569.8  11.0 4691.1..7085.1                                        
quadrature   8 rayon-join-left  rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork                                                           5755.9  11.5 5091.8..6913.4                                        
quadrature   8 parlay-left      ParlayLib native scheduler, parallel_for granularity 1, left-offer fork                                                              6124.8   4.2 5496.3..7981.9                                        
quadrature   8 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       6366.7   1.3 6284.6..6481.9                                        
quadrature   8 wf               compiler-chosen                                                                                                                     26588.9   7.9 24501.3..29103.8       5.098 [4.95-5.72]   0/5    3575 
quadrature   8 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew   519971.6   1.5 511990.9..527986.8                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join-left FASTEST = rayon-join   WF fastest: n/a (oversubscribed)

quadrature   1 serial           none: one thread, a loop over all callbacks                                                                                         15602.0   0.7 15271.2..16031.7                                      
quadrature   1 wf-seq           control                                                                                                                             16398.7   2.3 15877.8..17043.8                                      
quadrature   1 wf               compiler-chosen                                                                                                                     22170.1   2.3 21665.9..25137.6                                      

records      2 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    16167.8   0.4 15852.1..16375.2                                      excursions retained
records      2 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                         16522.4   2.3 16146.4..18562.1                                      
records      2 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                          16710.4   3.4 16149.0..19446.5                                      
records      2 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                            16877.1   0.6 16524.2..17751.9                                      
records      2 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                      17130.0   5.7 16153.0..24810.0                                      
records      2 wf               compiler-chosen                                                                                                                     17969.0   0.8 17791.2..24028.5       1.113 [1.10-1.49]   0/5       1 32 chunks
records      2 BEST REFERENCE = static       FASTEST = static       WF fastest: no

records      4 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             8248.0   0.5 8137.9..8522.8                                        
records      4 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       8292.8   0.7 8213.1..9122.8                                        
records      4 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           8507.5   2.5 8292.6..9314.4                                        
records      4 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          8655.4   1.5 8405.6..8927.3                                        
records      4 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew     9049.2   1.8 8199.9..9384.4                                        excursions retained
records      4 wf               compiler-chosen                                                                                                                      9508.3   1.3 9357.4..9884.3         1.152 [1.14-1.20]   0/5       5 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no

records      8 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             8561.8   1.3 8391.3..10723.4                                       
records      8 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       8640.4   1.8 8470.5..9162.9                                        
records      8 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           8656.5   2.2 8464.3..9673.7                                        
records      8 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          8710.7   2.9 8461.9..9702.9                                        
records      8 wf               compiler-chosen                                                                                                                      9911.9   0.4 9714.3..10058.0        1.169 [1.14-1.20]   0/5      19 64 chunks
records      8 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    16225.1   3.6 12287.3..20164.6                                      excursions retained
records      8 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: n/a (oversubscribed)

records      1 serial           none: one thread, a loop over all callbacks                                                                                         32433.8   1.5 31925.6..33284.6                                      
records      1 wf               compiler-chosen                                                                                                                     35973.0   0.8 35368.5..36615.2                                      
records      1 wf-seq           control                                                                                                                             36113.0   2.1 35370.8..38773.6                                      

fir          2 wf               compiler-chosen                                                                                                                     14189.4   3.0 13739.1..15151.3       0.923 [0.85-0.98]   5/5       1 32 chunks
fir          2 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                            15845.1   2.2 15194.4..16791.1                                      
fir          2 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                         15923.4   1.9 15397.0..16925.7                                      
fir          2 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                          16263.2   1.3 16048.1..16699.6                                      
fir          2 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                      16387.6   1.4 15843.3..16609.5                                      
fir          2 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    16697.6   2.4 15919.2..17345.7                                      excursions retained
fir          2 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes

fir          4 wf               compiler-chosen                                                                                                                      7398.1   3.0 7169.2..7824.6         0.938 [0.90-0.97]   5/5       7 64 chunks
fir          4 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          7959.5   0.9 7813.9..8099.9                                        
fir          4 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             8251.8   2.8 8022.1..9754.8                                        
fir          4 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           8282.1   1.9 8085.0..8770.7                                        
fir          4 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       8498.8   5.2 7971.1..9932.9                                        
fir          4 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew     8612.6   1.9 8451.6..10393.3                                       excursions retained
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes

fir          8 wf               compiler-chosen                                                                                                                      7647.3   0.5 7455.1..8092.9         0.942 [0.90-1.02]   3/5      21 64 chunks
fir          8 parlay           ParlayLib native scheduler, parallel_for granularity 1, right-offer fork                                                             8285.3   5.0 7875.2..9380.2                                        
fir          8 rayon-iter       rayon 1.12.0 parallel iterator, its own adaptive splitting                                                                           8313.3   1.6 7956.7..8446.6                                        
fir          8 rayon-join       rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork                                                          8319.9   2.4 7972.9..8753.6                                        
fir          8 tbb              oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1                                                                       8515.0  10.8 7592.9..9794.9                                        
fir          8 static           equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew    12302.9   1.5 12123.7..12669.9                                      excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)

fir          1 wf               compiler-chosen                                                                                                                     26664.9   1.4 26289.1..30263.6                                      
fir          1 wf-seq           control                                                                                                                             26803.3   0.5 26290.7..27263.0                                      
fir          1 serial           none: one thread, a loop over all callbacks                                                                                         31621.0   2.4 30259.4..32405.5                                      

```

What it says: at the recorded W=4 the compiled Whitefoot program is the fastest
form for FIR and is not the fastest for the other three. FIR's `wf` row is
0.938 of the best parallel reference with all five paired passes lower;
mandelbrot's is 1.053 with none of five lower, records' is 1.152 with none of
five lower, and quadrature's is 4.962 with none of five lower. The W=8 blocks
on this four-CPU host are oversubscribed and carry no verdict. Three rows need
their column read rather than their number: the `static` row is a regular-work
reference with no stealing, so its skewed-shape medians are three to four times
the dynamic forms' and, at W=8 on quadrature, 520 ms — that is the policy, not
a broken run, and its excursions are retained; the `wf` chunk counts in `note`
are what plain `--par` emitted, 16 for mandelbrot against the references'
1,536 callbacks at grain 64; and `ratio` is a median of within-pass matched
pairs, so recomputing it from the two printed medians does not reproduce it.

## 2026-09-11 — macos-14 (Darwin arm64, 3 logical CPUs), run 34574271919 at `5dd1eb7b`

The first hosted table, and the first on an Apple host: the `macos-14` leg of
`.github/workflows/compute-bench.yml` on the run CI made for this branch's
head. Three logical CPUs, so the recorded block is W=2, W=4 is emitted as
oversubscribed and W=8 is not emitted. `BENCH_ARCH` is empty on this host,
which is the second reading rule above: this section pools with nothing else
in this file, and it reads against the Linux sections only as a different host
under a different `BENCH_ARCH` and a different clang.

- host: `Darwin sat12-bq151-98237d8e-370a-4461-b970-509db0d8f6f4-8A411ABF2B4E.local 23.6.0 Darwin Kernel Version 23.6.0: Tue Jul 21 21:56:54 PDT 2026; root:xnu-10063.141.1.713.39~1/RELEASE_ARM64_VMAPPLE arm64`
- logical CPUs: 3   inherited mask: unqualified (no `taskset` on this host;
  `cgroup_cpu_max` and `cpuset` both absent and recorded as unqualified)
- recorded block: W=2   oversubscribed blocks emitted: W=4
- compiler revision: `5dd1eb7b78b4c5942c72faa0b57b0ea6bdd3dc6a`
- clang: Apple clang version 15.0.0 (clang-1500.3.9.4)   clang++: the same
  rustc: rustc 1.98.0 (88d9e12ae 2026-08-18), host `aarch64-apple-darwin`,
  LLVM 22.1.8   cargo: cargo 1.98.0 (797e8a9bc 2026-08-05)   cmake: cmake
  version 4.4.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: empty
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2 and W=4; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4;
  fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  `chunks=na`
- workflow run: `34574271919`, job `bench (macos-14)` `103183083848`, artifact
  `compute-bench-macos-14` (`10189050462`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 21.949 ms, quadrature 7.987 ms, records
  16.378 ms, fir 14.189 ms; every `wf` median at the recorded W=2 above 1 ms —
  mandelbrot 11.591 ms, quadrature 6.590 ms, records 7.744 ms, fir 7.453 ms;
  and `steals > 0` on the `wf` row at both parallel widths — mandelbrot 3/5,
  quadrature 701/1,257, records 1/11, fir 1/10 at W=2/W=4. No size constant
  changed.

```text
compute-bench  host=Darwin arm64  cpus=3  mask=unqualified (no taskset on this host)  date=2026-09-11T07:27:40Z
run=34574271919  compiler=5dd1eb7b78b4c5942c72faa0b57b0ea6bdd3dc6a  clang=Apple clang version 15.0.0 (clang-1500.3.9.4)  rustc=rustc 1.98.0 (88d9e12ae 2026-08-18)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread  -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto 
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 wf                  11590.6   6.1 10889.2..12694.5       0.984 [0.86-1.20]   3/5       3 16 chunks
mandelbrot   2 rayon-iter          11777.1   6.4 11024.3..15431.3                                      
mandelbrot   2 tbb                 12246.9   6.5 10585.4..16496.3                                      
mandelbrot   2 rayon-join          12981.0   2.3 12680.7..14779.1                                      
mandelbrot   2 parlay              13771.3   5.7 12812.3..14698.5                                      
mandelbrot   2 static              21684.7   2.1 21228.0..23551.0                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 rayon-iter           7706.0   4.1 7386.2..10197.1                                       
mandelbrot   4 parlay               7918.3   7.7 7310.0..8816.2                                        
mandelbrot   4 rayon-join           8078.2   7.9 7443.1..9737.8                                        
mandelbrot   4 tbb                  8346.7   2.3 8153.2..9701.4                                        
mandelbrot   4 wf                  11220.1  11.1 9344.0..13689.3        1.293 [1.19-1.87]   0/5       5 16 chunks
mandelbrot   4 static              33213.3   6.1 30065.5..35243.6                                      excursions retained
mandelbrot   4 BEST REFERENCE = parlay       FASTEST = rayon-iter   WF fastest: n/a (oversubscribed)
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf-seq              21949.1   6.0 20556.0..24408.1                                      
mandelbrot   1 wf                  22439.9   6.7 20554.0..23948.7                                      
mandelbrot   1 serial              22900.3   1.4 20545.6..25057.9                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

quadrature   2 static               4287.9   3.0 4161.2..6584.3                                        excursions retained
quadrature   2 parlay               4859.5   6.2 4166.7..6433.6                                        
quadrature   2 rayon-join           5205.4  16.0 4223.3..11401.4                                       
quadrature   2 parlay-left          5404.4  12.3 4363.7..6066.4                                        
quadrature   2 rayon-join-left      5544.9  15.6 4463.2..22391.2                                       
quadrature   2 wf                   6589.5   1.6 6485.2..9786.0         1.565 [1.41-2.28]   0/5     701 
quadrature   2 tbb                 11590.5  20.2 9253.3..17425.1                                       
quadrature   2 BEST REFERENCE = static       FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

quadrature   4 parlay               3500.8   2.7 3406.6..5863.5                                        
quadrature   4 parlay-left          4206.2  14.0 3615.8..7945.7                                        
quadrature   4 wf                   4360.2   2.0 4272.9..6276.4         1.254 [1.00-1.79]   1/5    1257 
quadrature   4 rayon-join           4375.8  13.5 3142.1..5239.0                                        
quadrature   4 rayon-join-left      5604.6  13.2 3389.5..6343.9                                        
quadrature   4 tbb                  7049.8   7.0 6557.3..10640.8                                       
quadrature   4 static             117222.1   0.5 113498.5..117835.2                                    excursions retained
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 wf-seq               7987.2   3.5 7249.7..8340.0                                        
quadrature   1 serial               8291.2   2.3 7975.1..12156.3                                       
quadrature   1 wf                  13503.7   1.6 12552.5..14020.1                                      
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen

records      2 wf                   7744.3   0.6 7687.2..7850.6         0.866 [0.86-0.89]   5/5       1 32 chunks
records      2 static               9015.5   1.4 8801.3..9152.7                                        excursions retained
records      2 rayon-join           9068.0   0.3 9043.7..9467.5                                        
records      2 parlay               9096.0   1.2 8985.4..12277.2                                       
records      2 tbb                  9108.8   1.3 8956.8..9259.8                                        
records      2 rayon-iter           9111.5   1.6 8962.2..9899.1                                        
records      2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

records      4 wf                   5637.2   2.2 5510.8..8432.7         0.890 [0.86-1.32]   4/5      11 64 chunks
records      4 rayon-join           6690.5   5.3 6317.1..37036.8                                       
records      4 tbb                  6702.0   4.9 6243.4..9010.0                                        
records      4 parlay               6715.8   7.9 6183.6..8050.9                                        
records      4 rayon-iter           8359.0  11.3 6509.7..10065.8                                       
records      4 static               8880.1   7.5 7735.5..12272.4                                       excursions retained
records      4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf                  14754.9   1.4 14555.6..16626.9                                      
records      1 wf-seq              16377.5   3.9 14587.5..20219.9                                      
records      1 serial              18658.9   5.2 17532.7..20535.2                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

fir          2 wf                   7452.9   4.1 7147.1..10460.5        0.954 [0.92-1.33]   3/5       1 32 chunks
fir          2 rayon-join           7884.5   0.8 7751.8..10991.2                                       
fir          2 parlay               7884.7   0.4 7751.5..14977.8                                       
fir          2 static               8056.1   3.0 7814.2..10014.9                                       excursions retained
fir          2 tbb                  8503.3   8.6 7770.7..13980.4                                       
fir          2 rayon-iter          22454.2  64.7 7921.0..43104.2                                       
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 parlay               5517.0   2.7 5370.2..7075.0                                        
fir          4 rayon-join           5642.8   1.4 5563.8..8138.3                                        
fir          4 tbb                  5718.7   3.7 5425.3..5991.8                                        
fir          4 wf                   5835.6   8.3 5350.2..20982.8        1.076 [0.96-3.91]   2/5      10 64 chunks
fir          4 static               7775.6  15.3 5888.0..10451.7                                       excursions retained
fir          4 rayon-iter           8904.8  37.3 5583.5..49329.7                                       
fir          4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          1 wf                  14174.7   3.4 13686.4..16281.3                                      
fir          1 wf-seq              14188.7   1.4 13909.8..17649.9                                      
fir          1 serial              16262.3   3.4 15703.8..19116.5                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: at the recorded W=2 three of the four `wf` rows are the fastest
form in their block. Records is 0.866 of the best parallel reference, `static`
on the median, with all five passes lower, a 0.6 percent MAD and a pass
interval of [0.86–0.89]; fir is 0.954 of `rayon-join` with three of five lower
and an interval of [0.92–1.33]; mandelbrot is 0.984 of `rayon-iter` with three
of five lower and an interval of [0.86–1.20]. Quadrature is 1.565 of `static`,
no pass lower, and at W=1 its `wf` row is 13.504 ms against 7.987 ms for
`wf-seq`, while the other three kernels' W=1 `wf` rows are within 2.3 percent
of, or below, their controls — records' is below, 14.755 ms against 16.378 ms.
The oversubscribed W=4 block is informational and keeps the order: records
0.890 with four of five lower, fir 1.076, quadrature 1.254, mandelbrot 1.293.
The spreads are wide on this host — MADs of 6 to 16 percent on most mandelbrot
and quadrature rows, 64.7 percent on `rayon-iter` for fir at W=2 — where the
Linux table below keeps every row of its recorded block under 3 percent.

## 2026-09-11 — ubuntu-24.04 (Linux x86_64, 4 logical CPUs), run 34574271919 at `5dd1eb7b`

The `ubuntu-24.04` leg of the same run: an Azure runner with four logical
CPUs, so the recorded block is W=4 and W=8 is emitted as oversubscribed, the
same widths as the local sections. It is taken under the same `BENCH_ARCH`
and the same clang as those sections, at `5dd1eb7b`, which differs from the
re-emitted local section's `11d1e4a2` in documents only; the eight module
hashes in its manifest equal that section's, so the two sections time the
same bytes on two hosts. It is a different host: another kernel, another
machine, and another `rustc` behind the Rayon references, and nothing below
is pooled with the local tables.

- host: `Linux runnervmlun5p 6.17.0-1022-azure #22-Ubuntu SMP Mon Jul 27 17:24:03 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as `taskset` recorded it;
  `cpuset.cpus.effective` also `0-3`; `cgroup_cpu_max` absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `5dd1eb7b78b4c5942c72faa0b57b0ea6bdd3dc6a`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: the same   rustc:
  rustc 1.98.1 (48a229cea 2026-09-01), host `x86_64-unknown-linux-gnu`, LLVM
  22.1.8   cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)   cmake: cmake version
  3.31.6
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`, the same setting as both local sections
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  `chunks=na`
- workflow run: `34574271919`, job `bench (ubuntu-24.04)` `103183083570`,
  artifact `compute-bench-ubuntu-24.04` (`10189060040`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 18.678 ms, quadrature 10.536 ms, records
  16.484 ms, fir 16.43 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 5.788 ms, quadrature 8.344 ms, records 8.756 ms, fir 6.641 ms;
  and `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/7/7,
  quadrature 738/2,277/2,428, records 1/5/24, fir 1/8/20 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 4495's current affinity list: 0-3  date=2026-09-11T07:28:00Z
run=34574271919  compiler=5dd1eb7b78b4c5942c72faa0b57b0ea6bdd3dc6a  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.98.1 (48a229cea 2026-09-01)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 rayon-iter           9300.2   0.1 9293.8..9456.7                                        
mandelbrot   2 tbb                  9308.1   0.1 9302.4..9420.6                                        
mandelbrot   2 rayon-join           9345.9   0.3 9317.1..9439.0                                        
mandelbrot   2 parlay               9590.4   0.6 9471.8..9645.4                                        
mandelbrot   2 wf                   9996.1   2.1 9450.4..10205.0        1.072 [1.02-1.09]   0/5       3 16 chunks
mandelbrot   2 static              18470.9   0.0 18464.2..18806.4                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  4899.5   0.1 4883.9..4930.9                                        
mandelbrot   4 rayon-join           4914.8   0.1 4908.1..4955.9                                        
mandelbrot   4 rayon-iter           4916.3   0.2 4897.0..4926.5                                        
mandelbrot   4 parlay               5109.1   0.2 5095.2..5131.9                                        
mandelbrot   4 wf                   5788.4   0.5 4987.5..5817.8         1.178 [1.02-1.19]   0/5       7 16 chunks
mandelbrot   4 static              17961.3   0.3 17911.3..18292.6                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 rayon-join           4948.3   0.2 4933.4..4958.9                                        
mandelbrot   8 tbb                  4976.7   0.2 4910.8..4987.0                                        
mandelbrot   8 rayon-iter           5013.9   0.3 4998.0..5185.7                                        
mandelbrot   8 parlay               5115.9   1.1 5059.4..5234.7                                        
mandelbrot   8 wf                   5667.1   8.5 5119.8..7036.1         1.148 [1.03-1.43]   0/5       7 16 chunks
mandelbrot   8 static              21402.4  12.0 17869.2..26848.7                                      excursions retained
mandelbrot   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              18538.5   0.0 18533.2..18735.6                                      
mandelbrot   1 wf                  18663.1   0.0 18655.6..18828.9                                      
mandelbrot   1 wf-seq              18678.1   0.0 18661.7..18685.2                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           5715.3   0.1 5712.1..5728.4                                        
quadrature   2 rayon-join-left      5880.4   0.4 5858.4..5905.8                                        
quadrature   2 static               6400.6   0.1 6396.1..6423.7                                        excursions retained
quadrature   2 tbb                  6522.6   0.1 6513.9..6531.8                                        
quadrature   2 wf                   9579.8   0.2 9556.9..9744.0         1.677 [1.67-1.71]   0/5     738 
quadrature   2 parlay               9879.2   3.2 9567.4..10520.8                                       
quadrature   2 parlay-left         17247.8   1.7 15713.8..18528.1                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5839.5   0.1 5824.1..5847.8                                        
quadrature   4 rayon-join-left      5953.6   0.2 5939.7..5976.7                                        
quadrature   4 tbb                  6332.4   0.1 6300.4..6339.0                                        
quadrature   4 static               6418.3   0.1 6408.7..6445.4                                        excursions retained
quadrature   4 wf                   8344.4   0.0 8341.7..8387.5         1.432 [1.43-1.43]   0/5    2277 
quadrature   4 parlay              10507.0   0.2 10457.5..10629.0                                      
quadrature   4 parlay-left         15200.2   2.3 14655.1..15551.9                                      
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 rayon-join           6213.5   0.8 6047.5..6266.1                                        
quadrature   8 tbb                  6659.8   0.2 6599.9..6672.6                                        
quadrature   8 rayon-join-left      6768.2   0.9 6644.2..6837.0                                        
quadrature   8 wf                   8917.1   0.2 8762.4..8991.7         1.436 [1.41-1.47]   0/5    2428 
quadrature   8 parlay              10112.4   2.0 9911.7..10851.6                                       
quadrature   8 parlay-left         15165.4   1.1 14291.1..15333.6                                      
quadrature   8 static             440978.8   3.9 404979.6..470002.5                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              10521.8   0.0 10520.1..10576.4                                      
quadrature   1 wf-seq              10536.2   0.0 10533.9..10730.7                                      
quadrature   1 wf                  11659.8   0.0 11643.2..11670.1                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                   8481.9   2.3 8285.8..8862.5         0.985 [0.95-1.03]   3/5       1 32 chunks
records      2 tbb                  8739.5   0.2 8629.9..8777.2                                        
records      2 rayon-iter           8746.1   0.6 8689.3..8889.2                                        
records      2 rayon-join           8765.0   0.3 8491.6..8824.5                                        
records      2 static               8821.4   0.5 8618.0..9129.9                                        excursions retained
records      2 parlay               9103.9   1.0 8961.4..9258.0                                        
records      2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

records      4 tbb                  7974.9   1.4 7854.4..8194.9                                        
records      4 rayon-iter           8056.3   1.7 7922.8..8479.4                                        
records      4 rayon-join           8197.6   0.7 7935.1..8256.5                                        
records      4 parlay               8262.9   2.7 8016.2..8620.0                                        
records      4 static               8306.8   1.3 7993.3..8414.8                                        excursions retained
records      4 wf                   8755.8   2.0 8441.9..8934.2         1.113 [1.06-1.13]   0/5       5 64 chunks
records      4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 tbb                  8218.3   1.0 8020.6..8350.8                                        
records      8 rayon-iter           8328.9   2.2 8141.7..8759.3                                        
records      8 parlay               8332.3   1.2 8184.2..8432.1                                        
records      8 rayon-join           8564.8   0.9 8052.2..8639.0                                        
records      8 wf                   9334.5   1.1 8929.8..9728.2         1.153 [1.09-1.21]   0/5      24 64 chunks
records      8 static              16998.7   0.1 16945.5..17021.4                                      excursions retained
records      8 BEST REFERENCE = parlay       FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              16484.4   0.9 16337.5..18825.7                                      
records      1 wf                  16801.9   1.8 16504.8..18867.9                                      
records      1 serial              17280.3   0.3 17221.0..17426.7                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

fir          2 static               8315.0   0.1 8305.7..8342.8                                        excursions retained
fir          2 rayon-iter           8330.0   0.1 8324.2..8357.5                                        
fir          2 tbb                  8332.1   0.2 8303.1..8353.5                                        
fir          2 rayon-join           8341.4   0.1 8315.3..8358.2                                        
fir          2 parlay               8486.8   0.4 8431.3..8534.1                                        
fir          2 wf                  10437.3   0.3 8257.6..10471.1        1.257 [0.99-1.26]   2/5       1 32 chunks
fir          2 BEST REFERENCE = static       FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

fir          4 static               5947.2   0.1 5938.3..5966.0                                        excursions retained
fir          4 tbb                  5957.5   0.0 5952.1..5968.6                                        
fir          4 rayon-iter           5981.0   0.0 5978.7..5998.5                                        
fir          4 rayon-join           5981.5   0.1 5972.0..5995.4                                        
fir          4 parlay               6090.9   0.1 6072.0..6128.1                                        
fir          4 wf                   6640.9   0.1 6379.3..6751.8         1.117 [1.07-1.14]   0/5       8 64 chunks
fir          4 BEST REFERENCE = static       FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

fir          8 tbb                  6017.2   0.2 5995.8..6034.2                                        
fir          8 rayon-join           6025.8   0.1 6017.8..6048.2                                        
fir          8 rayon-iter           6075.9   0.5 6039.6..6126.6                                        
fir          8 parlay               6214.0   0.6 6153.2..6269.6                                        
fir          8 wf                   6912.1   2.4 6535.4..7079.5         1.148 [1.09-1.18]   0/5      20 64 chunks
fir          8 static               9007.3   0.9 8765.1..11557.5                                       excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  16399.3   0.0 16394.5..16412.0                                      
fir          1 wf-seq              16432.5   0.0 16408.5..16463.4                                      
fir          1 serial              16566.4   0.0 16558.5..16575.6                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: at the recorded W=4 no kernel's `wf` row is the fastest form in
its block, and no block has a pass lower: fir is 1.117 of `static`, records
1.113 of oneTBB, mandelbrot 1.178 of oneTBB, quadrature 1.432 of `rayon-join`.
At W=2 records is fastest, 0.985 of oneTBB with three of five passes lower,
and fir's W=2 row is split — a median of 10.437 ms over a p10 of 8.258 ms, two
of five passes lower than `static` at 8.315 ms. At W=1 every `wf` row is
within two percent of its `wf-seq` control except quadrature, 11.660 ms
against 10.536 ms, and fir's W=1 `wf` row, 16.399 ms, is below the C `serial`
loop at 16.566 ms. Read against the re-emitted local section that follows —
the same eight modules on the development host — the references are faster
here on three kernels and the `wf` rows follow them less than fully: fir's
best reference is 5.947 ms here against 8.144 ms there while its `wf` row is
6.641 ms against 7.834 ms, which turns 0.978 and fastest into 1.117;
mandelbrot's oneTBB is 4.900 ms against 6.819 ms with `wf` at 5.788 ms against
6.985 ms, 1.178 against 1.034. Quadrature runs the other way: `rayon-join` is
5.840 ms here against 4.860 ms there while `wf` is 8.344 ms against 7.955 ms,
1.432 against 1.581, and at W=1 `wf` is 11.660 ms against `wf-seq` 10.536 ms
here, 17.206 against 16.305 ms there. Records is 1.113 here and 1.138 there.
Every row of the recorded block carries a MAD under 3 percent.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), merged tree re-emitted at `11d1e4a2`

**The after side, taken twice.** The merged tree — this branch after merging
`main` `395042f4`, which carries PR #33's scalar-leaf default and PR #34 —
was first tabled at `ed7eb0a7`, and that table stood in this file until the
runtime design review read the hashes: `build/quadrature-par.ll` had the same
SHA-256, `b4ee7493…`, in that run's manifest and in the baseline's. The
bundle's module rule named the compiler only as an order-only prerequisite,
so the compiler rebuilt at `ed7eb0a7` never re-emitted the modules the
baseline compiler had written, and the "merged tree" table timed quadrature
on the `33ed2c00` module: three `wf__par_acquire_lane` sites in
`@wf_adaptive`, no scalar leaf pruned. The other three kernels' modules are
byte-identical under both compilers, so their rows sampled the right bytes;
quadrature's did not. That table is out of this file — git keeps it at
`11d1e4a2` — and its quadrature rows are quoted here as what the defect cost:
`wf` 21.710 ms at W=1 against `wf-seq` 16.216 ms, 24.870 ms at W=4, ratio
4.930 against `rayon-join`; and its sentence attributing a 22.170 → 21.710 ms
move to PR #33 compared two runs of one module and described noise. The
Makefile now names `$(WFC)` as a normal prerequisite of every module, and
this section is the table taken with the modules that rule re-emitted: the
ledger reports `scalar leaf limit 16: omitted 2 compute offers` for
`adaptive` and for `integrate`, `@wf_adaptive` carries one acquisition site,
and all eight module hashes equal the ones the `ubuntu-24.04` section
recorded, so that section and this one time the same bytes on two hosts.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded; `cgroup_cpu_max` and
  `cpuset.cpus.effective` both absent and recorded as unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `11d1e4a24708538c20815690dd1f3522d901c87b`; nothing under
  `compiler/` differs from `ed7eb0a7` or from the hosted run's `5dd1eb7b`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: the same   rustc:
  rustc 1.94.1 (e408947bf 2026-03-25), host `x86_64-unknown-linux-gnu`, LLVM
  21.1.8   cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version
  3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`, the same setting as the baseline section
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 26.757 ms, quadrature 16.305 ms, records
  35.934 ms, fir 27.288 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 6.985 ms, quadrature 7.955 ms, records 9.636 ms, fir 7.834 ms;
  and `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/8,
  quadrature 679/2,125/2,259, records 1/5/18, fir 1/11/20 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 7432's current affinity list: 0-3  date=2026-09-11T07:51:50Z
run=local  compiler=11d1e4a24708538c20815690dd1f3522d901c87b  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13371.2   0.3 13326.1..13683.7                                      
mandelbrot   2 rayon-join          13429.5   0.9 13314.1..13985.7                                      
mandelbrot   2 rayon-iter          13454.3   1.1 13182.3..13728.7                                      
mandelbrot   2 parlay              13613.5   0.8 13388.7..13715.7                                      
mandelbrot   2 wf                  13817.5   0.8 13593.2..14561.9       1.033 [1.02-1.08]   0/5       3 16 chunks
mandelbrot   2 static              26483.8   1.2 26162.3..27296.8                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6818.9   1.8 6693.7..6997.5                                        
mandelbrot   4 parlay               6963.3   1.0 6893.4..7765.0                                        
mandelbrot   4 wf                   6984.7   1.9 6825.1..7505.0         1.034 [0.98-1.07]   1/5       6 16 chunks
mandelbrot   4 rayon-join           7191.1   0.8 7111.2..7297.9                                        
mandelbrot   4 rayon-iter           7344.2   0.8 7096.0..7400.5                                        
mandelbrot   4 static              26374.6   0.4 26018.2..26675.7                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6853.5   0.9 6789.2..6979.6                                        
mandelbrot   8 rayon-join           7027.6   0.6 6982.5..7272.9                                        
mandelbrot   8 parlay               7091.3   2.4 6852.7..7263.5                                        
mandelbrot   8 wf                   7283.7   3.5 6986.1..7558.0         1.050 [1.02-1.08]   0/5       8 16 chunks
mandelbrot   8 rayon-iter           7315.4   2.1 6894.2..7542.5                                        
mandelbrot   8 static              31476.4   0.1 31434.5..47504.8                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf-seq              26756.6   0.4 26488.5..26866.4                                      
mandelbrot   1 serial              26790.5   1.4 26246.8..27172.7                                      
mandelbrot   1 wf                  26811.8   1.3 26393.7..27160.1                                      
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen

quadrature   2 rayon-join           8512.3   4.1 8165.0..9096.8                                        
quadrature   2 rayon-join-left      8615.9   2.0 8440.9..8976.0                                        
quadrature   2 static               8993.1   1.0 8650.1..9085.9                                        excursions retained
quadrature   2 parlay              10031.3   2.6 8985.6..10296.4                                       
quadrature   2 tbb                 10107.6   1.7 9921.1..12101.6                                       
quadrature   2 parlay-left         12254.0   5.2 11611.4..15899.6                                      
quadrature   2 wf                  14549.8   2.3 14215.5..14996.6       1.762 [1.64-1.78]   0/5     679 
quadrature   2 BEST REFERENCE = rayon-join-left FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen

quadrature   4 rayon-join           4859.5   1.7 4762.9..5055.9                                        
quadrature   4 rayon-join-left      5115.5   4.6 4850.1..5461.3                                        
quadrature   4 static               5240.3   6.1 4913.9..6556.4                                        excursions retained
quadrature   4 tbb                  5828.8   1.1 5766.6..6094.7                                        
quadrature   4 parlay               7750.7   8.9 7037.8..8611.0                                        
quadrature   4 wf                   7955.3   5.3 7531.9..8873.0         1.581 [1.57-1.80]   0/5    2125 
quadrature   4 parlay-left         10520.5   3.0 10209.2..11689.9                                      
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 parlay               5156.6   5.9 4854.2..5816.6                                        
quadrature   8 rayon-join           5188.7   1.8 4821.6..5281.8                                        
quadrature   8 rayon-join-left      5821.6   6.7 5256.6..8605.9                                        
quadrature   8 parlay-left          5984.2  10.3 5370.1..10312.1                                       
quadrature   8 tbb                  6176.7   1.2 6085.2..6329.7                                        
quadrature   8 wf                   8148.5   0.7 7983.9..8971.3         1.581 [1.56-1.82]   0/5    2259 
quadrature   8 static             519954.0   0.8 511988.7..523986.4                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15462.3   0.8 14474.3..15578.8                                      
quadrature   1 wf-seq              16305.4   0.7 14488.7..16632.7                                      
quadrature   1 wf                  17205.6   0.9 16752.8..21871.5                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 tbb                 16341.5   0.5 16164.4..17709.7                                      
records      2 rayon-iter          16359.4   0.5 16232.9..16674.4                                      
records      2 static              16572.8   2.9 16052.6..17330.7                                      excursions retained
records      2 rayon-join          16656.7   1.1 16472.0..20514.6                                      
records      2 parlay              16805.6   1.6 16204.3..19920.3                                      
records      2 wf                  18073.2   1.0 17901.5..18528.8       1.117 [1.11-1.14]   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      4 rayon-join           8520.4   2.3 8328.4..11069.0                                       
records      4 tbb                  8626.4   2.6 8401.1..9451.1                                        
records      4 parlay               8782.4   4.2 8414.9..10896.5                                       
records      4 rayon-iter           8890.9   1.7 8704.0..9493.4                                        
records      4 wf                   9636.1   2.1 9388.0..9835.2         1.138 [1.02-1.17]   0/5       5 64 chunks
records      4 static               9884.2  16.2 8280.9..16336.5                                       excursions retained
records      4 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 rayon-join           8720.5   0.8 8521.3..8968.4                                        
records      8 rayon-iter           9246.5   4.5 8673.0..9666.1                                        
records      8 parlay               9467.9   1.8 8737.4..10167.8                                       
records      8 tbb                  9497.3   4.3 8427.9..9901.0                                        
records      8 wf                  10197.2   0.5 9946.2..10244.1        1.176 [1.11-1.18]   0/5      18 64 chunks
records      8 static              15847.0   0.2 14958.4..15880.9                                      excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32443.9   0.9 31817.7..34186.3                                      
records      1 wf                  35607.0   0.4 35459.1..36710.0                                      
records      1 wf-seq              35933.7   1.0 35184.0..37875.8                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14188.2   2.5 13830.0..16503.2       0.906 [0.89-1.06]   4/5       1 32 chunks
fir          2 static              15884.2   1.5 15265.0..16823.5                                      excursions retained
fir          2 rayon-join          16274.4   0.5 15706.3..16357.0                                      
fir          2 tbb                 16302.3   2.2 15778.9..16767.8                                      
fir          2 rayon-iter          16405.0   0.5 16131.4..16685.0                                      
fir          2 parlay              16552.3   1.7 15524.1..16992.6                                      
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   7833.7   7.0 7282.6..8714.8         0.978 [0.87-1.06]   3/5      11 64 chunks
fir          4 tbb                  8143.7   2.0 7978.1..8485.6                                        
fir          4 parlay               8335.9   1.7 8191.1..8767.3                                        
fir          4 static               8381.8   2.3 8190.9..8939.1                                        excursions retained
fir          4 rayon-join           8449.6   4.3 8085.8..9740.1                                        
fir          4 rayon-iter           8862.4   6.4 8195.5..10324.6                                       
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   7808.4   2.3 7545.4..7990.7         0.933 [0.92-0.96]   5/5      20 64 chunks
fir          8 parlay               8289.8   2.0 8120.5..8675.0                                        
fir          8 rayon-join           8376.0   2.9 8024.1..8644.3                                        
fir          8 tbb                  8590.9   1.4 8054.8..9026.0                                        
fir          8 rayon-iter           8714.1   1.5 8341.9..9876.0                                        
fir          8 static              12508.0   2.6 12180.6..18072.5                                      excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27288.2   0.9 26464.8..28191.9                                      
fir          1 wf                  27304.4   0.7 26967.0..27963.1                                      
fir          1 serial              31741.8   0.7 30620.8..32086.3                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says: at the recorded W=4 FIR is again the only kernel whose compiled
Whitefoot program is the fastest form in its block, 0.978 of oneTBB with three
of five passes lower and a 7.0 percent MAD on the `wf` row — a narrower margin
than the 0.903 the stale-module table showed for the same bytes. Mandelbrot is
1.034 of oneTBB with one pass lower; records is 1.138 of oneTBB with none.
Quadrature, the one kernel whose module the re-emission changed, is 1.581 of
`rayon-join` with none lower: 7.955 ms at W=4 against 24.870 ms for the
three-offer module, and 17.206 ms at W=1 against 16.305 ms for `wf-seq`, where
the three-offer module was 21.710 ms against 16.216 ms. So PR #33's scalar-leaf
default, measured on this kernel for the first time, takes the W=4 row from
1.53 times its own sequential control to 0.49 times it and the gap to the
fastest reference from 4.9x to 1.6x; the W=1 row is still 5.5 percent over
`wf-seq`, and the W=4 row still 1.6 times a reference that stops forking at
depth 8. Read against the `ubuntu-24.04` section above, which timed the same
eight modules: quadrature 1.581 here against 1.432 there, fir 0.978 against
1.117, mandelbrot 1.034 against 1.178, records 1.138 against 1.113.
