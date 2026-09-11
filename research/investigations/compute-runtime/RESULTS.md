<!-- Serves research/experiments/compute-bench: the durable record of the
     tables that matter. Job logs and artifacts expire, so a table a decision
     rests on is copied here by hand with its run id and the host that
     produced it. Nothing in this file is generated, and nothing here is a
     gate: no number in it passes or fails anything. -->

# Compute scoreboard results

Status: **one table recorded**, from a local four-logical-CPU Linux host, in the
dated section at the end of this file. The bundle's workflow also runs on
`ubuntu-24.04` and `macos-14`; each hosted run that matters is added the same
way, newest last.

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

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs)

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
