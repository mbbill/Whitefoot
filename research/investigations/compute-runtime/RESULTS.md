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

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the recursive-frontier depth sweep

The five sections below are one experiment and are written to be read together:
plain `--par` on this tree, and the same tree's four `--par-recursive-frontier`
depths as an A/B control. They answer the design question of
`research/investigations/compute-runtime` (and of PR #33's opt-in flag) on this
host: **would defaulting the recursive frontier under plain `--par` make the
quadrature row win?**

Read them against the `merged tree re-emitted at 11d1e4a2` section above. The
compiler content is identical to that section's — `git diff 11d1e4a2 c0ff6d4e
-- compiler/` is empty — so the plain section below re-measures the same bytes,
and the difference between the two plain tables is this host's run-to-run
spread and nothing else. That spread is the first thing the sweep shows and it
bounds how finely anything here can be read: quadrature `wf` at W=4 is
7,955.3 us there and 9,089.6 us here, 14 percent apart, on a
`quadrature-par.ll` with the same SHA-256 in both manifests.

The control is the bundle's new `WF_PAR_CONTROL_FLAGS`, appended to the `--par`
emission and empty by default. The four control tables carry a
`WF --par control flags=` line in their header for that reason; the plain table
does not, and only the plain table is a table about the program the compiler
produces.

**Only `quadrature-par.ll` moves.** Every one of the five runs emitted
`mandelbrot-par.ll` `f07c190e…`, `records-par.ll` `9ceebaed…` and
`fir-par.ll` `e0af1ed0…` — the same three hashes the `11d1e4a2` section timed.
That is the synthesized-function exclusion in
`compiler/src/backend/emitter/frontier.rs:73` doing exactly what it claims:
the three map kernels reach the runtime through a synthesized splitter, which
is never given a frontier, so the depth cannot touch them. Their rows still
move between runs, by up to 4 percent of the ratio on byte-identical bytes,
which is the same spread measured a second way.

What the depth does to quadrature, reading the `wf` row of each run:

| depth | W=2 ratio (lower) | W=4 ratio (lower) | W=8 ratio (lower) | W=1 `wf` vs `wf-seq` | W=4 steals | `.ll` bytes | `.o` bytes |
|-------|-------------------|-------------------|-------------------|----------------------|------------|-------------|------------|
| plain (no frontier) | 1.701 (0/5) | 1.763 (0/5) | 1.770 (0/5) | +3.70 % | 1,910 | 21,627 | 6,104 |
| 4  | 1.597 (0/5) | 3.110 (0/5) | 3.041 (0/5) | −0.35 % |   390 |  47,841 | 10,888 |
| 6  | 1.026 (0/5) | 1.221 (0/5) | 1.079 (1/5) | −0.57 % |   754 |  65,299 | 14,368 |
| 8  | 1.103 (0/5) | **1.017 (1/5)** | 1.039 (2/5) | +2.32 % | 1,024 |  82,757 | 17,840 |
| 12 | 1.155 (0/5) | 1.096 (0/5) | 1.131 (1/5) | +1.09 % | 1,596 | 117,685 | 24,792 |

The sizes are the emitted module and its object; the other three kernels' sizes
are identical in all five runs (`mandelbrot-par.ll` 34,678 / `.o` 6,104,
`records-par.ll` 49,700 / `.o` 5,752, `fir-par.ll` 43,238 / `.o` 6,280).

Three things follow, and none of them is "flip the default".

1. **The frontier is worth a great deal and it is not enough.** At depth 8 the
   W=4 quadrature row falls from 9,089.6 to 5,098.0 us and becomes the fastest
   form in its block — the first time a `wf` quadrature row has been that — but
   its ratio is 1.017 with **one** of five paired passes lower, not below 1.000
   with five. The ratio is the median of within-pass matched pairs against the
   *lowest* reference in that pass, so a `wf` median under `rayon-join`'s and a
   ratio over 1.000 are consistent: `static` and `rayon-join-left` take the
   per-pass minimum often enough to keep it there.
2. **The best depth moves with the width.** Depth 6 is best at W=2 (1.026
   against depth 8's 1.103) and depth 8 is best at W=4 and W=8. A compile-time
   constant cannot be right at both, which is trigger (β) of the design brief's
   §3 and points at a width-adaptive mechanism rather than a wider sweep.
3. **Depth 4 is not a smaller version of depth 8; it is a different failure.**
   At W=4 it is 3.110 — worse than no frontier at all — because 16 leaves over
   a Lorentz profile leave one very heavy subtree on the critical path with
   nothing to steal. The W=1 column is the mirror image: depths 4 and 6 put
   `wf` *below* its own `wf-seq` control, which is the accumulator
   tail-recursion the frontier's sequential clone gives back.

Auxiliary, and never a table row: quadrature process CPU (user + system,
median of five alternating pairs, GNU `time` is not installed on this host so
bash's `time` keyword supplied the child's own times). Plain: `wf` at W=4
0.255 s against `wf-seq` at W=1 0.152 s, a factor of **1.68**. Depth 8: 0.178 s
against 0.153 s, a factor of **1.16**. Both are inside the 2.0 the design brief
asks for, and the frontier cuts the parallel build's CPU by 30 percent.

**Nothing in the compiler changed for these tables**, and on this evidence
nothing should: the depth that would be defaulted does not clear "below 1.000
with 5/5 lower pairs" at any width, and the depth that would be chosen is not
the same depth at W=2 and W=4.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), plain `--par` at `c0ff6d4e`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `c0ff6d4e24f76752088d21f59f9c6109a3ca449e`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so the `wf`
  row is the program plain `--par` produces and this is the only one of the
  five that is a table about that program
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `e6753646eef31512…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5
  ms, 60 ms] — mandelbrot 26.636 ms, quadrature 16.406 ms, records 36.172 ms,
  fir 27.255 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 7.071 ms, quadrature 9.090 ms, records 9.704 ms, fir 7.627 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/7/9,
  quadrature 662/1910/2172, records 1/11/19, fir 1/12/20 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 11692's current affinity list: 0-3  date=2026-09-11T08:06:53Z
run=local  compiler=c0ff6d4e24f76752088d21f59f9c6109a3ca449e  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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
mandelbrot   2 tbb                 13435.0   0.7 13246.8..13546.1                                      
mandelbrot   2 rayon-iter          13455.1   1.1 13305.0..13832.4                                      
mandelbrot   2 parlay              13522.1   0.4 13472.6..13683.9                                      
mandelbrot   2 rayon-join          13637.9   1.1 13382.8..14039.3                                      
mandelbrot   2 wf                  13722.5   0.4 13531.3..13772.3       1.020 [1.01-1.04]   0/5       3 16 chunks
mandelbrot   2 static              26297.6   0.1 26184.6..26708.8                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6816.8   1.2 6735.4..6958.8                                        
mandelbrot   4 parlay               7052.2   1.1 6865.8..7481.7                                        
mandelbrot   4 wf                   7070.6   2.0 6928.4..7770.3         1.037 [1.01-1.15]   0/5       7 16 chunks
mandelbrot   4 rayon-join           7205.4   1.0 7134.5..7310.4                                        
mandelbrot   4 rayon-iter           7395.4   2.3 7071.4..8886.7                                        
mandelbrot   4 static              26499.8   0.5 26136.7..30156.2                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6941.0   1.3 6754.8..7161.5                                        
mandelbrot   8 rayon-iter           7307.8   3.2 6983.0..7622.8                                        
mandelbrot   8 rayon-join           7314.1   2.8 7045.7..7745.2                                        
mandelbrot   8 wf                   7362.6   0.3 7304.0..8695.0         1.062 [1.03-1.24]   0/5       9 16 chunks
mandelbrot   8 parlay               7631.3   5.5 7063.4..9164.2                                        
mandelbrot   8 static              31466.9   0.1 30305.4..31514.1                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26567.3   0.5 26425.6..27609.7                                      
mandelbrot   1 wf-seq              26635.8   0.4 26537.1..27288.3                                      
mandelbrot   1 wf                  26876.8   0.5 26645.4..28004.5                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 static               8749.2   3.3 8464.1..9766.6                                        excursions retained
quadrature   2 rayon-join           8880.3   0.9 7940.6..8959.5                                        
quadrature   2 rayon-join-left      9130.9   4.5 8526.7..9672.3                                        
quadrature   2 parlay              10708.5   2.6 9956.1..11637.2                                       
quadrature   2 tbb                 10899.2   4.1 10133.9..11348.6                                      
quadrature   2 parlay-left         13130.5   5.3 12436.1..15519.4                                      
quadrature   2 wf                  14714.8   2.5 14213.9..15095.9       1.701 [1.68-1.79]   0/5     662 
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen

quadrature   4 rayon-join           5190.3   5.5 4832.1..5541.4                                        
quadrature   4 static               5618.3   6.6 5180.7..6252.9                                        excursions retained
quadrature   4 rayon-join-left      5674.0   4.1 4963.1..5931.1                                        
quadrature   4 tbb                  6408.4   6.1 5675.4..6944.8                                        
quadrature   4 parlay               7330.2   1.0 7127.1..8089.4                                        
quadrature   4 wf                   9089.6   4.9 8644.8..11561.9        1.763 [1.61-2.33]   0/5    1910 
quadrature   4 parlay-left         10367.8   9.9 8674.3..11393.2                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 rayon-join           5601.8   6.2 5256.6..6413.2                                        
quadrature   8 parlay               5721.3   4.8 5412.8..6234.9                                        
quadrature   8 tbb                  6748.5   1.3 6217.4..6837.2                                        
quadrature   8 rayon-join-left      6933.0   4.9 6181.0..7333.5                                        
quadrature   8 parlay-left          7402.6  23.4 5424.3..9131.6                                        
quadrature   8 wf                   9531.6   3.1 9035.0..9835.3         1.770 [1.59-1.82]   0/5    2172 
quadrature   8 static             525766.6   1.2 516021.2..535995.9                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15784.1   0.2 15592.0..16815.0                                      
quadrature   1 wf-seq              16406.4   0.7 15851.2..16908.8                                      
quadrature   1 wf                  17012.7   3.7 16211.7..18189.3                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 tbb                 16590.2   2.1 16247.2..18677.4                                      
records      2 rayon-iter          16763.0   1.2 16411.9..17606.1                                      
records      2 static              16792.0   1.1 16451.6..18188.5                                      excursions retained
records      2 parlay              16956.1   1.5 16551.2..17804.5                                      
records      2 rayon-join          17176.2   1.7 16404.3..21407.8                                      
records      2 wf                  19079.3   1.5 18414.3..22714.5       1.156 [1.12-1.40]   0/5       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      4 parlay               8644.7   2.8 8174.4..11344.7                                       
records      4 rayon-join           8910.0   3.9 8566.7..11218.2                                       
records      4 tbb                  8986.0   4.1 8622.1..10622.6                                       
records      4 rayon-iter           9522.2   8.2 8740.6..12263.8                                       
records      4 wf                   9703.7   0.9 9620.4..10006.0        1.147 [1.14-1.19]   0/5      11 64 chunks
records      4 static              11758.1  28.6 8394.6..16401.8                                       excursions retained
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 tbb                  9055.8   5.0 8600.1..11705.6                                       
records      8 rayon-iter           9425.0   0.2 8573.3..9742.3                                        
records      8 rayon-join           9449.6   8.6 8635.8..11534.6                                       
records      8 parlay               9733.0   5.8 8814.1..12293.3                                       
records      8 wf                  10509.9   4.0 10086.8..11167.5       1.217 [1.07-1.27]   0/5      19 64 chunks
records      8 static              19990.8   2.1 15469.0..22528.8                                      excursions retained
records      8 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32969.5   1.2 32051.9..36278.6                                      
records      1 wf-seq              36171.5   1.2 35732.6..39335.7                                      
records      1 wf                  36672.3   1.8 35600.4..40987.5                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  13917.2   1.5 13711.0..16174.3       0.887 [0.86-1.02]   4/5       1 32 chunks
fir          2 parlay              15946.3   2.2 15561.2..16441.3                                      
fir          2 tbb                 16003.5   1.7 15729.2..16931.9                                      
fir          2 rayon-join          16024.3   1.4 15793.3..16361.7                                      
fir          2 rayon-iter          16149.3   1.3 15934.5..16476.1                                      
fir          2 static              16231.6   1.5 15934.9..16470.2                                      excursions retained
fir          2 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7626.7   3.5 7359.1..9110.2         0.957 [0.91-1.10]   4/5      12 64 chunks
fir          4 rayon-join           8339.0   5.6 7835.3..9310.6                                        
fir          4 static               8410.7   1.8 8260.8..8757.5                                        excursions retained
fir          4 tbb                  8518.7   1.0 8047.7..8600.0                                        
fir          4 parlay               8571.8   0.6 8519.0..11236.0                                       
fir          4 rayon-iter           8654.8   0.6 8138.3..13974.4                                       
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   8080.6   4.8 7477.5..9244.6         0.951 [0.92-1.12]   4/5      20 64 chunks
fir          8 tbb                  8333.4   2.2 7806.1..8637.3                                        
fir          8 rayon-iter           8429.5   3.3 8152.7..9212.5                                        
fir          8 parlay               8621.9   4.2 8169.2..9365.7                                        
fir          8 rayon-join           8911.4   1.6 8342.7..11676.1                                       
fir          8 static              12553.8   3.5 12113.0..18987.3                                      excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27235.6   0.8 26702.5..27447.4                                      
fir          1 wf-seq              27254.5   1.2 26922.5..28079.2                                      
fir          1 serial              32397.3   0.3 31441.3..32912.1                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: on this tree's plain `--par`, quadrature at the recorded W=4 is
1.763 of `rayon-join` with none of five paired passes lower, and 9,090 us
against its own 16,406 us sequential control. FIR is again the only kernel
whose compiled Whitefoot program is the fastest form in its block, 0.957 of
oneTBB with four of five passes lower; mandelbrot is 1.037 and records 1.147,
both with none lower. Every `.ll` hash equals the `11d1e4a2` section's, and
`git diff 11d1e4a2 c0ff6d4e -- compiler/` is empty, so the 7,955 to 9,090 us
move on the quadrature `wf` row between the two tables is this host's
run-to-run spread on identical bytes and not a change in the program.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), A/B control `--par-recursive-frontier 8`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `3c35c6ca20319782e5bc4a6f79e462a5b53f9e66`
- `--par` control flags: `--par-recursive-frontier 8`, appended to the
  `--par` emission through the bundle's `WF_PAR_CONTROL_FLAGS`; this is **not**
  the plain `--par` program and the table header says so
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `332c0e75203f0a5a…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5
  ms, 60 ms] — mandelbrot 26.594 ms, quadrature 15.755 ms, records 36.139 ms,
  fir 27.024 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 6.957 ms, quadrature 5.098 ms, records 9.474 ms, fir 7.310 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/5/8,
  quadrature 419/1024/1038, records 1/5/20, fir 1/5/19 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 15338's current affinity list: 0-3  date=2026-09-11T08:09:42Z
run=local  compiler=3c35c6ca20319782e5bc4a6f79e462a5b53f9e66  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF --par control flags=--par-recursive-frontier 8
      (an A/B control appended to the --par emission: this table is NOT the
      plain --par program and must not be recorded as one -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13328.2   0.3 13281.8..14444.2                                      
mandelbrot   2 rayon-iter          13449.0   0.5 13249.9..13917.3                                      
mandelbrot   2 rayon-join          13452.3   0.7 13355.7..13833.8                                      
mandelbrot   2 parlay              13507.5   0.6 13418.7..13913.6                                      
mandelbrot   2 wf                  13744.2   0.5 13646.3..13832.2       1.028 [1.02-1.04]   0/5       3 16 chunks
mandelbrot   2 static              26599.9   1.2 26162.0..27193.6                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6747.3   1.0 6670.0..6823.4                                        
mandelbrot   4 rayon-join           6951.1   0.4 6925.2..7267.5                                        
mandelbrot   4 wf                   6957.0   1.5 6849.7..7172.5         1.020 [1.01-1.07]   0/5       5 16 chunks
mandelbrot   4 parlay               6981.0   0.6 6790.1..7222.6                                        
mandelbrot   4 rayon-iter           7013.3   0.2 6926.1..7714.9                                        
mandelbrot   4 static              26187.1   0.4 25975.3..28028.2                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6812.2   0.9 6750.3..6941.3                                        
mandelbrot   8 rayon-join           7035.3   1.6 6892.0..7179.6                                        
mandelbrot   8 parlay               7114.2   1.2 6905.4..7198.5                                        
mandelbrot   8 rayon-iter           7296.6   1.1 6947.3..7374.5                                        
mandelbrot   8 wf                   7667.8   5.6 7236.4..10422.6        1.126 [1.07-1.53]   0/5       8 16 chunks
mandelbrot   8 static              31534.2   0.0 25359.8..31539.0                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26329.1   0.0 26319.2..26435.4                                      
mandelbrot   1 wf                  26522.4   0.1 26463.2..26551.1                                      
mandelbrot   1 wf-seq              26594.3   0.4 26484.2..26727.3                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8407.4   0.9 8201.4..10615.6                                       
quadrature   2 rayon-join-left      8948.7   2.1 8763.3..11190.6                                       
quadrature   2 wf                   9324.5   1.4 8603.7..10672.0        1.103 [1.02-1.24]   0/5     419 
quadrature   2 static               9365.9   5.9 8634.3..12782.8                                       excursions retained
quadrature   2 tbb                 10153.6   3.2 9829.0..11142.2                                       
quadrature   2 parlay              10528.9   2.6 9154.8..10801.2                                       
quadrature   2 parlay-left         12456.6   3.1 12069.6..15639.7                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   5098.0   3.0 4657.2..5249.4         1.017 [0.93-1.06]   1/5    1024 
quadrature   4 rayon-join           5161.7   6.2 4566.5..5760.1                                        
quadrature   4 rayon-join-left      5336.8   7.8 4634.5..6006.3                                        
quadrature   4 static               6063.2   4.2 5772.4..6472.8                                        excursions retained
quadrature   4 tbb                  6601.7   5.8 5873.7..7068.1                                        
quadrature   4 parlay               7032.1   1.9 6901.4..7870.3                                        
quadrature   4 parlay-left         10296.6   7.1 9567.3..12854.9                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5296.2   1.7 4852.0..6579.5         1.039 [0.97-1.19]   2/5    1038 
quadrature   8 rayon-join           5334.7   2.1 4898.3..5559.9                                        
quadrature   8 parlay               5496.4   0.5 4670.9..6007.3                                        
quadrature   8 tbb                  6414.0   4.7 5958.7..6718.9                                        
quadrature   8 parlay-left          6582.1   7.5 5325.5..7255.3                                        
quadrature   8 rayon-join-left      6978.3   1.5 6184.0..7084.0                                        
quadrature   8 static             516017.7   0.8 511987.9..543985.2                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15505.3   0.1 15441.6..16098.1                                      
quadrature   1 wf-seq              15755.5   3.7 14620.9..16354.0                                      
quadrature   1 wf                  16121.7   1.0 14747.7..16287.3                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 rayon-iter          16371.6   1.1 16192.1..16813.4                                      
records      2 rayon-join          16602.4   0.9 16098.5..17008.4                                      
records      2 parlay              16643.8   3.2 16109.0..18443.3                                      
records      2 static              16698.0   3.4 16121.9..17869.3                                      excursions retained
records      2 tbb                 17174.3   6.2 16117.1..26926.0                                      
records      2 wf                  18224.2   1.4 17971.3..18829.7       1.116 [1.11-1.17]   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen

records      4 parlay               8300.1   3.0 8051.2..9970.7                                        
records      4 tbb                  8432.5   2.3 8240.7..8948.2                                        
records      4 rayon-iter           8492.8   2.6 8275.1..10751.8                                       
records      4 static               8557.9   3.0 8106.5..8937.7                                        excursions retained
records      4 rayon-join           8671.5   2.1 8473.6..9083.3                                        
records      4 wf                   9474.3   2.5 9235.1..10450.4        1.150 [1.09-1.26]   0/5       5 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      8 tbb                  8391.8   0.3 8363.1..9219.8                                        
records      8 parlay               8588.8   3.3 8307.1..9244.9                                        
records      8 rayon-join           8672.8   3.5 8372.7..11126.7                                       
records      8 rayon-iter           9399.4  10.2 8445.2..13006.5                                       
records      8 wf                   9944.1   3.0 9448.8..13333.3        1.157 [1.13-1.61]   0/5      20 64 chunks
records      8 static              14084.7  12.7 12196.3..23453.8                                      excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32724.5   0.6 31836.8..33013.9                                      
records      1 wf                  35431.7   0.7 35200.6..37613.9                                      
records      1 wf-seq              36139.4   2.9 35085.3..37897.0                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  13968.8   1.1 13772.3..14126.3       0.875 [0.87-0.91]   5/5       1 32 chunks
fir          2 parlay              16076.2   2.3 15415.5..16580.4                                      
fir          2 rayon-join          16135.0   2.1 15757.7..18251.6                                      
fir          2 static              16197.8   1.3 15991.1..19230.3                                      excursions retained
fir          2 rayon-iter          16242.0   1.0 15948.4..17107.1                                      
fir          2 tbb                 16356.9   3.0 15748.4..17691.0                                      
fir          2 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

fir          4 wf                   7309.7   2.1 6998.0..7862.7         0.917 [0.88-0.93]   5/5       5 64 chunks
fir          4 tbb                  8141.2   4.6 7766.3..9344.7                                        
fir          4 static               8381.3   2.8 8009.5..8837.1                                        excursions retained
fir          4 rayon-join           8479.5   4.2 7788.4..10190.4                                       
fir          4 parlay               8551.4   7.1 7940.5..16023.8                                       
fir          4 rayon-iter           8560.3   2.3 7996.0..9727.3                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   7969.7   6.3 7292.5..8561.9         0.950 [0.88-1.01]   4/5      19 64 chunks
fir          8 rayon-join           8494.7   1.3 8275.2..9684.0                                        
fir          8 parlay               8774.1   5.4 8161.7..9965.8                                        
fir          8 rayon-iter           8866.2   1.9 8519.0..10123.8                                       
fir          8 tbb                  8941.8   6.0 8409.7..9660.0                                        
fir          8 static              13749.8   7.2 12466.7..16305.8                                      excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27023.6   1.5 26550.0..27712.2                                      
fir          1 wf                  27152.5   2.4 26504.0..28333.4                                      
fir          1 serial              31363.3   0.7 28933.0..31607.1                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says: the frontier at depth 8 takes the quadrature `wf` row from
9,090 to 5,098 us at W=4 and makes it the fastest form in the block for the
first time — and its ratio is still 1.017, with one of five paired passes
lower, because the ratio is taken against the lowest reference in each pass
and `static` and `rayon-join-left` take that minimum in the passes `wf` does
not win. W=2 is 1.103 and W=8 1.039. At W=1 `wf` is 16,122 us against a
15,756 us `wf-seq`, 2.32 percent over, which is inside the 3 percent the
design brief asks of the frontier's sequential clone. The steal median at W=4
falls from 1,910 to 1,024. `mandelbrot-par.ll`, `records-par.ll` and
`fir-par.ll` are byte-identical to the plain run's and to the `11d1e4a2`
section's, which is the synthesized-function exclusion holding: only the one
module the frontier can reach moved.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), A/B control `--par-recursive-frontier 4`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `3c35c6ca20319782e5bc4a6f79e462a5b53f9e66`
- `--par` control flags: `--par-recursive-frontier 4`, appended to the
  `--par` emission through the bundle's `WF_PAR_CONTROL_FLAGS`; this is **not**
  the plain `--par` program and the table header says so
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `ff80e9b09788130e…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5
  ms, 60 ms] — mandelbrot 26.686 ms, quadrature 16.235 ms, records 35.847 ms,
  fir 27.677 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 7.042 ms, quadrature 14.585 ms, records 9.659 ms, fir 7.537 ms;
  and `steals > 0` on the `wf` row at every parallel width — mandelbrot
  3/6/9, quadrature 256/390/420, records 1/11/18, fir 1/12/22 at W=2/4/8. No
  size constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 20388's current affinity list: 0-3  date=2026-09-11T08:15:23Z
run=local  compiler=3c35c6ca20319782e5bc4a6f79e462a5b53f9e66  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF --par control flags=--par-recursive-frontier 4
      (an A/B control appended to the --par emission: this table is NOT the
      plain --par program and must not be recorded as one -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13347.4   0.2 13267.1..13429.0                                      
mandelbrot   2 rayon-iter          13395.9   0.6 13304.7..13608.9                                      
mandelbrot   2 parlay              13457.5   0.2 13412.0..13522.9                                      
mandelbrot   2 rayon-join          13509.8   0.9 13384.8..13683.8                                      
mandelbrot   2 wf                  13662.8   0.1 13526.5..14072.6       1.026 [1.02-1.05]   0/5       3 16 chunks
mandelbrot   2 static              26414.7   0.3 26202.4..26496.8                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6797.2   0.9 6683.7..7010.1                                        
mandelbrot   4 wf                   7042.0   1.8 6868.2..8532.6         1.028 [1.03-1.26]   0/5       6 16 chunks
mandelbrot   4 rayon-iter           7092.0   1.0 7017.8..7763.2                                        
mandelbrot   4 rayon-join           7099.8   1.2 6970.0..7213.5                                        
mandelbrot   4 parlay               7289.4   2.8 6888.9..9548.5                                        
mandelbrot   4 static              26157.1   0.6 25992.4..26423.9                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6898.6   0.3 6878.8..7013.6                                        
mandelbrot   8 parlay               7094.4   1.6 6977.7..7230.9                                        
mandelbrot   8 rayon-join           7313.4   4.7 6886.2..9893.8                                        
mandelbrot   8 rayon-iter           7474.0   2.5 7257.7..8158.7                                        
mandelbrot   8 wf                   7661.0   4.5 7138.6..8988.0         1.100 [1.04-1.28]   0/5       9 16 chunks
mandelbrot   8 static              31489.9   0.4 30493.2..34262.9                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26504.0   0.8 26252.3..28227.5                                      
mandelbrot   1 wf-seq              26686.5   0.3 26538.8..26917.8                                      
mandelbrot   1 wf                  26708.3   0.7 26460.7..27342.5                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8600.7   2.6 8375.4..8908.6                                        
quadrature   2 static               8962.6   3.2 8673.5..9749.2                                        excursions retained
quadrature   2 rayon-join-left      9020.8   1.2 8826.4..9615.0                                        
quadrature   2 tbb                 10262.8   1.0 10145.8..10420.2                                      
quadrature   2 parlay              10704.1   1.7 10426.3..11238.0                                      
quadrature   2 parlay-left         12495.4   9.9 10783.8..15261.8                                      
quadrature   2 wf                  13852.3   0.9 13365.2..14241.2       1.597 [1.51-1.70]   0/5     256 
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen

quadrature   4 rayon-join           4770.9   2.3 4662.4..4954.9                                        
quadrature   4 rayon-join-left      5204.1   3.3 5000.6..8484.6                                        
quadrature   4 static               5515.1   6.3 5167.2..6177.8                                        excursions retained
quadrature   4 tbb                  6196.0   4.1 5942.7..6660.6                                        
quadrature   4 parlay               7579.0   4.5 6825.9..8342.2                                        
quadrature   4 parlay-left         10001.5   4.5 9303.3..10453.7                                       
quadrature   4 wf                  14584.5   6.0 13655.3..15559.3       3.110 [2.76-3.14]   0/5     390 
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen

quadrature   8 rayon-join           5459.1   5.5 4842.5..6055.9                                        
quadrature   8 parlay               5918.9   3.6 5213.0..6131.0                                        
quadrature   8 tbb                  6045.8   0.7 5988.4..7755.5                                        
quadrature   8 parlay-left          6251.3  11.8 5413.9..12905.5                                       
quadrature   8 rayon-join-left      6329.5   1.0 6264.9..7868.7                                        
quadrature   8 wf                  15985.7   2.5 14947.7..16975.2       3.041 [2.66-3.38]   0/5     420 
quadrature   8 static             520014.7   0.8 511977.4..543979.2                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15517.2   0.3 14556.7..16506.8                                      
quadrature   1 wf                  16177.0   0.6 16082.4..19735.4                                      
quadrature   1 wf-seq              16234.5   0.6 16017.8..17207.7                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 parlay              16301.2   0.3 16256.6..17054.5                                      
records      2 tbb                 16416.2   0.5 16282.3..16601.6                                      
records      2 static              16482.4   1.6 16224.4..18192.7                                      excursions retained
records      2 rayon-join          16551.9   1.9 16232.0..17235.7                                      
records      2 rayon-iter          16738.2   1.4 16503.5..17960.2                                      
records      2 wf                  18008.9   1.7 17701.6..20438.3       1.108 [1.09-1.26]   0/5       1 32 chunks
records      2 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

records      4 tbb                  8527.3   2.0 8306.2..11606.2                                       
records      4 rayon-join           8592.7   4.2 8231.6..11369.7                                       
records      4 parlay               8652.7   2.8 8188.2..11738.5                                       
records      4 wf                   9659.4   3.2 9351.1..10279.7        1.161 [1.10-1.26]   0/5      11 64 chunks
records      4 static              10152.5  19.5 8056.8..15864.6                                       excursions retained
records      4 rayon-iter          11632.1  11.5 8590.8..13983.5                                       
records      4 BEST REFERENCE = parlay       FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

records      8 tbb                  8571.1   1.3 8258.0..8850.1                                        
records      8 rayon-join           8607.1   3.4 8312.2..8992.9                                        
records      8 parlay               8699.3   1.0 8531.5..9122.5                                        
records      8 rayon-iter           8762.1   0.4 8728.5..8878.0                                        
records      8 wf                  10920.3   5.3 10345.1..12968.9       1.274 [1.21-1.57]   0/5      18 64 chunks
records      8 static              12226.7   2.0 11986.3..15758.7                                      excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32770.5   1.9 32017.6..33888.5                                      
records      1 wf-seq              35847.1   0.3 35190.9..35951.0                                      
records      1 wf                  35928.7   0.8 35633.6..36820.3                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14308.9   2.9 13864.7..15654.6       0.918 [0.89-0.98]   5/5       1 32 chunks
fir          2 rayon-join          15716.2   1.3 15511.2..16720.8                                      
fir          2 parlay              15996.1   2.1 15582.9..16869.3                                      
fir          2 rayon-iter          16436.4   0.9 16289.8..16876.4                                      
fir          2 tbb                 16548.3   0.5 15654.4..18119.5                                      
fir          2 static              16677.8   1.1 16313.7..17187.9                                      excursions retained
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7537.1   3.2 7297.2..10645.3        0.954 [0.91-1.26]   3/5      12 64 chunks
fir          4 rayon-join           8424.8   1.1 8054.9..8928.9                                        
fir          4 rayon-iter           8426.3   3.1 8045.5..8890.4                                        
fir          4 tbb                  8504.5   3.3 7898.0..9356.5                                        
fir          4 parlay               8666.8   5.3 8203.5..10013.0                                       
fir          4 static               9989.0  12.1 8444.6..11201.5                                       excursions retained
fir          4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7676.0   0.4 7643.5..8249.8         0.923 [0.89-0.98]   5/5      22 64 chunks
fir          8 parlay               8582.8   4.2 8223.4..10218.0                                       
fir          8 rayon-join           8627.6   3.4 8265.3..9012.0                                        
fir          8 rayon-iter           8908.6   4.2 8408.7..9339.0                                        
fir          8 tbb                  9065.2   5.8 8284.1..9950.3                                        
fir          8 static              14483.2  13.4 12542.9..21524.9                                      excursions retained
fir          8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27228.5   2.8 26260.1..28228.7                                      
fir          1 wf-seq              27677.4   1.5 26955.7..28088.6                                      
fir          1 serial              32829.1   4.0 30609.3..34495.8                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: depth 4 is the sweep's failure case and it fails upward, not
downward: at W=4 the quadrature `wf` row is 14,585 us and 3.110 of
`rayon-join`, worse than the no-frontier 9,090 us in the plain section above,
with a steal median of 390 against that run's 1,910. Sixteen leaves over a
Lorentz profile put one very heavy subtree on the critical path and leave
nothing for the other three lanes to take. The same cut is what makes its W=1
column the best in the sweep — 16,177 us against a 16,235 us `wf-seq`, 0.35
percent *below* its own sequential control, which is the accumulator tail
recursion the sequential clone restores below the cut. Depth is therefore not
a single-signed knob: it trades W=1 cost against W=4 parallelism, and 4 is on
the wrong side of that trade.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), A/B control `--par-recursive-frontier 6`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `3c35c6ca20319782e5bc4a6f79e462a5b53f9e66`
- `--par` control flags: `--par-recursive-frontier 6`, appended to the
  `--par` emission through the bundle's `WF_PAR_CONTROL_FLAGS`; this is **not**
  the plain `--par` program and the table header says so
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `e13b47cf4a5c718f…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5
  ms, 60 ms] — mandelbrot 26.815 ms, quadrature 16.469 ms, records 35.913 ms,
  fir 27.530 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 6.875 ms, quadrature 6.030 ms, records 9.885 ms, fir 7.294 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/8,
  quadrature 376/754/750, records 1/13/21, fir 1/5/21 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 23934's current affinity list: 0-3  date=2026-09-11T08:17:17Z
run=local  compiler=3c35c6ca20319782e5bc4a6f79e462a5b53f9e66  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF --par control flags=--par-recursive-frontier 6
      (an A/B control appended to the --par emission: this table is NOT the
      plain --par program and must not be recorded as one -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 rayon-iter          13298.0   0.4 13248.3..13523.9                                      
mandelbrot   2 tbb                 13355.7   1.4 13123.2..13597.3                                      
mandelbrot   2 rayon-join          13424.0   0.7 13333.3..13940.8                                      
mandelbrot   2 parlay              13583.7   0.9 13335.2..13826.2                                      
mandelbrot   2 wf                  13708.0   1.6 13485.6..14146.1       1.033 [1.01-1.07]   0/5       3 16 chunks
mandelbrot   2 static              26350.0   0.3 26108.9..26449.5                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 wf                   6875.0   0.2 6855.2..7130.2         0.990 [0.98-1.02]   3/5       6 16 chunks
mandelbrot   4 tbb                  6958.1   2.9 6688.5..8039.7                                        
mandelbrot   4 parlay               7105.4   2.0 6960.5..7472.4                                        
mandelbrot   4 rayon-join           7210.6   3.7 6944.9..8445.8                                        
mandelbrot   4 rayon-iter           7214.1   2.4 7038.9..8648.4                                        
mandelbrot   4 static              26415.3   0.8 26129.2..26806.2                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6842.6   0.1 6824.7..7074.5                                        
mandelbrot   8 parlay               7130.2   2.1 6983.5..7756.8                                        
mandelbrot   8 rayon-join           7255.2   1.6 6892.1..7455.1                                        
mandelbrot   8 wf                   7305.5   3.3 7065.5..8968.5         1.060 [1.03-1.31]   0/5       8 16 chunks
mandelbrot   8 rayon-iter           7333.8   1.4 7129.0..8126.8                                        
mandelbrot   8 static              31452.5   0.3 30260.9..31535.2                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26452.8   0.5 26312.3..27069.1                                      
mandelbrot   1 wf-seq              26815.4   0.9 26538.5..27365.5                                      
mandelbrot   1 wf                  27507.0   0.3 26628.0..27584.6                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8475.0   2.9 8229.6..9561.8                                        
quadrature   2 rayon-join-left      8890.6   2.7 8651.0..9857.9                                        
quadrature   2 static               8917.6   9.5 8001.5..10242.2                                       excursions retained
quadrature   2 wf                   9062.2   0.7 8443.5..9216.0         1.026 [1.00-1.14]   0/5     376 
quadrature   2 parlay              10229.3   2.3 9989.5..10695.9                                       
quadrature   2 tbb                 10259.9   1.1 10151.2..10630.2                                      
quadrature   2 parlay-left         12657.9   4.7 12059.9..14915.3                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5052.4   3.7 4792.2..5238.5                                        
quadrature   4 rayon-join-left      5388.1   6.0 5066.3..5896.1                                        
quadrature   4 static               5778.7   4.7 5496.7..8627.1                                        excursions retained
quadrature   4 wf                   6030.1   3.2 5835.0..7178.5         1.221 [1.15-1.41]   0/5     754 
quadrature   4 tbb                  6747.0  10.1 6065.7..8993.0                                        
quadrature   4 parlay               7771.2   5.1 7018.7..8167.4                                        
quadrature   4 parlay-left          9670.9   0.9 9586.6..13462.9                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 rayon-join           5681.8   8.5 5092.6..6262.3                                        
quadrature   8 wf                   5962.3   6.4 5582.7..7232.6         1.079 [0.95-1.39]   1/5     750 
quadrature   8 parlay               6015.1  12.5 5261.3..8546.4                                        
quadrature   8 parlay-left          6646.8   1.6 6120.6..7046.5                                        
quadrature   8 rayon-join-left      6794.1  11.1 5579.0..7548.6                                        
quadrature   8 tbb                  6889.0   1.2 6072.6..6969.2                                        
quadrature   8 static             543986.0   2.2 527978.6..587991.4                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15507.5   0.5 15422.6..15858.8                                      
quadrature   1 wf                  16374.9   1.4 14943.5..16824.5                                      
quadrature   1 wf-seq              16469.1   0.2 16172.8..17734.1                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 tbb                 16432.4   0.3 16024.1..16605.0                                      
records      2 static              16540.5   2.6 16112.0..18914.4                                      excursions retained
records      2 rayon-iter          16846.5   2.7 16280.9..17520.3                                      
records      2 parlay              16951.0   2.2 16168.4..17345.5                                      
records      2 rayon-join          17484.9   3.0 16138.0..20470.0                                      
records      2 wf                  18650.1   2.2 18240.6..20504.7       1.164 [1.11-1.24]   0/5       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      4 parlay               8629.2   3.5 8171.2..9518.4                                        
records      4 rayon-join           8789.1   2.1 8606.6..10734.2                                       
records      4 tbb                  8832.7   4.6 8276.6..11348.1                                       
records      4 rayon-iter           8892.2   4.6 8449.4..9933.6                                        
records      4 static               9461.6   4.5 8195.4..10282.3                                       excursions retained
records      4 wf                   9884.6   4.4 9388.8..10808.6        1.170 [1.10-1.21]   0/5      13 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 rayon-join           8790.5   0.8 8724.0..9003.6                                        
records      8 parlay               8965.1   2.6 8576.2..9213.7                                        
records      8 tbb                  9453.0   3.8 8565.4..12544.6                                       
records      8 rayon-iter           9530.2   5.7 8844.5..11354.9                                       
records      8 wf                  10828.0   5.2 10061.9..11394.8       1.241 [1.15-1.33]   0/5      21 64 chunks
records      8 static              19853.8   4.0 16596.4..32094.7                                      excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32531.2   1.4 31731.3..33315.6                                      
records      1 wf-seq              35913.1   0.7 35526.1..36400.0                                      
records      1 wf                  35998.6   1.3 35235.8..37114.4                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14125.7   1.5 13549.1..14943.3       0.894 [0.84-0.93]   5/5       1 32 chunks
fir          2 tbb                 16238.9   1.3 15393.7..17969.2                                      
fir          2 static              16336.4   1.5 16042.2..19468.7                                      excursions retained
fir          2 rayon-iter          16341.2   0.6 15844.1..16685.8                                      
fir          2 parlay              16411.3   1.8 15879.5..16877.0                                      
fir          2 rayon-join          16805.1   5.1 15860.1..18477.3                                      
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

fir          4 wf                   7293.9   1.5 7171.0..8124.8         0.932 [0.89-0.99]   5/5       5 64 chunks
fir          4 rayon-iter           8193.0   0.7 8133.7..8999.0                                        
fir          4 rayon-join           8201.3   3.0 7844.0..8622.9                                        
fir          4 tbb                  8270.2   0.5 7693.9..8707.6                                        
fir          4 parlay               8783.5   5.1 8260.8..10750.6                                       
fir          4 static               9812.8   9.2 8229.3..10718.8                                       excursions retained
fir          4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7706.5   5.4 7285.1..8504.1         0.976 [0.88-1.03]   4/5      21 64 chunks
fir          8 parlay               8196.4   0.9 7893.3..8296.2                                        
fir          8 tbb                  8360.9   1.9 8203.2..9019.2                                        
fir          8 rayon-join           8372.1   1.2 8231.6..8665.0                                        
fir          8 rayon-iter           8600.9   2.2 8249.4..9399.0                                        
fir          8 static              14484.3  12.1 12448.3..20493.4                                      excursions retained
fir          8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27288.9   1.9 26422.9..28168.1                                      
fir          1 wf-seq              27529.7   1.4 26466.1..27903.5                                      
fir          1 serial              31416.8   1.2 30231.3..32227.1                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: depth 6 is the best of the sweep at W=2 — 1.026 against depth
8's 1.103 — and clearly worse at W=4, 1.221 against 1.017, with none of
five passes lower at either width. Its W=1 row is 16,375 us against a 16,469
us `wf-seq`, 0.57 percent below its own control. Read with the depth-8
section, this is the width sensitivity the design brief named as its trigger
for preferring a runtime budget over a compile-time constant: the depth that
is best at two lanes is not the depth that is best at four.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), A/B control `--par-recursive-frontier 12`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `3c35c6ca20319782e5bc4a6f79e462a5b53f9e66`
- `--par` control flags: `--par-recursive-frontier 12`, appended to the
  `--par` emission through the bundle's `WF_PAR_CONTROL_FLAGS`; this is **not**
  the plain `--par` program and the table header says so
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `bfcbb2b655e8119a…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`, on the development host
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5
  ms, 60 ms] — mandelbrot 26.627 ms, quadrature 16.190 ms, records 35.778 ms,
  fir 27.080 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 6.992 ms, quadrature 5.847 ms, records 9.410 ms, fir 7.300 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/8,
  quadrature 541/1596/1662, records 1/6/21, fir 1/8/20 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 27447's current affinity list: 0-3  date=2026-09-11T08:19:10Z
run=local  compiler=3c35c6ca20319782e5bc4a6f79e462a5b53f9e66  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF --par control flags=--par-recursive-frontier 12
      (an A/B control appended to the --par emission: this table is NOT the
      plain --par program and must not be recorded as one -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13429.3   0.8 13249.1..13624.6                                      
mandelbrot   2 rayon-iter          13439.7   0.6 13323.5..13613.1                                      
mandelbrot   2 rayon-join          13582.5   0.4 13400.1..14684.4                                      
mandelbrot   2 parlay              13628.8   0.5 13470.6..13691.5                                      
mandelbrot   2 wf                  13647.5   0.2 13564.5..13938.9       1.018 [1.01-1.05]   0/5       3 16 chunks
mandelbrot   2 static              26395.1   0.3 26290.1..26823.5                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6763.6   0.9 6701.8..6990.9                                        
mandelbrot   4 wf                   6992.0   0.6 6822.5..7939.3         1.032 [1.00-1.16]   0/5       6 16 chunks
mandelbrot   4 parlay               7082.5   2.0 6941.9..7645.6                                        
mandelbrot   4 rayon-join           7101.2   1.4 7001.5..8011.8                                        
mandelbrot   4 rayon-iter           7179.1   3.7 6913.2..8376.3                                        
mandelbrot   4 static              26341.3   0.9 26085.6..26604.9                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6821.2   1.2 6738.4..9260.5                                        
mandelbrot   8 parlay               7087.7   0.3 7000.3..7108.3                                        
mandelbrot   8 rayon-join           7088.7   0.9 6974.5..7339.6                                        
mandelbrot   8 wf                   7206.4   0.4 7178.2..12698.7        1.056 [1.03-1.87]   0/5       8 16 chunks
mandelbrot   8 rayon-iter           7263.5   2.2 7090.1..7473.9                                        
mandelbrot   8 static              31506.2   0.0 31492.4..37404.7                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26337.3   0.3 26269.1..26699.1                                      
mandelbrot   1 wf-seq              26626.7   0.4 26414.8..26970.9                                      
mandelbrot   1 wf                  26845.1   1.0 26547.8..27446.8                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join-left      8668.4   1.2 8104.8..9040.5                                        
quadrature   2 rayon-join           8803.4   0.9 8137.8..8885.1                                        
quadrature   2 static               8892.3   0.9 8809.0..12378.7                                       excursions retained
quadrature   2 tbb                  9826.6   2.4 9588.8..10294.7                                       
quadrature   2 wf                  10193.2   1.7 9033.6..10610.7        1.155 [1.11-1.27]   0/5     541 
quadrature   2 parlay              10541.6   6.6 9354.9..11232.3                                       
quadrature   2 parlay-left         12121.7  11.0 10739.6..14685.6                                      
quadrature   2 BEST REFERENCE = rayon-join-left FASTEST = rayon-join-left WF fastest: no
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           4823.9   3.7 4643.2..5900.1                                        
quadrature   4 rayon-join-left      4963.2   5.5 4691.5..6547.1                                        
quadrature   4 static               5408.6   5.1 5132.9..6774.0                                        excursions retained
quadrature   4 wf                   5846.8   7.1 5142.8..6530.4         1.096 [1.05-1.35]   0/5    1596 
quadrature   4 tbb                  6570.0   2.6 5822.0..6738.1                                        
quadrature   4 parlay               7157.1   1.5 7051.3..7763.9                                        
quadrature   4 parlay-left         10126.8   5.4 9281.7..11798.6                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 parlay               5404.0   5.4 4947.6..6059.8                                        
quadrature   8 rayon-join           5609.4   4.1 5013.5..5904.6                                        
quadrature   8 wf                   5671.8   7.1 5269.3..6345.5         1.131 [0.97-1.20]   1/5    1662 
quadrature   8 rayon-join-left      5796.7   6.9 5397.4..7125.8                                        
quadrature   8 tbb                  6217.5   2.3 6075.7..7118.6                                        
quadrature   8 parlay-left          7586.8  13.0 6050.0..8592.0                                        
quadrature   8 static             540004.9   2.2 515998.0..555989.6                                    excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15596.2   2.2 14220.8..15967.3                                      
quadrature   1 wf-seq              16189.8   2.0 14571.6..16509.1                                      
quadrature   1 wf                  16366.2   1.1 14609.3..16623.4                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 tbb                 16316.7   1.6 15886.4..16639.0                                      
records      2 static              16377.1   0.5 15836.9..16452.7                                      excursions retained
records      2 parlay              16541.8   1.9 16132.1..17443.7                                      
records      2 rayon-join          16590.4   1.6 16223.9..17165.6                                      
records      2 rayon-iter          16743.5   1.5 16380.6..17015.7                                      
records      2 wf                  18478.2   1.8 18149.0..20219.7       1.146 [1.13-1.27]   0/5       1 32 chunks
records      2 BEST REFERENCE = static       FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

records      4 tbb                  8262.4   1.6 8130.2..11603.4                                       
records      4 rayon-join           8380.2   0.8 8291.4..8700.1                                        
records      4 rayon-iter           8484.3   0.8 8416.2..9657.6                                        
records      4 static               8598.5   4.6 8206.5..16022.2                                       excursions retained
records      4 parlay               8616.3   5.0 8185.6..11962.0                                       
records      4 wf                   9409.7   0.8 9313.9..9739.5         1.142 [1.11-1.15]   0/5       6 64 chunks
records      4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      8 rayon-join           8596.7   3.0 8227.9..9552.4                                        
records      8 tbb                  8639.8   3.3 8277.8..8925.9                                        
records      8 rayon-iter           8859.1   4.2 8487.9..11387.5                                       
records      8 parlay               9311.8  10.9 8296.2..12251.1                                       
records      8 wf                  10115.1   4.1 9570.8..11840.8        1.180 [1.15-1.44]   0/5      21 64 chunks
records      8 static              16014.4   3.8 13167.9..20498.8                                      excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32094.0   0.0 31778.3..32306.8                                      
records      1 wf                  35551.0   0.7 35123.0..36106.4                                      
records      1 wf-seq              35777.5   1.9 35024.8..37309.7                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14067.2   0.3 13921.8..14118.8       0.913 [0.89-0.93]   5/5       1 32 chunks
fir          2 tbb                 15639.3   2.1 15064.5..16595.3                                      
fir          2 rayon-join          15790.6   0.4 15457.1..16198.3                                      
fir          2 parlay              16130.6   1.2 15773.9..16712.2                                      
fir          2 rayon-iter          16193.2   1.6 15852.3..16523.0                                      
fir          2 static              16533.9   1.7 15621.5..20555.8                                      excursions retained
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7300.2   1.7 7060.3..9986.2         0.920 [0.87-1.23]   4/5       8 64 chunks
fir          4 rayon-iter           8209.9   2.0 8030.2..8382.2                                        
fir          4 parlay               8505.1   3.2 8234.9..10847.4                                       
fir          4 static               8572.6   1.2 8176.9..10996.2                                       excursions retained
fir          4 tbb                  8632.4   5.5 8127.5..9706.8                                        
fir          4 rayon-join           8752.9   9.4 7885.8..11353.9                                       
fir          4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

fir          8 wf                   7619.3   1.5 7503.0..8656.0         0.937 [0.93-1.03]   4/5      20 64 chunks
fir          8 parlay               8295.5   2.0 8064.6..8537.3                                        
fir          8 rayon-join           8422.1   3.7 8015.7..8846.8                                        
fir          8 rayon-iter           8435.0   2.2 8250.3..8715.4                                        
fir          8 tbb                  8507.4   1.8 8284.8..8696.5                                        
fir          8 static              12863.0   1.1 12372.0..14214.3                                      excursions retained
fir          8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27051.8   1.3 26623.9..27500.8                                      
fir          1 wf-seq              27080.3   1.4 26324.9..27457.7                                      
fir          1 serial              31658.6   0.9 30059.2..31947.8                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: depth 12 does not recover what depth 8 has: 1.096 at W=4 with
none of five passes lower, against depth 8's 1.017 with one, and 1.155 at W=2
against 1.026 at depth 6. Its steal median at W=4 is 1,596, three quarters of
the way back to the no-frontier 1,910, and its module is 117,685 bytes of
`.ll` and 24,792 of `.o` against the plain 21,627 and 6,104 — five and four
times the code for a row that is worse than depth 8's. The curve over this
host therefore has an interior best near 8 and pays for both sides of it.

