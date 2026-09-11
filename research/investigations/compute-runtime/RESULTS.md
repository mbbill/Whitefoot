<!-- Serves research/experiments/compute-bench: the durable record of the
     tables that matter. Job logs and artifacts expire, so a table a decision
     rests on is copied here by hand with its run id and the host that
     produced it. Nothing in this file is generated, and nothing here is a
     gate: no number in it passes or fails anything. -->

# Compute scoreboard results

Status: **thirty-eight tables recorded**, in the dated sections at the end of this
file: a baseline at compiler `33ed2c00` on the local four-logical-CPU Linux
host, then the first hosted run, `34574271919` at `5dd1eb7b`, one section per
leg of the bundle's workflow — `macos-14` (three CPUs, recorded block W=2) and
`ubuntu-24.04` (four CPUs, recorded block W=4) — then the merged tree re-emitted
at `11d1e4a2` on the local host, which replaced a merged-tree table that had
timed one stale module; then two experiments on the recursive frontier, each
opening with a section that carries no table of its own and is the argument the
tables under it support: the depth sweep (plain and four pinned depths) and the
recursion budget (plain and its runtime-derived control); then the table those
two experiments settled — plain `--par` at `53359d73`, where the
runtime-derived budget is the default and `off` and a pinned `N` are the
controls; then four hosted tables of that default flip, two runs of the
record-only `compute-bench` workflow with both legs each and no control flags
in either: run `34592005664` at `51debad8`, `ubuntu-24.04` then `macos-14`,
where plain `--par` still offered at every node of a recursive component, and
run `34597909514` at `9d0c9485`, the same two legs with the budget defaulted;
and last four sections about the wait path and the grain, each an argument with
its own tables under it — the CPU column added to the bundle at `78867abd`, a
`PASSES=2` pair over byte-identical images that answers an agreement question
and is not a candidate record; one random victim per spin round at `d47223c0`,
a refusal measured in two runs per arm; the spin bound sized to a measured
park-and-wake at the same revision, where `WF_PAR_SPIN_ROUNDS` moves from 4,096
to 1,024; and the split work unit swept against the Mandelbrot grain, a second
refusal, where three control runs of the same bytes spread further than the
effect the sweep was cutting for and the constant does not move. That refusal
is why the section after it is an instrument rather than a table: the bundle
gained an **A/B twin**, a second Whitefoot image timed inside the same passes as
the first, and its own null check over byte-identical images says what a
within-pass pair is worth at two passes on this host. The section after it
carries that twin's first experiment in three tables, the oversubscription cap
raised with the work unit in three pairs: a third refusal, where the twin's own
null arm moved as far as the effect it was built to select and
`WF_PAR_SPLIT_OVERSUBSCRIBE` stays at 16. The last three sections take that
refusal's cause as their subject rather than its symptom, and are read together:
a null arm that is **identical in behaviour and shifted in placement** — a
never-called 587-byte function under `#ifdef WF_PLACEMENT_PAD`, which nothing
that ships defines — moves a block median by up to 11.7 percent and a single
paired reading from 0.68 to 1.30; compiling **both** arms with
`-falign-functions=64 -falign-loops=32` does not bring those lines back, and
moves the median line twice as far from 1.000 as leaving them alone; and the
remedy's own A/B, alignment on the Whitefoot side of the link against the flags
`whitefootc` passes clang, holds every one of its sixteen lines inside
[0.954, 1.010]. **So the compiler driver is unchanged**: the flags cost nothing
and buy less than this host can resolve, and the sensitivity they were meant to
remove is not the kind of thing they reach.
Each run that matters is added the same way, newest last.

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

A run that carries a `wf-b` row is an A/B instrument and is never recorded as a
table of what this tree produces: record it as the arms of an experiment, with
the control flags in the heading and the `A/B  wf-b/wf` lines quoted in the
reading, and say in the host block that no width block of it is a candidate
record. See the compute-bench README, "The A/B twin".

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

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the recursion budget as an opt-in control

The two sections below are one experiment and are written to be read
together: plain `--par` on this tree, and the same tree's
`--par-recursive-frontier auto` as its A/B control. They answer the question
the depth sweep above left open — a compile-time depth is the wrong mechanism
because the best depth moves with the width, so **does a width-adaptive budget
earn the `--par` default?**

The mechanism measured here is the one the compiler now carries behind
`--par-recursive-frontier`: each ordinary cyclic call-graph component gets one
synthesized variant per member with a hidden trailing budget, every
intra-component call and published callback enters the callee's variant with
one level less, and a variant handed nothing left enters its own sequential
clone. `auto` takes the initial budget from the runtime, once per call into the
component — `floor(log2(64 * lanes))`, so 6 with no pool, 7 at two lanes, 8 at
four, 9 at eight — and `N` pins it. Unlike the sweep's N private layers, the
body is emitted once: `quadrature-par.ll` is 21,627 bytes plain, 23,069 under
`auto` and 22,937 pinned at 8, against 82,757 at the sweep's depth 8.

**Only `quadrature-par.ll` moves.** Every run emitted `mandelbrot-par.ll`
`f07c190e…`, `records-par.ll` `9ceebaed…` and `fir-par.ll` `e0af1ed0…` — the
same three hashes every section above this one timed. And the plain module is
`e6753646…` at 21,627 bytes, byte-identical to the plain module of the depth
sweep's own plain section, which is what makes the two experiments comparable
at all.

What the budget does to quadrature, reading the `wf` row of each run. The two
recorded sections are the first two rows; the last three were taken the same
way, on this branch at `d70017ed`, whose `auto` module has the same SHA-256 as
the recorded one, `dc6aeaf3…`, and whose only difference from `39d810c4` is
which of these two the default selects. The three `auto` rows are the same
bytes measured in three runs within an hour, and all three are given because
the difference between them is what this host's spread does to a ratio:

| run | W=2 ratio (lower) | W=4 ratio (lower) | W=8 ratio (lower) | W=1 `wf` vs `wf-seq` | W=4 steals | `.ll` bytes |
|-----|-------------------|-------------------|-------------------|----------------------|------------|-------------|
| plain (budget off) | 1.685 (0/5) | 1.564 (0/5) | 1.665 (0/5) | +5.61 % | 2,069 | 21,627 |
| `auto`, recorded   | **1.031 (0/5)** | **0.987 (3/5)** | **1.000 (2/5)** | −0.30 % | 1,011 | 23,069 |
| `auto`, earlier run | 1.056 (0/5) | 1.008 (2/5) | 0.985 (3/5) | −0.67 % | 1,016 | 23,069 |
| `auto`, third run   | 0.978 (3/5) | 0.966 (4/5) | 1.015 (2/5) | +0.20 % | 1,026 | 23,069 |
| pinned 8           | 1.067 (1/5) | 0.934 (4/5) | 0.966 (4/5) | −0.29 % | 1,021 | 22,937 |

Three things follow, and the third is why the default did not move.

1. **The budget is worth a great deal, at every width.** `auto` takes the
   quadrature `wf` row from 14,534.7 to 8,751.7 us at W=2, from 8,216.5 to
   4,582.5 at W=4 and from 8,753.9 to 4,766.7 at W=8, and makes it the fastest
   form in its block at W=4 and W=8. At W=1 it also removes the cost the
   parallel lowering pays for nothing: +5.61 percent over its own sequential
   control becomes −0.30 percent, which is the accumulator tail-recursion the
   sequential clone gives back below the cut.
2. **One runtime-derived budget matches the best fixed depth without being
   it.** Against the sweep's depth 8 — the best fixed depth there, 1.103 at
   W=2 and 1.017 at W=4 — `auto` reads 1.031 and 0.987 here, and against the
   pinned 8 measured on this same tree it is better at W=2 and within the
   spread at W=4 and W=8. It costs 132 bytes more than the pinned form and
   1,442 more than no family at all, where the sweep's depth 8 cost 61,130.
3. **The acceptance bound for defaulting it was not met.** The bound was: the
   three map modules byte-identical (met), W=1 within 3 percent of `wf-seq`
   (met, −0.30 percent), W=4 ratio at or below 1.02 with at least one of five
   passes lower (met, 0.987 with 3/5), `quadrature-par.ll` at most 40 KB (met,
   23,069), W=4 process CPU at most 2.0 times the W=1 `wf-seq` CPU (met, 1.17),
   every test green (met) — and **W=2 ratio at or below 1.03, which is not
   met**: 1.031 recorded, 1.056 and 0.978 in the other two runs of the same
   bytes. The bound came from the sweep's best W=2 depth of 6 at 1.026; the
   runtime's answer at two lanes is 7, which lands between that and depth 8's
   1.103, exactly as the mechanism predicts. Three readings of one module
   straddling a bound by less than the 0.078 of ratio that separates two of
   them is not a bound that was met, so `--par` still offers at every node of a
   recursive component and the family is written.

Auxiliary, and never a table row: quadrature process CPU (user + system, median
of five alternating pairs, GNU `time` is not installed on this host so bash's
`time` keyword supplied the child's own times). Plain: `wf` at W=4 0.269 s
against `wf-seq` at W=1 0.149 s, a factor of **1.81**. `auto`: 0.179 s against
0.153 s, a factor of **1.17**. Both are inside the 2.0 the design brief asks
for, and the budget cuts the parallel build's CPU by a third.

**What would reopen it**: a host whose quadrature `wf` spread is below this
one's — three runs of one module read 1.031, 1.056 and 0.978 at W=2 here — or
a W=2 reading at or below 1.03 that a second run of the same bytes confirms.
Nothing else about the mechanism is in question: it is cheap, it is
width-adaptive, and it cannot touch a kernel that reaches the runtime through a
synthesized splitter.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), plain `--par` at `39d810c4`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `39d810c4b8ef07d3125ef3daa9cc3abf060ac503`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so the `wf`
  row is the program plain `--par` produces and this is the only one of the
  three that is a table about that program
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
  ms, 60 ms] — mandelbrot 26.714 ms, quadrature 16.175 ms, records 35.398 ms,
  fir 27.309 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 7.125 ms, quadrature 8.216 ms, records 9.409 ms, fir 7.563 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/9,
  quadrature 667/2069/2197, records 1/11/19, fir 1/11/19 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 26589's current affinity list: 0-3  date=2026-09-11T10:33:55Z
run=local  compiler=39d810c4b8ef07d3125ef3daa9cc3abf060ac503  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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
mandelbrot   2 tbb                 13333.7   0.4 13282.4..13782.9                                      
mandelbrot   2 parlay              13431.0   0.0 13426.8..13727.4                                      
mandelbrot   2 rayon-join          13454.9   0.2 13356.5..13647.6                                      
mandelbrot   2 rayon-iter          13474.1   0.6 13290.2..13609.2                                      
mandelbrot   2 wf                  13718.3   0.2 13593.4..13995.0       1.025 [1.02-1.05]   0/5       3 16 chunks
mandelbrot   2 static              26298.5   0.3 26198.3..26488.3                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6772.9   0.5 6740.6..6890.9                                        
mandelbrot   4 parlay               6905.4   0.4 6880.6..7060.8                                        
mandelbrot   4 wf                   7124.5   1.7 6900.2..7246.6         1.048 [1.00-1.07]   0/5       6 16 chunks
mandelbrot   4 rayon-join           7184.7   1.9 7017.0..7660.8                                        
mandelbrot   4 rayon-iter           7265.8   0.2 7144.8..7281.2                                        
mandelbrot   4 static              26546.3   0.5 26112.4..26680.4                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 rayon-join           7022.5   0.6 6981.6..7118.9                                        
mandelbrot   8 parlay               7199.7   0.9 7122.8..7512.2                                        
mandelbrot   8 rayon-iter           7255.3   2.0 7106.9..7525.4                                        
mandelbrot   8 wf                   7342.1   2.7 7145.5..7865.5         1.048 [1.02-1.13]   0/5       9 16 chunks
mandelbrot   8 tbb                  7499.4   7.5 6932.8..9066.6                                        
mandelbrot   8 static              31426.7   0.5 29151.7..39516.4                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26365.5   0.2 26208.1..26459.5                                      
mandelbrot   1 wf                  26632.9   0.4 26494.9..26819.9                                      
mandelbrot   1 wf-seq              26713.7   0.2 26514.7..26755.1                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8511.5   1.6 8377.1..9510.3                                        
quadrature   2 static               8972.2   2.5 8522.4..9431.2                                        excursions retained
quadrature   2 rayon-join-left      9212.2   5.8 8445.5..10791.1                                       
quadrature   2 tbb                 10020.0   5.3 9489.1..10752.4                                       
quadrature   2 parlay              10299.4   1.4 9295.6..10587.2                                       
quadrature   2 parlay-left         14072.9   9.1 12719.3..15795.0                                      
quadrature   2 wf                  14534.7   2.6 14106.9..15113.6       1.685 [1.66-1.71]   0/5     667 
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen

quadrature   4 rayon-join           5153.2   5.2 4702.5..5836.9                                        
quadrature   4 rayon-join-left      5447.7   6.0 4642.2..5784.7                                        
quadrature   4 static               6030.5   5.5 5345.4..6435.3                                        excursions retained
quadrature   4 tbb                  6575.3   2.3 6221.5..6866.4                                        
quadrature   4 parlay               7064.9   1.8 6939.4..7952.6                                        
quadrature   4 wf                   8216.5   2.8 7683.8..8459.4         1.564 [1.55-1.68]   0/5    2069 
quadrature   4 parlay-left         10344.8   1.9 8699.4..11381.0                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 rayon-join           5476.2   6.3 5108.1..6092.8                                        
quadrature   8 parlay               5858.6   9.3 5205.0..6403.3                                        
quadrature   8 parlay-left          6023.4   3.4 5681.0..6540.6                                        
quadrature   8 rayon-join-left      6405.1   8.1 5888.4..7849.1                                        
quadrature   8 tbb                  6948.5   2.6 6527.3..7127.0                                        
quadrature   8 wf                   8753.9   0.7 7857.7..8816.0         1.665 [1.29-1.73]   0/5    2197 
quadrature   8 static             523983.8   0.8 511993.2..543995.4                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15416.4   2.1 14224.4..16418.3                                      
quadrature   1 wf-seq              16175.5   0.7 14408.1..16746.6                                      
quadrature   1 wf                  17082.6   1.0 15405.7..17780.4                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 static              16453.9   2.1 16104.3..17401.4                                      excursions retained
records      2 tbb                 16637.1   0.5 16297.8..16712.4                                      
records      2 rayon-iter          16722.7   2.2 16223.0..22993.7                                      
records      2 rayon-join          16739.0   0.4 16670.5..17196.9                                      
records      2 parlay              16775.7   1.9 16358.6..17102.3                                      
records      2 wf                  18454.9   1.5 18177.2..21311.5       1.129 [1.11-1.31]   0/5       1 32 chunks
records      2 BEST REFERENCE = static       FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      4 parlay               8265.1   1.3 8068.3..8559.6                                        
records      4 rayon-join           8399.1   1.2 8295.7..8604.7                                        
records      4 rayon-iter           8445.3   0.8 8380.9..9016.5                                        
records      4 tbb                  8567.9   1.9 8167.3..8789.2                                        
records      4 static               8947.2   3.1 8670.1..12950.9                                       excursions retained
records      4 wf                   9408.6   1.6 9261.7..10297.2        1.151 [1.14-1.24]   0/5      11 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 rayon-join           8575.5   2.3 8374.3..9887.0                                        
records      8 tbb                  8676.1   2.9 8256.9..8980.1                                        
records      8 parlay               8687.0   1.0 8488.6..9077.6                                        
records      8 rayon-iter           9408.2   0.3 8503.4..9992.9                                        
records      8 wf                   9801.4   0.5 9748.8..10164.2        1.184 [1.14-1.20]   0/5      19 64 chunks
records      8 static              19495.4   2.3 14815.0..19939.3                                      excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32199.8   0.4 31633.6..35565.2                                      
records      1 wf-seq              35398.5   0.1 35268.0..36573.0                                      
records      1 wf                  35912.8   1.4 35223.9..36433.0                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14380.1   2.0 13925.1..16123.1       0.907 [0.89-1.00]   5/5       1 32 chunks
fir          2 tbb                 16078.6   2.3 15580.4..16595.1                                      
fir          2 rayon-iter          16114.7   1.6 15639.4..16855.4                                      
fir          2 rayon-join          16121.9   0.4 15388.7..16183.4                                      
fir          2 parlay              16174.3   3.3 15424.6..18320.8                                      
fir          2 static              16303.9   2.3 15906.5..17231.2                                      excursions retained
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7562.9   6.3 7084.6..11824.5        0.915 [0.87-1.33]   3/5      11 64 chunks
fir          4 rayon-join           8113.6   0.4 8080.9..9590.7                                        
fir          4 rayon-iter           8346.2   2.9 8103.6..9383.4                                        
fir          4 tbb                  8346.7   1.9 8191.2..9180.0                                        
fir          4 parlay               8423.3   2.1 8015.5..9162.4                                        
fir          4 static               8661.2   8.0 7970.3..10486.5                                       excursions retained
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7827.5   3.7 7540.2..8580.8         0.945 [0.92-1.00]   5/5      19 64 chunks
fir          8 rayon-iter           8285.9   0.7 8230.4..8940.0                                        
fir          8 rayon-join           8322.0   3.4 8042.0..10989.3                                       
fir          8 tbb                  8539.7   4.8 8129.0..9441.0                                        
fir          8 parlay               9048.7   5.0 8262.0..10535.5                                       
fir          8 static              12434.7   3.6 11983.8..18225.1                                      excursions retained
fir          8 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  26686.5   0.8 26038.9..27402.0                                      
fir          1 wf-seq              27309.4   1.0 26913.9..27614.9                                      
fir          1 serial              32095.8   1.2 29778.6..32465.7                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

What it says: the quadrature `wf` row is what a recursive component costs
when every node of it offers: 1.685, 1.564 and 1.665 of the best per-pass
reference at W=2, W=4 and W=8, none of five paired passes lower at any width,
and 5.61 percent over its own sequential control at W=1 on a program that
overlapped nothing there. The other three kernels are the modules every
section above timed, byte for byte, and read within their usual spread: fir
0.915, mandelbrot 1.048, records 1.151 at W=4.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), A/B control `--par-recursive-frontier auto`

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `39d810c4b8ef07d3125ef3daa9cc3abf060ac503`
- `--par` control flags: `--par-recursive-frontier auto`: the budget-carrying
  clone family, entered with the runtime's own answer. This is a control table
  and not a table about the program plain `--par` produces
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `dc6aeaf3fda497ba…`
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
  ms, 60 ms] — mandelbrot 26.554 ms, quadrature 16.178 ms, records 35.644 ms,
  fir 27.420 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 7.059 ms, quadrature 4.582 ms, records 9.265 ms, fir 7.286 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/8,
  quadrature 425/1011/1205, records 1/6/18, fir 1/7/21 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 29501's current affinity list: 0-3  date=2026-09-11T10:35:29Z
run=local  compiler=39d810c4b8ef07d3125ef3daa9cc3abf060ac503  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF --par control flags=--par-recursive-frontier auto
      (an A/B control appended to the --par emission: this table is NOT the
      plain --par program and must not be recorded as one -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13400.8   0.3 13276.0..13693.9                                      
mandelbrot   2 rayon-iter          13513.2   0.8 13408.1..13856.3                                      
mandelbrot   2 parlay              13548.0   0.6 13468.8..13814.7                                      
mandelbrot   2 rayon-join          13557.6   0.4 13501.5..13906.5                                      
mandelbrot   2 wf                  13710.1   1.2 13474.3..13910.8       1.023 [1.01-1.04]   0/5       3 16 chunks
mandelbrot   2 static              26418.6   0.1 26205.8..26656.4                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6780.0   0.0 6724.2..6835.8                                        
mandelbrot   4 parlay               6974.8   0.1 6959.1..7105.1                                        
mandelbrot   4 wf                   7059.3   0.0 7057.4..7675.3         1.050 [1.03-1.13]   0/5       6 16 chunks
mandelbrot   4 rayon-join           7139.1   0.7 7075.9..7381.5                                        
mandelbrot   4 rayon-iter           7334.2   3.3 7074.6..7872.3                                        
mandelbrot   4 static              26210.4   0.1 25966.1..26482.2                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  7002.8   2.5 6754.5..7286.5                                        
mandelbrot   8 parlay               7089.0   0.8 6960.5..8414.4                                        
mandelbrot   8 rayon-iter           7194.6   0.8 7047.9..7430.0                                        
mandelbrot   8 rayon-join           7213.3   1.7 6941.5..13217.8                                       
mandelbrot   8 wf                   7405.0   1.3 7017.2..7857.9         1.062 [0.99-1.16]   1/5       8 16 chunks
mandelbrot   8 static              38143.3  17.3 31476.5..47371.6                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26488.7   0.5 26349.3..27073.5                                      
mandelbrot   1 wf-seq              26553.8   0.4 26459.5..26926.5                                      
mandelbrot   1 wf                  26592.5   0.2 26512.4..27257.4                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8405.3   1.7 8180.8..8689.1                                        
quadrature   2 rayon-join-left      8734.6   1.7 8590.4..8987.4                                        
quadrature   2 wf                   8751.7   3.4 8456.5..13554.4        1.031 [1.01-1.60]   0/5     425 
quadrature   2 static               8918.2   1.0 8827.6..12800.5                                       excursions retained
quadrature   2 tbb                 10080.0   0.1 10035.6..11768.9                                      
quadrature   2 parlay              10094.2   0.7 10027.4..10579.1                                      
quadrature   2 parlay-left         12521.9   3.6 9800.9..15619.4                                       
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   4582.5   1.3 4438.2..5129.0         0.987 [0.96-1.12]   3/5    1011 
quadrature   4 rayon-join           4582.8   0.9 4425.9..4769.1                                        
quadrature   4 static               5181.5   1.5 5102.7..5775.1                                        excursions retained
quadrature   4 rayon-join-left      5249.3   7.3 4855.4..5929.6                                        
quadrature   4 tbb                  5796.3   1.6 5706.3..6943.2                                        
quadrature   4 parlay               7391.2   3.8 6503.1..7673.6                                        
quadrature   4 parlay-left          9896.0   3.8 9502.5..10578.3                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   4766.7   2.6 4584.4..5018.4         1.000 [0.87-1.00]   2/5    1205 
quadrature   8 rayon-join           5233.8  12.7 4570.1..6257.4                                        
quadrature   8 parlay               5342.6   8.5 4889.8..8259.3                                        
quadrature   8 parlay-left          5592.4   1.9 5486.7..7643.9                                        
quadrature   8 tbb                  6019.2   0.6 5981.4..6473.6                                        
quadrature   8 rayon-join-left      6380.5  18.5 5063.5..8077.2                                        
quadrature   8 static             515987.6   0.0 515981.7..547973.1                                    excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15360.7   0.8 14340.7..15476.3                                      
quadrature   1 wf                  16129.5   1.6 14440.4..18070.9                                      
quadrature   1 wf-seq              16178.0   1.2 14429.2..16398.3                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 tbb                 16254.9   1.2 15963.7..17018.4                                      
records      2 rayon-iter          16350.7   1.4 16052.5..16707.1                                      
records      2 parlay              16472.3   0.8 15987.5..17089.3                                      
records      2 rayon-join          16554.3   1.1 16370.0..18518.8                                      
records      2 static              16557.6   0.9 15908.3..16704.0                                      excursions retained
records      2 wf                  18521.6   2.5 18053.4..23680.4       1.139 [1.13-1.49]   0/5       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      4 static               8400.0   1.7 8140.6..8670.3                                        excursions retained
records      4 tbb                  8426.5   0.9 8128.3..9350.1                                        
records      4 rayon-join           8550.8   1.3 8176.2..11130.7                                       
records      4 rayon-iter           8808.0   0.7 8335.0..9091.5                                        
records      4 parlay               8895.6   3.0 8213.1..9165.4                                        
records      4 wf                   9264.9   1.5 9124.2..9872.7         1.133 [1.10-1.20]   0/5       6 64 chunks
records      4 BEST REFERENCE = tbb          FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      8 rayon-iter           8724.9   1.1 8608.6..11099.6                                       
records      8 rayon-join           8751.3   3.4 8451.4..11112.3                                       
records      8 parlay               8771.0   3.7 8448.1..11105.1                                       
records      8 tbb                  9477.8   1.4 8332.0..10953.9                                       
records      8 wf                  10216.2   3.3 9686.7..11197.5        1.186 [1.11-1.33]   0/5      18 64 chunks
records      8 static              15437.4   4.9 12063.2..16545.3                                      excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = rayon-iter   WF fastest: n/a (oversubscribed)
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32477.0   1.4 31755.1..33187.1                                      
records      1 wf                  35592.9   0.5 35126.5..35965.7                                      
records      1 wf-seq              35643.9   0.3 35287.1..36033.2                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  13984.8   0.6 13692.0..14066.7       0.877 [0.87-0.89]   5/5       1 32 chunks
fir          2 static              16028.1   0.8 15899.7..16519.0                                      excursions retained
fir          2 parlay              16114.8   0.9 15893.4..16427.3                                      
fir          2 rayon-join          16160.2   1.0 15967.8..16379.6                                      
fir          2 tbb                 16197.5   1.2 15355.1..17136.6                                      
fir          2 rayon-iter          16297.2   1.5 16014.9..16535.3                                      
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   7286.0   2.9 7068.7..7643.4         0.921 [0.87-0.94]   5/5       7 64 chunks
fir          4 tbb                  8099.9   1.8 7860.2..8242.4                                        
fir          4 static               8247.4   3.1 7987.7..9304.1                                        excursions retained
fir          4 rayon-join           8295.1   1.0 8093.2..8513.2                                        
fir          4 parlay               8354.3   1.6 7985.7..8889.3                                        
fir          4 rayon-iter           8580.0   3.2 8303.0..9112.6                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   7716.9   1.0 7489.2..7793.8         0.942 [0.89-0.95]   5/5      21 64 chunks
fir          8 tbb                  8192.7   0.6 8101.0..9102.9                                        
fir          8 rayon-join           8400.7   1.8 8249.2..10705.6                                       
fir          8 parlay               8540.9   2.9 8084.9..8838.3                                        
fir          8 rayon-iter           8876.0   4.0 8379.6..10426.1                                       
fir          8 static              14390.4   0.9 12251.2..18312.5                                      excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27419.8   1.2 27094.2..28630.2                                      
fir          1 wf                  27868.0   2.1 26674.1..28619.8                                      
fir          1 serial              32032.6   1.2 30503.5..32432.4                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says: the budget moves the quadrature `wf` row and nothing else.
Quadrature falls from 14,534.7 to 8,751.7 us at W=2, 8,216.5 to 4,582.5 at
W=4 and 8,753.9 to 4,766.7 at W=8, becoming the fastest form in its block at
W=4 and W=8, and its W=1 row goes from 5.61 percent above its sequential
control to 0.30 percent below it. The other three kernels are byte-identical
modules to the plain run's and read fir 0.921, mandelbrot 1.050 and records
1.133 at W=4, inside their own spread. Its W=2 ratio of 1.031 is the one
acceptance bound the lead section records as not met.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), plain `--par` with the recursion budget defaulted, at `53359d73`

**The default moved, and this is the first table of the program it produces.**
Every section above this one was taken on a tree where plain `--par` offered
at every node of a recursive component; from `4c09484c` it gives each ordinary
cyclic call-graph component the budget-carrying clone family and enters it with
the budget the runtime derives from the pool width, and
`--par-recursive-frontier off` and a pinned `N` are the controls over that one
mechanism. `WF_PAR_CONTROL_FLAGS` was empty here, so this is a plain table in
the sense the bundle's README fixes: the `wf` row is the program plain `--par`
produces.

The rule the decision was made under, stated before the numbers it is read
against: **the family must beat `off` at every width, and be no worse than the
best fixed depth at every width.** The first half was never in doubt — the
section above measures it on this host at 1.685 to 1.031 at two lanes, 1.564 to
0.987 at four and 1.665 to 1.000 at eight on the quadrature `wf` row, W=1 from
5.61 percent above its sequential control to 0.30 percent below it, and W=4
process CPU from 1.81 to 1.17 times that control. What held the default at
`off` was the second half read through a single number: a W=2 ratio of 1.031
against a bound of 1.03, which came from the sweep's best fixed depth, 6 at
1.026. Three runs of the same `auto` bytes read **1.031, 1.056 and 0.978** at
W=2 — they straddle 1.026, and they straddle each other by 0.078, which is
twenty times the 0.004 the bound was being asked to resolve. A bound below a
host's own spread over identical bytes decides nothing about the mechanism, so
"no worse than the best fixed depth at every width" holds within what this host
can say, and the quieter reading is now the hosted compute-bench workflow's:
it runs on every push that touches the emitter and, with the default flipped,
times this program on `ubuntu-24.04` (recorded W=4) and `macos-14` (recorded
W=2).

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded;
  `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `53359d738edbfdee6d9fd13540e1c699a3c2033c`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so the `wf`
  row is the program plain `--par` produces — which now carries the budget
  family the section above measured as a control
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8   cargo: cargo 1.94.1 (29ea6fb6a
  2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `dc6aeaf3fda497ba…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
  The three map modules are the same bytes every section above this one timed,
  and `quadrature-par.ll` is now, under plain `--par`, the module the `auto`
  control emitted in the section above.
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `local`
- sizing window, read off the table: every `wf-seq` median at W=1 inside [5 ms,
  60 ms] — mandelbrot 26.793 ms, quadrature 16.499 ms, records 35.729 ms, fir
  27.241 ms; every `wf` median at the recorded W=4 above 1 ms — mandelbrot
  7.024 ms, quadrature 4.771 ms, records 9.696 ms, fir 7.489 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/9,
  quadrature 401/1,012/1,188, records 1/5/21, fir 1/7/18 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 7285's current affinity list: 0-3  date=2026-09-11T12:11:39Z
run=local  compiler=53359d738edbfdee6d9fd13540e1c699a3c2033c  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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
mandelbrot   2 tbb                 13393.7   0.5 13286.6..13605.6                                      
mandelbrot   2 rayon-iter          13443.5   0.4 13331.4..13757.9                                      
mandelbrot   2 rayon-join          13500.5   0.5 13419.8..13567.3                                      
mandelbrot   2 wf                  13705.2   1.3 13491.0..14716.1       1.031 [1.01-1.10]   0/5       3 16 chunks
mandelbrot   2 parlay              13901.4   1.4 13433.9..14092.2                                      
mandelbrot   2 static              26291.6   0.4 26150.3..26525.2                                      excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6846.9   1.8 6725.2..7290.9                                        
mandelbrot   4 parlay               7002.5   1.5 6878.2..7294.6                                        
mandelbrot   4 wf                   7024.2   1.0 6810.9..7275.3         1.032 [0.96-1.05]   1/5       6 16 chunks
mandelbrot   4 rayon-join           7068.5   1.0 6998.0..7256.7                                        
mandelbrot   4 rayon-iter           7271.7   1.7 7037.1..7840.4                                        
mandelbrot   4 static              26346.0   0.2 26296.3..26662.5                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6935.0   0.3 6848.8..7000.2                                        
mandelbrot   8 rayon-join           7081.8   0.4 6964.6..7172.0                                        
mandelbrot   8 parlay               7267.4   2.4 6870.3..7573.5                                        
mandelbrot   8 rayon-iter           7349.7   2.2 7173.6..7893.9                                        
mandelbrot   8 wf                   7501.4   2.8 7213.3..8268.8         1.095 [1.04-1.19]   0/5       9 16 chunks
mandelbrot   8 static              31521.2   0.0 31470.7..32743.3                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26415.6   0.2 26326.0..26533.9                                      
mandelbrot   1 wf                  26693.8   0.3 26552.2..26996.0                                      
mandelbrot   1 wf-seq              26793.1   0.2 26605.4..27217.2                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8349.6   1.9 8194.7..8587.3                                        
quadrature   2 rayon-join-left      8645.8   2.2 8381.3..8954.8                                        
quadrature   2 wf                   8757.4   1.9 8594.5..10158.0        1.049 [1.00-1.21]   0/5     401 
quadrature   2 static               8868.7   1.4 8740.4..9286.8                                        excursions retained
quadrature   2 tbb                 10129.5   1.3 9995.9..10828.7                                       
quadrature   2 parlay              10151.0   1.9 9742.0..10346.0                                       
quadrature   2 parlay-left         12186.3   4.6 11627.7..15540.9                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   4770.6   2.1 4668.2..6284.6         1.024 [0.92-1.23]   2/5    1012 
quadrature   4 rayon-join-left      5091.7   6.8 4744.7..6358.7                                        
quadrature   4 rayon-join           5122.1   1.5 4566.5..5198.3                                        
quadrature   4 tbb                  6490.7   6.5 5830.7..7169.8                                        
quadrature   4 static               6595.4  12.8 5184.4..7442.9                                        excursions retained
quadrature   4 parlay               7634.5   6.1 6727.1..8098.9                                        
quadrature   4 parlay-left         10469.3   5.6 9735.8..11082.7                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   4789.2   0.9 4633.6..6025.2         0.924 [0.81-1.20]   3/5    1188 
quadrature   8 rayon-join           5383.9   7.5 4781.0..5864.1                                        
quadrature   8 parlay               5855.1   5.6 5032.5..7326.0                                        
quadrature   8 parlay-left          6051.7   6.9 5422.6..7019.4                                        
quadrature   8 tbb                  6105.4   0.8 6053.8..6533.2                                        
quadrature   8 rayon-join-left      6235.1   8.5 5668.5..7548.9                                        
quadrature   8 static             523987.4   2.3 512008.1..544009.8                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15342.1   1.1 15166.0..15820.8                                      
quadrature   1 wf                  16365.2   0.6 16272.4..16995.7                                      
quadrature   1 wf-seq              16498.5   3.0 15160.9..16990.3                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 static              16605.4   1.3 16393.1..18034.9                                      excursions retained
records      2 rayon-join          16666.9   1.5 16157.0..17021.3                                      
records      2 tbb                 16758.0   0.8 16224.4..16886.6                                      
records      2 rayon-iter          17023.7   2.4 16617.2..29723.4                                      
records      2 parlay              17118.3   3.0 16603.1..19718.4                                      
records      2 wf                  18307.2   1.4 18048.0..19310.8       1.108 [1.10-1.17]   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-join   FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      4 rayon-join           8440.3   2.0 8198.2..11565.7                                       
records      4 parlay               8768.9   4.3 8391.6..11051.0                                       
records      4 tbb                  8800.0   5.6 8310.4..11682.5                                       
records      4 static               8969.0   7.5 8296.7..16212.9                                       excursions retained
records      4 rayon-iter           9357.9   9.3 8491.0..12288.9                                       
records      4 wf                   9695.8   4.3 9161.5..12389.2        1.169 [1.12-1.50]   0/5       5 64 chunks
records      4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

records      8 parlay               8677.1   2.9 8423.2..9306.4                                        
records      8 rayon-iter           9014.3   6.4 8386.3..11795.7                                       
records      8 tbb                  9192.4   2.5 8762.5..9768.9                                        
records      8 rayon-join           9455.0   5.1 8291.5..9938.6                                        
records      8 wf                  10165.1   2.8 9879.2..13185.7        1.197 [1.11-1.52]   0/5      21 64 chunks
records      8 static              18360.6   7.8 15920.0..21036.8                                      excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32482.5   0.7 31731.2..36587.5                                      
records      1 wf                  35500.5   0.3 35368.0..36051.9                                      
records      1 wf-seq              35728.6   0.8 35443.0..36452.8                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14011.7   1.2 13668.8..14205.9       0.884 [0.87-0.91]   5/5       1 32 chunks
fir          2 tbb                 15747.4   1.0 15585.3..16016.9                                      
fir          2 rayon-join          16020.7   1.3 15578.7..16298.6                                      
fir          2 static              16031.9   1.8 15743.0..17520.8                                      excursions retained
fir          2 parlay              16117.9   0.4 16012.6..16225.4                                      
fir          2 rayon-iter          16353.9   1.8 15993.3..16934.0                                      
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   7488.9   4.1 7180.1..8778.4         0.925 [0.88-1.05]   4/5       7 64 chunks
fir          4 rayon-join           8303.4   1.6 8116.9..8540.5                                        
fir          4 rayon-iter           8424.0   0.1 8365.0..10490.3                                       
fir          4 parlay               8425.2   0.5 7906.8..8464.5                                        
fir          4 tbb                  8518.7   5.0 8094.6..9606.8                                        
fir          4 static               8744.5   5.5 8260.8..10187.1                                       excursions retained
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7623.8   2.5 7436.3..7852.2         0.934 [0.93-0.97]   5/5      18 64 chunks
fir          8 tbb                  8161.0   1.0 8039.1..8324.3                                        
fir          8 parlay               8264.3   2.0 8097.9..8494.2                                        
fir          8 rayon-iter           8575.1   1.5 8261.2..9134.1                                        
fir          8 rayon-join           8680.7   4.5 8184.6..9345.8                                        
fir          8 static              14315.2  15.8 9705.0..18106.4                                       excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27188.4   1.2 26861.4..27986.3                                      
fir          1 wf-seq              27241.0   1.2 26671.9..27556.7                                      
fir          1 serial              31886.0   0.7 31130.0..32104.3                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
```

What it says: the quadrature `wf` row now reads 1.049 at W=2 with none of five
passes lower, 1.024 at W=4 with two of five lower and the fastest median in its
block, and 0.924 at W=8 with three of five lower and the lowest median there
too, in a block that is oversubscribed and therefore adjudicates no winner. At
W=1 it is 16.365 ms against its own sequential control's 16.499, 0.81 percent
below it, so the parallel lowering is no longer paying for a pool it does not
have. Against the last plain table of a tree with no family in it — the merged
tree re-emitted at `11d1e4a2`, this host, these references, these sizes —
quadrature moves from 1.762 to 1.049 at W=2 (14,549.8 to 8,757.4 us), from
1.581 to 1.024 at W=4 (7,955.3 to 4,770.6) and from 1.581 to 0.924 at W=8
(8,148.5 to 4,789.2), and its W=1 row from 5.52 percent above its sequential
control to 0.81 percent below. The three map kernels are byte-identical modules
and read as such: mandelbrot 1.031/1.032/1.095 against that section's
1.033/1.034/1.050, records 1.108/1.169/1.197 against 1.117/1.138/1.176, and fir
0.884/0.925/0.934 against 0.906/0.978/0.933 — differences inside the spread
this host puts on one module, and the recursion budget cannot reach a kernel
that meets the runtime through a synthesized splitter at all.

## 2026-09-11 — ubuntu-24.04 (Linux x86_64, 4 logical CPUs), run 34592005664 at `51debad8`

**The hosted before side of the default flip, Linux leg.** This section and the
`macos-14` one under it are the two legs of one record-only `compute-bench` run
at `51debad8`, the last revision on this branch at which plain `--par` still
offered at every node of a recursive component: `--par-recursive-frontier`
defaulted to `off` there. `WF_PAR_CONTROL_FLAGS` is empty in this run's
manifest, so the `wf` row is the program plain `--par` produced at that
revision, and its four emitted module hashes are the ones every earlier plain
table in this file recorded — `quadrature-par.ll` `e6753646…`. The two
`9d0c9485` sections at the end of the file are read against these two.

The runner prints the same `uname` hostname as the `34574271919` ubuntu section
above, and it is not the same machine: that section's `wf-seq` W=1 medians are
mandelbrot 18.678 ms, quadrature 10.536 ms, records 16.484 ms and fir 16.43 ms,
against 18.216, 9.393, 16.968 and 9.952 ms here. Each run is its own host and
nothing is pooled across them.

- host: `Linux runnervmlun5p 6.17.0-1022-azure #22-Ubuntu SMP Mon Jul 27 17:24:03 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as `taskset` recorded it;
  `cpuset.cpus.effective` also `0-3`; `cgroup_cpu_max` absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `51debad89740c7d1b425d5c328c6fafd776dcc81`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so this is a
  plain table
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: the same   rustc:
  rustc 1.98.1 (48a229cea 2026-09-01), host `x86_64-unknown-linux-gnu`, LLVM
  22.1.8   cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)   cmake: cmake version
  3.31.6
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `e6753646eef31512…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
  All four are the hashes the plain local sections above recorded.
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `34592005664`, job `bench (ubuntu-24.04)` `103239198739`,
  artifact `compute-bench-ubuntu-24.04` (`10196082291`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 18.216 ms, quadrature 9.393 ms, records
  16.968 ms, fir 9.952 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 5.262 ms, quadrature 6.985 ms, records 7.963 ms, fir 4.280 ms;
  and `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/8,
  quadrature 710/2,175/2,276, records 1/11/21, fir 1/5/20 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 4437's current affinity list: 0-3  date=2026-09-11T11:04:14Z
run=34592005664  compiler=51debad89740c7d1b425d5c328c6fafd776dcc81  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.98.1 (48a229cea 2026-09-01)
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
mandelbrot   2 tbb                  9237.4   0.2 9194.5..9351.7                                        
mandelbrot   2 wf                   9242.9   0.4 9118.4..9279.4         1.004 [0.99-1.01]   2/5       3 16 chunks
mandelbrot   2 rayon-iter           9246.5   0.2 9223.7..9275.1                                        
mandelbrot   2 rayon-join           9270.8   0.5 9208.2..9374.9                                        
mandelbrot   2 parlay               9410.5   0.1 9361.4..9416.6                                        
mandelbrot   2 static              18282.2   0.1 18231.3..18345.5                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  5125.2   0.2 5116.4..5432.4                                        
mandelbrot   4 rayon-join           5173.0   0.2 5162.2..5181.4                                        
mandelbrot   4 rayon-iter           5178.3   0.2 5161.5..5239.0                                        
mandelbrot   4 wf                   5261.6   0.4 5213.2..5302.6         1.026 [1.01-1.03]   0/5       6 16 chunks
mandelbrot   4 parlay               5346.7   0.2 5328.7..5448.4                                        
mandelbrot   4 static              18007.8   0.0 18003.3..18235.4                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  5186.7   0.4 5166.2..5289.7                                        
mandelbrot   8 rayon-join           5218.1   0.7 5175.3..5300.5                                        
mandelbrot   8 parlay               5367.4   0.4 5254.8..5437.3                                        
mandelbrot   8 rayon-iter           5378.1   1.1 5257.7..5438.3                                        
mandelbrot   8 wf                   5420.5   0.5 5392.0..5637.6         1.043 [1.04-1.09]   0/5       8 16 chunks
mandelbrot   8 static              21205.0  13.4 18359.3..29773.4                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf                  18026.1   0.1 18002.5..18448.8                                      
mandelbrot   1 wf-seq              18216.1   0.1 18197.0..18245.4                                      
mandelbrot   1 serial              18321.1   0.1 18295.4..18346.7                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

quadrature   2 rayon-join           5113.0   0.0 5110.9..5162.8                                        
quadrature   2 rayon-join-left      5202.0   0.1 5197.3..5231.5                                        
quadrature   2 tbb                  6059.8   0.0 6046.4..6239.4                                        
quadrature   2 static               6069.3   0.2 6046.9..6081.4                                        excursions retained
quadrature   2 wf                   8719.1   0.1 8708.3..8824.6         1.706 [1.70-1.71]   0/5     710 
quadrature   2 parlay               9401.3   2.6 9155.3..9846.6                                        
quadrature   2 parlay-left         14431.3   5.3 13229.4..16470.1                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5048.0   0.0 5035.9..5052.3                                        
quadrature   4 rayon-join-left      5171.8   0.1 5161.5..5188.7                                        
quadrature   4 tbb                  5653.9   0.1 5631.1..5668.0                                        
quadrature   4 static               6012.0   0.2 5997.3..6039.3                                        excursions retained
quadrature   4 wf                   6984.7   0.1 6963.9..7001.0         1.386 [1.38-1.39]   0/5    2175 
quadrature   4 parlay               8698.7   0.9 8580.3..8890.0                                        
quadrature   4 parlay-left         12967.9   2.1 12183.8..13234.5                                      
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 rayon-join           5225.7   0.2 5214.2..5319.3                                        
quadrature   8 rayon-join-left      5719.1   2.2 5591.5..5880.1                                        
quadrature   8 tbb                  5822.4   0.1 5801.6..5832.4                                        
quadrature   8 parlay               7039.4   0.3 6496.2..7137.4                                        
quadrature   8 wf                   7901.3   0.9 7312.7..8777.9         1.513 [1.40-1.65]   0/5    2276 
quadrature   8 parlay-left         10053.5   2.0 9135.3..10251.7                                       
quadrature   8 static             411014.3   1.7 404032.6..488016.8                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial               9364.8   0.0 9356.4..9372.6                                        
quadrature   1 wf                   9386.5   0.0 9382.1..9429.6                                        
quadrature   1 wf-seq               9392.8   0.0 9388.2..9479.3                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 wf                   8894.0   0.6 8838.2..9052.1         0.989 [0.99-1.01]   3/5       1 32 chunks
records      2 tbb                  8970.5   0.4 8931.6..9084.2                                        
records      2 static               8974.2   0.3 8949.0..9127.5                                        excursions retained
records      2 rayon-iter           8980.8   0.3 8941.7..9036.0                                        
records      2 rayon-join           9095.9   1.1 8953.3..9358.2                                        
records      2 parlay               9137.0   0.1 9029.6..9160.9                                        
records      2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

records      4 wf                   7962.7   0.4 7933.1..8063.9         0.951 [0.94-0.97]   5/5      11 64 chunks
records      4 tbb                  8446.3   0.5 8331.2..8571.0                                        
records      4 rayon-iter           8480.7   1.1 8297.7..8572.1                                        
records      4 rayon-join           8483.4   0.5 8325.3..8526.5                                        
records      4 parlay               8535.7   1.0 8314.4..8669.2                                        
records      4 static               8686.4   0.7 8496.3..8769.1                                        excursions retained
records      4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 wf                   8369.6   1.1 8274.5..8586.0         1.001 [0.98-1.01]   2/5      21 64 chunks
records      8 rayon-join           8470.5   0.6 8400.5..8617.4                                        
records      8 parlay               8518.5   0.9 8445.9..8852.0                                        
records      8 tbb                  8520.1   1.7 8358.8..8686.4                                        
records      8 rayon-iter           8530.0   1.5 8405.5..8799.0                                        
records      8 static              16830.8   0.1 16790.2..16870.5                                      excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              16968.2   0.6 16847.7..17103.0                                      
records      1 wf                  17531.9   0.1 17504.1..17570.8                                      
records      1 serial              17752.1   0.2 17572.9..17912.4                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

fir          2 wf                   5127.7   0.3 5106.3..5783.1         0.867 [0.86-0.98]   5/5       1 32 chunks
fir          2 rayon-iter           5922.4   0.2 5910.7..6194.8                                        
fir          2 static               5924.6   0.3 5891.5..6159.0                                        excursions retained
fir          2 tbb                  5937.1   0.1 5924.0..5976.4                                        
fir          2 rayon-join           5975.8   1.0 5917.6..6067.0                                        
fir          2 parlay               6073.0   0.1 5998.6..6076.3                                        
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   4280.3   0.1 4277.3..4298.2         0.854 [0.85-0.86]   5/5       5 64 chunks
fir          4 static               5015.7   0.2 5006.7..5050.9                                        excursions retained
fir          4 tbb                  5027.9   0.1 5022.8..5045.8                                        
fir          4 rayon-iter           5040.7   0.1 5033.7..5056.2                                        
fir          4 rayon-join           5044.4   0.1 5037.7..5064.7                                        
fir          4 parlay               5196.5   0.5 5143.0..5291.3                                        
fir          4 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          8 wf                   4651.2   1.0 4598.6..4704.1         0.918 [0.91-0.93]   5/5      20 64 chunks
fir          8 tbb                  5072.2   0.1 5068.2..5084.8                                        
fir          8 rayon-join           5079.3   0.0 5077.6..5092.4                                        
fir          8 rayon-iter           5128.5   0.3 5112.9..5142.8                                        
fir          8 parlay               5206.1   0.1 5166.4..5217.2                                        
fir          8 static               8373.3   1.9 5573.3..8531.8                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq               9952.4   0.1 9942.7..9983.6                                        
fir          1 wf                   9968.4   0.1 9941.5..9980.3                                        
fir          1 serial              11632.2   0.2 11605.2..11653.9                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says: at the recorded W=4 two of the four `wf` rows are the fastest
form in their block — fir at ratio 0.854 with all five passes lower,
4,280.3 us against `static` at 5,015.7, and records at 0.951 with all five
lower, 7,962.7 against oneTBB at 8,446.3. Mandelbrot is 1.026 with none
of five lower, 5,261.6 against oneTBB's 5,125.2, and quadrature is 1.386
with none lower, 6,984.7 against `rayon-join` at 5,048.0. At W=2 the same
two are fastest, fir 0.867 with five of five lower and records 0.989 with
three of five; mandelbrot is 1.004 with two of five lower and oneTBB ahead
of it by 5.5 us; quadrature is 1.706 with none lower, 8,719.1 against
`rayon-join` at 5,113.0. The oversubscribed W=8 block keeps that order:
fir 0.918, records 1.001, mandelbrot 1.043, quadrature 1.513. At W=1 the
`wf` row is below its `wf-seq` control on mandelbrot, 18,026.1 against
18,216.1 us, and on quadrature, 9,386.5 against 9,392.8; it is 0.16 percent
above on fir, 9,968.4 against 9,952.4, and 3.3 percent above on records,
17,531.9 against 16,968.2. Quadrature's `wf` row carries far more steals
than any other kernel, 710/2,175/2,276 at W=2/4/8 against single digits for
mandelbrot and tens for records and fir. Every row of the recorded block
has a MAD at or under 2.1 percent, and every `wf` row in it at or under 0.4.

## 2026-09-11 — macos-14 (Darwin arm64, 3 logical CPUs), run 34592005664 at `51debad8`

The Apple leg of the same record-only run, a minute after the Linux one: three
logical CPUs, so the recorded block is W=2, W=4 is emitted as oversubscribed
and W=8 is not emitted. `BENCH_ARCH` is empty on this host, which is the second
reading rule above — this section pools with nothing else in this file, and it
reads against the Linux sections only as a different host under a different
`BENCH_ARCH` and a different clang. `WF_PAR_CONTROL_FLAGS` is empty here too,
so this is a plain table: the `wf` row is the program plain `--par` produced at
`51debad8`, where `--par-recursive-frontier` still defaulted to `off`.

The `.ll` module hashes on this host are not the Linux ones — the emitted
module carries its own target — so the four below are this leg's own before
side, and the `9d0c9485` macOS section at the end of the file is read against
them.

- host: `Darwin sat12-bq163-0b4893de-d2e4-4f6d-be36-0ce0fcfbe5dd-0A807D0CE304.local 23.6.0 Darwin Kernel Version 23.6.0: Tue Jul 21 21:56:54 PDT 2026; root:xnu-10063.141.1.713.39~1/RELEASE_ARM64_VMAPPLE arm64`
- logical CPUs: 3   inherited mask: unqualified (no `taskset` on this host;
  `cgroup_cpu_max` and `cpuset` both absent and recorded as unqualified)
- recorded block: W=2   oversubscribed blocks emitted: W=4
- compiler revision: `51debad89740c7d1b425d5c328c6fafd776dcc81`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so this is a
  plain table
- clang: Apple clang version 15.0.0 (clang-1500.3.9.4)   clang++: the same
  rustc: rustc 1.98.0 (88d9e12ae 2026-08-18), host `aarch64-apple-darwin`,
  LLVM 22.1.8   cargo: cargo 1.98.0 (797e8a9bc 2026-08-05)   cmake: cmake
  version 4.4.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: empty
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `658df6c0d59843b4…`
  - `quadrature-par.ll` `5a30fb6c4eef2b18…`
  - `records-par.ll` `5aba473c12f26845…`
  - `fir-par.ll` `3f90b97e6e60dc98…`
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2 and W=4; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4;
  fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at W=4;
  quadrature M=64 integrations at tolerance `0x1p-54`, depth 24, `chunks=na`
- workflow run: `34592005664`, job `bench (macos-14)` `103239198951`, artifact
  `compute-bench-macos-14` (`10196105405`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 25.661 ms, quadrature 7.308 ms, records
  14.801 ms, fir 13.863 ms; every `wf` median at the recorded W=2 above 1 ms —
  mandelbrot 21.265 ms, quadrature 6.097 ms, records 7.860 ms, fir 7.240 ms;
  and `steals > 0` on the `wf` row at both parallel widths — mandelbrot 3/5,
  quadrature 681/1,253, records 1/12, fir 1/10 at W=2/W=4. No size constant
  changed.

```text
compute-bench  host=Darwin arm64  cpus=3  mask=unqualified (no taskset on this host)  date=2026-09-11T11:05:01Z
run=34592005664  compiler=51debad89740c7d1b425d5c328c6fafd776dcc81  clang=Apple clang version 15.0.0 (clang-1500.3.9.4)  rustc=rustc 1.98.0 (88d9e12ae 2026-08-18)
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
mandelbrot   2 parlay              12360.3   3.9 11179.7..12852.6                                      
mandelbrot   2 tbb                 15857.1  28.9 10612.3..25956.2                                      
mandelbrot   2 wf                  21264.9  25.5 13739.8..27089.6       2.004 [1.07-2.37]   0/5       3 16 chunks
mandelbrot   2 static              21958.5   0.2 21869.2..22062.8                                      excursions retained
mandelbrot   2 rayon-iter          22004.8   0.4 12938.0..22119.2                                      
mandelbrot   2 rayon-join          44355.6   6.2 22810.7..47088.1                                      
mandelbrot   2 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

mandelbrot   4 parlay               7542.5   2.5 7351.2..8399.9                                        
mandelbrot   4 tbb                  9911.7  21.9 7745.5..19646.2                                       
mandelbrot   4 wf                  20349.9   7.3 9338.2..21841.9        2.626 [1.24-2.97]   0/5       5 16 chunks
mandelbrot   4 rayon-join          23115.0  67.0 7619.6..54619.4                                       
mandelbrot   4 rayon-iter          27171.4  61.7 8314.3..51610.0                                       
mandelbrot   4 static              31064.8   4.0 29710.0..35698.0                                      excursions retained
mandelbrot   4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf                  22315.3   1.0 22016.0..26548.6                                      
mandelbrot   1 serial              25490.0   6.9 22268.2..27256.9                                      
mandelbrot   1 wf-seq              25661.4   7.4 22276.8..27571.2                                      
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
  wf-seq: control

quadrature   2 rayon-join           4010.4   1.0 3971.7..8586.8                                        
quadrature   2 parlay               4029.6   1.1 3961.0..4481.5                                        
quadrature   2 static               4119.9   0.3 4108.7..5381.6                                        excursions retained
quadrature   2 parlay-left          4141.7   1.1 4089.5..5203.8                                        
quadrature   2 rayon-join-left      4458.9  10.1 4007.0..7921.7                                        
quadrature   2 wf                   6096.5   1.3 6001.5..6669.9         1.515 [1.49-1.53]   0/5     681 
quadrature   2 tbb                  9955.4   4.7 9056.5..10425.2                                       
quadrature   2 BEST REFERENCE = parlay       FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

quadrature   4 rayon-join           3338.1   5.8 3144.0..17368.8                                       
quadrature   4 parlay               3578.3   4.1 3348.0..5084.2                                        
quadrature   4 rayon-join-left      3630.5   6.6 3392.5..7957.0                                        
quadrature   4 parlay-left          3826.2   1.9 3620.1..5939.5                                        
quadrature   4 wf                   4277.7   0.5 4233.9..4306.4         1.290 [1.20-1.35]   0/5    1253 
quadrature   4 tbb                  6812.0   0.8 6754.7..58982.7                                       
quadrature   4 static             118264.5   4.4 112272.2..124000.7                                    excursions retained
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial               7276.3   0.9 7211.5..9301.6                                        
quadrature   1 wf-seq               7308.2   1.2 7217.6..7834.7                                        
quadrature   1 wf                  13465.8   1.6 12903.0..15236.0                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                   7860.3   2.2 7686.2..9289.6         0.877 [0.85-1.02]   4/5       1 32 chunks
records      2 tbb                  9098.5   1.0 9003.9..10791.5                                       
records      2 parlay               9114.8   0.5 9069.7..9525.6                                        
records      2 rayon-iter           9157.5   2.3 8948.2..16405.0                                       
records      2 rayon-join           9164.0   0.8 9087.7..9665.9                                        
records      2 static               9316.1   3.6 8978.5..16034.7                                       excursions retained
records      2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      4 wf                   5510.9   0.3 5494.9..7457.2         0.890 [0.75-1.21]   4/5      12 64 chunks
records      4 tbb                  6201.4   1.2 6128.7..7326.5                                        
records      4 parlay               6226.3   0.6 6191.2..8132.2                                        
records      4 rayon-join           6316.9   0.5 6278.9..7965.2                                        
records      4 rayon-iter           6480.4   2.9 6295.7..8937.2                                        
records      4 static               8126.1   8.7 7324.3..10397.0                                       excursions retained
records      4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf                  14736.7   2.5 14369.8..15799.5                                      
records      1 wf-seq              14800.6   0.7 14635.2..15620.1                                      
records      1 serial              17711.3   2.3 17299.0..18821.1                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

fir          2 wf                   7240.2   1.8 7106.9..7817.9         0.906 [0.90-0.99]   5/5       1 32 chunks
fir          2 static               8004.6   0.6 7952.7..8561.2                                        excursions retained
fir          2 rayon-iter           8008.0   2.1 7842.9..11325.2                                       
fir          2 tbb                  8055.6   2.7 7836.5..8837.7                                        
fir          2 rayon-join           8099.0   2.6 7885.0..36192.5                                       
fir          2 parlay              10230.7  16.6 7903.9..11925.0                                       
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   5258.2   1.6 5172.3..7287.7         0.963 [0.69-1.34]   3/5      10 64 chunks
fir          4 tbb                  5578.4   2.3 5451.2..8652.4                                        
fir          4 rayon-join           5723.0   1.5 5547.5..10461.3                                       
fir          4 rayon-iter           5760.0   2.2 5635.3..18804.4                                       
fir          4 parlay               5862.7   6.9 5458.2..8534.2                                        
fir          4 static               6646.8   5.4 6286.3..7599.7                                        excursions retained
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              13862.5   0.5 13419.5..15290.2                                      
fir          1 wf                  13886.1   1.5 13488.7..15264.1                                      
fir          1 serial              17179.3   7.5 15476.0..19359.0                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says: at the recorded W=2 two of the four `wf` rows are the fastest
form in their block — records at ratio 0.877 with four of five passes
lower, 7,860.3 us against oneTBB at 9,098.5, and fir at 0.906 with all five
lower, 7,240.2 against `static` at 8,004.6. Quadrature is 1.515 with none
of five lower, 6,096.5 against `rayon-join` at 4,010.4, and mandelbrot is
2.004 with none lower, a 21,264.9 us median at a MAD of 25.5 percent over
a p10 of 13,739.8, against `parlay` at 12,360.3. At W=1 quadrature's `wf`
row is 13,465.8 us against its `wf-seq` control's 7,308.2, 84.3 percent
above it, while the other three are within 1.5 percent of their controls
or below — mandelbrot 22,315.3 against 25,661.4, records 14,736.7
against 14,800.6, fir 13,886.1 against 13,862.5. The oversubscribed
W=4 block keeps the order records 0.890, fir 0.963, quadrature 1.290,
mandelbrot 2.626. Spreads on this host are wide: MADs of 25.5 and 28.9
percent on the mandelbrot W=2 block, 16.6 percent on `parlay` for fir,
and p90 excursions to 36,192.5 us on `rayon-join` for fir and 47,088.1 on
`rayon-join` for mandelbrot, where the quadrature block's own rows sit at
or under 4.7 percent apart from `rayon-join-left`'s 10.1.

## 2026-09-11 — ubuntu-24.04 (Linux x86_64, 4 logical CPUs), run 34597909514 at `9d0c9485`

**The hosted after side, Linux leg.** The same record-only workflow on this
branch's head, where plain `--par` defaults `--par-recursive-frontier` to the
budget the runtime derives from the pool width. `WF_PAR_CONTROL_FLAGS` is empty
in the manifest, so this is a plain table in the same sense as the `51debad8`
one: the `wf` row is the program plain `--par` produces, now with the budget in
it. The manifest's hashes say exactly what changed — `quadrature-par.ll` is
`dc6aeaf3…` where the `51debad8` leg emitted `e6753646…`, and
`mandelbrot-par.ll`, `records-par.ll` and `fir-par.ll` are the same three
hashes that leg recorded, as the synthesized-splitter exclusion requires. The
`dc6aeaf3…` module is the one the local `53359d73` section above timed.

The `uname` hostname string is again the one both earlier ubuntu sections
print, and again the timings say it is a different machine: this run's `wf-seq`
W=1 medians are mandelbrot 20.197 ms, quadrature 12.743 ms, records 17.107 ms
and fir 19.112 ms, against 18.216, 9.393, 16.968 and 9.952 ms in the
`51debad8` section — fir's control is nearly twice as slow here. Nothing is
pooled across the two runs; the comparison below is stated leg by leg and row
by row.

- host: `Linux runnervmlun5p 6.17.0-1022-azure #22-Ubuntu SMP Mon Jul 27 17:24:03 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as `taskset` recorded it;
  `cpuset.cpus.effective` also `0-3`; `cgroup_cpu_max` absent and recorded as
  unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `9d0c9485a0b501ed3b2221a9b327d4f042eb3708`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so this is a
  plain table, and the recursion budget in it is the default rather than a
  control
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: the same   rustc:
  rustc 1.98.1 (48a229cea 2026-09-01), host `x86_64-unknown-linux-gnu`, LLVM
  22.1.8   cargo: cargo 1.98.1 (797e8a9bc 2026-08-05)   cmake: cmake version
  3.31.6
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `f07c190e3a396fc9…`
  - `quadrature-par.ll` `dc6aeaf3fda497ba…`
  - `records-par.ll` `9ceebaedf90e4a60…`
  - `fir-par.ll` `e0af1ed03ef2bc52…`
  The three map modules equal the `51debad8` leg's; only quadrature's moved.
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4 and
  W=8; fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at
  W=4 and W=8; quadrature M=64 integrations at tolerance `0x1p-54`, depth 24,
  no independent-map split and therefore `chunks=na`
- workflow run: `34597909514`, job `bench (ubuntu-24.04)` `103257899918`,
  artifact `compute-bench-ubuntu-24.04` (`10262866687`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 20.197 ms, quadrature 12.743 ms, records
  17.107 ms, fir 19.112 ms; every `wf` median at the recorded W=4 above 1 ms —
  mandelbrot 5.741 ms, quadrature 6.582 ms, records 9.793 ms, fir 6.824 ms;
  and `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/9,
  quadrature 445/1,049/1,214, records 1/6/21, fir 1/5/22 at W=2/4/8. No size
  constant changed.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 4599's current affinity list: 0-3  date=2026-09-11T12:18:09Z
run=34597909514  compiler=9d0c9485a0b501ed3b2221a9b327d4f042eb3708  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.98.1 (48a229cea 2026-09-01)
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
mandelbrot   2 tbb                 10086.4   0.0 10081.5..10104.9                                      
mandelbrot   2 rayon-iter          10093.2   0.0 10089.1..10127.3                                      
mandelbrot   2 rayon-join          10131.2   0.2 10111.6..10162.6                                      
mandelbrot   2 wf                  10265.3   0.2 10203.9..10308.8       1.018 [1.01-1.02]   0/5       3 16 chunks
mandelbrot   2 parlay              10473.6   0.2 10386.0..10508.8                                      
mandelbrot   2 static              20005.3   0.0 20002.1..20045.3                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  5667.8   0.1 5659.7..5680.2                                        
mandelbrot   4 rayon-join           5709.2   0.2 5697.4..5819.1                                        
mandelbrot   4 rayon-iter           5734.6   0.4 5680.8..5758.6                                        
mandelbrot   4 wf                   5740.7   0.1 5733.9..5844.7         1.013 [1.01-1.03]   0/5       6 16 chunks
mandelbrot   4 parlay               6149.8   0.5 6057.3..6181.0                                        
mandelbrot   4 static              20454.9   0.1 20424.1..20476.3                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 rayon-join           5740.9   0.2 5718.9..5777.6                                        
mandelbrot   8 tbb                  5747.8   0.1 5739.7..5776.9                                        
mandelbrot   8 rayon-iter           5866.6   0.7 5823.4..5932.4                                        
mandelbrot   8 wf                   5928.6   0.3 5905.4..5948.1         1.033 [1.03-1.04]   0/5       9 16 chunks
mandelbrot   8 parlay               6206.6   1.2 6039.9..6395.8                                        
mandelbrot   8 static              25875.4  11.2 22980.8..35870.8                                      excursions retained
mandelbrot   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              20101.0   0.0 20088.1..20152.4                                      
mandelbrot   1 wf                  20187.5   0.0 20181.0..20325.2                                      
mandelbrot   1 wf-seq              20197.2   0.1 20187.1..20210.9                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 wf                   6601.1   0.2 6585.3..6692.9         0.972 [0.97-0.98]   5/5     445 
quadrature   2 rayon-join           6792.2   0.1 6781.7..6812.1                                        
quadrature   2 rayon-join-left      6996.8   0.2 6975.8..11193.8                                       
quadrature   2 static               7007.6   0.3 6971.5..8758.1                                        excursions retained
quadrature   2 tbb                  7857.7   0.4 7811.7..7923.0                                        
quadrature   2 parlay              14847.5   0.3 14623.1..15039.0                                      
quadrature   2 parlay-left         23530.0   1.5 22794.9..24513.2                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   6581.7   0.3 6531.3..6618.6         0.974 [0.97-0.99]   5/5    1049 
quadrature   4 rayon-join           6737.5   0.6 6697.0..6776.1                                        
quadrature   4 rayon-join-left      6821.0   0.4 6763.2..6915.5                                        
quadrature   4 static               7201.8   0.1 7195.5..7237.3                                        excursions retained
quadrature   4 tbb                  7393.0   0.2 7379.3..7476.6                                        
quadrature   4 parlay              14242.4   3.7 13654.7..14770.2                                      
quadrature   4 parlay-left         23353.9   0.5 22527.7..23472.5                                      
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   6847.7   0.1 6837.9..6861.3         0.959 [0.94-0.97]   5/5    1214 
quadrature   8 rayon-join           7139.7   0.5 7072.3..7265.1                                        
quadrature   8 rayon-join-left      7630.3   0.3 7609.6..7876.5                                        
quadrature   8 tbb                  7773.7   0.6 7673.9..7819.1                                        
quadrature   8 parlay              14146.3   0.8 13242.7..14261.0                                      
quadrature   8 parlay-left         22785.4   2.1 22267.4..23973.2                                      
quadrature   8 static             412991.4   2.9 400991.4..455021.6                                    excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              12683.2   0.1 12575.9..12700.2                                      
quadrature   1 wf                  12711.3   0.1 12656.2..12725.9                                      
quadrature   1 wf-seq              12742.6   0.2 12717.6..12809.9                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 tbb                  8198.8   1.1 8088.8..8723.2                                        
records      2 rayon-join           8242.7   1.1 8154.7..8900.5                                        
records      2 rayon-iter           8448.0   2.3 8196.6..9253.8                                        
records      2 static               8464.3   3.6 8083.1..9000.4                                        excursions retained
records      2 parlay               8810.8   0.6 8378.1..8902.8                                        
records      2 wf                   9069.0   3.1 8770.3..9554.1         1.108 [1.08-1.18]   0/5       1 32 chunks
records      2 BEST REFERENCE = static       FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      4 tbb                  8857.7   0.4 8783.6..9058.2                                        
records      4 rayon-iter           8890.6   0.8 8789.6..9797.9                                        
records      4 parlay               9097.3   1.9 8856.5..9268.2                                        
records      4 rayon-join           9159.4   0.6 8793.5..9228.7                                        
records      4 static               9332.1   3.7 8853.9..10572.2                                       excursions retained
records      4 wf                   9793.4   1.3 9509.0..10152.7        1.114 [1.07-1.16]   0/5       6 64 chunks
records      4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 rayon-iter           8986.8   0.3 8963.9..9332.6                                        
records      8 rayon-join           9245.9   0.2 8877.1..9264.2                                        
records      8 parlay               9251.8   0.7 9099.3..10035.7                                       
records      8 tbb                  9362.0   1.4 8922.4..9692.4                                        
records      8 wf                  10093.6   0.7 9938.3..10252.5        1.118 [1.08-1.15]   0/5      21 64 chunks
records      8 static              16862.2   1.4 11080.1..17104.9                                      excursions retained
records      8 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: n/a (oversubscribed)
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              16348.8   1.7 16071.3..17194.9                                      
records      1 wf-seq              17106.6   0.2 17043.7..17480.2                                      
records      1 wf                  17329.1   0.6 17048.9..17436.2                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                   9669.2   0.3 9637.0..9702.3         0.997 [1.00-1.00]   4/5       1 32 chunks
fir          2 static               9695.1   0.0 9675.0..9731.9                                        excursions retained
fir          2 tbb                  9700.5   0.1 9678.6..9705.7                                        
fir          2 rayon-join           9712.1   0.0 9707.3..9737.5                                        
fir          2 rayon-iter           9733.7   0.2 9713.2..9753.7                                        
fir          2 parlay               9923.4   0.5 9854.5..9988.2                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 static               6726.0   0.1 6719.6..6751.5                                        excursions retained
fir          4 tbb                  6731.2   0.0 6726.9..6733.6                                        
fir          4 rayon-iter           6753.9   0.1 6745.7..6780.4                                        
fir          4 rayon-join           6766.9   0.1 6754.1..6775.9                                        
fir          4 wf                   6823.9   0.1 6810.2..6829.7         1.015 [1.01-1.02]   0/5       5 64 chunks
fir          4 parlay               7060.9   0.9 6923.7..7165.0                                        
fir          4 BEST REFERENCE = static       FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          8 tbb                  6789.9   0.1 6775.7..6797.2                                        
fir          8 rayon-join           6797.5   0.2 6779.4..6842.8                                        
fir          8 rayon-iter           6852.2   0.5 6814.5..6905.3                                        
fir          8 parlay               7066.1   0.7 6929.5..7169.4                                        
fir          8 wf                   7284.7   0.5 7071.9..7403.2         1.073 [1.04-1.09]   0/5      22 64 chunks
fir          8 static              13494.5  14.7 11505.4..17506.2                                      excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              19111.5   0.1 19075.9..19126.6                                      
fir          1 wf                  19154.7   0.1 19126.2..19207.9                                      
fir          1 serial              19280.4   0.0 19256.0..19288.9                                      
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

What it says, read against the `51debad8` ubuntu section — the same
workflow, the same references, the same four sizes, an hour and a quarter
earlier, and the same three map modules, with `quadrature-par.ll` the one
file that differs: the quadrature `wf` row is the fastest form in its block
at every width for the first time on a hosted leg. At the recorded W=4 it is
0.974 with all five passes lower, 6,581.7 us against `rayon-join` at 6,737.5,
where the before table read 1.386 with none of five lower and `rayon-join`
fastest; at W=2 it is 0.972 with five of five lower against that table's
1.706 with none; at W=8, oversubscribed and adjudicating no winner, 0.959
with five of five against 1.513 with none. Its steal counts fall with the
budget in place, 445/1,049/1,214 against 710/2,175/2,276. At W=1 quadrature's
`wf` row is 12,711.3 us against its own `wf-seq` control's 12,742.6, 0.25
percent below it, where the before table read 9,386.5 against 9,392.8,
0.07 percent below — both sit under their controls, on runners whose
controls differ by 36 percent. The three map kernels are byte-identical
modules and are quoted here so the reader can see what the day did to the
host: mandelbrot 1.018/1.013/1.033 at W=2/4/8 against the before table's
1.004/1.026/1.043, records 1.108/1.114/1.118 against 0.989/0.951/1.001,
fir 0.997/1.015/1.073 against 0.867/0.854/0.918. Records and fir lose their
fastest-form standing at the recorded width while their emitted code does not
change, and their references move with them — fir's `static` is 6,726.0
us here against 5,015.7 there and its `wf` row 6,823.9 against 4,280.3,
records' oneTBB 8,857.7 against 8,446.3 and its `wf` row 9,793.4 against
7,962.7 — on a run whose fir `wf-seq` W=1 control is 19,112 us against
9,952. Every row of the recorded block has a MAD at or under 3.7 percent.

## 2026-09-11 — macos-14 (Darwin arm64, 3 logical CPUs), run 34597909514 at `9d0c9485`

**The hosted after side, Apple leg.** The macOS leg of the same head run, a
minute and a half after the Linux one. Three logical CPUs, so the recorded
block is W=2 and W=4 is emitted as oversubscribed. `BENCH_ARCH` is empty here
as on every macOS run, which is the second reading rule above: this section
pools with nothing, and it reads against the `51debad8` macOS section as the
same leg of the same workflow on a different day-part and a different machine.
`WF_PAR_CONTROL_FLAGS` is empty, so it is a plain table, with the runtime-derived
recursion budget as the default rather than as a control.

The manifest's hashes make the same statement the Linux leg's do, in this
host's own module bytes: `quadrature-par.ll` moves from `5a30fb6c…` to
`259c3d89…`, while `mandelbrot-par.ll` `658df6c0…`, `records-par.ll`
`5aba473c…` and `fir-par.ll` `3f90b97e…` are unchanged from the `51debad8`
leg.

- host: `Darwin iad20-fj917-2e82cc35-015b-413b-8e15-ceeb52f9f292-4EF4BFCC91CF.local 23.6.0 Darwin Kernel Version 23.6.0: Tue Jul 21 21:56:54 PDT 2026; root:xnu-10063.141.1.713.39~1/RELEASE_ARM64_VMAPPLE arm64`
- logical CPUs: 3   inherited mask: unqualified (no `taskset` on this host;
  `cgroup_cpu_max` and `cpuset` both absent and recorded as unqualified)
- recorded block: W=2   oversubscribed blocks emitted: W=4
- compiler revision: `9d0c9485a0b501ed3b2221a9b327d4f042eb3708`
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty, so this is a
  plain table
- clang: Apple clang version 15.0.0 (clang-1500.3.9.4)   clang++: the same
  rustc: rustc 1.98.0 (88d9e12ae 2026-08-18), host `aarch64-apple-darwin`,
  LLVM 22.1.8   cargo: cargo 1.98.0 (797e8a9bc 2026-08-05)   cmake: cmake
  version 4.4.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: empty
- emitted `--par` module SHA-256, as recorded before and after the run:
  - `mandelbrot-par.ll` `658df6c0d59843b4…`
  - `quadrature-par.ll` `259c3d89f26bad27…`
  - `records-par.ll` `5aba473c12f26845…`
  - `fir-par.ll` `3f90b97e6e60dc98…`
  The three map modules equal the `51debad8` leg's; only quadrature's moved.
- sizes and emitted chunk counts: mandelbrot 98,304 points at limit 256, shape
  `trailing`, **16 chunks** at W=2 and W=4; records 131,072 records at
  `max_length` 255, shape `unicode`, **32 chunks** at W=2 and **64** at W=4;
  fir K=64 taps over N=524,288 outputs, **32 chunks** at W=2 and **64** at W=4;
  quadrature M=64 integrations at tolerance `0x1p-54`, depth 24, `chunks=na`
- workflow run: `34597909514`, job `bench (macos-14)` `103257899562`, artifact
  `compute-bench-macos-14` (`10263271887`)
- sizing window, read off the table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 23.528 ms, quadrature 8.001 ms, records
  18.292 ms, fir 19.771 ms; every `wf` median at the recorded W=2 above 1 ms —
  mandelbrot 11.570 ms, quadrature 4.011 ms, records 8.948 ms, fir 10.057 ms;
  and `steals > 0` on the `wf` row at both parallel widths — mandelbrot 3/5,
  quadrature 442/695, records 1/11, fir 2/10 at W=2/W=4. No size constant
  changed.

```text
compute-bench  host=Darwin arm64  cpus=3  mask=unqualified (no taskset on this host)  date=2026-09-11T12:19:54Z
run=34597909514  compiler=9d0c9485a0b501ed3b2221a9b327d4f042eb3708  clang=Apple clang version 15.0.0 (clang-1500.3.9.4)  rustc=rustc 1.98.0 (88d9e12ae 2026-08-18)
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
mandelbrot   2 parlay              11332.5   1.7 11140.5..11987.6                                      
mandelbrot   2 wf                  11570.0   2.0 11344.0..14320.6       1.042 [1.04-1.20]   0/5       3 16 chunks
mandelbrot   2 rayon-join          11854.7   6.5 11085.4..13243.6                                      
mandelbrot   2 tbb                 12008.3   6.3 10915.5..12759.9                                      
mandelbrot   2 rayon-iter          12150.1   8.2 10994.4..14460.0                                      
mandelbrot   2 static              22903.0   3.1 22191.7..24426.1                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  7890.2   3.7 7394.6..9125.5                                        
mandelbrot   4 parlay               7966.8   7.9 7306.2..9363.4                                        
mandelbrot   4 rayon-join           8579.7  10.8 7653.5..10478.6                                       
mandelbrot   4 wf                   9741.5   2.9 9391.3..12060.8        1.285 [1.22-1.32]   0/5       5 16 chunks
mandelbrot   4 rayon-iter          10576.4   9.8 8463.2..11617.3                                       
mandelbrot   4 static              32048.2   7.2 29738.5..35875.2                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf                  22870.7   3.5 22074.5..24400.2                                      
mandelbrot   1 serial              23491.3   1.7 22147.7..24744.4                                      
mandelbrot   1 wf-seq              23527.6   0.3 22171.8..25639.4                                      
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
  wf-seq: control

quadrature   2 wf                   4011.2   3.9 3854.1..6105.2         1.004 [0.93-1.47]   2/5     442 
quadrature   2 rayon-join           4194.6   0.6 4145.7..4275.7                                        
quadrature   2 rayon-join-left      4255.8   6.1 3996.9..7022.0                                        
quadrature   2 static               4281.4   2.7 4154.4..4659.1                                        excursions retained
quadrature   2 parlay               4436.0   8.1 4030.7..6536.5                                        
quadrature   2 parlay-left          5271.3  10.5 4300.7..6064.8                                        
quadrature   2 tbb                  9781.3   4.8 9211.6..15758.2                                       
quadrature   2 BEST REFERENCE = rayon-join-left FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

quadrature   4 wf                   2856.4   2.0 2798.8..4914.3         0.907 [0.68-1.34]   3/5     695 
quadrature   4 rayon-join           3426.6   8.1 3148.6..5940.6                                        
quadrature   4 parlay               3451.7   3.7 3324.5..5820.3                                        
quadrature   4 rayon-join-left      3688.5   7.9 3396.3..5996.7                                        
quadrature   4 parlay-left          3926.6   6.1 3666.8..4312.5                                        
quadrature   4 tbb                  7334.8   4.3 7022.2..9927.0                                        
quadrature   4 static             120074.0   2.9 116586.0..125927.5                                    excursions retained
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 wf-seq               8000.8   2.3 7672.3..8692.7                                        
quadrature   1 serial               8036.1   4.6 7486.7..8679.8                                        
quadrature   1 wf                   8272.5   7.3 7669.8..9143.0                                        
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen

records      2 wf                   8948.2  14.3 7666.8..15745.2        0.905 [0.86-1.68]   4/5       1 32 chunks
records      2 rayon-iter           9890.7   8.7 9033.8..15160.6                                       
records      2 rayon-join           9949.8   6.7 9283.5..11354.2                                       
records      2 parlay              10247.6   8.8 9316.9..13347.2                                       
records      2 static              10469.0  14.9 8912.7..15093.4                                       excursions retained
records      2 tbb                 14565.1  26.3 8874.4..23990.6                                       
records      2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

records      4 wf                   5719.4   3.5 5518.7..28408.0        0.904 [0.88-3.06]   4/5      11 64 chunks
records      4 parlay               6494.1   4.5 6185.6..11381.0                                       
records      4 tbb                  6612.4   4.5 6154.2..10682.5                                       
records      4 rayon-iter           6835.6   5.0 6490.8..9931.9                                        
records      4 rayon-join           7085.1   6.5 6625.4..9856.2                                        
records      4 static               8612.6   5.8 8062.1..12034.9                                       excursions retained
records      4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf                  17233.0   8.1 15835.5..19870.5                                      
records      1 wf-seq              18292.2   7.3 16954.8..21443.4                                      
records      1 serial              20972.6  15.3 17763.2..28110.0                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

fir          2 wf                  10057.0  13.0 8752.7..33938.8        0.878 [0.70-3.42]   3/5       2 32 chunks
fir          2 parlay              12059.3  16.0 10128.9..16714.6                                      
fir          2 static              12298.9  12.2 10269.3..14289.3                                      excursions retained
fir          2 rayon-iter          12935.8   8.3 11865.2..26709.2                                      
fir          2 rayon-join          14224.8  29.1 10089.0..41718.8                                      
fir          2 tbb                 17208.9  33.8 9922.5..23025.6                                       
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

fir          4 rayon-join           9405.0  17.7 7736.5..41879.3                                       
fir          4 parlay               9505.1  11.7 7173.8..10905.7                                       
fir          4 rayon-iter          10076.2   0.7 8874.6..45790.4                                       
fir          4 tbb                 10167.3  26.9 7270.7..17637.3                                       
fir          4 wf                  10697.6   6.9 8119.4..24679.5        1.302 [1.12-2.45]   0/5      10 64 chunks
fir          4 static              11008.7  12.9 9117.0..14337.3                                       excursions retained
fir          4 BEST REFERENCE = rayon-iter   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  17306.3   9.9 15505.1..20614.6                                      
fir          1 serial              18494.3   5.8 17430.7..24691.4                                      
fir          1 wf-seq              19771.1   4.0 16685.9..20846.7                                      
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
  wf-seq: control

```

What it says, read against the `51debad8` macOS section — the same workflow
leg, the same references, the same sizes, an hour and a quarter earlier,
and the same three map modules, with `quadrature-par.ll` the one file
that differs: at the recorded W=2 the quadrature `wf` row is 1.004 with
two of five passes lower and the fastest median in its block, 4,011.2 us
against `rayon-join` at 4,194.6, where the before table read 1.515 with
none of five lower and `rayon-join` fastest at 4,010.4 against a `wf`
median of 6,096.5. In the oversubscribed W=4 block it is 0.907 with three
of five lower and again the fastest median, 2,856.4 against `rayon-join`
at 3,426.6, where the before table read 1.290 with none lower. Its steal
counts fall, 442/695 against 681/1,253. At W=1 quadrature's `wf` row is
8,272.5 us against its `wf-seq` control's 8,000.8, 3.4 percent above it,
where the before table read 13,465.8 against 7,308.2, 84.3 percent above:
the W=1 penalty this kernel paid for its parallel lowering on this leg
is the part that moved most. The three map kernels are byte-identical
modules and their rows moved in both directions: mandelbrot 1.042/1.285
at W=2/W=4 against the before table's 2.004/2.626, records 0.905/0.904
against 0.877/0.890, fir 0.878/1.302 against 0.906/0.963. This host's
spread is what those numbers are read through — the mandelbrot before
row carried a 25.5 percent MAD and this one 2.0, fir's `wf` row here has
a 13.0 percent MAD and a p90 of 33,938.8 us against a median of 10,057.0,
and the W=2 block's reference rows run to MADs of 26.3 and 33.8 percent
— while quadrature's own rows stay at or under 10.5 percent in both tables.

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the CPU column added to the bundle, at `78867abd`

**A harness change, not a runtime change, and a `PASSES=2` pair rather than a
recorded block.** `raw.tsv` recorded wall, outputs and steals only, so nothing
in this record could say what a form spent in CPU to reach its wall time — the
number the process-CPU target is read off. The `time` subcommand now reads
`CLOCK_PROCESS_CPUTIME_ID` around the same interval the wall clock brackets and
records it as a `cpu_ns` column beside `wall_ns`; the reducer prints a `cpu_us`
median per row and, on the `wf` row, a `cpu_r` beside the wall `ratio`, paired
the same way — within each pass, WF's CPU against the CPU of the reference that
was fastest **by wall** in that pass, so the two ratios are about the same
pairs.

The two tables below are the same tree, the same four images and the same
emitted modules, timed before and after that harness change, and they exist for
one purpose: to show that the extra clock reads did not move the wall figures.
They are `PASSES=2`, which is half the bundle's recorded discipline, and
**neither is a candidate record**; the pair answers an agreement question, not a
scheduling one.

**What moved, and what that says.** Over the 87 rows both runs share, the median
absolute change in `median_us` is **3.07 percent**, the largest is 46.8 percent
(`quadrature` W=4 `rayon-join`, a row whose own `p10..p90` in the second run
spans 12 percent), and the signed mean is **+1.04 percent** with 41 rows down
and 46 up. That is this host's two-pass spread, not a shift: a systematic cost
would push every row the same way, and the `wf` rows themselves move both ways
(`records` W=4 −19.0, `quadrature` W=2 −16.3, `mandelbrot` W=2 +3.3,
`mandelbrot` W=4 +0.5, `records` W=8 −0.01). The nested reads are also bounded
directly rather than only inferred: 100,000 back-to-back `mono/cpu/cpu/mono`
quadruples on this host put the minimum wall cost of the **two** nested
`CLOCK_PROCESS_CPUTIME_ID` reads at **757 ns** — that clock is a syscall on
Linux and not a vDSO read, so it is far from free — which against the smallest
median in either table, `quadrature` W=4 at about 4.9 ms, is **0.016 percent**,
two orders below that row's own MAD of 3.2 percent and three below the spread
just described.

The wall clock stays the outermost pair and the two CPU reads are nested inside
it, so the wall interval encloses exactly one `call()` plus those two reads and
nothing else, and no CPU a call spends can fall outside the wall interval. That
ordering is stated in a comment in `harness.c` beside the loop.

The CPU column reads immediately. At W=4 `quadrature`'s `wf` row spends
**17,738 us of process CPU for a 5,002 us wall**, 3.55 times its own wall on a
four-CPU host, against a `wf-seq` W=1 control that spends 15,806 us — so the
parallel form buys a 3.16x wall speedup for 1.12x the sequential CPU, and the
`cpu_r` against `rayon-join` is 0.874. `records` at W=4 reads `cpu_r` **1.176**
beside a wall ratio of 1.178, and `fir` reads 0.835 beside 0.915. These are the
numbers the wait-path steps are about.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core — this host has no SMT
  siblings)   inherited mask: `0-3` (as recorded; `cgroup_cpu_max` and
  `cpuset.cpus.effective` both absent and recorded as unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `78867abdb5119d0751de1514a0befeaeeb5d6724` for both runs
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` was empty in both runs, so
  the `wf` row is the program plain `--par` produces
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host `x86_64-unknown-linux-gnu`
  cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- emitted `--par` module SHA-256, identical in both runs and identical to the
  plain `--par` section at `53359d73`: `mandelbrot-par.ll` `f07c190e3a396fc9…`,
  `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`,
  `fir-par.ll` `e0af1ed03ef2bc52…`. The harness is not the module, and this is
  the check that says so.
- sizes and emitted chunk counts: unchanged — mandelbrot 98,304 points at limit
  256, **16 chunks** at every parallel width; records 131,072 records,
  **32 chunks** at W=2 and **64** at W=4 and W=8; fir K=64 over N=524,288,
  **32/64/64**; quadrature M=64, no independent-map split, `chunks=na`
- workflow run: `local`. The push carrying this change touches
  `research/experiments/compute-bench/**`, so the hosted `compute-bench`
  workflow runs on `ubuntu-24.04` and `macos-14` and its two tables are the
  quieter reading; they are not waited on here.
- sizing window, read off the second table: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 26.916 ms, quadrature 15.807 ms, records 35.541 ms,
  fir 28.345 ms; every `wf` median at the recorded W=4 above 1 ms — mandelbrot
  7.191 ms, quadrature 5.002 ms, records 9.769 ms, fir 7.758 ms; and
  `steals > 0` on the `wf` row at every parallel width — mandelbrot 3/6/7,
  quadrature 399/1,010/1,202, records 1/11/20, fir 1/9/17 at W=2/4/8. No size
  constant changed.

Before the change, `PASSES=2 CALLS=5`, no `cpu_us` column:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 11550's current affinity list: 0-3  date=2026-09-11T12:54:38Z
run=local  compiler=78867abdb5119d0751de1514a0befeaeeb5d6724  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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
passes=2 calls=5

kernel       w form              median_us  mad% p10..p90_us            ratio            lower  steals note
mandelbrot   2 tbb                 13394.8   0.0 13391.3..13398.3                                      
mandelbrot   2 parlay              13584.1   0.4 13526.7..13641.5                                      
mandelbrot   2 wf                  13657.5   0.2 13633.4..13681.5       1.020 [1.02-1.02]   0/2       3 16 chunks
mandelbrot   2 rayon-join          13698.0   1.2 13529.0..13867.0                                      
mandelbrot   2 rayon-iter          13731.0   0.3 13694.6..13767.4                                      
mandelbrot   2 static              26439.4   0.3 26367.6..26511.3                                      excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6911.1   0.7 6864.6..6957.7                                        
mandelbrot   4 parlay               7134.1   1.5 7024.9..7243.2                                        
mandelbrot   4 rayon-join           7140.2   2.2 6984.7..7295.8                                        
mandelbrot   4 wf                   7158.7   2.8 6955.9..7361.4         1.036 [1.01-1.06]   0/2       6 16 chunks
mandelbrot   4 rayon-iter           7649.4   5.9 7196.9..8102.0                                        
mandelbrot   4 static              26605.9   1.1 26318.6..26893.1                                      excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 rayon-join           7180.1   0.9 7118.0..7242.1                                        
mandelbrot   8 tbb                  7224.7   4.0 6932.7..7516.7                                        
mandelbrot   8 parlay               7360.1   1.5 7251.8..7468.4                                        
mandelbrot   8 rayon-iter           7445.5   0.4 7418.0..7473.0                                        
mandelbrot   8 wf                   7724.4   3.3 7472.3..7976.5         1.091 [1.03-1.15]   0/2       9 16 chunks
mandelbrot   8 static              35042.8  10.4 31385.1..38700.4                                      excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26502.8   0.6 26348.6..26657.0                                      
mandelbrot   1 wf-seq              26769.6   0.3 26694.5..26844.8                                      
mandelbrot   1 wf                  26786.0   0.1 26760.6..26811.3                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8312.2   3.5 8018.1..8606.3                                        
quadrature   2 rayon-join-left      8748.0   1.4 8627.9..8868.1                                        
quadrature   2 tbb                 10229.8   3.0 9922.9..10536.8                                       
quadrature   2 parlay              10439.9   2.3 10199.2..10680.6                                      
quadrature   2 wf                  10444.1   6.5 9761.8..11126.5        1.261 [1.13-1.39]   0/2     387 
quadrature   2 static              10628.9  13.0 9242.2..12015.5                                       excursions retained
quadrature   2 parlay-left         14194.5   5.6 13396.9..14992.0                                      
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           4825.7   0.2 4813.8..4837.6                                        
quadrature   4 wf                   4895.0   5.1 4645.6..5144.3         1.014 [0.97-1.06]   1/2    1019 
quadrature   4 rayon-join-left      5143.9   4.4 4920.1..5367.7                                        
quadrature   4 static               5333.1   3.6 5139.4..5526.7                                        excursions retained
quadrature   4 tbb                  6046.8   5.2 5733.2..6360.3                                        
quadrature   4 parlay               7612.2   1.7 7485.8..7738.7                                        
quadrature   4 parlay-left         10036.8  10.6 8977.9..11095.7                                       
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   4953.8   2.5 4827.9..5079.7         0.791 [0.75-0.83]   2/2    1174 
quadrature   8 tbb                  6271.5   2.3 6125.7..6417.4                                        
quadrature   8 parlay               6321.5   3.2 6119.3..6523.7                                        
quadrature   8 rayon-join           6577.7   3.5 6347.3..6808.2                                        
quadrature   8 rayon-join-left      6807.5   4.8 6477.5..7137.6                                        
quadrature   8 parlay-left          7531.0  11.7 6649.8..8412.3                                        
quadrature   8 static             561964.8   8.9 511993.0..611936.5                                    excursions retained
quadrature   8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15472.6   1.5 15240.5..15704.6                                      
quadrature   1 wf                  15652.2   6.0 14709.9..16594.5                                      
quadrature   1 wf-seq              16344.4   0.4 16283.6..16405.2                                      
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 rayon-iter          16627.1   2.5 16203.6..17050.6                                      
records      2 tbb                 16788.5   1.5 16538.3..17038.6                                      
records      2 rayon-join          16965.3   0.3 16919.5..17011.1                                      
records      2 static              17098.5   1.0 16931.3..17265.7                                      excursions retained
records      2 parlay              18761.7   8.7 17123.0..20400.4                                      
records      2 wf                  19010.1   4.2 18208.2..19812.0       1.144 [1.12-1.16]   0/2       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen

records      4 rayon-iter           8830.7   1.9 8658.8..9002.5                                        
records      4 rayon-join           8982.2   3.9 8631.7..9332.8                                        
records      4 parlay               9082.6   3.6 8757.4..9407.9                                        
records      4 tbb                  9662.0   8.5 8844.3..10479.7                                       
records      4 static              11580.5  22.1 9025.6..14135.3                                       excursions retained
records      4 wf                  12065.3   3.3 11663.5..12467.2       1.396 [1.35-1.44]   0/2      11 64 chunks
records      4 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 tbb                  8451.8   0.5 8413.5..8490.1                                        
records      8 parlay               8479.3   1.6 8341.3..8617.2                                        
records      8 rayon-join           8698.7   2.0 8522.1..8875.4                                        
records      8 rayon-iter           8863.2   4.2 8492.2..9234.2                                        
records      8 wf                  10067.7   2.8 9790.3..10345.1        1.202 [1.16-1.24]   0/2      20 64 chunks
records      8 static              18741.9   2.3 18318.4..19165.3                                      excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              33872.6   4.6 32326.7..35418.4                                      
records      1 wf-seq              36250.9   0.6 36048.0..36453.8                                      
records      1 wf                  38851.3   8.6 35502.8..42199.9                                      
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14314.8   2.1 14014.2..14615.5       0.905 [0.89-0.92]   2/2       1 32 chunks
fir          2 rayon-iter          15900.0   0.6 15802.3..15997.6                                      
fir          2 rayon-join          16047.4   1.3 15846.5..16248.2                                      
fir          2 parlay              16122.2   1.3 15918.2..16326.3                                      
fir          2 tbb                 17059.8   5.5 16120.7..17999.0                                      
fir          2 static              18353.5  10.5 16434.8..20272.1                                      excursions retained
fir          2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7542.8   1.2 7454.9..7630.8         0.937 [0.93-0.94]   2/2      10 64 chunks
fir          4 rayon-iter           8213.7   2.6 8001.7..8425.8                                        
fir          4 parlay               8268.8   0.1 8259.9..8277.8                                        
fir          4 tbb                  8333.3   2.8 8096.0..8570.7                                        
fir          4 rayon-join           8560.0   3.3 8276.9..8843.1                                        
fir          4 static               9395.9  11.4 8327.8..10464.1                                       excursions retained
fir          4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7542.2   0.7 7485.8..7598.7         0.949 [0.93-0.97]   2/2      20 64 chunks
fir          8 tbb                  7951.1   2.7 7740.2..8162.0                                        
fir          8 rayon-iter           8563.2   4.4 8185.6..8940.9                                        
fir          8 rayon-join           8623.3   4.8 8208.2..9038.5                                        
fir          8 parlay               8778.9   7.2 8144.4..9413.4                                        
fir          8 static              14083.2  14.4 12061.2..16105.2                                      excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27814.2   3.5 26831.0..28797.4                                      
fir          1 wf-seq              27872.5   1.9 27329.3..28415.6                                      
fir          1 serial              31760.0   0.5 31592.0..31927.9                                      
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
```

After the change, `PASSES=2 CALLS=5`:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 12997's current affinity list: 0-3  date=2026-09-11T12:57:00Z
run=local  compiler=78867abdb5119d0751de1514a0befeaeeb5d6724  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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
passes=2 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 rayon-join          13654.1   0.0 13654.0..13654.3          27030.4                                        
mandelbrot   2 tbb                 13670.6   0.2 13647.7..13693.6          26947.7                                        
mandelbrot   2 parlay              13696.4   1.3 13516.3..13876.5          27037.8                                        
mandelbrot   2 rayon-iter          13759.8   0.2 13728.3..13791.3          27217.9                                        
mandelbrot   2 wf                  14112.2   3.3 13653.0..14571.4          25647.0 1.039 [1.01-1.07]  0.954   0/2       3 16 chunks
mandelbrot   2 static              26578.4   0.4 26465.1..26691.7          54470.1                                        excursions retained
mandelbrot   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6837.6   0.9 6779.2..6895.9            25629.9                                        
mandelbrot   4 wf                   7190.8   2.9 6979.9..7401.7            26023.8 1.051 [1.03-1.07]  1.015   0/2       6 16 chunks
mandelbrot   4 rayon-iter           7215.9   1.0 7141.8..7289.9            27491.5                                        
mandelbrot   4 parlay               7239.2   2.4 7063.3..7415.2            27832.6                                        
mandelbrot   4 rayon-join           7253.0   1.1 7171.8..7334.2            27976.4                                        
mandelbrot   4 static              26457.1   0.5 26318.3..26595.9         109658.2                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  7019.4   0.7 6967.7..7071.1            27423.4                                        
mandelbrot   8 rayon-join           7088.8   1.3 6997.3..7180.4            27650.6                                        
mandelbrot   8 parlay               7164.6   0.6 7121.9..7207.4            27976.0                                        
mandelbrot   8 wf                   7304.0   1.5 7195.5..7412.5            27436.0 1.041 [1.02-1.06]  1.000   0/2       7 16 chunks
mandelbrot   8 rayon-iter           7333.3   2.4 7157.4..7509.1            27705.5                                        
mandelbrot   8 static              31440.1   0.0 31435.4..31444.7         126604.8                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26275.9   0.2 26219.1..26332.8          26273.1                                        
mandelbrot   1 wf                  26814.0   0.4 26715.7..26912.2          26810.0                                        
mandelbrot   1 wf-seq              26915.6   0.6 26755.3..27075.8          26911.5                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8591.4   0.6 8542.4..8640.5            17168.6                                        
quadrature   2 wf                   8739.9   1.1 8647.8..8832.0            17046.5 1.017 [1.00-1.03]  0.993   0/2     399 
quadrature   2 static               9020.8   0.3 8990.4..9051.2            17149.8                                        excursions retained
quadrature   2 parlay               9740.8   4.1 9341.5..10140.0           18610.8                                        
quadrature   2 rayon-join-left      9863.1  10.5 8828.1..10898.1           19636.7                                        
quadrature   2 tbb                 10281.9   0.7 10212.1..10351.7          20528.0                                        
quadrature   2 parlay-left         13974.9   4.1 13402.1..14547.6          21609.1                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   5001.8   3.2 4843.8..5159.8            17738.4 0.861 [0.78-0.95]  0.874   2/2    1010 
quadrature   4 rayon-join-left      5993.5   4.7 5710.3..6276.7            22809.2                                        
quadrature   4 tbb                  6161.6   1.4 6075.8..6247.4            24072.5                                        
quadrature   4 static               6317.1  13.6 5454.9..7179.3            24288.9                                        excursions retained
quadrature   4 rayon-join           7084.6   7.8 6530.8..7638.4            24115.7                                        
quadrature   4 parlay               7552.1   3.7 7275.6..7828.7            22949.6                                        
quadrature   4 parlay-left          9924.4   1.4 9787.2..10061.5           26482.6                                        
quadrature   4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5481.8   2.7 5333.5..5630.1            23776.2 0.878 [0.84-0.91]  0.957   2/2    1202 
quadrature   8 rayon-join           6257.2   6.6 5847.1..6667.3            24585.9                                        
quadrature   8 parlay               6852.7  12.9 5966.3..7739.0            26695.3                                        
quadrature   8 tbb                  6927.3   3.4 6692.4..7162.2            27370.7                                        
quadrature   8 rayon-join-left      6954.0  13.6 6010.1..7897.8            27274.1                                        
quadrature   8 parlay-left          7164.0   1.5 7054.2..7273.9            23837.7                                        
quadrature   8 static             531992.7   0.7 528009.9..535975.5      2102926.3                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              14835.9   3.5 14314.3..15357.5          14834.2                                        
quadrature   1 wf-seq              15806.5   0.4 15736.0..15877.0          15804.5                                        
quadrature   1 wf                  16764.2   2.7 16310.8..17217.6          16756.9                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 tbb                 16316.2   0.2 16284.6..16347.9          32476.3                                        
records      2 static              16534.4   1.3 16311.3..16757.6          32507.8                                        excursions retained
records      2 rayon-iter          17062.1   1.0 16886.9..17237.4          33096.6                                        
records      2 rayon-join          17172.5   2.2 16787.2..17557.9          33864.2                                        
records      2 wf                  18240.4   2.3 17826.2..18654.6          35665.0 1.118 [1.09-1.15]  1.098   0/2       1 32 chunks
records      2 parlay              19916.3   4.8 18956.5..20876.1          38064.9                                        
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

records      4 tbb                  8436.3   0.5 8393.8..8478.7            30733.7                                        
records      4 rayon-iter           8559.2   3.7 8244.5..8873.9            33004.1                                        
records      4 parlay               8923.9   6.5 8339.6..9508.2            29436.6                                        
records      4 static               8978.6   0.7 8911.8..9045.5            38593.9                                        excursions retained
records      4 rayon-join           9169.3   1.2 9054.8..9283.9            35305.7                                        
records      4 wf                   9769.0   2.0 9569.1..9969.0            34900.8 1.178 [1.15-1.21]  1.176   0/2      11 64 chunks
records      4 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      8 tbb                  8733.1   0.4 8701.6..8764.7            33338.1                                        
records      8 rayon-iter           8888.8   0.3 8860.3..8917.3            33682.8                                        
records      8 parlay               8988.2   6.1 8443.1..9533.3            35436.2                                        
records      8 rayon-join           9088.4   0.4 9049.2..9127.5            34605.7                                        
records      8 wf                  10066.2   1.2 9948.1..10184.4           37801.8 1.171 [1.14-1.21]  1.140   0/2      20 64 chunks
records      8 static              18302.9   8.6 16734.6..19871.2          78184.2                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32681.2   2.0 32032.6..33329.7          32676.5                                        
records      1 wf-seq              35541.3   1.2 35114.0..35968.7          35537.1                                        
records      1 wf                  35806.7   1.6 35245.5..36367.8          35791.7                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14075.4   2.8 13683.6..14467.2          26341.1 0.872 [0.86-0.89]  0.846   2/2       1 32 chunks
fir          2 tbb                 16341.1   0.1 16328.9..16353.2          31531.7                                        
fir          2 parlay              16408.8   1.0 16243.3..16574.2          31672.0                                        
fir          2 static              16412.1   0.4 16354.1..16470.0          34402.4                                        excursions retained
fir          2 rayon-join          16627.5   4.2 15932.2..17322.7          32694.7                                        
fir          2 rayon-iter          16786.0   0.9 16632.3..16939.7          32774.3                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   7758.3   1.1 7670.0..7846.7            26655.7 0.915 [0.89-0.94]  0.835   2/2       9 64 chunks
fir          4 rayon-join           8701.8   5.8 8193.2..9210.3            32412.0                                        
fir          4 static               8753.5   0.3 8723.4..8783.7            32765.0                                        excursions retained
fir          4 tbb                  8987.8   4.8 8557.7..9417.8            32052.5                                        
fir          4 parlay               9336.3   4.1 8955.2..9717.4            35737.8                                        
fir          4 rayon-iter          10336.8  13.6 8928.1..11745.5           33148.8                                        
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   8278.0   0.8 8208.2..8347.8            30082.5 0.986 [0.96-1.01]  0.947   1/2      17 64 chunks
fir          8 rayon-iter           8560.9   0.7 8504.7..8617.0            32587.9                                        
fir          8 tbb                  8829.0   6.3 8273.4..9384.5            32765.5                                        
fir          8 rayon-join           8843.2   3.5 8530.3..9156.1            32795.5                                        
fir          8 parlay               9010.5   1.2 8904.3..9116.7            34789.8                                        
fir          8 static              17869.1   0.5 17786.4..17951.8          77139.9                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27687.4   2.6 26975.1..28399.7          27683.3                                        
fir          1 wf-seq              28344.9   1.8 27824.6..28865.2          28340.0                                        
fir          1 serial              32017.7   0.7 31804.2..32231.2          32008.0                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), one random victim per spin round, at `d47223c0`

**This step did not meet its acceptance and does not land.** Its record is here
because a measured refusal is the result, and because what it refused is not
what it looked like it was refusing.

The change measured: in `wf__par_wait` and `wf__par_worker_main`, the per-round
`wf__par_find` — a rotation over *every other lane*, two sequentially-consistent
remote loads each — became one victim drawn from the lane's own LCG and one
`wf__par_steal`, with the round budget left at 4,096 and the yield budget at 16.
The join's help-first `wf__par_pop` was untouched, and the idle worker's scan
*after* it announces its idle bit stayed a full scan, because that one is the
announce-then-scan handshake and a probe there could park on work whose wake had
already been missed. Two runs per arm, alternating arms, one tree, plain `--par`
with `WF_PAR_CONTROL_FLAGS` empty in all four manifests.

**Acceptance.** No kernel's wall ratio was to regress beyond that row's MAD at
any width, and the W=4 CPU ratio was to fall on quadrature and records.

*Wall: met.* Over four runs no row regresses reproducibly. Reading each row as
the pair of control readings against the pair of probe readings at the recorded
W=4: mandelbrot 1.037/1.037 against 1.012/1.071, quadrature 1.013/0.925 against
1.004/0.992, records 1.180/1.223 against 1.164/1.156, fir 0.983/0.925 against
0.924/0.919. The two rows where both probe readings sit outside both control
readings are records W=4 and fir W=8, and the probe is the better one in both.

*W=4 CPU ratio: not met.* quadrature went **0.852/0.818 to 0.926/0.863** — higher
in both pairings — and records **1.227/1.331 to 1.262/1.159**, which straddles.
Neither fell. Nor did the absolute figure: quadrature's W=4 `wf` process CPU read
18.2/17.0 ms against 17.0/17.3, and records 36.0/37.9 against 36.5/37.8.

**What actually moved, and why the criterion could not have been met by this
change.** The one reproducible CPU effect is a *rise*, on mandelbrot, at every
parallel width: W=2 25.0/25.1 ms to 27.5/27.0, W=4 24.6/23.7 to 27.1/27.2, W=8
28.0/25.8 to 27.4/27.7, with the wall unchanged. Mandelbrot is the kernel with
the most idle lane-time in the block — 16 chunks, a median of six or seven
steals at W=4 — so it is where a change to the idle path shows first.

The cause is not the probe. It is that **the round count is the residency**, and
cheapening the round shortened it. Measured on this host (with the park-and-wake
probe the next step adds): one full-scan round costs **18.9 ns** at four lanes
uncontended and one single-victim round **3.2 ns**, so an unchanged budget of
4,096 rounds is a spin window of **77.4 us** before the change and **13.1 us**
after — against a measured park-and-wake of **16.3 us** on the same host. The
change therefore moved two things at once: the cost of a round, which it was
meant to move, and the time a lane stays hot before parking, which it was not.
On this host the second dominates, and it dominates in the direction the
recorded caution predicted — "backoff on the steal scan is monotonically worse
because pickup latency dominates probe traffic"
(`mcts_mem/whitefoot/parallelism/runtime.md`, 2026-08-22) — a lane that parks
sooner pays 16.3 us to be woken again more often than it saves by not scanning.

So §5a as specified is not a one-variable change, and no probe with an unchanged
round count could have met a criterion that asks CPU to fall while the residency
falls with it. The experiment that would settle the probe on its own merits
holds the *window* fixed rather than the count — about 24,000 probe rounds for
this host's 77 us, or a bound expressed in time — and is not run here. The
spin-round sweep in the section that follows separates the two effects directly
and confirms this reading: at 256 and 1,024 **full-scan** rounds, mandelbrot's
process CPU rises to the same 27 ms the probe produced, so the rise tracks the
window and not the round.

**One dimension of the original hypothesis is untested here.** The idea being
probed was that the spin costs most on hosts with SMT siblings, where a spinning
lane slows the lane doing work. This host is four cores with **one thread per
core** and has no SMT siblings at all, so it cannot answer that question; the
hosted `ubuntu-24.04` runner, two cores with two threads each, can. Nothing on
this branch puts the probe in front of that runner, because the change does not
land.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `d47223c0` in all four runs; the two arms differ only in
  `compiler/src/backend/sched/core.c`, which the compiler does not read — the
  four emitted `--par` modules are byte-identical across all four runs and
  identical to the plain table at `53359d73` (`mandelbrot-par.ll`
  `f07c190e3a396fc9…`, `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll`
  `9ceebaedf90e4a60…`, `fir-par.ll` `e0af1ed03ef2bc52…`)
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` empty in all four manifests
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host `x86_64-unknown-linux-gnu`
  cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes and emitted chunk counts: unchanged in every run — mandelbrot 98,304
  points at limit 256, **16 chunks**; records 131,072 records, **32/64/64**
  chunks at W=2/4/8; fir K=64 over N=524,288, **32/64/64**; quadrature M=64, no
  independent-map split, `chunks=na`
- workflow run: `local`. The push carrying this record touches
  `compiler/src/backend/sched/**` only through the step that does not land, so
  the hosted `compute-bench` workflow runs on both runners against a tree whose
  runtime is unchanged; its tables are a reading of the control, not of the
  probe.
- sizing window, all four runs: every `wf-seq` median at W=1 inside [5 ms,
  60 ms] — mandelbrot 27.013 / 26.701 / 26.790 / 26.739 ms, quadrature 16.641 /
  16.561 / 16.241 / 16.426, records 36.684 / 36.573 / 35.499 / 36.062, fir
  27.849 / 28.025 / 27.632 / 27.309; every `wf` median at the recorded W=4 above
  1 ms; and `steals > 0` on the `wf` row at every parallel width. No size
  constant changed.

Control, the tree at `d47223c0` with the full scan, first run:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 14693's current affinity list: 0-3  date=2026-09-11T13:03:24Z
run=local  compiler=d47223c0dedbf90e4dc928b6c5bbad0b4acc1a67  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13386.4   0.2 13342.7..13523.1          26603.5                                        
mandelbrot   2 rayon-join          13549.7   0.4 13490.5..13933.8          26954.2                                        
mandelbrot   2 rayon-iter          13628.4   0.5 13465.8..13714.7          26958.1                                        
mandelbrot   2 parlay              13727.3   0.6 13463.5..13865.0          27073.8                                        
mandelbrot   2 wf                  13768.5   0.7 13607.2..13980.9          25022.3 1.030 [1.02-1.04]  0.947   0/5       3 16 chunks
mandelbrot   2 static              26660.3   0.7 26307.2..26838.5          54392.2                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6863.8   1.6 6755.0..7161.4            26319.3                                        
mandelbrot   4 parlay               7076.4   0.4 7046.6..7292.7            27288.6                                        
mandelbrot   4 rayon-iter           7248.0   1.5 7023.9..7379.3            27744.6                                        
mandelbrot   4 rayon-join           7279.4   0.2 7049.8..7394.0            27546.0                                        
mandelbrot   4 wf                   7295.2   3.3 6944.5..7636.9            24623.0 1.037 [1.00-1.11]  0.935   1/5       6 16 chunks
mandelbrot   4 static              26958.8   0.8 26336.0..27186.9         109602.3                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6988.2   1.3 6826.4..7203.3            27542.4                                        
mandelbrot   8 rayon-join           7007.6   0.3 6967.3..7119.2            27394.2                                        
mandelbrot   8 rayon-iter           7241.9   1.2 7156.0..7580.5            27676.8                                        
mandelbrot   8 parlay               7249.5   2.8 6971.1..7499.8            28298.5                                        
mandelbrot   8 wf                   7348.4   2.0 7161.0..7973.3            27973.1 1.072 [1.03-1.14]  1.016   0/5       7 16 chunks
mandelbrot   8 static              31687.1   0.9 31392.4..37792.7         127501.8                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26742.2   0.4 26292.6..27020.8          26738.9                                        
mandelbrot   1 wf                  26875.5   0.2 26627.2..27017.2          26870.4                                        
mandelbrot   1 wf-seq              27012.8   1.1 26553.5..27311.7          27010.1                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 wf                   8581.2   0.8 8510.4..10143.8           16595.7 1.031 [0.97-1.13]  1.022   2/5     428 
quadrature   2 static               8859.7   1.3 8690.8..9053.3            16892.6                                        excursions retained
quadrature   2 rayon-join           9201.4  12.0 8099.7..10780.7           18204.6                                        
quadrature   2 rayon-join-left      9742.1   1.9 8719.5..10831.4           19384.4                                        
quadrature   2 parlay              10382.2   3.6 10009.9..11223.6          18876.0                                        
quadrature   2 tbb                 10536.8   2.4 10030.3..13017.6          20919.7                                        
quadrature   2 parlay-left         15418.5   4.2 13648.2..16537.7          22389.5                                        
quadrature   2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5237.3   4.8 4898.8..7120.1            20723.8                                        
quadrature   4 rayon-join-left      5436.1   5.0 4680.4..6910.3            21525.1                                        
quadrature   4 wf                   5449.7   6.0 4996.8..6319.8            18207.7 1.013 [0.92-1.23]  0.852   2/5    1009 
quadrature   4 tbb                  6419.3   3.4 6198.3..7082.0            25412.2                                        
quadrature   4 static               6770.6   8.5 5611.9..7347.5            30271.9                                        excursions retained
quadrature   4 parlay               7306.3   5.7 6886.6..8075.8            23542.5                                        
quadrature   4 parlay-left         10861.9   6.1 9872.3..11819.1           29298.6                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5266.5   3.6 4803.4..5542.3            17528.4 0.899 [0.86-1.02]  0.773   4/5    1162 
quadrature   8 rayon-join           5735.5   2.9 5430.1..6172.2            22477.7                                        
quadrature   8 rayon-join-left      6415.1   2.5 6176.1..7773.1            25057.3                                        
quadrature   8 parlay               6638.5  11.6 5719.4..7439.8            25918.8                                        
quadrature   8 tbb                  6853.5   3.1 6246.7..7065.7            26993.7                                        
quadrature   8 parlay-left          6941.8   1.3 6213.3..7287.6            24291.5                                        
quadrature   8 static             556252.1   5.8 519983.3..591987.1      2207078.0                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15685.6   0.4 15503.2..15881.5          15683.7                                        
quadrature   1 wf                  16327.3   3.3 14608.3..16871.7          16325.8                                        
quadrature   1 wf-seq              16641.2   2.2 16208.3..17751.8          16640.2                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 static              16612.2   2.4 16220.4..17550.1          32609.7                                        excursions retained
records      2 parlay              16737.4   2.0 16408.4..18156.1          31764.8                                        
records      2 rayon-iter          16896.0   2.5 16466.3..20157.4          33273.1                                        
records      2 rayon-join          16963.1   2.1 16545.9..17313.4          33149.1                                        
records      2 tbb                 17093.1   1.2 16326.6..17944.9          33725.1                                        
records      2 wf                  18619.2   2.1 18223.1..20946.5          34747.4 1.143 [1.11-1.24]  1.111   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen

records      4 parlay               8407.0   1.0 8292.4..10060.6           27504.1                                        
records      4 rayon-join           8865.8   1.7 8540.7..9488.7            33446.3                                        
records      4 tbb                  8888.7   3.6 8341.4..9205.7            32494.8                                        
records      4 rayon-iter           9261.7   1.9 9084.0..11503.6           34544.1                                        
records      4 static               9315.9   6.0 8532.0..12549.5           44359.7                                        excursions retained
records      4 wf                  10069.0   2.4 9747.9..10715.1           36003.1 1.180 [1.18-1.28]  1.227   0/5      10 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen

records      8 parlay               8513.7   1.0 8428.7..11400.9           33591.3                                        
records      8 tbb                  8598.5   0.1 8380.6..8685.2            33112.3                                        
records      8 rayon-join           8890.3   0.9 8477.5..8966.0            33613.3                                        
records      8 rayon-iter           9032.6   1.0 8944.7..9316.2            34866.4                                        
records      8 wf                  10231.9   1.8 9797.6..10854.1           38866.3 1.202 [1.14-1.30]  1.161   0/5      21 64 chunks
records      8 static              15898.7   3.8 14148.7..18690.4          63441.4                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              33091.9   1.3 32276.4..33512.3          33087.6                                        
records      1 wf                  36050.1   1.5 35499.8..37792.8          36044.5                                        
records      1 wf-seq              36683.5   1.3 35690.6..37467.8          36624.1                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14439.7   1.3 14180.2..14803.8          27091.2 0.901 [0.85-0.92]  0.841   5/5       1 32 chunks
fir          2 parlay              16410.9   1.4 16130.4..17209.6          31325.5                                        
fir          2 static              16687.1   2.3 16118.9..17314.1          33074.3                                        excursions retained
fir          2 tbb                 16732.9   1.7 16278.1..17019.2          31416.3                                        
fir          2 rayon-iter          16858.7   0.2 16403.6..16890.8          32163.0                                        
fir          2 rayon-join          16886.4   4.0 15738.3..17558.1          32845.9                                        
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

fir          4 wf                   7991.8   4.2 7652.7..10037.6           28233.4 0.983 [0.92-1.07]  0.913   3/5      11 64 chunks
fir          4 tbb                  8280.0   1.8 8128.4..11426.0           31175.6                                        
fir          4 rayon-join           8354.1   3.5 8065.8..10244.4           32239.9                                        
fir          4 rayon-iter           8652.7   3.2 8330.6..9373.0            32190.4                                        
fir          4 parlay               8706.1   4.3 8229.6..9806.1            33012.9                                        
fir          4 static              10604.5  21.3 8293.1..19582.6           46427.0                                        excursions retained
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7805.5   2.7 7518.7..8974.2            28884.4 0.953 [0.92-1.03]  0.916   4/5      21 64 chunks
fir          8 tbb                  8393.4   2.4 8192.8..8787.9            32100.6                                        
fir          8 rayon-join           8508.1   2.9 8261.1..10084.2           31794.8                                        
fir          8 parlay               8863.9   0.8 8789.1..11443.3           33771.8                                        
fir          8 rayon-iter           9066.9   4.4 8199.1..10144.0           32790.5                                        
fir          8 static              14144.2  10.3 12207.3..15605.1          62001.0                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27848.7   1.5 26826.0..28451.0          27844.0                                        
fir          1 wf                  28082.0   5.0 26676.1..39562.5          28078.4                                        
fir          1 serial              32559.0   1.7 31606.4..33097.7          32555.0                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```

Candidate, one random victim per spin round, first run:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 18600's current affinity list: 0-3  date=2026-09-11T13:06:48Z
run=local  compiler=d47223c0dedbf90e4dc928b6c5bbad0b4acc1a67  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13469.6   0.4 13321.2..13686.4          26757.1                                        
mandelbrot   2 rayon-iter          13471.1   0.6 13391.7..13681.5          26679.2                                        
mandelbrot   2 rayon-join          13551.1   0.6 13421.2..13626.3          26897.3                                        
mandelbrot   2 parlay              13580.6   0.1 13476.4..13598.0          26775.4                                        
mandelbrot   2 wf                  14031.6   0.4 13567.5..14092.6          27458.5 1.043 [1.01-1.05]  1.029   0/5       3 16 chunks
mandelbrot   2 static              26440.0   0.2 26168.9..26495.8          54286.2                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6842.8   1.1 6699.9..6957.7            26054.7                                        
mandelbrot   4 wf                   7029.4   2.3 6867.4..7710.9            27051.4 1.012 [1.00-1.11]  1.025   0/5       7 16 chunks
mandelbrot   4 parlay               7132.4   1.0 6908.9..7206.0            27246.3                                        
mandelbrot   4 rayon-iter           7248.5   2.5 6992.7..7783.4            27512.2                                        
mandelbrot   4 rayon-join           7372.9   2.6 7024.1..10442.2           27610.1                                        
mandelbrot   4 static              26526.6   0.4 26326.7..26817.4         110148.4                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6919.2   1.1 6818.6..6997.0            27477.2                                        
mandelbrot   8 rayon-join           7058.4   1.8 6928.6..7315.9            27590.4                                        
mandelbrot   8 parlay               7299.9   3.1 6867.1..7544.9            28563.3                                        
mandelbrot   8 wf                   7383.4   2.5 7009.6..10319.1           27401.0 1.075 [1.03-1.48]  1.008   0/5      10 16 chunks
mandelbrot   8 rayon-iter           7486.4   2.1 7262.5..7798.9            28022.1                                        
mandelbrot   8 static              37789.0  16.5 31509.6..47419.5         145562.1                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26489.4   0.1 26263.4..26568.7          26486.9                                        
mandelbrot   1 wf                  26738.4   0.4 26537.9..27009.5          26734.4                                        
mandelbrot   1 wf-seq              26790.3   0.8 26564.9..27597.0          26787.0                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8551.7   1.7 8051.4..8893.1            17060.7                                        
quadrature   2 wf                   8600.8   1.0 8210.3..8836.8            16496.5 1.006 [0.96-1.04]  0.966   2/5     419 
quadrature   2 rayon-join-left      9028.9   2.2 8585.6..9225.3            18048.2                                        
quadrature   2 static               9179.6   1.4 8855.2..9312.0            17331.7                                        excursions retained
quadrature   2 parlay              10084.8   3.6 9718.6..10751.2           17800.0                                        
quadrature   2 tbb                 10238.8   0.9 9586.1..10612.3           20456.6                                        
quadrature   2 parlay-left         12650.4   3.6 12197.9..13933.6          20179.6                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   4879.0   5.3 4619.1..5902.8            16988.8 1.004 [0.92-1.14]  0.926   2/5    1067 
quadrature   4 rayon-join           4963.6   7.2 4606.9..7345.3            19742.6                                        
quadrature   4 rayon-join-left      5213.8   3.5 5004.6..5426.6            19925.8                                        
quadrature   4 static               5474.6   6.2 5132.6..8016.5            17604.2                                        excursions retained
quadrature   4 tbb                  5974.7   2.4 5829.4..7211.5            23659.8                                        
quadrature   4 parlay               7628.6   6.8 6777.0..9003.4            22334.7                                        
quadrature   4 parlay-left         10201.5   1.6 9125.0..10851.9           26884.6                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   4922.9   4.0 4727.2..5622.6            16999.1 0.971 [0.80-1.05]  0.782   3/5    1217 
quadrature   8 parlay               5878.9  10.1 4591.5..7039.4            22101.6                                        
quadrature   8 rayon-join           5913.0   4.9 5623.4..6844.3            22842.6                                        
quadrature   8 tbb                  6282.1   2.7 5905.7..6454.6            24875.7                                        
quadrature   8 rayon-join-left      6436.4  10.2 5334.2..8406.3            25426.2                                        
quadrature   8 parlay-left          6634.1  23.6 5069.5..12437.6           26806.2                                        
quadrature   8 static             512014.1   0.0 511997.5..568027.7      2044644.1                                        excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15583.5   1.2 14002.7..15763.7          15581.8                                        
quadrature   1 wf-seq              16241.1   2.1 14348.0..16579.4          16239.3                                        
quadrature   1 wf                  16372.6   0.5 14689.4..16537.0          16371.1                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 static              16660.5   2.4 16100.0..18281.2          36403.7                                        excursions retained
records      2 tbb                 16870.0   0.5 16451.9..17064.2          33021.8                                        
records      2 rayon-join          16896.1   3.2 15938.5..17436.5          33568.0                                        
records      2 parlay              17015.5   2.0 16323.4..19793.5          31688.2                                        
records      2 rayon-iter          17031.4   1.6 16285.1..19041.0          33478.6                                        
records      2 wf                  18336.9   0.4 18023.7..19068.6          35979.9 1.126 [1.08-1.17]  1.116   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-join   FASTEST = static       WF fastest: no
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

records      4 parlay               8536.0   4.6 8111.2..9366.8            28259.1                                        
records      4 tbb                  8617.8   0.9 8211.5..8730.9            32770.2                                        
records      4 rayon-iter           8862.2   2.8 8230.8..9871.5            33813.4                                        
records      4 static               8926.8   6.1 8380.5..17141.8           33474.1                                        excursions retained
records      4 rayon-join           9014.7   4.5 8512.2..9562.9            35326.1                                        
records      4 wf                   9722.3   1.9 9440.0..9935.1            36543.3 1.164 [1.14-1.19]  1.262   0/5       6 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      8 tbb                  8400.6   1.4 8286.9..9071.6            32499.7                                        
records      8 rayon-iter           8711.9   3.8 8378.2..10419.1           33869.7                                        
records      8 rayon-join           8858.0   2.6 8356.8..9791.6            33807.7                                        
records      8 parlay               8899.3   5.4 8377.4..10337.6           35181.0                                        
records      8 wf                   9622.4   2.1 9420.3..10313.9           36537.3 1.140 [1.12-1.24]  1.104   0/5      20 64 chunks
records      8 static              15975.3   8.1 12099.6..17268.8          63256.0                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32095.0   1.4 31651.0..34088.1          32089.9                                        
records      1 wf-seq              35499.1   1.0 35140.2..36816.0          35495.6                                        
records      1 wf                  35798.6   1.3 35332.4..36868.8          35793.5                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

fir          2 wf                  14067.5   2.3 13743.4..16302.5          27275.0 0.884 [0.85-1.07]  0.890   4/5       1 32 chunks
fir          2 static              16011.6   0.6 15909.2..16475.7          32011.9                                        excursions retained
fir          2 rayon-join          16266.4   1.3 15265.7..16474.2          31381.0                                        
fir          2 rayon-iter          16291.9   0.7 16170.4..16666.2          31749.4                                        
fir          2 tbb                 16332.0   2.6 15716.4..17132.6          31221.5                                        
fir          2 parlay              16500.6   1.6 15702.1..16763.6          31971.7                                        
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   7717.2   0.9 7382.2..7785.2            27575.2 0.924 [0.87-0.94]  0.854   5/5       8 64 chunks
fir          4 static               8243.7   2.2 8010.4..10970.5           32245.4                                        excursions retained
fir          4 tbb                  8296.0   1.6 8161.5..11314.5           31359.2                                        
fir          4 parlay               8484.9   1.0 8299.0..9764.0            32232.3                                        
fir          4 rayon-join           8729.1   2.7 8344.2..8962.9            32884.6                                        
fir          4 rayon-iter           8801.7   1.9 8601.9..9802.5            32789.6                                        
fir          4 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   8071.6   3.5 7500.3..8470.3            28726.9 0.945 [0.90-1.00]  0.912   4/5      17 64 chunks
fir          8 rayon-join           8463.1   1.7 8315.1..9145.0            31458.2                                        
fir          8 tbb                  8564.8   3.4 8277.3..9139.1            32020.9                                        
fir          8 rayon-iter           8731.5   1.5 8373.2..8985.8            32724.3                                        
fir          8 parlay               8969.4   1.9 8402.0..9142.3            34485.5                                        
fir          8 static              12606.8   1.4 9698.5..13148.7           48777.3                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27442.0   1.0 26807.3..28419.9          27437.7                                        
fir          1 wf-seq              27632.2   1.3 26836.4..28515.1          27628.3                                        
fir          1 serial              31970.0   1.0 29859.7..32291.3          31965.7                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
```

The second run of each arm is not reproduced as a whole table: it differs from
the one above it only in its numbers, and it is here to bound this host's spread
over identical bytes rather than to record a second configuration. Its `wf` and
`wf-seq` rows, control then candidate:

```text
mandelbrot   2 wf                  13925.3   0.7 13463.1..14017.9          25113.9 1.026 [1.02-1.04]  0.964   0/5       3 16 chunks
mandelbrot   4 wf                   7029.0   3.0 6817.4..7448.7            23695.7 1.037 [1.00-1.05]  0.899   1/5       7 16 chunks
mandelbrot   8 wf                   7588.4   1.2 7424.7..8730.6            25799.2 1.096 [1.06-1.23]  0.956   0/5       7 16 chunks
mandelbrot   1 wf-seq              26700.7   1.0 26357.6..27074.9          26686.1                                        
mandelbrot   1 wf                  26712.6   0.4 26469.8..27083.8          26709.3                                        
quadrature   2 wf                   8717.3   3.3 8300.5..9364.6            16821.4 1.011 [0.95-1.09]  0.992   1/5     415 
quadrature   4 wf                   5026.0   1.5 4685.2..5607.2            16997.0 0.925 [0.88-1.12]  0.818   4/5    1012 
quadrature   8 wf                   5121.5   1.5 5046.0..5376.7            17337.1 0.975 [0.96-1.00]  0.855   3/5    1184 
quadrature   1 wf                  16420.0   1.7 15609.0..20674.6          16418.8                                        
quadrature   1 wf-seq              16561.1   0.5 16098.6..16667.6          16559.7                                        
records      2 wf                  18258.8   1.4 17998.3..18845.2          35487.6 1.105 [1.08-1.15]  1.096   0/5       1 32 chunks
records      4 wf                  10155.8   2.7 9861.3..12597.8           37859.9 1.223 [1.15-1.51]  1.331   0/5      12 64 chunks
records      8 wf                   9779.3   2.9 9357.4..10071.5           37196.2 1.128 [1.11-1.19]  1.113   0/5      21 64 chunks
records      1 wf-seq              36573.0   2.2 35576.0..38609.0          36568.5                                        
records      1 wf                  36690.9   1.1 35496.3..37086.7          36686.1                                        
fir          2 wf                  14468.0   4.4 13584.0..17178.1          27469.8 0.891 [0.87-1.08]  0.871   4/5       1 32 chunks
fir          4 wf                   7772.0   3.0 7371.9..8003.1            27644.0 0.925 [0.89-0.98]  0.873   5/5      11 64 chunks
fir          8 wf                   7854.0   2.2 7540.7..9078.2            28726.1 0.976 [0.90-1.07]  0.948   4/5      19 64 chunks
fir          1 wf                  27479.1   1.0 26964.8..27764.6          27462.3                                        
fir          1 wf-seq              28025.4   0.6 27771.3..28876.5          28022.2                                        
```

```text
mandelbrot   2 wf                  13728.0   0.2 13697.5..14224.0          26975.7 1.023 [1.01-1.04]  1.012   0/5       3 16 chunks
mandelbrot   4 wf                   7303.5   2.3 7136.9..7775.3            27244.3 1.071 [1.04-1.15]  1.006   0/5       7 16 chunks
mandelbrot   8 wf                   7483.0   1.7 7078.3..9347.1            27671.9 1.091 [1.03-1.37]  1.008   0/5       9 16 chunks
mandelbrot   1 wf-seq              26738.8   0.6 26567.1..27081.3          26736.0                                        
mandelbrot   1 wf                  26797.4   0.5 26552.3..27542.3          26737.5                                        
quadrature   2 wf                   8934.1   3.0 8553.9..9527.0            16800.0 1.029 [0.99-1.05]  1.007   1/5     426 
quadrature   4 wf                   4949.4   1.8 4862.2..5266.2            17302.9 0.992 [0.96-1.10]  0.863   3/5    1037 
quadrature   8 wf                   5173.1   8.1 4455.6..5594.4            17486.6 0.932 [0.81-1.14]  0.843   4/5    1224 
quadrature   1 wf                  16299.3   5.0 15481.7..17646.3          16297.7                                        
quadrature   1 wf-seq              16425.9   4.1 15236.3..17197.4          16423.7                                        
records      2 wf                  18449.8   3.4 17830.0..25296.9          36286.6 1.127 [1.09-1.49]  1.229   0/5       1 32 chunks
records      4 wf                   9955.5   1.3 9694.1..12736.0           37817.3 1.156 [1.14-1.52]  1.159   0/5       6 64 chunks
records      8 wf                  10330.7   2.5 9951.8..10655.3           37952.1 1.157 [1.14-1.21]  1.108   0/5      18 64 chunks
records      1 wf-seq              36061.9   0.4 35910.4..39982.2          36058.7                                        
records      1 wf                  36116.6   0.8 35830.1..44459.0          36112.3                                        
fir          2 wf                  14497.8   3.1 14041.9..15245.5          28130.3 0.904 [0.86-0.96]  0.919   5/5       1 32 chunks
fir          4 wf                   7759.5   1.3 7560.0..8724.4            27583.2 0.919 [0.92-1.05]  0.935   4/5       9 64 chunks
fir          8 wf                   7729.4   3.7 7441.4..8914.9            28528.9 0.928 [0.82-1.12]  0.890   4/5      19 64 chunks
fir          1 wf                  27293.2   0.6 27019.1..27676.3          27288.3                                        
fir          1 wf-seq              27308.9   1.3 26594.7..28007.9          27304.8                                        
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the spin bound sized to a measured park-and-wake, at `d47223c0`

**`WF_PAR_SPIN_ROUNDS` moves from 4,096 to 1,024.** The constant was never
measured — it is a count of misses standing in for a length of time, and neither
end of that substitution had a number on this host. Both now do.

**The measurement.** `compiler/src/backend/sched/wake_probe.c` parks and wakes
through the core's own `wf__par_signal`, `posted` flag and `wf_prim_wait_sleep`
— the same path an idle worker takes — two threads alternating so that each side
genuinely sleeps once per round, and reports the median over 2,000 rounds after
200 warm-up rounds. It also times 200,000 uncontended `wf__par_find` rounds over
a prepared four-lane pool of empty deques. Seven consecutive runs on this host
read a round trip of 32.7 / 31.4 / 31.7 / 32.4 / 35.1 / 34.6 / 32.5 us, so
**one park-and-wake is 16.3 us** (the median of the halves: 16.36, 15.72, 15.84,
16.22, 17.53, 17.31, 16.26), and **one spin round is 18.9 ns** (18.6 to 19.7 in
the same seven). The probe is a measurement and never a timed table row: it
applies no threshold to either number, it passes on a round having really
parked, and nothing in the compiler or the scoreboard reads its output. That
16.3 us also sits beside the retired park-on-miss bundle's 16.2 us for a
different scheduler core on a different host — the agreement is a coincidence of
two Linux futex paths and not a transferred constant.

**The rule and the sweep.** The spin window each candidate buys is its round
count times 18.9 ns: **256 → 4.8 us, 1,024 → 19.4 us, 4,096 → 77.4 us**, against
a wake cost of 16.3 us. The choice is the value whose window is nearest that
cost among those that regress no kernel's wall ratio beyond its MAD at any
width. One plain `compare PASSES=5 CALLS=5` per value, yields fixed at 16; the
4,096 cell is the control pair recorded in the section above, on the same tree
and the same bytes, and is not reproduced here.

**256 is excluded on the wall.** Quadrature's W=4 ratio goes from 1.013/0.925 to
**1.182 with none of five paired passes lower**, on a row whose MAD is 1.5 to
7.8 percent, and its W=4 process CPU rises to 21.6 ms against 18.2/17.0. A 4.8 us
window is well under the 16.3 us it costs to undo a park, and the recursive
kernel — the one whose lanes miss and retry most — pays for it directly.

**1,024 regresses nothing beyond MAD and improves several rows.** Reading the
two 4,096 readings against the two 1,024 readings: fir **0.901/0.891 to
0.894/0.876** at W=2, **0.983/0.925 to 0.905/0.907** at W=4 and **0.953/0.976 to
0.941/0.904** at W=8 — better in every reading at every width; records W=4
**1.180/1.223 to 1.175/1.174**; mandelbrot W=8 **1.072/1.096 to 1.040/1.048**;
quadrature W=4 **1.013/0.925 to 1.004/0.924**, unchanged. The one row worse in
both readings is mandelbrot W=2, 1.030/1.026 to 1.032/1.036 — a rise of 0.58
percent on a row whose MAD reads 0.4 to 0.7 percent, inside it. So the rule
selects 1,024, and it is the value that would have been selected on the wall
alone.

**What it costs, stated rather than buried.** Mandelbrot's process CPU rises
with the shorter window exactly as the section above described: W=4 24.6/23.7 ms
at 4,096 against 27.3/27.4 at 1,024 and 27.0 at 256, with the wall flat. That is
the same effect the single-victim probe produced, reached here by shortening the
window with the round cost held fixed, which is what separates the two and
exonerates the probe. Mandelbrot is the block with the most idle lane-time — 16
chunks and a median of six or seven steals at W=4 — so a lane there parks,
is woken, and parks again; each cycle costs 16.3 us of mostly system time, and
below some window that churn costs more than staying hot. Everything else is
flat or better: quadrature's W=4 CPU reads 19.0/16.9 against 18.2/17.0, records
36.3/36.6 against 36.0/37.9, fir 27.3/27.3 against 28.2/27.6. The reading this
leaves open is whether the right shape is a longer window on the map kernels
rather than a shorter one everywhere, which is a time-shaped residency and is
brief §5c, blocked on a cadence column.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `d47223c0` in every run of the sweep; the arms differ only
  in `WF_PAR_SPIN_ROUNDS` in `compiler/src/backend/sched/core.c`, which the
  compiler does not read — the four emitted `--par` modules are byte-identical
  across every run of the sweep and identical to the plain table at `53359d73`
  (`mandelbrot-par.ll` `f07c190e3a396fc9…`, `quadrature-par.ll`
  `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`, `fir-par.ll`
  `e0af1ed03ef2bc52…`)
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` empty in every manifest
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host `x86_64-unknown-linux-gnu`
  cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes and emitted chunk counts: unchanged — mandelbrot 98,304 points at limit
  256, **16 chunks**; records 131,072 records, **32/64/64** chunks at W=2/4/8;
  fir K=64 over N=524,288, **32/64/64**; quadrature M=64, no independent-map
  split, `chunks=na`
- workflow run: `local`. The push carrying this change touches
  `compiler/src/backend/sched/**`, so the hosted `compute-bench` workflow runs
  on `ubuntu-24.04` (recorded W=4) and `macos-14` (recorded W=2) against the
  1,024-round bound. Those two tables are the quieter reading of this choice on
  hosts with different core counts and, on the Linux runner, with SMT siblings
  this host does not have; they are not waited on here and are not part of this
  section.
- sizing window: every `wf-seq` median at W=1 inside [5 ms, 60 ms] — at 256
  mandelbrot 27.066 ms, quadrature 16.276, records 36.377, fir 27.632; at 1,024
  mandelbrot 26.788 / 27.007, quadrature 16.411 / 16.105, records 37.001 /
  36.176, fir 27.132 / 27.084; every `wf` median at the recorded W=4 above 1 ms;
  and `steals > 0` on the `wf` row at every parallel width. No size constant
  changed.

`WF_PAR_SPIN_ROUNDS` = 256, a 4.8 us window:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 27618's current affinity list: 0-3  date=2026-09-11T13:19:23Z
run=local  compiler=d47223c0dedbf90e4dc928b6c5bbad0b4acc1a67  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 rayon-iter          13473.6   0.3 13433.1..14041.7          26732.8                                        
mandelbrot   2 tbb                 13515.4   0.2 13376.6..13555.8          26746.3                                        
mandelbrot   2 rayon-join          13579.3   0.3 13544.8..13873.9          26899.1                                        
mandelbrot   2 parlay              13615.2   0.9 13405.2..13863.4          26907.2                                        
mandelbrot   2 wf                  13960.6   1.0 13678.3..14277.0          27064.2 1.036 [1.02-1.05]  1.009   0/5       3 16 chunks
mandelbrot   2 static              26330.3   0.4 26176.8..26946.1          51537.1                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = rayon-iter   WF fastest: no
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6757.1   0.7 6703.3..6921.6            26164.0                                        
mandelbrot   4 rayon-iter           7076.4   0.5 6938.2..7727.6            27208.9                                        
mandelbrot   4 parlay               7108.2   1.3 6891.7..7300.8            27073.3                                        
mandelbrot   4 wf                   7131.7   2.1 6955.4..11284.6           26998.7 1.059 [1.04-1.63]  1.047   0/5       7 16 chunks
mandelbrot   4 rayon-join           7234.2   3.7 6969.4..7625.7            27667.4                                        
mandelbrot   4 static              26418.4   0.4 26320.2..26814.6         110020.2                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6866.0   0.9 6801.8..6978.3            27197.7                                        
mandelbrot   8 parlay               7062.5   1.3 6973.6..7264.0            27595.8                                        
mandelbrot   8 rayon-join           7084.6   0.6 7000.9..7440.8            27594.9                                        
mandelbrot   8 wf                   7162.8   2.8 6962.8..7654.2            27252.7 1.026 [1.02-1.10]  0.998   0/5       9 16 chunks
mandelbrot   8 rayon-iter           7303.6   1.0 7228.7..7675.7            27713.2                                        
mandelbrot   8 static              31537.5   0.1 31509.8..37299.8         127513.1                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26649.7   0.4 26443.8..26768.9          26646.1                                        
mandelbrot   1 wf                  26730.4   0.5 26603.1..26960.8          26714.7                                        
mandelbrot   1 wf-seq              27066.0   0.8 26749.5..28358.3          27063.8                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 wf                   8630.0   2.0 8358.3..9134.9            16550.8 0.999 [0.95-1.04]  0.966   3/5     403 
quadrature   2 rayon-join           8676.9   1.4 8274.0..9030.0            17352.0                                        
quadrature   2 rayon-join-left      8804.9   1.3 8447.7..9121.9            17590.7                                        
quadrature   2 static               9321.9   4.9 8866.2..10147.1           17320.1                                        excursions retained
quadrature   2 tbb                 10375.0   2.7 9350.3..10658.5           20032.2                                        
quadrature   2 parlay              10599.9   4.7 9288.2..11097.5           18248.1                                        
quadrature   2 parlay-left         12934.9   3.3 12363.2..13575.4          20624.5                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join-left      4977.8   2.6 4849.2..5701.1            19354.4                                        
quadrature   4 rayon-join           5079.3   1.1 4775.0..5134.4            19846.2                                        
quadrature   4 static               5369.3   5.8 5057.2..7413.1            18376.6                                        excursions retained
quadrature   4 tbb                  5908.1   0.9 5855.8..6682.7            23591.0                                        
quadrature   4 wf                   5980.1   7.8 4888.3..7783.9            21556.7 1.182 [1.02-1.61]  1.176   0/5    1018 
quadrature   4 parlay               7597.1   1.8 7434.1..8213.2            22670.9                                        
quadrature   4 parlay-left         10008.5   4.3 9491.0..10966.5           26354.7                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join-left WF fastest: no
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   4870.9   1.5 4796.5..5177.8            18448.7 0.907 [0.78-0.99]  0.926   5/5    1183 
quadrature   8 rayon-join           5512.1   5.0 4930.1..6486.1            21415.0                                        
quadrature   8 parlay               6043.0   4.5 5773.5..8496.3            22000.7                                        
quadrature   8 parlay-left          6490.2   4.0 6112.1..7213.0            23573.5                                        
quadrature   8 tbb                  6756.9   5.5 5789.1..7486.1            27024.6                                        
quadrature   8 rayon-join-left      6880.7   4.2 6591.4..7240.5            26728.0                                        
quadrature   8 static             515990.8   0.8 511984.8..523990.3      2052637.3                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15620.5   0.8 15251.5..15843.7          15619.3                                        
quadrature   1 wf                  16006.2   1.6 15734.0..16905.6          16004.3                                        
quadrature   1 wf-seq              16275.5   0.8 15685.6..16551.7          16274.3                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 tbb                 16685.4   1.0 16511.9..20421.8          32875.0                                        
records      2 parlay              16811.6   0.3 16648.2..16890.3          32514.0                                        
records      2 static              17162.0   2.3 16731.0..18257.4          33183.2                                        excursions retained
records      2 rayon-iter          17168.8   1.7 16833.9..26165.6          33412.3                                        
records      2 rayon-join          17175.2   1.9 16807.5..17495.2          34110.4                                        
records      2 wf                  19136.7   2.2 18390.8..19659.6          37062.9 1.147 [1.10-1.17]  1.127   0/5       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen

records      4 rayon-join           8775.0   2.4 8563.1..9601.2            33905.2                                        
records      4 parlay               8810.3   2.0 8420.7..9059.1            29273.4                                        
records      4 rayon-iter           8912.1   2.9 8515.3..10149.0           33269.5                                        
records      4 tbb                  9395.5  10.3 8428.9..11148.8           33855.4                                        
records      4 wf                  10019.6   2.8 9613.4..12360.6           37367.1 1.143 [1.13-1.47]  1.136   0/5       7 64 chunks
records      4 static              10507.5  15.4 8483.6..12466.7           34364.2                                        excursions retained
records      4 BEST REFERENCE = rayon-iter   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 parlay               8606.5   0.7 8491.1..10320.7           33871.9                                        
records      8 rayon-join           8806.8   2.4 8592.4..9458.5            34568.5                                        
records      8 rayon-iter           8946.3   2.6 8715.1..9309.0            33953.9                                        
records      8 tbb                  9024.4   2.7 8561.8..9264.8            33615.4                                        
records      8 wf                  10936.9   2.6 9932.8..11222.6           40052.0 1.216 [1.15-1.31]  1.190   0/5      20 64 chunks
records      8 static              18293.1  12.8 15811.9..23508.1          78126.5                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = parlay       WF fastest: n/a (oversubscribed)
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32624.3   0.4 32264.3..32954.0          32620.7                                        
records      1 wf                  36101.9   1.8 35275.2..36762.7          36097.3                                        
records      1 wf-seq              36377.5   0.4 36095.6..37317.5          36374.5                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14627.9   1.9 13413.4..14982.5          28027.6 0.917 [0.84-0.97]  0.943   5/5       1 32 chunks
fir          2 rayon-join          15915.2   0.9 15459.3..16449.9          31382.3                                        
fir          2 tbb                 16016.0   0.4 15946.6..17452.9          30903.5                                        
fir          2 parlay              16101.2   1.3 15387.8..16581.3          31638.7                                        
fir          2 rayon-iter          16420.1   1.4 16184.2..17020.1          31633.4                                        
fir          2 static              16561.6   1.1 16375.8..16796.2          32558.7                                        excursions retained
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          4 wf                   7789.3   8.4 7132.1..9287.4            27979.6 0.977 [0.89-1.09]  0.956   3/5       9 64 chunks
fir          4 tbb                  8223.5   3.0 7973.3..8907.3            29673.1                                        
fir          4 parlay               8285.3   0.9 8094.2..9106.3            31533.7                                        
fir          4 rayon-join           8398.5   3.6 8077.5..9535.5            31737.4                                        
fir          4 static               8602.4   3.2 8330.8..11247.4           32627.7                                        excursions retained
fir          4 rayon-iter           8890.3   1.1 8144.6..9961.0            32729.3                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   8141.6   3.3 7829.0..9071.8            29758.9 0.958 [0.92-1.02]  0.945   3/5      19 64 chunks
fir          8 rayon-join           8531.5   1.7 8387.2..10382.1           32583.2                                        
fir          8 tbb                  8740.9   2.2 8423.4..9882.0            32805.4                                        
fir          8 parlay               8774.5   1.6 8522.2..8918.8            34045.1                                        
fir          8 rayon-iter           8879.5   3.1 8602.4..10066.4           33036.8                                        
fir          8 static              14166.7   1.4 11958.0..21150.4          61643.7                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27631.6   1.9 26987.7..28194.8          27627.6                                        
fir          1 wf                  28033.9   2.3 27143.3..28941.3          28017.4                                        
fir          1 serial              30557.6   2.2 29871.7..33782.6          30541.8                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```

`WF_PAR_SPIN_ROUNDS` = 1,024, a 19.4 us window — the landed value:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 30561's current affinity list: 0-3  date=2026-09-11T13:21:52Z
run=local  compiler=d47223c0dedbf90e4dc928b6c5bbad0b4acc1a67  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13408.9   0.7 13250.7..13537.3          26751.2                                        
mandelbrot   2 rayon-join          13540.2   0.4 13459.0..13726.2          26903.4                                        
mandelbrot   2 rayon-iter          13629.5   1.4 13433.7..13952.0          26985.6                                        
mandelbrot   2 wf                  13757.4   0.6 13653.5..14354.1          27083.3 1.032 [1.02-1.06]  1.015   0/5       3 16 chunks
mandelbrot   2 parlay              13761.3   1.6 13497.8..13977.8          27151.1                                        
mandelbrot   2 static              26768.4   0.5 26358.8..26899.6          54590.8                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6951.2   2.2 6750.4..7161.6            26776.2                                        
mandelbrot   4 parlay               7074.8   0.8 6912.3..7173.5            27462.0                                        
mandelbrot   4 rayon-join           7103.8   0.6 7059.6..7534.1            27354.5                                        
mandelbrot   4 rayon-iter           7163.4   1.6 7000.9..7279.9            27606.7                                        
mandelbrot   4 wf                   7164.7   1.5 6895.2..8026.5            27314.5 1.021 [1.02-1.15]  1.009   0/5       6 16 chunks
mandelbrot   4 static              26705.5   1.5 26228.5..27150.6         110633.8                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6911.2   0.2 6897.4..7502.6            27179.6                                        
mandelbrot   8 wf                   7173.2   0.8 7118.4..7451.2            27319.3 1.040 [0.96-1.08]  1.008   1/5       8 16 chunks
mandelbrot   8 rayon-join           7186.5   0.6 7146.8..7446.6            27940.2                                        
mandelbrot   8 parlay               7393.5   1.4 7179.8..7548.4            28763.1                                        
mandelbrot   8 rayon-iter           7474.3   1.9 7201.0..7775.5            28066.4                                        
mandelbrot   8 static              31489.3   0.1 31460.9..45337.9         127476.8                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26521.6   0.1 26466.0..27742.0          26519.0                                        
mandelbrot   1 wf-seq              26787.9   0.4 26693.2..27244.9          26784.1                                        
mandelbrot   1 wf                  27155.8   0.4 26851.2..27274.3          27152.5                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join-left      9120.7   1.3 8737.1..9855.3            18157.3                                        
quadrature   2 wf                   9149.6   4.2 8765.2..13609.0           17281.6 1.015 [0.96-1.50]  0.956   1/5     408 
quadrature   2 rayon-join           9189.1   1.4 9042.6..9539.4            18098.4                                        
quadrature   2 static               9688.0   3.9 9058.4..14159.8           17724.0                                        excursions retained
quadrature   2 parlay              10293.0   3.7 9363.0..11032.4           18512.7                                        
quadrature   2 tbb                 10301.6   0.8 10216.1..10752.2          20526.9                                        
quadrature   2 parlay-left         12415.6   5.1 10538.8..13387.2          20286.1                                        
quadrature   2 BEST REFERENCE = rayon-join-left FASTEST = rayon-join-left WF fastest: no
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5117.0   4.1 4892.7..5331.3            20134.5                                        
quadrature   4 wf                   5144.6   5.1 4884.3..6715.0            19026.3 1.004 [0.97-1.26]  0.945   2/5    1015 
quadrature   4 rayon-join-left      5236.8   3.1 5055.1..5535.8            20753.1                                        
quadrature   4 static               5597.1   2.1 5415.6..5967.3            17584.3                                        excursions retained
quadrature   4 tbb                  6526.5   6.1 5774.3..6926.2            25849.3                                        
quadrature   4 parlay               7504.6   1.4 6582.0..7977.6            22829.4                                        
quadrature   4 parlay-left         10790.6   8.9 9664.3..14436.8           28631.5                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5050.2   5.4 4517.8..5398.4            16977.1 0.910 [0.78-0.94]  0.771   5/5    1168 
quadrature   8 rayon-join           5707.2   4.4 5415.4..6729.2            21819.8                                        
quadrature   8 parlay               6311.2   1.6 5964.7..6411.2            21783.8                                        
quadrature   8 tbb                  6790.9   2.6 6224.9..6967.0            26514.6                                        
quadrature   8 parlay-left          7051.9   8.8 6134.1..7737.5            25474.3                                        
quadrature   8 rayon-join-left      7572.9   4.7 6131.5..8063.9            28154.8                                        
quadrature   8 static             519986.6   1.5 511984.4..543989.9      2068178.2                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15618.6   0.8 15329.6..15745.1          15617.3                                        
quadrature   1 wf                  16360.1   1.1 14679.6..16541.8          16358.2                                        
quadrature   1 wf-seq              16411.4   1.5 16167.6..16689.6          16409.4                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

records      2 rayon-join          16477.1   1.1 16253.8..17703.7          32717.9                                        
records      2 parlay              16656.2   0.7 16280.8..16873.3          31337.9                                        
records      2 rayon-iter          16783.4   2.2 16028.7..17720.3          32942.2                                        
records      2 static              16834.6   2.8 16250.9..19557.9          32838.1                                        excursions retained
records      2 tbb                 16851.9   1.1 16380.0..17234.3          33149.1                                        
records      2 wf                  18409.5   1.0 18103.3..19676.2          36101.6 1.140 [1.10-1.19]  1.123   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen

records      4 tbb                  8287.7   1.1 8198.0..8402.9            30880.8                                        
records      4 rayon-iter           8583.6   1.2 8293.0..10612.1           32973.5                                        
records      4 rayon-join           8852.4   2.3 8423.2..9085.1            34277.2                                        
records      4 parlay               9293.4  11.7 8205.6..10958.2           29792.3                                        
records      4 wf                   9876.2   3.8 9451.3..10798.5           36327.1 1.175 [1.15-1.32]  1.220   0/5      10 64 chunks
records      4 static              10340.7  12.9 8300.8..11674.4           42885.4                                        excursions retained
records      4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 tbb                  8636.1   0.1 8628.1..8744.2            33716.4                                        
records      8 parlay               8795.2   1.7 8520.7..10880.4           34357.8                                        
records      8 rayon-iter           8939.6   5.1 8486.0..9936.8            33670.9                                        
records      8 rayon-join           8960.2   1.1 8463.1..9108.1            34394.0                                        
records      8 wf                   9853.8   3.0 9551.3..10532.3           37064.4 1.128 [1.11-1.20]  1.100   0/5      20 64 chunks
records      8 static              16358.9  14.5 12753.4..20046.6          76207.4                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32772.2   0.7 32520.3..34584.9          32765.5                                        
records      1 wf                  36058.6   0.6 35843.3..36598.6          36055.5                                        
records      1 wf-seq              37000.7   2.8 35836.1..38294.5          36996.4                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

fir          2 wf                  14224.9   0.9 13845.9..14717.9          27375.2 0.894 [0.87-0.94]  0.880   5/5       1 32 chunks
fir          2 rayon-join          16018.1   1.0 15595.5..16211.4          31587.9                                        
fir          2 tbb                 16135.4   2.8 15669.9..17201.5          30510.5                                        
fir          2 static              16361.7   1.6 16102.4..20191.7          32358.7                                        excursions retained
fir          2 rayon-iter          16362.5   3.5 15710.4..17063.4          31582.3                                        
fir          2 parlay              16404.0   1.7 15902.6..16746.3          32336.2                                        
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   7478.7   0.5 7205.5..7519.4            27293.8 0.905 [0.85-0.92]  0.874   5/5       8 64 chunks
fir          4 tbb                  8322.4   1.5 8161.5..8868.9            29315.9                                        
fir          4 static               8340.5   2.9 8094.6..8844.0            32262.4                                        excursions retained
fir          4 rayon-join           8423.4   2.5 8208.7..8843.4            32162.5                                        
fir          4 parlay               8530.5   1.6 7960.5..9622.1            32203.4                                        
fir          4 rayon-iter           8596.2   1.2 8421.1..8885.9            31912.2                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf                   7796.0   4.1 7401.4..8371.2            28207.2 0.941 [0.92-1.02]  0.900   3/5      19 64 chunks
fir          8 rayon-join           8278.9   0.8 8041.1..8376.7            31337.3                                        
fir          8 parlay               8506.2   1.6 8054.3..8915.9            33262.8                                        
fir          8 rayon-iter           8637.5   1.2 8420.5..9292.2            32762.9                                        
fir          8 tbb                  8688.9   1.5 8350.6..8930.5            32321.4                                        
fir          8 static              14423.3  15.5 12183.3..18237.4          62400.8                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27131.7   2.2 26545.1..29783.5          27128.0                                        
fir          1 wf                  27340.9   0.3 27270.3..28107.4          27337.9                                        
fir          1 serial              31881.8   0.6 31243.4..32078.8          31877.8                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```

The second run at 1,024 is not reproduced as a whole table: it differs from the
one above it only in its numbers, and it is here to bound this host's spread over
identical bytes. Its `wf` and `wf-seq` rows:

```text
mandelbrot   2 wf                  13894.3   0.4 13603.5..14325.8          27081.4 1.036 [1.01-1.04]  1.019   0/5       3 16 chunks
mandelbrot   4 wf                   7209.0   2.1 7056.1..9825.1            27444.9 1.046 [0.97-1.43]  1.035   1/5       7 16 chunks
mandelbrot   8 wf                   7553.6   3.1 7104.1..7789.9            27533.4 1.048 [1.04-1.11]  0.991   0/5       8 16 chunks
mandelbrot   1 wf                  26711.6   0.5 26590.3..27303.3          26706.9                                        
mandelbrot   1 wf-seq              27007.1   1.3 26643.5..27504.5          27002.9                                        
quadrature   2 wf                   9432.0   5.7 8492.5..12267.6           17231.8 1.060 [0.96-1.42]  0.969   1/5     407 
quadrature   4 wf                   5225.1   4.1 4690.7..6383.9            16948.6 0.924 [0.84-1.11]  0.799   3/5     979 
quadrature   8 wf                   5238.0  10.8 4671.5..5873.4            17227.7 1.005 [0.85-1.01]  0.811   2/5    1193 
quadrature   1 wf-seq              16105.1   5.1 15018.1..16929.9          16103.2                                        
quadrature   1 wf                  16486.5   0.8 16361.8..16654.6          16484.5                                        
records      2 wf                  18405.6   0.5 18306.2..19863.7          36144.7 1.133 [1.11-1.19]  1.130   0/5       1 32 chunks
records      4 wf                   9942.2   7.3 9213.2..13173.7           36572.6 1.174 [1.06-1.59]  1.187   0/5       9 64 chunks
records      8 wf                   9813.2   0.5 9760.3..10778.8           36917.0 1.166 [1.12-1.27]  1.103   0/5      20 64 chunks
records      1 wf                  35767.7   1.4 35262.0..37500.7          35718.7                                        
records      1 wf-seq              36176.2   2.0 35452.3..37390.5          36126.7                                        
fir          2 wf                  13724.7   0.7 13206.2..14691.9          26545.5 0.876 [0.84-0.93]  0.868   5/5       1 32 chunks
fir          4 wf                   7439.5   2.6 7121.7..11064.4           27343.6 0.907 [0.88-1.43]  0.868   4/5      10 64 chunks
fir          8 wf                   7591.8   4.5 7250.6..8201.1            27619.7 0.904 [0.88-1.01]  0.866   4/5      18 64 chunks
fir          1 wf-seq              27083.6   0.4 26750.4..27191.2          27079.7                                        
fir          1 wf                  27160.8   1.1 26874.1..28465.2          27120.2                                        
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the split work unit swept against the Mandelbrot grain

**No value meets the acceptance and the splitter's work unit stays at
1,200,000.** The record is here because the refusal has a measured reason, and
because the reason is not the one the experiment set out to test.

**The question.** Mandelbrot's `wf` row runs at **16 chunks at every width**
while oneTBB's `auto_partitioner` hands the same map out as about 1,536
callbacks, and the timed input is skewed (`shape=trailing`), so sixteen chunks
over four lanes leaves the tail unbalanced. Sixteen is what the admission rule
affords, not what the oversubscription term wants: at weight 219 over a span of
98,304 the work term allows `98,304 / ceil(1,200,000 / 219)` = 17 chunks, which
rounds down to 16, far under the `16 * lanes` cap of 64. Records and fir already
sit at that cap, so at W=2 and W=4 the work unit can only move mandelbrot. The
experiment moves that one variable and asks whether a finer grain is what
mandelbrot's 1.06 at W=4 is missing.

**The method.** One plain `compare PASSES=5 CALLS=5` per value into a fresh
`RESULTS=` on an otherwise idle host, with the runtime built at 1,200,000
(control), 600,000, 300,000 and 150,000 through the bundle's new
`WF_RUNTIME_CONTROL_FLAGS`, which appends `-DWF_PAR_SPLIT_WORK_UNIT=...` to the
compile of the four Whitefoot runtime translation units and to nothing else. The
order was 600,000, control, 300,000, control, 150,000, and a third control
followed on the committed tree, so **the control is neither first nor alone:
three of the six runs are the same bytes three times**, and they are what the
candidates have to be read against. `WF_PAR_CONTROL_FLAGS` was empty in all six,
so every `wf` row is the program plain `--par` produces.

**The chunk counts moved exactly as the rule predicts**, read off the `note`
column of each table rather than recomputed:

| work unit | mandelbrot W=2/4/8 | records W=2/4/8 | fir W=2/4/8 | quadrature |
|---|---|---|---|---|
| 1,200,000 | **16 / 16 / 16** | 32 / 64 / 64 | 32 / 64 / 64 | `chunks=na` |
| 600,000 | **32 / 32 / 32** | 32 / 64 / **128** | 32 / 64 / **128** | `chunks=na` |
| 300,000 | **32 / 64 / 64** | 32 / 64 / **128** | 32 / 64 / **128** | `chunks=na` |
| 150,000 | **32 / 64 / 128** | 32 / 64 / **128** | 32 / 64 / **128** | `chunks=na` |

Mandelbrot reaches the `16 * lanes` cap of 64 at W=4 from 300,000 down, and 32
at W=2 at every candidate because 32 *is* the cap there. **Records and fir did
not move at W=2 or W=4 at any value**, which is the check that what changed was
the work term and not the oversubscription term: both were already at the cap at
those two widths. Both move at the oversubscribed W=8, where the cap is 128.

**Acceptance: not met, at any value.** The criterion was that mandelbrot's W=4
wall ratio improve against the control with at least as many paired passes
lower, that no kernel's wall ratio regress beyond its own MAD at any width, that
no kernel's `cpu_r` rise beyond its MAD, and that the W=1 rows be unchanged. The
three control readings of mandelbrot W=4 are **1.037 (0/5), 1.015 (0/5) and
1.030 (1/5)**. The candidates read **1.057 (0/5)** at 600,000, **1.067 (1/5)**
at 300,000 and **1.017 (1/5)** at 150,000. Not one is below every control
reading; two are above all three, and the third sits inside them. The first
clause fails at every value, so nothing is chosen and nothing changes. Two
values also fail the later clauses on their own: 300,000 takes mandelbrot W=4 to
1.067 against a best control of 1.037 on a row whose MAD is 2.3 percent, and
150,000 takes mandelbrot W=2 to **1.063** against 1.020/1.023/1.031 with
`cpu_r` **1.075** against 1.012/1.013/1.014, both beyond that row's 2.4 percent
MAD.

**Why this host cannot answer the question, in its own numbers.** Three rows in
this sweep are *the same computation at the same grain in all six runs*, and
they say what one reading here is worth:

- **quadrature emits no `wf__par_split_budget` call at all** — it parallelizes
  by recursion structure — so the work unit cannot reach it and all six
  quadrature blocks time identical work. Its W=4 ratio reads **1.035 / 1.010 /
  0.916 / 1.017 / 0.987 / 0.968**: a spread of **13.0 percent** over six runs of
  a decomposition that did not change, with three of those readings being the
  same bytes three times.
- **records W=4 holds 64 chunks in all six** and reads 0.910 / 0.941 / 0.949 /
  0.954 / 0.939 / 0.987 — **8.5 percent**.
- **fir W=4 holds 64 chunks in all six** and reads 0.963 / 0.894 / 0.898 /
  0.973 / 0.909 / 0.963 — **8.8 percent**.

The effect being chased is mandelbrot's 1.06 against 1.00, about six percent. It
is smaller than the spread of rows that did not change at all, and the two
mandelbrot W=4 readings at an *identical* 64 chunks — 1.067 at 300,000 and 1.017
at 150,000 — differ by 4.9 percent by themselves. One reading per value cannot
separate a grain effect from this host's run-to-run noise here, and it is the
sweep's own controls that say so rather than a rule of thumb.

**What the readings do support.** Finer is not better for mandelbrot on this
host, and at W=2 it is worse. W=2 is the one width where the grain moves for
every candidate and nothing else does, 16 chunks against 32: the three 16-chunk
readings are **1.023, 1.020 and 1.031**, and the three 32-chunk readings are
**1.040, 1.025 and 1.063** — two above every control, one inside them, none
below. At W=4, across grains of 16, 32, 64 and 64, the readings are
1.015/1.030/1.037, 1.057, 1.067 and 1.017: no trend, and nothing below the
control band. **So the sixteen chunks are not what mandelbrot's W=4 row is
missing, as far as this host can see**, and something other than the split
bounds that row from below — sixty-four chunks is four to one over four lanes
and still does not reach the reference.

**One observation outside the recorded block, offered as a pointer and not as a
result.** At the oversubscribed W=8, records and fir move from 64 to 128 chunks
at 600,000 and below. Fir improves in every candidate reading against every
control — 0.929/0.928/0.920 against **0.896 / 0.886 / 0.896** — and records in
two of three, 0.921/0.994/0.935 against **0.883 / 0.893** and a 0.950 that lands
inside the controls; records' `cpu_r` falls with it (0.898/0.932/0.906 against
0.886/0.871/0.901) while fir's stays inside its control band. A W=8 block on a
four-CPU host is oversubscribed, rewards schedulers that yield, and prints
`n/a (oversubscribed)` in place of a verdict, so it cannot answer this bundle's
question. It is a reason to run this sweep again on an eight-CPU host, where W=8
is the recorded block and 128 chunks would be the cap rather than a consequence
of oversubscription.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- recorded block: W=4   oversubscribed blocks emitted: W=8
- compiler revision: `ecd86a6c` in the first five runs, which were taken on this
  branch's working tree before its first commit, and `b4c8e747` in the third
  control, taken after it. That commit changes the bundle and one comment in
  `compiler/src/backend/sched/entry.c`; the compiler is the parent's in every
  run, and the four emitted `--par` modules are byte-identical across all six
  and identical to the plain table at `53359d73` (`mandelbrot-par.ll`
  `f07c190e3a396fc9…`, `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll`
  `9ceebaedf90e4a60…`, `fir-par.ll` `e0af1ed03ef2bc52…`). The arms differ in one
  `unsigned long` initializer in `sched_entry.o`, which the compiler does not
  read.
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` empty in all six manifests
- runtime control flags: `WF_RUNTIME_CONTROL_FLAGS` empty in the three control
  runs and `-DWF_PAR_SPLIT_WORK_UNIT=<value>` in the three candidate runs, as
  each manifest and each candidate table's own header line records. **The three
  candidate tables are not plain tables and are not recorded as ones**: they are
  the arms of this sweep, and the value that ships is unchanged.
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version 18.1.3 (1ubuntu1)
- rustc: rustc 1.94.1 (e408947bf 2026-03-25), host `x86_64-unknown-linux-gnu`
  cargo: cargo 1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes: unchanged — mandelbrot 98,304 points at limit 256, quadrature M=64 at
  tolerance `0x1p-54` and depth 24, records 131,072 at `max_length` 255, fir
  K=64 over N=524,288. Emitted chunk counts are the table above, from the `note`
  column of each run. No size constant changed.
- workflow run: `local`. The push carrying this record touches
  `compiler/src/backend/sched/**` in comments only and
  `research/experiments/compute-bench/**` in the bundle, so the hosted
  `compute-bench` workflow runs on `ubuntu-24.04` (recorded W=4) and `macos-14`
  (recorded W=2) against an unchanged runtime constant; those tables are a
  reading of the control on two other machines and are not waited on here.
- sizing window, all six runs: every `wf-seq` median at W=1 inside [5 ms, 60 ms]
  — mandelbrot 26.686 / 26.590 / 26.654 / 26.950 / 26.627 / 27.115 ms,
  quadrature 16.061 / 16.194 / 16.138 / 16.016 / 15.846 / 16.350, records
  28.035 / 28.608 / 28.606 / 29.229 / 28.335 / 28.944, fir 27.452 / 27.790 /
  27.687 / 27.600 / 27.405 / 27.798; every `wf` median at the recorded W=4 above
  1 ms; and `steals > 0` on the `wf` row at every parallel width in every run,
  so no row carries `no-lanes`.
- **These `wf-seq` medians are not comparable with the sections above.** Records
  reads about 28.6 ms at W=1 here against about 36.2 ms in the section before
  it, on a byte-identical `records-seq.ll`. What changed between them is
  `harness.o`, whose size moved when the harness stopped carrying its own copy
  of the work unit, and code placement is this bundle's largest confound. All
  six runs here share that layout, so the sweep is internally comparable; a
  reading of it against an earlier section is not.

The control, `WF_RUNTIME_CONTROL_FLAGS` empty, first of three:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 16727's current affinity list: 0-3  date=2026-09-11T13:58:27Z
run=local  compiler=ecd86a6c9a3c1e1a15138b478e6f3cac0cc77192  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
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

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 rayon-join          13523.1   0.5 13449.0..13707.8          26872.2                                        
mandelbrot   2 tbb                 13534.5   0.5 13371.9..13784.8          26929.2                                        
mandelbrot   2 parlay              13559.3   0.8 13417.0..13717.7          26764.0                                        
mandelbrot   2 rayon-iter          13578.2   1.1 13435.1..13895.6          26913.1                                        
mandelbrot   2 wf                  13723.2   0.3 13578.2..14247.1          26854.6 1.023 [1.01-1.06]  1.012   0/5       3 16 chunks
mandelbrot   2 static              26373.0   0.7 26196.0..26902.9          54204.1                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6797.6   1.0 6729.7..7039.8            25877.3                                        
mandelbrot   4 rayon-iter           7082.7   1.5 6938.2..7512.0            27149.4                                        
mandelbrot   4 wf                   7172.7   1.3 6899.3..7265.8            26999.0 1.037 [1.02-1.07]  1.041   0/5       7 16 chunks
mandelbrot   4 parlay               7205.5   1.1 6978.2..7348.5            27664.2                                        
mandelbrot   4 rayon-join           7671.7   6.7 7065.4..8613.7            27549.8                                        
mandelbrot   4 static              26460.7   0.9 26080.3..26798.2         109859.5                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  7121.5   0.9 6819.0..7184.7            27670.2                                        
mandelbrot   8 parlay               7196.1   1.3 7104.8..7673.1            28464.1                                        
mandelbrot   8 rayon-join           7239.5   1.6 6995.9..7782.6            28058.5                                        
mandelbrot   8 rayon-iter           7342.2   0.4 7029.0..7626.3            28204.4                                        
mandelbrot   8 wf                   7566.3   1.2 7043.1..7660.8            27392.4 1.053 [1.03-1.07]  0.981   0/5       8 16 chunks
mandelbrot   8 static              31491.5   2.6 30669.5..37438.1         127193.3                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26466.0   0.5 26300.7..26916.1          26463.4                                        
mandelbrot   1 wf-seq              26686.4   0.3 26537.9..26764.0          26681.5                                        
mandelbrot   1 wf                  26845.9   0.9 26616.3..27280.9          26842.3                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8653.5   2.8 8209.8..8959.8            17304.8                                        
quadrature   2 rayon-join-left      8874.3   0.9 8795.0..10288.3           17706.7                                        
quadrature   2 wf                   8897.1   2.6 8436.6..9162.7            16848.9 1.030 [0.97-1.08]  0.991   1/5     404 
quadrature   2 static               8977.3   3.1 8700.8..10282.1           16975.8                                        excursions retained
quadrature   2 tbb                 10574.0   5.4 9685.6..12141.2           20317.9                                        
quadrature   2 parlay              10677.3   7.6 9866.1..11714.4           18558.1                                        
quadrature   2 parlay-left         12889.0  10.6 10978.0..14670.8          20713.3                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           4975.3   3.4 4592.9..5163.7            19739.9                                        
quadrature   4 wf                   5009.5   5.5 4732.9..8379.0            17183.1 1.035 [0.95-1.82]  0.890   2/5    1009 
quadrature   4 rayon-join-left      5881.4   7.7 4819.2..8070.5            21991.4                                        
quadrature   4 static               5908.6   5.5 5266.1..6909.9            20243.6                                        excursions retained
quadrature   4 tbb                  6265.2   7.9 5769.1..6900.0            24861.9                                        
quadrature   4 parlay               7703.8   4.5 7034.3..8053.8            22849.7                                        
quadrature   4 parlay-left          9973.6   0.3 9939.6..10746.5           26673.5                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5161.7   0.9 4703.9..5205.8            17446.0 0.989 [0.90-1.06]  0.890   3/5    1188 
quadrature   8 rayon-join           5348.1   8.4 4843.5..6452.4            20971.8                                        
quadrature   8 parlay-left          5945.3  19.5 4786.5..8607.9            22967.2                                        
quadrature   8 tbb                  6278.8   4.0 5817.1..7394.2            24262.5                                        
quadrature   8 rayon-join-left      6429.4   3.1 5645.6..6628.9            24511.8                                        
quadrature   8 parlay               6464.8   1.7 4755.1..6574.1            23310.2                                        
quadrature   8 static             559993.9   0.7 524040.4..591963.0      2205142.9                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15479.8   1.2 14734.3..15756.2          15478.4                                        
quadrature   1 wf-seq              16060.9   0.8 14610.4..16245.1          16059.9                                        
quadrature   1 wf                  16278.0   0.9 14596.8..16420.8          16276.7                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                  14731.1   1.5 14367.8..15136.0          28693.6 0.911 [0.88-0.95]  0.903   5/5       1 32 chunks
records      2 rayon-join          16387.3   0.3 16332.3..16873.4          32518.6                                        
records      2 parlay              16470.7   0.8 16001.4..16609.9          30240.5                                        
records      2 static              16502.1   0.9 16138.7..16647.3          32496.8                                        excursions retained
records      2 rayon-iter          16656.9   0.5 16030.5..16742.4          32343.1                                        
records      2 tbb                 16709.7   1.8 16097.4..17047.5          31990.6                                        
records      2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

records      4 wf                   7657.4   2.2 7355.1..7953.6            28793.9 0.910 [0.89-0.97]  0.897   5/5       6 64 chunks
records      4 tbb                  8360.8   1.6 8230.7..9164.6            29946.5                                        
records      4 static               8487.5   1.0 8376.1..13728.7           33120.9                                        excursions retained
records      4 rayon-iter           8518.4   3.2 8249.7..9900.9            32707.0                                        
records      4 parlay               8548.0   2.7 8317.4..8999.3            27702.4                                        
records      4 rayon-join           8809.9   5.3 8226.3..9873.8            32771.0                                        
records      4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

records      8 wf                   7720.6   1.5 7604.7..9060.0            29437.8 0.921 [0.86-1.03]  0.898   4/5      19 64 chunks
records      8 tbb                  8806.6   1.1 8356.2..8908.2            33363.4                                        
records      8 parlay               8823.7   4.2 8422.3..10474.1           34053.6                                        
records      8 rayon-iter           8926.9   1.3 8372.3..10385.2           33922.7                                        
records      8 rayon-join           9154.5   5.6 8643.3..10906.4           34651.7                                        
records      8 static              18953.5  18.1 13090.6..23899.0          78927.0                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              28035.4   0.2 27979.3..28297.9          28032.0                                        
records      1 wf                  29140.2   2.0 28088.8..37136.9          29135.8                                        
records      1 serial              32534.2   0.6 32328.7..33776.1          32517.4                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

fir          2 wf                  15078.2   2.0 14111.8..15718.9          28746.7 0.933 [0.90-0.99]  0.913   5/5       2 32 chunks
fir          2 rayon-iter          16221.7   0.5 15913.9..17694.1          31638.1                                        
fir          2 tbb                 16244.4   1.6 15754.6..17244.4          31677.8                                        
fir          2 rayon-join          16286.6   2.0 15652.8..16612.1          31901.7                                        
fir          2 static              16364.2   0.4 16255.2..16465.5          32282.7                                        excursions retained
fir          2 parlay              16487.3   1.0 15947.4..17233.0          32319.9                                        
fir          2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf                   7962.7   6.6 7160.8..8776.5            27464.4 0.963 [0.90-1.04]  0.918   3/5       7 64 chunks
fir          4 static               8277.3   1.2 8180.8..10300.8           32319.4                                        excursions retained
fir          4 rayon-join           8405.0   2.4 8177.3..8817.8            31554.3                                        
fir          4 tbb                  8453.1   1.5 7986.9..8584.1            29905.8                                        
fir          4 rayon-iter           8592.6   2.4 8316.7..8796.4            31612.9                                        
fir          4 parlay               8782.8   3.5 8382.0..9255.6            32927.6                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          8 wf                   7915.6   4.7 7485.8..9469.6            28064.5 0.929 [0.88-1.11]  0.891   4/5      18 64 chunks
fir          8 rayon-join           8591.1   0.7 8533.7..9974.6            32314.4                                        
fir          8 parlay               8621.3   1.6 8484.9..9102.6            33537.7                                        
fir          8 rayon-iter           8858.2   4.0 8351.5..10859.6           32845.4                                        
fir          8 tbb                  8906.4   1.6 8519.2..10161.2           32447.7                                        
fir          8 static              13137.6   6.7 12251.0..24727.5          49151.4                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27439.4   0.3 27363.4..28316.0          27435.1                                        
fir          1 wf-seq              27452.0   0.7 27135.9..30070.0          27446.0                                        
fir          1 serial              32666.6   0.9 31368.5..33227.9          32651.3                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

The work unit at 600,000, which doubles mandelbrot to 32 chunks at every
width and takes records and fir from 64 to 128 at the oversubscribed W=8:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 13998's current affinity list: 0-3  date=2026-09-11T13:56:53Z
run=local  compiler=ecd86a6c9a3c1e1a15138b478e6f3cac0cc77192  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_WORK_UNIT=600000
      (an A/B control appended to the compile of the Whitefoot runtime: this
      table is NOT the runtime this tree ships and must not be recorded as one
      -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 rayon-join          13791.3   1.9 13444.7..14194.6          27230.2                                        
mandelbrot   2 rayon-iter          13855.5   2.2 13404.3..14402.0          27244.7                                        
mandelbrot   2 parlay              13872.3   1.8 13622.9..14396.1          27404.2                                        
mandelbrot   2 tbb                 14080.9   0.7 13245.7..14181.8          27516.6                                        
mandelbrot   2 wf                  14117.4   1.8 13774.4..14855.6          27449.0 1.040 [1.00-1.07]  1.019   0/5       3 32 chunks
mandelbrot   2 static              27014.4   0.4 26601.9..27246.1          54949.8                                        excursions retained
mandelbrot   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6920.1   3.0 6709.5..7497.5            27060.7                                        
mandelbrot   4 parlay               7061.2   1.0 6929.8..7548.1            27219.0                                        
mandelbrot   4 wf                   7201.0   1.7 6914.9..7473.5            27243.5 1.057 [1.00-1.08]  1.004   0/5       8 32 chunks
mandelbrot   4 rayon-join           7417.0   2.7 7184.7..10216.8           27934.8                                        
mandelbrot   4 rayon-iter           7624.7   4.5 7198.1..8002.0            28575.9                                        
mandelbrot   4 static              26548.7   1.3 26195.4..27488.9         110019.8                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6864.2   0.3 6845.3..7203.6            27366.7                                        
mandelbrot   8 rayon-join           7218.4   2.2 7057.7..7525.1            27991.5                                        
mandelbrot   8 parlay               7324.6   1.9 6989.9..7563.6            28552.7                                        
mandelbrot   8 rayon-iter           7439.2   1.9 7121.6..7577.5            27983.9                                        
mandelbrot   8 wf                   7447.5   1.9 7304.7..7765.5            28242.6 1.073 [1.03-1.13]  1.024   0/5      11 32 chunks
mandelbrot   8 static              31531.2   0.1 31458.5..47481.1         127236.1                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf                  26745.1   0.3 26654.2..27856.8          26741.8                                        
mandelbrot   1 serial              26805.7   1.3 26370.5..27216.5          26802.8                                        
mandelbrot   1 wf-seq              26949.6   0.3 26725.0..27043.0          26946.0                                        
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
  wf-seq: control

quadrature   2 wf                   8717.6   3.1 8451.4..9569.4            17557.3 0.995 [0.95-1.05]  0.994   3/5     407 
quadrature   2 rayon-join           9150.6   2.3 8490.4..9636.0            18225.1                                        
quadrature   2 static               9205.4   0.6 8773.7..10623.3           17210.2                                        excursions retained
quadrature   2 rayon-join-left      9289.7   3.6 8593.2..9628.0            18548.1                                        
quadrature   2 tbb                 10044.1   0.9 9395.9..11412.1           20085.9                                        
quadrature   2 parlay              10556.1   1.4 10238.1..10964.4          18470.7                                        
quadrature   2 parlay-left         12766.7   8.8 10634.8..18324.0          20582.5                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5062.6   3.3 4579.7..5350.5            20242.0                                        
quadrature   4 wf                   5089.2   1.1 4663.7..5596.7            18631.8 1.017 [0.96-1.07]  0.914   2/5    1028 
quadrature   4 rayon-join-left      5632.6   1.3 4855.0..6284.8            22216.5                                        
quadrature   4 static               6336.2   5.3 5150.5..7508.9            29836.8                                        excursions retained
quadrature   4 tbb                  6519.9   1.7 5681.2..6722.9            25958.0                                        
quadrature   4 parlay               7498.1   5.5 7086.8..8662.7            22704.0                                        
quadrature   4 parlay-left         10019.2   4.4 9582.6..10789.5           27162.8                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5093.7   4.9 4845.7..5603.7            17160.8 0.969 [0.87-1.10]  0.999   3/5    1194 
quadrature   8 rayon-join           5758.0   4.3 5140.7..6367.0            22040.3                                        
quadrature   8 parlay-left          5849.4  10.1 5189.5..7042.1            17748.7                                        
quadrature   8 parlay               6382.8   6.0 4742.5..6768.5            18185.2                                        
quadrature   8 tbb                  7230.8   7.6 6332.9..7999.7            26615.3                                        
quadrature   8 rayon-join-left      7458.8   8.7 6640.0..8545.4            26977.1                                        
quadrature   8 static             527943.4   1.5 519965.1..536012.5      2093806.4                                        excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15829.0   0.2 15454.1..16349.3          15827.0                                        
quadrature   1 wf-seq              16016.3   2.8 15518.8..17006.7          16014.8                                        
quadrature   1 wf                  16318.3   0.5 16231.8..17126.5          16315.9                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                  14990.9   2.1 14609.0..15436.9          29062.2 0.919 [0.91-0.95]  0.910   5/5       1 32 chunks
records      2 parlay              16572.6   2.8 16114.3..17321.4          30253.4                                        
records      2 rayon-join          16584.0   1.1 16394.9..17314.6          32903.6                                        
records      2 tbb                 16594.9   0.5 16047.5..16679.6          31925.9                                        
records      2 static              16692.1   4.1 16007.2..24836.0          33287.6                                        excursions retained
records      2 rayon-iter          16703.7   1.2 16503.8..22757.4          32600.6                                        
records      2 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

records      4 wf                   8273.4   5.2 7572.0..8702.4            31243.7 0.954 [0.89-1.00]  0.951   4/5      10 64 chunks
records      4 rayon-join           8698.9   2.9 8447.0..10031.3           33858.4                                        
records      4 static               8797.9   2.9 8467.5..14734.8           32594.9                                        excursions retained
records      4 parlay               8815.8   1.6 8571.6..9054.1            27944.0                                        
records      4 tbb                  8823.3   2.0 8643.9..10334.9           32598.2                                        
records      4 rayon-iter          10067.6  13.4 8715.1..12667.8           34787.5                                        
records      4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

records      8 wf                   7943.5   2.1 7575.3..8458.2            30412.0 0.883 [0.87-0.94]  0.886   5/5      22 128 chunks
records      8 tbb                  8867.9   3.9 8523.3..10273.9           34421.7                                        
records      8 parlay               9173.5   2.6 8931.2..9856.5            35323.8                                        
records      8 rayon-join           9403.9   6.7 8770.5..11725.5           35365.6                                        
records      8 rayon-iter          10070.9   7.8 9139.5..11486.9           38647.6                                        
records      8 static              16411.9   2.8 14433.6..19739.1          73894.2                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf                  28636.6   0.4 27881.1..29126.5          28631.0                                        
records      1 wf-seq              29228.8   0.7 28328.8..29418.9          29224.4                                        
records      1 serial              33044.4   0.8 31937.6..35438.3          33041.1                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

fir          2 wf                  14004.5   2.9 13594.4..14586.2          27434.4 0.872 [0.85-0.92]  0.881   5/5       1 32 chunks
fir          2 rayon-join          16169.9   1.8 15885.0..18022.2          31521.7                                        
fir          2 static              16306.1   2.2 15858.1..19352.8          32303.3                                        excursions retained
fir          2 parlay              16359.5   1.7 16079.9..18440.3          31869.0                                        
fir          2 tbb                 16429.4   2.6 15925.8..17404.4          31227.6                                        
fir          2 rayon-iter          16791.2   1.0 16409.4..19502.7          32346.7                                        
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   8082.5   9.1 7323.7..11126.6           29192.6 0.973 [0.92-1.39]  0.918   3/5      10 64 chunks
fir          4 tbb                  8096.8   1.5 7973.9..8970.7            30274.1                                        
fir          4 parlay               8303.7   1.6 7956.5..8434.2            31810.2                                        
fir          4 rayon-join           8408.6   4.7 8010.5..9520.7            31878.9                                        
fir          4 rayon-iter           8611.7   3.4 8067.6..11674.9           31719.1                                        
fir          4 static               9147.2   4.8 8260.4..9703.9            33103.9                                        excursions retained
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7445.9   1.4 7126.3..8319.0            27671.8 0.896 [0.89-0.96]  0.879   5/5      23 128 chunks
fir          8 tbb                  8305.6   0.9 8231.9..8695.0            31879.0                                        
fir          8 rayon-join           8522.7   4.4 8151.3..8958.8            32401.9                                        
fir          8 rayon-iter           8556.4   0.8 8484.9..9860.0            32378.4                                        
fir          8 parlay               8689.4   3.0 7980.2..9487.7            33892.7                                        
fir          8 static              12416.2   0.1 12234.5..14803.7          48456.5                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27355.2   1.5 26756.1..28086.5          27351.0                                        
fir          1 wf-seq              27600.0   2.9 26594.6..30274.0          27596.9                                        
fir          1 serial              32466.7   0.8 30421.9..32716.1          32462.8                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

The work unit at 300,000, which takes mandelbrot to the `16 * lanes` cap of
64 at W=4:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 19412's current affinity list: 0-3  date=2026-09-11T13:59:59Z
run=local  compiler=ecd86a6c9a3c1e1a15138b478e6f3cac0cc77192  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_WORK_UNIT=300000
      (an A/B control appended to the compile of the Whitefoot runtime: this
      table is NOT the runtime this tree ships and must not be recorded as one
      -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13364.1   0.3 13312.1..13405.5          26697.2                                        
mandelbrot   2 rayon-iter          13411.4   0.4 13356.6..13671.9          26673.6                                        
mandelbrot   2 parlay              13513.2   0.2 13488.8..14483.4          26658.6                                        
mandelbrot   2 rayon-join          13587.9   1.4 13305.9..13853.1          27070.3                                        
mandelbrot   2 wf                  13652.0   0.8 13543.8..14244.3          26786.9 1.025 [1.02-1.07]  1.013   0/5       3 32 chunks
mandelbrot   2 static              26438.8   1.1 26095.1..27300.2          54253.2                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6793.1   1.1 6685.7..7391.8            27022.8                                        
mandelbrot   4 parlay               7063.8   0.8 6984.4..7437.4            27256.7                                        
mandelbrot   4 rayon-join           7128.4   0.4 7013.8..7333.8            27600.6                                        
mandelbrot   4 rayon-iter           7145.2   1.1 7001.7..7312.1            27667.0                                        
mandelbrot   4 wf                   7249.7   2.3 7022.4..7449.2            27148.1 1.067 [0.99-1.10]  1.024   1/5      10 64 chunks
mandelbrot   4 static              26423.4   0.4 26319.0..26678.2         109525.4                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  7037.8   0.8 6834.0..7374.2            27585.6                                        
mandelbrot   8 rayon-join           7138.4   1.3 6950.1..7262.3            27569.8                                        
mandelbrot   8 wf                   7268.0   1.1 7058.4..7382.9            27412.2 1.046 [1.00-1.06]  0.995   1/5      16 64 chunks
mandelbrot   8 parlay               7406.4   4.0 7011.8..9912.4            28625.6                                        
mandelbrot   8 rayon-iter           7831.2   6.6 7315.6..10023.0           28993.5                                        
mandelbrot   8 static              31448.7   0.7 31201.0..37535.2         127042.1                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26553.2   0.9 26320.5..27480.5          26551.4                                        
mandelbrot   1 wf-seq              26627.3   0.1 26597.4..27037.8          26625.2                                        
mandelbrot   1 wf                  27058.8   1.2 26615.2..27576.6          27054.9                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

quadrature   2 rayon-join           8362.4   2.3 8054.8..9121.3            16705.0                                        
quadrature   2 static               8716.8   0.4 8681.7..9413.7            16717.9                                        excursions retained
quadrature   2 wf                   8960.0   3.5 8650.6..11638.9           17120.3 1.057 [1.03-1.28]  1.010   0/5     391 
quadrature   2 rayon-join-left      9476.4   4.8 8791.7..10358.1           18919.4                                        
quadrature   2 tbb                 10153.5   3.5 9700.9..10789.8           20142.7                                        
quadrature   2 parlay              11243.2   4.1 9976.7..13550.9           19348.1                                        
quadrature   2 parlay-left         15357.1  19.0 12442.1..19155.8          23062.0                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join           5458.1   3.2 4924.5..6360.1            21589.3                                        
quadrature   4 wf                   5639.6   3.3 4736.6..5825.4            17614.1 0.987 [0.89-1.16]  0.874   3/5    1010 
quadrature   4 static               6256.4   7.1 5345.1..6700.1            27647.5                                        excursions retained
quadrature   4 rayon-join-left      6401.1  10.5 4799.0..7443.7            20413.5                                        
quadrature   4 tbb                  6621.7   5.0 6169.4..6949.8            26114.3                                        
quadrature   4 parlay               7479.0   5.3 6983.2..7880.1            23541.1                                        
quadrature   4 parlay-left         10162.9   4.6 9697.7..12462.9           26851.7                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5494.0  11.9 4838.8..7274.4            18282.6 1.008 [0.91-1.22]  0.909   2/5    1213 
quadrature   8 parlay               6075.3  10.9 5217.8..9334.9            22213.9                                        
quadrature   8 rayon-join           6496.7  13.0 4801.6..7884.8            22316.5                                        
quadrature   8 tbb                  6740.5   3.5 6048.6..6976.9            26223.5                                        
quadrature   8 rayon-join-left      6837.9  11.4 5451.4..7952.0            26454.8                                        
quadrature   8 parlay-left          7606.4   9.9 6051.3..10205.6           27242.8                                        
quadrature   8 static             519953.2   0.8 511986.8..568038.9      2066753.0                                        excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15494.9   0.5 15078.3..15751.8          15492.9                                        
quadrature   1 wf-seq              15845.6   2.7 14938.6..16333.7          15844.4                                        
quadrature   1 wf                  16491.6   3.1 15413.2..17005.9          16490.5                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                  14813.9   0.7 14716.4..15364.3          29057.8 0.914 [0.90-0.92]  0.917   5/5       1 32 chunks
records      2 rayon-iter          16341.2   1.4 16107.6..18092.1          32273.7                                        
records      2 rayon-join          16750.2   0.9 16214.6..17370.0          33124.2                                        
records      2 static              16755.8   0.6 16587.9..18559.3          32807.9                                        excursions retained
records      2 tbb                 16907.9   2.1 16133.1..17270.9          31969.3                                        
records      2 parlay              17160.7   4.5 16384.9..26468.3          32555.0                                        
records      2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

records      4 wf                   7760.7   2.4 7536.5..8883.8            29023.3 0.939 [0.91-1.03]  0.925   4/5       9 64 chunks
records      4 rayon-iter           8374.5   0.7 8305.9..8649.8            32534.1                                        
records      4 parlay               8471.8   0.9 8393.9..9636.5            29896.5                                        
records      4 tbb                  8535.1   2.6 8184.6..8842.8            31861.3                                        
records      4 rayon-join           8673.2   2.4 8297.3..9398.6            33101.2                                        
records      4 static               9330.9  11.3 8280.3..17874.2           33045.7                                        excursions retained
records      4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 wf                   7708.1   1.6 7587.2..8321.0            29499.1 0.893 [0.83-0.98]  0.871   5/5      25 128 chunks
records      8 tbb                  8500.8   0.6 8448.2..9120.8            33061.1                                        
records      8 rayon-iter           8905.6   2.4 8578.5..10452.0           34765.3                                        
records      8 rayon-join           9078.3   3.3 8534.9..9740.1            34698.4                                        
records      8 parlay               9341.5   2.0 8700.5..9529.3            36958.3                                        
records      8 static              21334.3   6.5 16802.7..23607.6          92169.5                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              28335.2   0.1 28309.4..30162.7          28329.9                                        
records      1 wf                  28926.8   1.4 28324.1..30231.0          28921.4                                        
records      1 serial              32164.8   0.8 31919.4..33788.6          32160.5                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

fir          2 wf                  14282.5   2.9 13546.2..17133.1          27531.5 0.877 [0.84-1.12]  0.885   4/5       1 32 chunks
fir          2 rayon-join          16119.9   0.9 15806.2..17161.9          31607.3                                        
fir          2 tbb                 16312.5   1.7 15918.6..16746.4          31276.2                                        
fir          2 static              16344.7   0.6 16253.5..17088.8          32341.9                                        excursions retained
fir          2 parlay              16459.7   3.9 15690.7..17126.1          32191.0                                        
fir          2 rayon-iter          16495.6   2.7 15344.2..17787.6          31943.2                                        
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   7558.2   6.1 7050.6..8144.8            27474.7 0.909 [0.90-0.95]  0.937   5/5       8 64 chunks
fir          4 tbb                  8149.6   2.1 7915.9..8559.2            29393.6                                        
fir          4 rayon-iter           8415.0   4.2 8064.1..9127.0            31150.2                                        
fir          4 parlay               8441.8   3.4 8156.9..9079.4            32723.1                                        
fir          4 rayon-join           8560.5   1.2 7758.2..9207.3            32811.2                                        
fir          4 static               9084.4   9.1 8228.6..10616.9           34926.7                                        excursions retained
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf                   7354.0   2.2 7153.1..7848.0            27501.2 0.886 [0.88-0.91]  0.880   5/5      24 128 chunks
fir          8 tbb                  8218.0   0.7 8157.8..9538.0            31576.3                                        
fir          8 parlay               8412.4   2.7 7981.5..10111.5           32749.7                                        
fir          8 rayon-join           8553.3   3.1 8096.1..8928.7            32095.0                                        
fir          8 rayon-iter           8689.3   3.0 8265.9..10674.7           32425.6                                        
fir          8 static              14184.8  14.1 12189.2..18237.3          61902.4                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27405.1   0.8 27184.5..37220.2          27401.2                                        
fir          1 wf                  27745.8   1.2 26987.9..28068.5          27742.1                                        
fir          1 serial              31388.5   2.8 29389.0..32252.1          31384.5                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

The work unit at 150,000, the finest grain swept, where every kernel is at
its cap at every width:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 24739's current affinity list: 0-3  date=2026-09-11T14:03:06Z
run=local  compiler=ecd86a6c9a3c1e1a15138b478e6f3cac0cc77192  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_WORK_UNIT=150000
      (an A/B control appended to the compile of the Whitefoot runtime: this
      table is NOT the runtime this tree ships and must not be recorded as one
      -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13403.1   0.3 13289.3..13454.6          26323.9                                        
mandelbrot   2 rayon-join          13504.4   0.6 13417.9..13749.5          26835.9                                        
mandelbrot   2 rayon-iter          13599.9   1.3 13429.8..15396.3          26902.2                                        
mandelbrot   2 parlay              13773.2   1.5 13476.7..13974.2          27141.6                                        
mandelbrot   2 wf                  14231.1   2.4 13774.3..16563.9          27627.6 1.063 [1.03-1.24]  1.075   0/5       3 32 chunks
mandelbrot   2 static              26927.0   1.7 26186.5..27391.0          54520.0                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6970.8   1.2 6886.8..7275.2            27864.4                                        
mandelbrot   4 wf                   7033.4   0.8 6977.0..8759.0            27206.2 1.017 [0.99-1.27]  1.010   1/5      10 64 chunks
mandelbrot   4 parlay               7033.6   0.5 6961.1..7276.9            27305.0                                        
mandelbrot   4 rayon-iter           7162.3   0.4 7132.1..7417.8            27520.0                                        
mandelbrot   4 rayon-join           7192.1   0.6 7152.5..9279.9            27414.8                                        
mandelbrot   4 static              26432.0   0.9 26199.2..26839.7         108559.4                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6962.3   0.8 6871.3..7768.5            27434.8                                        
mandelbrot   8 rayon-join           7104.3   1.0 7032.9..8476.7            27609.1                                        
mandelbrot   8 wf                   7234.7   1.2 7133.4..7354.7            27646.3 1.039 [1.03-1.07]  1.004   0/5      16 128 chunks
mandelbrot   8 parlay               7246.5   1.8 7107.9..7395.4            28598.7                                        
mandelbrot   8 rayon-iter           7716.7   2.5 7095.0..7908.7            28616.8                                        
mandelbrot   8 static              31513.4   0.1 31458.6..38032.2         127127.1                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26391.9   0.8 26192.1..26675.7          26388.4                                        
mandelbrot   1 wf                  27058.5   0.9 26556.6..28969.1          27044.6                                        
mandelbrot   1 wf-seq              27114.5   1.1 26555.4..27412.1          27095.4                                        
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control

quadrature   2 wf                   8984.9   9.2 8075.4..11073.7           17240.1 0.995 [0.92-1.34]  0.959   3/5     398 
quadrature   2 rayon-join           8994.4   2.7 8243.4..10176.2           17943.8                                        
quadrature   2 rayon-join-left      9513.9   8.0 8751.9..13718.1           18900.2                                        
quadrature   2 static              10142.0   3.6 9229.7..13062.0           18499.6                                        excursions retained
quadrature   2 tbb                 10631.1   1.1 10251.7..10836.1          21036.0                                        
quadrature   2 parlay              10898.9   5.2 10060.5..11468.2          19326.8                                        
quadrature   2 parlay-left         13228.0   3.7 12735.8..15071.3          20644.0                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   5074.0   2.4 4843.3..6829.6            16950.1 0.968 [0.88-1.45]  0.936   3/5    1036 
quadrature   4 rayon-join           5188.7   6.2 4865.1..5755.2            20594.2                                        
quadrature   4 rayon-join-left      5477.6   7.6 4721.1..6264.1            20849.2                                        
quadrature   4 static               5845.4   8.8 5329.6..7465.1            27984.0                                        excursions retained
quadrature   4 tbb                  6202.4   4.9 5896.1..7188.1            24500.8                                        
quadrature   4 parlay               7481.0   4.5 6882.8..8745.2            22689.5                                        
quadrature   4 parlay-left         10047.5   2.8 9376.9..10629.5           26787.8                                        
quadrature   4 BEST REFERENCE = rayon-join-left FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5378.8   4.3 4732.4..6098.6            17893.1 1.097 [0.86-1.11]  1.027   2/5    1207 
quadrature   8 parlay               5503.5   8.9 4844.9..8746.6            24877.5                                        
quadrature   8 rayon-join           5560.7   9.4 5036.9..6262.0            21843.6                                        
quadrature   8 rayon-join-left      6802.6   3.5 6202.5..7639.2            26259.4                                        
quadrature   8 parlay-left          6811.6   7.4 6305.8..8078.3            23616.2                                        
quadrature   8 tbb                  6980.1   5.6 6183.7..7759.5            26550.7                                        
quadrature   8 static             515983.9   0.8 512000.8..527982.0      2055440.5                                        excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15853.5   1.3 14572.6..16087.2          15852.4                                        
quadrature   1 wf-seq              16349.9   1.3 15884.7..17229.5          16348.5                                        
quadrature   1 wf                  16462.4   1.0 15302.6..16627.5          16395.3                                        
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen

records      2 wf                  15052.0   2.0 14638.9..16418.2          29104.7 0.931 [0.90-0.99]  0.909   5/5       1 32 chunks
records      2 static              16505.1   0.8 16376.2..17656.9          32502.4                                        excursions retained
records      2 tbb                 16542.0   0.7 16312.2..16651.9          32355.6                                        
records      2 parlay              16868.8   2.8 16393.5..20628.9          32127.0                                        
records      2 rayon-join          16882.3   2.7 16426.3..17698.0          33560.7                                        
records      2 rayon-iter          16909.5   3.6 16175.6..20339.7          33289.0                                        
records      2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

records      4 wf                   8416.2   1.3 7514.4..8524.0            30014.0 0.987 [0.90-1.03]  0.961   4/5      11 64 chunks
records      4 parlay               8629.2   1.1 8325.2..9042.5            30118.2                                        
records      4 tbb                  8735.3   1.9 8528.5..9131.8            32342.9                                        
records      4 static               8866.4   1.1 8268.9..13621.8           32774.9                                        excursions retained
records      4 rayon-iter           9238.5   4.3 8839.2..11544.0           35113.8                                        
records      4 rayon-join           9532.0   2.9 8812.4..10016.1           36029.1                                        
records      4 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

records      8 wf                   7946.4   3.6 7663.4..8349.6            30259.1 0.950 [0.90-0.96]  0.901   5/5      22 128 chunks
records      8 tbb                  8561.9   1.1 8468.5..8886.0            33236.4                                        
records      8 rayon-join           8712.0   0.5 8665.8..11977.8           33834.1                                        
records      8 parlay               8802.2   1.3 8363.3..8915.0            34020.2                                        
records      8 rayon-iter           9190.8   3.7 8809.6..11855.3           35530.8                                        
records      8 static              20534.4   9.7 15812.1..23971.9          82520.0                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              28943.5   1.5 28159.5..29390.0          28940.2                                        
records      1 wf                  28999.2   1.8 28192.2..29519.0          28993.7                                        
records      1 serial              32779.6   1.5 32298.0..33724.3          32773.4                                        
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

fir          2 wf                  15257.7   9.0 13578.0..19985.7          28716.2 0.939 [0.85-1.21]  0.872   4/5       1 32 chunks
fir          2 parlay              16485.9   0.9 16333.1..17794.4          32360.5                                        
fir          2 static              16525.1   2.5 16111.6..20861.3          34551.8                                        excursions retained
fir          2 rayon-join          16669.7   0.9 16511.6..20189.4          32640.4                                        
fir          2 tbb                 16770.1   1.9 16002.6..17230.8          31684.2                                        
fir          2 rayon-iter          17244.8   4.0 16404.2..19723.9          32571.0                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf                   7857.9   3.5 7523.4..9303.1            28729.2 0.963 [0.90-1.15]  0.958   4/5      10 64 chunks
fir          4 tbb                  8319.0   2.4 8104.6..8824.2            29728.2                                        
fir          4 rayon-join           8519.7   3.9 8190.3..9029.4            31792.0                                        
fir          4 static               8538.4   2.7 8307.3..11362.0           33063.7                                        excursions retained
fir          4 rayon-iter           8682.9   3.9 8105.5..9602.3            32233.3                                        
fir          4 parlay               9245.5   0.5 8313.4..9291.0            32304.7                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          8 wf                   7481.6   1.1 7320.8..8468.9            27778.6 0.896 [0.89-1.07]  0.887   4/5      23 128 chunks
fir          8 parlay               8398.4   1.1 8236.8..8763.2            32800.8                                        
fir          8 tbb                  8417.6   2.4 7883.0..8640.1            31510.5                                        
fir          8 rayon-iter           8642.0   1.4 8522.0..8886.5            32551.1                                        
fir          8 rayon-join           8722.7   4.3 8240.4..9130.5            32683.3                                        
fir          8 static              12764.7   3.3 12348.1..16002.6          52606.9                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27580.9   0.7 27385.2..29128.2          27577.1                                        
fir          1 wf-seq              27797.8   0.4 27358.6..31149.1          27793.3                                        
fir          1 serial              32386.8   0.8 31697.9..33137.1          32337.7                                        
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

The second control run, the same bytes as the first and differing from it
only in its numbers. It and the third below are here to bound this host's
spread over identical bytes, which is what the argument above rests on.
Its `wf` and `wf-seq` rows:

```text
mandelbrot   2 wf                  13683.9   0.9 13564.7..14004.4          26950.7 1.020 [1.00-1.05]  1.013   1/5       3 16 chunks
mandelbrot   4 wf                   6926.8   1.2 6826.2..7257.2            26771.8 1.015 [1.00-1.07]  1.018   0/5       6 16 chunks
mandelbrot   8 wf                   7265.4   1.6 7149.7..12240.6           27345.2 1.061 [1.03-1.82]  1.010   0/5       8 16 chunks
mandelbrot   1 wf-seq              26589.9   0.3 26516.1..26817.4          26586.2                                        
mandelbrot   1 wf                  26603.6   0.3 26512.8..27047.4          26599.4                                        
quadrature   2 wf                   8634.2   3.5 8330.7..9844.0            16933.8 1.005 [0.93-1.03]  0.978   1/5     418 
quadrature   4 wf                   4967.2   1.9 4871.9..6444.8            17040.7 1.010 [0.79-1.28]  0.867   2/5    1008 
quadrature   8 wf                   5148.0   1.8 5017.3..5563.8            18123.9 0.922 [0.86-1.09]  0.843   4/5    1167 
quadrature   1 wf-seq              16194.4   4.4 15478.3..16938.5          16192.7                                        
quadrature   1 wf                  16236.4   5.9 14987.6..18865.9          16235.2                                        
records      2 wf                  15337.5   5.7 14438.6..16956.5          29295.7 0.939 [0.85-1.03]  0.903   4/5       1 32 chunks
records      4 wf                   7869.1   1.6 7744.9..10770.7           29261.2 0.941 [0.91-1.27]  0.902   4/5       7 64 chunks
records      8 wf                   8410.7   2.0 8232.4..9453.7            31279.5 0.994 [0.91-1.07]  0.932   3/5      18 64 chunks
records      1 wf-seq              28608.2   0.4 28331.4..29147.7          28602.3                                        
records      1 wf                  29291.9   0.7 28089.6..29488.8          29238.5                                        
fir          2 wf                  14149.5   1.0 14014.3..16364.7          27439.4 0.903 [0.88-1.02]  0.892   4/5       2 32 chunks
fir          4 wf                   7507.8   5.5 7098.0..8037.3            27320.6 0.894 [0.88-0.97]  0.866   5/5      10 64 chunks
fir          8 wf                   7643.1   1.0 7448.3..8879.5            27961.2 0.928 [0.84-1.04]  0.894   4/5      19 64 chunks
fir          1 wf-seq              27790.1   0.9 27535.4..32703.7          27786.1                                        
fir          1 wf                  28057.0   0.3 27978.9..28607.6          28050.3                                        
```

The third control run, taken after this branch's first commit, so its
manifest names a revision on this branch rather than its parent. Its `wf`
and `wf-seq` rows:

```text
mandelbrot   2 wf                  13800.7   1.3 13588.3..14119.4          27023.5 1.031 [1.02-1.06]  1.014   0/5       3 16 chunks
mandelbrot   4 wf                   6980.3   1.1 6900.3..7348.5            27027.1 1.030 [0.99-1.10]  1.054   1/5       6 16 chunks
mandelbrot   8 wf                   6990.1   1.2 6877.2..7244.7            27097.1 1.030 [0.98-1.05]  1.003   1/5       9 16 chunks
mandelbrot   1 wf                  26601.9   0.6 26402.4..27183.7          26599.2                                        
mandelbrot   1 wf-seq              26654.2   0.2 26521.7..26707.7          26650.8                                        
quadrature   2 wf                   9158.6   2.4 8939.1..11148.3           18070.2 1.069 [1.01-1.43]  1.037   0/5     403 
quadrature   4 wf                   5445.3   8.2 5000.2..6326.9            17421.8 0.916 [0.83-1.17]  0.747   3/5    1035 
quadrature   8 wf                   5460.5   3.9 5155.1..5853.3            21293.9 0.986 [0.83-1.02]  1.016   4/5    1186 
quadrature   1 wf-seq              16137.5   3.5 14845.2..16798.9          16135.7                                        
quadrature   1 wf                  16480.0   0.6 16373.1..17071.7          16478.4                                        
records      2 wf                  14526.2   3.9 13961.2..15147.8          28446.5 0.887 [0.86-0.93]  0.883   5/5       1 32 chunks
records      4 wf                   7872.8   2.0 7508.1..9988.1            29403.1 0.949 [0.89-1.21]  1.179   4/5      12 64 chunks
records      8 wf                   8031.4   1.4 7725.9..8194.9            29999.6 0.935 [0.89-1.00]  0.906   5/5      19 64 chunks
records      1 wf                  28204.7   1.1 27903.5..29233.7          28200.0                                        
records      1 wf-seq              28606.3   2.0 27980.0..29241.6          28603.1                                        
fir          2 wf                  14411.1   1.0 13733.8..14648.2          27505.3 0.903 [0.85-0.92]  0.899   5/5       1 32 chunks
fir          4 wf                   7316.8   2.8 7108.8..8509.7            27384.9 0.898 [0.86-0.99]  0.853   5/5       9 64 chunks
fir          8 wf                   7683.2   2.9 7333.4..8220.4            28094.3 0.920 [0.89-0.97]  0.871   5/5      18 64 chunks
fir          1 wf-seq              27687.0   2.3 26858.1..28364.0          27682.9                                        
fir          1 wf                  27889.7   0.4 27458.1..27995.9          27884.7                                        
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the A/B twin and its null check

**This is the instrument, not a result.** The compute scoreboard now builds an
optional **second Whitefoot image per kernel** and times it inside the same
passes as the first, so two runtimes or two compiler controls can be compared
without comparing two separate runs. The run below is that instrument measured
against itself: the twin was built with `-DWF_PAR_SPLIT_WORK_UNIT=1200000`,
which is the value the runtime already compiles, and **every twin image came out
byte-identical to its plain image** — the manifest records both hashes and each
pair matches. So every `A/B` line here is this host's own within-pass paired
spread over identical bytes at two passes, and nothing else.

**Why the twin exists.** The section before this one could not select between
four values of the splitter's work unit because the host's run-to-run spread was
wider than the effect: quadrature's W=4 ratio spread 13.0 percent over six runs
of a decomposition that never changed, and records and fir W=4 spread 8.5 and
8.8 percent at an unchanged 64 chunks. The reducer already answers that for the
references — within each pass every form runs as its own process in a rotated,
alternating order, and the `wf` row's `ratio` is the median of within-pass
matched pairs rather than a quotient of two run medians. What was missing was a
second Whitefoot image in those same passes. `wf-b` is that image: the same cell
list, the same rotation and reversal, its own process, its own row with its own
`ratio`/`cpu_r`/`lower` against the best reference of each pass, and one line
per block pairing it with `wf` pass by pass.

**What the null check says.** Eleven of the sixteen `A/B` lines read within 1.6
percent of 1.000, and the `lower` counts are mixed as they should be over
identical bytes — 1/2 on nine lines, 2/2 on four, 0/2 on three:

```text
mandelbrot   2 A/B  wf-b/wf  wall 0.998 [1.00-1.00]  lower 1/2  cpu 1.002
mandelbrot   4 A/B  wf-b/wf  wall 1.004 [0.97-1.04]  lower 1/2  cpu 0.994
mandelbrot   8 A/B  wf-b/wf  wall 0.976 [0.96-0.99]  lower 2/2  cpu 1.023
mandelbrot   1 A/B  wf-b/wf  wall 1.004 [0.99-1.01]  lower 1/2  cpu 1.004
quadrature   2 A/B  wf-b/wf  wall 1.042 [1.02-1.07]  lower 0/2  cpu 1.056
quadrature   4 A/B  wf-b/wf  wall 1.014 [0.98-1.05]  lower 1/2  cpu 1.086
quadrature   8 A/B  wf-b/wf  wall 1.016 [0.99-1.04]  lower 1/2  cpu 0.922
quadrature   1 A/B  wf-b/wf  wall 1.048 [1.03-1.07]  lower 0/2  cpu 1.048
records      2 A/B  wf-b/wf  wall 0.935 [0.87-1.00]  lower 1/2  cpu 0.957
records      4 A/B  wf-b/wf  wall 0.998 [0.97-1.02]  lower 1/2  cpu 0.989
records      8 A/B  wf-b/wf  wall 0.974 [0.94-1.01]  lower 1/2  cpu 0.974
records      1 A/B  wf-b/wf  wall 0.978 [0.97-0.99]  lower 2/2  cpu 0.978
fir          2 A/B  wf-b/wf  wall 0.996 [0.99-1.00]  lower 2/2  cpu 1.007
fir          4 A/B  wf-b/wf  wall 0.880 [0.74-1.02]  lower 1/2  cpu 0.987
fir          8 A/B  wf-b/wf  wall 1.014 [1.01-1.02]  lower 0/2  cpu 1.019
fir          1 A/B  wf-b/wf  wall 1.004 [1.00-1.01]  lower 1/2  cpu 1.004
```

**And it says what the instrument's resolution is at two passes, which is the
more useful half.** Five lines are further out than 1.6 percent, and each one
has a visible cause in the rows above it rather than a difference between the
arms, because there is no difference between the arms:

- **fir W=4 reads 0.880 [0.74-1.02]** because one of the two `wf` processes was
  disturbed: that row's MAD is **18.5 percent** with a p10..p90 of
  7,342.7..10,673.8 us against the twin's 2.6 percent and 7,494.6..7,888.0. One
  excursion in one process moves a median of two pairs by twelve percent. The
  bundle retains excursions and discards no process, here as everywhere.
- **records W=2 reads 0.935 [0.87-1.00]** the same way, from a `wf` row with a
  6.2 percent MAD and a 18,440.9..20,891.1 spread.
- **quadrature W=1 1.048 (0/2) and W=2 1.042 (0/2)** are the only pair of lines
  that lean one way in both passes; quadrature is also the kernel with no
  independent-map split at all, so the two arms there are the same code taking
  the same path, and its W=1 row is the longest-running single-threaded cell in
  the run.

So the twin does not remove this host's spread; it removes the part of it that
lies *between runs*, and what remains is what a single disturbed process is
worth inside a pass. **At two passes that is a few percent, not a fraction of
one**, and an experiment that needs to resolve five percent should read five
paired passes and the `lower` count together rather than the wall median alone.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4   inherited mask: `0-3` (as recorded; `cgroup_cpu_max` and
  `cpuset.cpus.effective` both absent and recorded as unqualified)
- recorded block: none — **this run is not a candidate record.** It carries a
  `wf-b` row, so it is an A/B instrument by construction and its `wf` rows are
  read here only as the other half of a pair. W=4 is this host's highest
  non-oversubscribed block and W=8 is oversubscribed, as in every section above.
- compiler revision: `a3fef4b65b74d5b1264405904e720e8828c2d11e`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes and emitted chunk counts: unchanged — mandelbrot 98,304 points at limit
  256, shape `trailing`, **16 chunks** at W=2, W=4 and W=8; records 131,072 at
  `max_length` 255, **32 / 64 / 64**; fir K=64 over N=524,288, **32 / 64 / 64**;
  quadrature M=64 at tolerance `0x1p-54`, depth 24, `chunks=na`. The `wf-b` rows
  report the same counts, which is the first thing the twin had to get right:
  the harness asks the linked runtime for the split rather than re-deriving it,
  so an image built at another work unit or another oversubscription cap labels
  itself correctly.
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` empty
- runtime control flags: `WF_RUNTIME_CONTROL_FLAGS=-DWF_PAR_SPLIT_WORK_UNIT=1200000`,
  the compiled default, reaching the twin only. `WF_AB_TWIN: yes` in the
  manifest, and the table header carries the twin's disclosure line.
- image identity: every twin image is byte-identical to its plain image —
  `mandelbrot` and `mandelbrot-b` both `e546b37823fa5472…`, `quadrature`
  `efabfb8d95808395…`, `records` `2d468d594e628e59…`, `fir`
  `505fd6772508ee12…` — and each `-par-b.ll` matches its `-par.ll`
  (`f07c190e…`, `dc6aeaf3…`, `9ceebaed…`, `e0af1ed0…`, the same four modules as
  every section since `53359d73`).
- workflow run: `local`
- sizing window: every `wf-seq` median at W=1 inside [5 ms, 60 ms] — mandelbrot
  26.546, quadrature 16.607, records 36.410, fir 28.453 ms; every `wf` median at
  W=4 above 1 ms; `steals > 0` on the `wf` row at every parallel width, and on
  every `wf-b` row too, so no row carries `no-lanes`.
- passes: 2, calls: 5

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 11272's current affinity list: 0-3  date=2026-09-11T14:55:02Z
run=local  compiler=a3fef4b65b74d5b1264405904e720e8828c2d11e  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_WORK_UNIT=1200000
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=2 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13251.3   0.2 13224.7..13277.9          26488.7                                        
mandelbrot   2 rayon-iter          13340.8   0.0 13337.1..13344.6          26413.7                                        
mandelbrot   2 rayon-join          13437.9   0.1 13428.8..13447.0          26725.2                                        
mandelbrot   2 parlay              13556.2   0.4 13498.2..13614.3          26625.9                                        
mandelbrot   2 wf-b                13617.6   0.9 13490.8..13744.5          26710.4 1.028 [1.02-1.04]  1.008   0/2       3 16 chunks
mandelbrot   2 wf                  13638.4   1.1 13486.1..13790.7          26650.5 1.029 [1.02-1.04]  1.006   0/2       3 16 chunks
mandelbrot   2 static              26136.7   0.2 26080.6..26192.7          52237.8                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 0.998 [1.00-1.00]  lower 1/2  cpu 1.002
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6782.6   0.6 6743.0..6822.2            25817.8                                        
mandelbrot   4 wf                   7088.1   1.2 7004.2..7172.0            26949.1 1.045 [1.03-1.06]  1.044   0/2       5 16 chunks
mandelbrot   4 parlay               7105.1   1.4 7003.8..7206.5            27442.9                                        
mandelbrot   4 wf-b                 7111.2   2.7 6921.3..7301.1            26776.5 1.048 [1.03-1.07]  1.037   0/2       6 16 chunks
mandelbrot   4 rayon-join           7123.5   0.3 7103.5..7143.4            27448.8                                        
mandelbrot   4 rayon-iter           7149.2   0.6 7107.7..7190.6            27320.0                                        
mandelbrot   4 static              26224.6   0.7 26049.7..26399.6         103952.6                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 1.004 [0.97-1.04]  lower 1/2  cpu 0.994
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6985.2   0.5 6948.1..7022.4            27489.0                                        
mandelbrot   8 wf-b                 7028.6   0.6 6988.1..7069.1            26913.1 1.006 [1.00-1.02]  0.979   1/2       8 16 chunks
mandelbrot   8 rayon-join           7093.5   0.2 7080.2..7106.8            27585.0                                        
mandelbrot   8 parlay               7100.5   0.2 7089.7..7111.3            27655.3                                        
mandelbrot   8 rayon-iter           7116.3   1.2 7030.6..7202.0            27511.8                                        
mandelbrot   8 wf                   7201.3   0.9 7137.4..7265.1            26328.6 1.031 [1.03-1.03]  0.958   0/2       8 16 chunks
mandelbrot   8 static              31521.0   0.1 31496.5..31545.6         127371.6                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.976 [0.96-0.99]  lower 2/2  cpu 1.023
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26309.8   0.4 26202.6..26417.0          26300.5                                        
mandelbrot   1 wf-seq              26545.8   0.5 26424.6..26666.9          26543.6                                        
mandelbrot   1 wf                  26662.0   0.0 26649.5..26674.6          26658.5                                        
mandelbrot   1 wf-b                26760.7   0.9 26511.0..27010.4          26758.0                                        
mandelbrot   1 A/B  wf-b/wf  wall 1.004 [0.99-1.01]  lower 1/2  cpu 1.004
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen
  wf-b: compiler-chosen

quadrature   2 wf                   8634.6   2.0 8461.9..8807.4            16779.1 0.972 [0.97-0.97]  0.945   2/2     420 
quadrature   2 rayon-join           8884.5   2.2 8687.2..9081.9            17767.0                                        
quadrature   2 wf-b                 8998.3   4.5 8593.2..9403.3            17716.6 1.012 [0.99-1.04]  0.997   1/2     398 
quadrature   2 rayon-join-left      9083.4   1.6 8940.1..9226.7            18160.8                                        
quadrature   2 static               9865.3   0.1 9857.1..9873.5            18567.4                                        excursions retained
quadrature   2 tbb                 10262.4   1.0 10161.8..10363.0          19960.5                                        
quadrature   2 parlay              10508.0   5.0 9981.6..11034.3           18442.9                                        
quadrature   2 parlay-left         14264.5   9.2 12946.1..15582.8          21244.4                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   2 A/B  wf-b/wf  wall 1.042 [1.02-1.07]  lower 0/2  cpu 1.056
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   5129.5   4.0 4923.1..5336.0            17172.1 0.991 [0.96-1.03]  0.843   1/2    1005 
quadrature   4 rayon-join           5174.4   0.5 5148.1..5200.7            20362.2                                        
quadrature   4 wf-b                 5208.0   7.2 4834.6..5581.3            18670.5 1.006 [0.94-1.07]  0.916   1/2    1026 
quadrature   4 rayon-join-left      5600.4   3.5 5403.5..5797.4            21776.7                                        
quadrature   4 static               6415.7   8.5 5870.9..6960.6            30321.0                                        excursions retained
quadrature   4 tbb                  6631.7   2.4 6474.4..6788.9            26067.9                                        
quadrature   4 parlay               7360.1   4.0 7065.7..7654.5            23124.0                                        
quadrature   4 parlay-left          9502.1   2.5 9260.9..9743.2            26119.8                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 1.014 [0.98-1.05]  lower 1/2  cpu 1.086
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5041.4   0.8 5002.7..5080.1            19095.0 0.999 [0.94-1.06]  1.015   1/2    1164 
quadrature   8 wf-b                 5119.0   1.5 5044.5..5193.5            17477.1 1.016 [0.93-1.10]  0.941   1/2    1141 
quadrature   8 parlay               5392.9  12.7 4709.6..6076.1            18262.4                                        
quadrature   8 rayon-join           5468.3   0.7 5432.6..5503.9            21690.6                                        
quadrature   8 tbb                  6408.3   1.5 6312.4..6504.2            25230.7                                        
quadrature   8 rayon-join-left      7276.4   6.3 6815.2..7737.6            26592.8                                        
quadrature   8 parlay-left          7430.1   1.0 7352.6..7507.6            23446.3                                        
quadrature   8 static             524001.8   0.8 520008.5..527995.0      2084017.6                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 1.016 [0.99-1.04]  lower 1/2  cpu 0.922
  wf: compiler-chosen
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15612.1   0.1 15601.5..15622.7          15610.6                                        
quadrature   1 wf                  15751.2   1.1 15579.1..15923.2          15749.6                                        
quadrature   1 wf-b                16496.0   0.9 16341.6..16650.4          16494.7                                        
quadrature   1 wf-seq              16606.8   0.6 16504.2..16709.4          16603.9                                        
quadrature   1 A/B  wf-b/wf  wall 1.048 [1.03-1.07]  lower 0/2  cpu 1.048
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control

records      2 tbb                 16467.9   0.0 16466.7..16469.2          32322.0                                        
records      2 rayon-join          16847.2   0.2 16818.8..16875.5          33268.1                                        
records      2 parlay              17090.7   0.5 17008.4..17173.1          32197.2                                        
records      2 static              17813.0   6.8 16593.2..19032.8          33789.0                                        excursions retained
records      2 wf-b                18308.3   1.2 18093.2..18523.4          36329.7 1.112 [1.10-1.12]  1.124   0/2       1 32 chunks
records      2 rayon-iter          18646.0   3.6 17969.1..19323.0          36208.4                                        
records      2 wf                  19666.0   6.2 18440.9..20891.1          38114.7 1.194 [1.12-1.27]  1.180   0/2       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
records      2 A/B  wf-b/wf  wall 0.935 [0.87-1.00]  lower 1/2  cpu 0.957
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf-b: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

records      4 rayon-join           8323.8   0.2 8306.3..8341.3            32653.4                                        
records      4 static               8418.2   1.3 8307.8..8528.6            32365.2                                        excursions retained
records      4 parlay               8655.2   3.1 8388.5..8921.9            28714.4                                        
records      4 rayon-iter           8783.0   2.6 8550.4..9015.5            33038.6                                        
records      4 tbb                  9450.0   9.6 8544.8..10355.3           33521.5                                        
records      4 wf-b                 9689.7   1.9 9503.7..9875.8            36458.9 1.164 [1.14-1.18]  1.117   0/2       9 64 chunks
records      4 wf                   9710.4   0.7 9643.5..9777.3            36878.8 1.167 [1.16-1.18]  1.129   0/2       7 64 chunks
records      4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
records      4 A/B  wf-b/wf  wall 0.998 [0.97-1.02]  lower 1/2  cpu 0.989
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  wf: compiler-chosen

records      8 parlay               8573.9   0.6 8523.5..8624.2            33892.7                                        
records      8 rayon-join           8655.7   1.7 8511.7..8799.6            33535.3                                        
records      8 tbb                  8940.9   3.0 8674.1..9207.7            34248.1                                        
records      8 rayon-iter           9377.1   6.9 8730.3..10023.8           35182.5                                        
records      8 wf-b                10014.2   1.5 9866.4..10162.0           37386.9 1.176 [1.16-1.19]  1.121   0/2      18 64 chunks
records      8 wf                  10285.6   2.2 10061.2..10510.0          38437.8 1.208 [1.18-1.23]  1.152   0/2      18 64 chunks
records      8 static              16005.9   1.1 15833.6..16178.1          63424.7                                        excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 0.974 [0.94-1.01]  lower 1/2  cpu 0.974
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32870.3   2.0 32219.6..33521.0          32859.2                                        
records      1 wf-seq              36410.3   1.7 35777.1..37043.6          36406.1                                        
records      1 wf-b                36727.9   1.4 36220.4..37235.4          36723.3                                        
records      1 wf                  37557.5   2.1 36771.6..38343.4          37553.1                                        
records      1 A/B  wf-b/wf  wall 0.978 [0.97-0.99]  lower 2/2  cpu 0.978
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf-b: compiler-chosen
  wf: compiler-chosen

fir          2 wf-b                14446.1   1.1 14288.4..14603.9          27832.5 0.901 [0.88-0.92]  0.886   2/2       1 32 chunks
fir          2 wf                  14503.6   0.8 14385.6..14621.6          27631.1 0.905 [0.88-0.93]  0.880   2/2       1 32 chunks
fir          2 rayon-join          16135.7   3.6 15549.9..16721.4          31657.0                                        
fir          2 rayon-iter          16383.8   0.9 16240.3..16527.3          31976.8                                        
fir          2 parlay              16509.9   3.6 15912.5..17107.4          32043.3                                        
fir          2 static              16542.5   0.0 16542.4..16542.7          32537.9                                        excursions retained
fir          2 tbb                 16845.9   1.4 16604.9..17087.0          32788.5                                        
fir          2 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 0.996 [0.99-1.00]  lower 2/2  cpu 1.007
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

fir          4 wf-b                 7691.3   2.6 7494.6..7888.0            27641.6 0.949 [0.93-0.97]  0.887   2/2      11 64 chunks
fir          4 parlay               8266.9   2.6 8054.2..8479.6            31660.8                                        
fir          4 rayon-join           8295.3   1.7 8157.4..8433.2            31535.1                                        
fir          4 tbb                  8675.3   3.0 8416.6..8934.1            31353.9                                        
fir          4 static               8821.9   0.3 8796.0..8847.7            32516.6                                        excursions retained
fir          4 rayon-iter           8935.1   3.3 8637.5..9232.7            33113.0                                        
fir          4 wf                   9008.3  18.5 7342.7..10673.8           27993.7 1.110 [0.91-1.31]  0.899   1/2      12 64 chunks
fir          4 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: no
fir          4 A/B  wf-b/wf  wall 0.880 [0.74-1.02]  lower 1/2  cpu 0.987
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen

fir          8 wf                   7572.3   1.4 7462.6..7682.1            27582.4 0.942 [0.94-0.94]  0.897   2/2      18 64 chunks
fir          8 wf-b                 7675.8   0.6 7627.2..7724.4            28095.4 0.955 [0.94-0.97]  0.913   2/2      20 64 chunks
fir          8 tbb                  8042.5   1.8 7897.9..8187.1            30775.0                                        
fir          8 parlay               8484.4   1.3 8373.4..8595.3            33041.1                                        
fir          8 rayon-iter           8551.7   0.6 8501.4..8602.0            32424.1                                        
fir          8 rayon-join           8736.8   1.3 8619.0..8854.5            33594.9                                        
fir          8 static              15571.7  19.9 12471.9..18671.6          63152.8                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 1.014 [1.01-1.02]  lower 0/2  cpu 1.019
  wf: compiler-chosen
  wf-b: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27219.3   0.2 27168.1..27270.4          27207.6                                        
fir          1 wf-b                27337.3   0.8 27117.1..27557.4          27327.2                                        
fir          1 wf-seq              28452.7   0.4 28328.5..28576.8          28447.8                                        
fir          1 serial              35179.1   8.2 32288.6..38069.7          35145.0                                        
fir          1 A/B  wf-b/wf  wall 1.004 [1.00-1.01]  lower 1/2  cpu 1.004
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the oversubscription cap measured with the A/B twin

**No pair meets the acceptance and `WF_PAR_SPLIT_OVERSUBSCRIBE` stays at 16.**
The record is here because the refusal has a measured reason, and because that
reason is about the instrument as much as about the cap: the twin's own null arm
moved as far as the effect the twin was built to select.

**The question.** The rule is
`chunks = 2^floor(log2(min(WF_PAR_SPLIT_OVERSUBSCRIBE * lanes, span / ceil(work_unit / weight))))`.
The section before last could not take mandelbrot past 64 chunks at W=4 because
the `16 * lanes` cap held there, and records and fir never moved at W=2 or W=4
at any work unit because both already sat at the cap. Mandelbrot is the kernel
the question is about: `shape=trailing`, severe skew — `static`, the regular-work
reference, reads 19,899 us against oneTBB's 5,532 at W=4 on the hosted runner —
and oneTBB's `auto_partitioner` hands that same map out as about 1,536
callbacks against the compiled program's 16 chunks. So this experiment raises
**the cap and the work unit together**: raising the cap alone changes nothing on
mandelbrot, whose 16 chunks are what the work term affords, and raising the work
unit alone stops at the cap.

**The method, and what is new in it.** Three candidate pairs, one
`compare PASSES=5 CALLS=5` each into a fresh `RESULTS=` on an otherwise idle
host, each run comparing the candidate against the shipped runtime **inside the
same passes** through the bundle's new A/B twin rather than against a separate
control run: the plain image is `wf`, the candidate is `wf-b`, they run as
separate processes in the same rotated alternating cell list, and the
`A/B  wf-b/wf` line under each block is the median of their **within-pass paired
ratios**. `WF_PAR_CONTROL_FLAGS` was empty in all three, so every `wf` row is
the program plain `--par` produces, and **the plain image is byte-identical in
all three runs** — `mandelbrot` `e546b37823fa5472…`, `quadrature`
`efabfb8d95808395…`, `records` `2d468d594e628e59…`, `fir` `505fd6772508ee12…` —
so the three `wf` rows are three readings of one build.

**The chunk counts moved exactly as the rule predicts**, read off the `note`
column and confirmed from each twin image before its passes began:

| cap, work unit | mandelbrot W=2/4/8 | records W=2/4/8 | fir W=2/4/8 | quadrature |
|---|---|---|---|---|
| 16, 1,200,000 (the plain arm) | 16 / 16 / 16 | 32 / 64 / 64 | 32 / 64 / 64 | `chunks=na` |
| 64, 300,000 | **64 / 64 / 64** | 128 / 256 / 256 | 128 / 256 / 256 | `chunks=na` |
| 256, 75,000 | **256 / 256 / 256** | 512 / 1024 / 1024 | 512 / 1024 / 1024 | `chunks=na` |
| 1,024, 20,000 | **1024 / 1024 / 1024** | 2048 / 4096 / 4096 | 2048 / 2048 / 2048 | `chunks=na` |

Mandelbrot reads 64, 256 and 1,024 at W=4 as predicted; at (1024, 20,000) the
work term binds again at 1,068 affordable chunks, which is why it stops at 1,024
rather than reaching the cap of 4,096. Records and fir move at every width for
the first time, because the term that held them is the one being raised.

**The twin lines, all four kernels at every width.** `wall` is the median of the
five within-pass paired ratios, `lower` the number of those pairs with the
candidate faster, `cpu` the same pairing over process CPU:

| kernel | W=1 | W=2 | W=4 | W=8 |
|---|---|---|---|---|
| **cap 64, unit 300,000** | | | | |
| mandelbrot | 1.007 (1/5) cpu 1.005 | 1.006 (2/5) cpu 1.005 | **0.999 (3/5) cpu 1.004** | 0.996 (3/5) cpu 0.995 |
| quadrature | 1.003 (2/5) cpu 1.003 | 1.019 (2/5) cpu 1.012 | 0.936 (3/5) cpu 1.018 | 0.997 (3/5) cpu 1.017 |
| records | 1.013 (2/5) cpu 1.013 | 0.993 (3/5) cpu 1.004 | 0.952 (4/5) cpu 0.987 | 0.958 (3/5) cpu 0.984 |
| fir | 1.014 (1/5) cpu 1.014 | 1.002 (2/5) cpu 1.003 | 0.994 (4/5) cpu 1.029 | 1.031 (2/5) cpu 1.011 |
| **cap 256, unit 75,000** | | | | |
| mandelbrot | 1.009 (1/5) cpu 1.008 | 1.013 (0/5) cpu 1.009 | **0.992 (3/5) cpu 1.003** | 0.979 (3/5) cpu 1.004 |
| quadrature | 0.997 (3/5) cpu 0.997 | 0.957 (3/5) cpu 0.976 | 1.086 (0/5) cpu 1.029 | 1.020 (2/5) cpu 1.054 |
| records | 1.003 (2/5) cpu 1.003 | 0.967 (4/5) cpu 0.977 | 0.985 (3/5) cpu 0.930 | 0.963 (4/5) cpu 1.007 |
| fir | 0.982 (3/5) cpu 0.982 | 0.987 (3/5) cpu 0.991 | 0.968 (4/5) cpu 0.810 | 0.909 (5/5) cpu 0.922 |
| **cap 1,024, unit 20,000** | | | | |
| mandelbrot | 1.010 (1/5) cpu 1.010 | 0.997 (4/5) cpu 0.999 | **0.970 (5/5) cpu 1.007** | 0.995 (3/5) cpu 1.011 |
| quadrature | 1.055 (1/5) cpu 1.055 | 1.012 (1/5) cpu 1.010 | 0.940 (5/5) cpu 1.001 | 0.970 (4/5) cpu 1.015 |
| records | 0.996 (3/5) cpu 0.996 | 1.004 (2/5) cpu 0.984 | 0.992 (3/5) cpu 0.852 | 1.002 (1/5) cpu 1.057 |
| fir | 0.999 (3/5) cpu 0.999 | 1.053 (1/5) cpu 0.965 | 1.000 (3/5) cpu 0.855 | 1.009 (2/5) cpu 1.032 |

**Acceptance: not met, at any pair.** The criterion was mandelbrot's W=4 wall
below 1.000 with at least 4 of 5 paired passes lower, no kernel reproducibly
worse at either recorded width, W=4 paired CPU no worse than 1.05 on any kernel,
and W=1 unchanged within MAD. Mandelbrot's W=4 pair reads **0.999 (3/5)**,
**0.992 (3/5)** and **0.970 (5/5)**. The first two fail the `lower` count
outright. The third meets the clause as written and is still not selectable,
for the reason below; it also takes **fir's W=2 pair to 1.053 with one of five
lower**, the only reproducible regression at a recorded width in the three runs,
and quadrature's W=1 pair to 1.055 against that row's 1.3 and 4.3 percent MADs.

**Why the third pair cannot be credited: quadrature is a null arm in every one
of these runs.** It emits no `wf__par_split_budget` call at all — it
parallelizes by recursion structure — so neither the cap nor the work unit can
reach it, and **its two images do identical work in every pass of every run**.
Its W=4 pair reads:

```text
null check (identical bytes)   1.014 [0.98-1.05]  lower 1/2
cap 64,    unit 300,000        0.936 [0.90-1.03]  lower 3/5
cap 256,   unit 75,000         1.086 [1.02-1.21]  lower 0/5
cap 1,024, unit 20,000         0.940 [0.93-0.99]  lower 5/5
```

**A 5/5-lower reading of 0.940 over identical work sits in the same run as
mandelbrot's 5/5-lower 0.970.** Nothing distinguishes them. The span 0.936 to
1.086 is fifteen percent on a decomposition that never changed, and it contains
both the strongest positive and the strongest negative shape the criterion can
read.

**What that says about the twin, which is the more durable half of this
result.** The instrument does what it was built to do and no more. It removes
the *between-run* half of this host's spread — the plain image is byte-identical
across the three runs and its `wf` rows are steady — but it cannot remove the
difference between two images. The twin's runtime objects carry different
constants, so the two binaries differ in bytes and therefore in **code
placement**, which this bundle's own README names as its largest confound: six
byte-identical kernel bodies at different offsets once split a width-one median
7.3 ms against 10.7 ms with no scheduler involved. The null check's arms were
byte-identical and its lines sat near 1.000; these arms are not, and quadrature
measures exactly that difference and nothing else. **Read the null arm first in
any A/B run on this host**, and treat a candidate as selected only when it beats
what the null arm did in the same run.

**What the readings do support, offered as pointers and not as results.** At the
oversubscribed W=8, which carries no verdict on a four-CPU host, the middle pair
takes fir to **0.909 with five of five lower and paired CPU 0.922** and records
to 0.963 (4/5); at W=4 the same pair reads fir 0.968 (4/5) with paired CPU
**0.810** and records 0.985 (3/5) with CPU 0.930, so a finer grain does buy
CPU on the two kernels the cap was holding even where it does not buy wall.
Those are the rows an eight-CPU host, where W=8 is the recorded block and 1,024
chunks would be a cap rather than a consequence of oversubscription, should
look at first.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- recorded block: none — **all three runs carry a `wf-b` row and none is a
  candidate record.** W=4 is this host's highest non-oversubscribed block and is
  where the criterion is read; W=8 is oversubscribed and carries no verdict.
- compiler revision: `641520ddfe50512a540cdfa1741d693273ae1f1b` in all three
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes: unchanged — mandelbrot 98,304 points at limit 256, shape `trailing`;
  quadrature M=64 at tolerance `0x1p-54`, depth 24; records 131,072 at
  `max_length` 255; fir K=64 over N=524,288. Emitted chunk counts are the table
  above, from the `note` column of each run. No size constant changed.
- `--par` control flags: none; `WF_PAR_CONTROL_FLAGS` empty in all three manifests
- runtime control flags: `-DWF_PAR_SPLIT_OVERSUBSCRIBE=<cap> -DWF_PAR_SPLIT_WORK_UNIT=<unit>`,
  reaching the **twin only**; `WF_AB_TWIN: yes` in each manifest and the
  disclosure line in each table header. **These three tables are not plain
  tables and are not recorded as ones**: they are the arms of this experiment,
  and the constant that ships is unchanged.
- module identity: the four emitted `--par` modules are byte-identical across
  all three runs and identical to every section since `53359d73` —
  `mandelbrot-par.ll` `f07c190e3a396fc9…`, `quadrature-par.ll`
  `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`, `fir-par.ll`
  `e0af1ed03ef2bc52…` — and each twin's `-par-b.ll` matches its plain module, so
  the arms differ in the runtime link and in nothing the compiler emitted. The
  twin images differ from the plain ones in bytes, which is the point of the
  null-arm paragraph above.
- workflow run: `local`. The push carrying this record changes
  `compiler/src/backend/sched/core.c` in a comment and an `#ifndef` guard and
  `compiler/src/backend/sched/entry.c` in a comment, so no compiled default
  moves; the hosted `compute-bench` workflow runs both legs against an unchanged
  runtime constant and is not waited on here.
- sizing window, all three runs: every `wf-seq` median at W=1 inside
  [5 ms, 60 ms] — mandelbrot 26.817 / 26.947 / 27.315 ms, quadrature 16.156 /
  16.288 / 16.728, records 35.952 / 36.348 / 36.581, fir 27.817 / 27.813 /
  26.933; every `wf` median at W=4 above 1 ms; `steals > 0` on every `wf` and
  every `wf-b` row at every parallel width, so no row carries `no-lanes`.
- passes: 5, calls: 5

The first arm, **cap 64 with work unit 300,000**:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 13950's current affinity list: 0-3  date=2026-09-11T15:02:37Z
run=local  compiler=641520ddfe50512a540cdfa1741d693273ae1f1b  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_OVERSUBSCRIBE=64 -DWF_PAR_SPLIT_WORK_UNIT=300000
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 rayon-iter          13514.6   1.0 13375.5..14024.9          26830.1                                        
mandelbrot   2 rayon-join          13540.8   0.9 13412.9..14011.2          26914.1                                        
mandelbrot   2 parlay              13703.1   0.4 13598.7..13781.2          27044.3                                        
mandelbrot   2 tbb                 13729.3   0.7 13562.2..14166.7          27359.9                                        
mandelbrot   2 wf                  13866.8   2.4 13529.8..14277.9          27181.0 1.037 [1.01-1.05]  1.022   0/5       3 16 chunks
mandelbrot   2 wf-b                13943.5   0.8 13642.9..14444.1          27263.0 1.024 [1.02-1.08]  1.011   0/5       3 64 chunks
mandelbrot   2 static              26620.0   0.3 26530.0..26861.5          54559.9                                        excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 1.006 [0.98-1.04]  lower 2/5  cpu 1.005
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6948.0   2.9 6746.1..7404.6            26078.5                                        
mandelbrot   4 parlay               7118.8   1.3 6963.8..9349.1            27194.5                                        
mandelbrot   4 rayon-iter           7158.7   0.7 7107.6..7742.6            27557.1                                        
mandelbrot   4 wf                   7169.0   3.9 6850.3..7618.1            27282.4 1.042 [0.99-1.13]  1.026   2/5       6 16 chunks
mandelbrot   4 wf-b                 7350.7   1.7 6917.1..7489.2            27551.8 1.072 [0.99-1.11]  1.053   1/5       9 64 chunks
mandelbrot   4 rayon-join           7365.8   1.9 7069.1..7508.2            27752.0                                        
mandelbrot   4 static              26323.9   0.9 26084.6..27259.2         110091.6                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.999 [0.97-1.09]  lower 3/5  cpu 1.004
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6872.0   0.8 6814.6..7091.3            27426.0                                        
mandelbrot   8 rayon-join           7079.5   1.7 6956.5..7441.9            27678.7                                        
mandelbrot   8 parlay               7166.3   0.8 7090.3..9733.4            27974.0                                        
mandelbrot   8 wf-b                 7204.8   2.4 7030.0..8262.4            27418.4 1.038 [1.03-1.20]  1.006   0/5      15 64 chunks
mandelbrot   8 rayon-iter           7398.5   1.3 7301.6..7592.4            28045.5                                        
mandelbrot   8 wf                   7662.0   2.8 6963.4..7879.1            27527.9 1.120 [0.98-1.16]  1.020   1/5       9 16 chunks
mandelbrot   8 static              31505.1   0.2 31427.4..41129.3         127233.3                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.996 [0.89-1.07]  lower 3/5  cpu 0.995
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26656.4   0.8 26453.3..27193.3          26652.7                                        
mandelbrot   1 wf                  26765.3   0.3 26675.8..27102.1          26762.2                                        
mandelbrot   1 wf-seq              26816.6   0.5 26675.3..27072.3          26814.0                                        
mandelbrot   1 wf-b                26955.2   0.7 26773.3..27581.2          26893.7                                        
mandelbrot   1 A/B  wf-b/wf  wall 1.007 [0.99-1.02]  lower 1/5  cpu 1.005
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen

quadrature   2 wf-b                 8592.7   3.0 8339.1..10383.8           16594.0 1.009 [0.93-1.16]  0.995   1/5     396 
quadrature   2 wf                   8624.0   1.0 8436.0..8949.5            16590.4 0.999 [0.96-1.03]  0.979   3/5     427 
quadrature   2 rayon-join           8962.5   0.2 8487.8..8983.5            17846.9                                        
quadrature   2 rayon-join-left      9110.1   1.7 8336.9..9264.7            18218.2                                        
quadrature   2 static               9881.0   1.6 8728.1..10037.0           21749.1                                        excursions retained
quadrature   2 parlay              10096.9   2.4 9856.3..11226.4           18344.1                                        
quadrature   2 tbb                 10390.2   5.1 9494.5..11718.8           20506.5                                        
quadrature   2 parlay-left         12479.0   9.1 10747.1..16750.9          20561.5                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   2 A/B  wf-b/wf  wall 1.019 [0.97-1.16]  lower 2/5  cpu 1.012
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf-b                 5078.5   2.7 4940.7..6132.5            17262.9 0.985 [0.96-1.26]  0.832   3/5    1017 
quadrature   4 wf                   5365.6   3.5 4931.0..6790.2            17341.9 1.039 [0.95-1.40]  0.844   2/5     987 
quadrature   4 rayon-join-left      5431.4   3.0 4850.0..5596.2            21616.2                                        
quadrature   4 rayon-join           5432.8   7.5 5026.3..6083.9            20202.3                                        
quadrature   4 static               5652.1   6.2 5257.4..6200.6            17885.1                                        excursions retained
quadrature   4 tbb                  6274.7   0.2 6264.0..6816.2            25249.5                                        
quadrature   4 parlay               7460.8   1.2 6918.1..7655.6            22852.1                                        
quadrature   4 parlay-left          9928.4   6.6 8469.5..10893.8           26784.5                                        
quadrature   4 BEST REFERENCE = rayon-join-left FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 0.936 [0.90-1.03]  lower 3/5  cpu 1.018
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5146.8   6.1 4715.2..6149.4            17095.2 0.991 [0.73-1.15]  0.852   3/5    1195 
quadrature   8 wf-b                 5400.2  12.0 4738.3..6129.2            17526.0 1.046 [0.73-1.16]  0.875   2/5    1187 
quadrature   8 rayon-join           5431.9   6.3 5075.9..6620.9            21152.2                                        
quadrature   8 parlay-left          5701.6   9.5 5161.3..7068.8            22433.7                                        
quadrature   8 parlay               6009.8   9.4 5194.5..8037.2            23553.5                                        
quadrature   8 tbb                  6477.7   1.6 6133.3..6579.6            25924.0                                        
quadrature   8 rayon-join-left      6735.5   9.2 5424.5..7760.1            26024.3                                        
quadrature   8 static             523948.9   0.0 510953.0..524001.8      2061865.4                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 0.997 [0.98-1.18]  lower 3/5  cpu 1.017
  wf: compiler-chosen
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15252.9   1.8 14081.7..15984.4          15251.4                                        
quadrature   1 wf                  15800.4   4.7 14783.5..16549.1          15799.3                                        
quadrature   1 wf-b                16055.0   3.4 14428.5..16602.7          16053.6                                        
quadrature   1 wf-seq              16155.9   2.6 14460.8..16577.0          16154.8                                        
quadrature   1 A/B  wf-b/wf  wall 1.003 [0.96-1.03]  lower 2/5  cpu 1.003
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control

records      2 rayon-join          16889.2   0.8 16759.2..21335.1          33343.2                                        
records      2 parlay              17049.8   0.4 16985.9..18406.4          32150.8                                        
records      2 tbb                 17144.0   1.9 16824.6..17843.7          33654.7                                        
records      2 rayon-iter          17276.5   1.1 17078.7..23070.1          33544.3                                        
records      2 static              17297.1   2.0 16277.3..17747.4          33293.3                                        excursions retained
records      2 wf-b                18995.1   1.1 18735.2..19731.1          37338.5 1.141 [1.12-1.17]  1.145   0/5       1 128 chunks
records      2 wf                  19298.0   0.2 18809.0..21568.0          37274.2 1.149 [1.11-1.29]  1.143   0/5       2 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = rayon-join   WF fastest: no
records      2 A/B  wf-b/wf  wall 0.993 [0.87-1.02]  lower 3/5  cpu 1.004
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf-b: compiler-chosen
  wf: compiler-chosen

records      4 rayon-join           8658.7   2.8 8412.7..9309.5            34062.8                                        
records      4 rayon-iter           8810.3   1.7 8664.3..11308.7           33750.6                                        
records      4 parlay               8908.0   5.2 8271.8..11552.4           31000.2                                        
records      4 static               8930.9   4.2 8552.5..16347.5           44520.4                                        excursions retained
records      4 wf-b                 9954.4   2.7 9255.0..10965.1           38901.4 1.118 [1.07-1.33]  1.067   0/5      13 256 chunks
records      4 wf                  10064.4   2.6 9807.6..10565.4           38292.6 1.177 [1.13-1.25]  1.086   0/5       7 64 chunks
records      4 tbb                 10139.8   9.1 8448.4..13448.4           35346.8                                        
records      4 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
records      4 A/B  wf-b/wf  wall 0.952 [0.92-1.11]  lower 4/5  cpu 0.987
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

records      8 parlay               8718.8   1.4 8535.6..8982.4            34342.9                                        
records      8 rayon-join           8836.3   2.2 8464.6..10110.8           34316.6                                        
records      8 tbb                  8851.1   3.0 8331.6..9526.7            32650.4                                        
records      8 wf-b                 9714.0   0.6 9445.9..10576.4           37294.5 1.134 [1.09-1.22]  1.132   0/5      31 256 chunks
records      8 wf                  10077.3   5.8 9488.3..12468.3           38043.3 1.154 [1.11-1.50]  1.160   0/5      19 64 chunks
records      8 rayon-iter          10311.5   9.4 9157.6..13774.0           38891.3                                        
records      8 static              16649.3  14.4 12576.9..20082.9          76152.1                                        excursions retained
records      8 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 0.958 [0.76-1.06]  lower 3/5  cpu 0.984
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32740.4   1.3 32050.2..34093.6          32735.2                                        
records      1 wf-seq              35951.9   0.3 35199.1..36057.3          35946.9                                        
records      1 wf-b                36444.3   0.1 35773.4..38752.2          36439.3                                        
records      1 wf                  36873.4   2.4 35822.7..38057.4          36855.2                                        
records      1 A/B  wf-b/wf  wall 1.013 [0.94-1.04]  lower 2/5  cpu 1.013
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf-b: compiler-chosen
  wf: compiler-chosen

fir          2 wf                  13923.9   1.2 13645.6..17046.2          27083.0 0.867 [0.86-1.16]  0.850   4/5       1 32 chunks
fir          2 wf-b                14198.0   1.9 13674.4..14872.5          27652.3 0.885 [0.86-1.01]  0.866   4/5       1 128 chunks
fir          2 static              15943.3   2.1 14750.6..16277.5          31940.2                                        excursions retained
fir          2 rayon-join          16150.2   0.7 15748.1..17781.0          31846.9                                        
fir          2 tbb                 16252.9   2.0 15413.1..19026.8          31700.9                                        
fir          2 rayon-iter          16461.3   1.4 16118.3..16988.1          31972.6                                        
fir          2 parlay              16472.7   1.5 16047.9..16727.2          32123.2                                        
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 1.002 [0.87-1.03]  lower 2/5  cpu 1.003
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf-b                 7306.0   1.9 7169.7..8679.8            27601.2 0.904 [0.83-1.06]  0.873   3/5      13 256 chunks
fir          4 wf                   7352.2   0.4 7211.4..9039.1            27035.6 0.887 [0.83-1.10]  0.865   4/5       8 64 chunks
fir          4 tbb                  8300.8   1.4 8184.0..9509.9            31575.1                                        
fir          4 parlay               8828.7   3.8 8427.0..9667.6            32855.8                                        
fir          4 rayon-join           9121.3   6.4 8085.0..9707.2            32979.0                                        
fir          4 static               9261.7  12.4 8113.6..10676.3           33306.2                                        excursions retained
fir          4 rayon-iter           9426.8   2.6 8531.2..9667.7            34432.6                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          4 A/B  wf-b/wf  wall 0.994 [0.96-1.18]  lower 4/5  cpu 1.029
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf-b                 8011.1   1.8 7713.6..9301.3            29480.0 0.941 [0.87-1.05]  0.872   4/5      24 256 chunks
fir          8 wf                   8166.0   5.8 7468.8..8753.7            29600.9 0.938 [0.81-1.07]  0.893   4/5      19 64 chunks
fir          8 rayon-join           8884.2   3.8 8118.8..9220.0            33618.0                                        
fir          8 parlay               9043.0   6.3 8294.4..9734.0            35369.4                                        
fir          8 rayon-iter           9395.7   4.3 8146.8..10390.5           34476.4                                        
fir          8 tbb                  9410.1   2.8 8495.7..10000.9           34411.3                                        
fir          8 static              14056.3  12.0 12270.9..17444.8          60728.6                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 1.031 [0.88-1.14]  lower 2/5  cpu 1.011
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27238.2   1.4 26269.3..27727.9          27233.8                                        
fir          1 wf-b                27342.9   1.2 26625.0..28558.9          27336.4                                        
fir          1 wf-seq              27817.2   3.6 26814.6..29059.5          27812.0                                        
fir          1 serial              31999.6   1.8 31433.8..32806.2          31994.8                                        
fir          1 A/B  wf-b/wf  wall 1.014 [0.99-1.03]  lower 1/5  cpu 1.014
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

```

The second arm, **cap 256 with work unit 75,000**:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 17368's current affinity list: 0-3  date=2026-09-11T15:04:58Z
run=local  compiler=641520ddfe50512a540cdfa1741d693273ae1f1b  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_OVERSUBSCRIBE=256 -DWF_PAR_SPLIT_WORK_UNIT=75000
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13503.1   0.2 13481.1..13598.9          26683.3                                        
mandelbrot   2 rayon-iter          13626.2   1.1 13440.3..13836.1          26877.5                                        
mandelbrot   2 parlay              13668.8   0.3 13504.7..14354.5          27007.5                                        
mandelbrot   2 rayon-join          13677.5   0.7 13547.4..14026.2          27161.4                                        
mandelbrot   2 wf                  13799.9   0.4 13661.1..14040.8          26942.4 1.020 [1.01-1.03]  1.012   0/5       3 16 chunks
mandelbrot   2 wf-b                13982.1   0.3 13821.8..14218.9          27338.3 1.035 [1.02-1.05]  1.020   0/5       4 256 chunks
mandelbrot   2 static              26930.6   0.8 26315.8..27223.0          54563.5                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 1.013 [1.00-1.02]  lower 0/5  cpu 1.009
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  7018.2   2.3 6752.3..7228.8            26872.7                                        
mandelbrot   4 parlay               7033.2   0.9 6972.3..7627.3            27272.6                                        
mandelbrot   4 wf-b                 7152.3   1.4 7051.4..9153.0            27369.1 1.022 [1.00-1.33]  1.012   1/5      14 256 chunks
mandelbrot   4 rayon-iter           7176.6   2.3 7012.8..7451.0            27481.2                                        
mandelbrot   4 rayon-join           7233.2   2.6 7044.7..7714.2            27574.6                                        
mandelbrot   4 wf                   7388.9   3.3 7022.4..7631.0            27296.4 1.040 [1.03-1.06]  1.022   0/5       6 16 chunks
mandelbrot   4 static              26915.0   0.7 26290.3..27327.5         110262.7                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.992 [0.94-1.30]  lower 3/5  cpu 1.003
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6979.2   1.0 6848.7..7093.2            27729.5                                        
mandelbrot   8 rayon-join           7163.8   2.3 6967.7..7342.4            27908.2                                        
mandelbrot   8 parlay               7205.4   1.5 7047.9..7448.6            28269.9                                        
mandelbrot   8 wf-b                 7342.3   0.6 7208.5..7388.9            27763.1 1.052 [1.04-1.08]  1.004   0/5      24 256 chunks
mandelbrot   8 wf                   7438.1   1.7 7016.3..7577.5            27490.7 1.060 [1.02-1.10]  1.003   0/5       7 16 chunks
mandelbrot   8 rayon-iter           7480.4   2.8 7269.6..10140.0           28067.1                                        
mandelbrot   8 static              31493.1   0.1 29250.9..31526.4         127214.0                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.979 [0.95-1.05]  lower 3/5  cpu 1.004
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26546.4   0.3 26475.3..26916.9          26543.9                                        
mandelbrot   1 wf                  26810.0   0.2 26564.2..26869.3          26805.6                                        
mandelbrot   1 wf-seq              26946.9   1.4 26578.0..27698.8          26943.6                                        
mandelbrot   1 wf-b                26989.8   0.5 26632.3..27241.8          26985.7                                        
mandelbrot   1 A/B  wf-b/wf  wall 1.009 [1.00-1.02]  lower 1/5  cpu 1.008
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen

quadrature   2 rayon-join           8434.5   0.7 8192.9..9117.1            16867.1                                        
quadrature   2 wf-b                 8805.0   4.0 8455.2..10930.3           16742.1 1.040 [1.03-1.30]  1.002   0/5     408 
quadrature   2 wf                   8839.1   4.8 8415.4..11647.1           17382.2 1.079 [1.00-1.28]  1.036   0/5     391 
quadrature   2 rayon-join-left      9195.5   3.4 8407.4..9504.9            18388.9                                        
quadrature   2 static               9610.6   4.7 8711.3..10132.3           17912.8                                        excursions retained
quadrature   2 tbb                 10200.1   4.3 9704.1..10718.6           20398.5                                        
quadrature   2 parlay              10441.4   5.5 9212.8..11272.0           18618.8                                        
quadrature   2 parlay-left         13150.6   8.0 12099.2..17007.8          20867.1                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
quadrature   2 A/B  wf-b/wf  wall 0.957 [0.86-1.26]  lower 3/5  cpu 0.976
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   4951.2   4.2 4654.0..5518.6            17336.1 0.967 [0.93-1.11]  0.855   4/5    1021 
quadrature   4 rayon-join           5019.4   1.4 4950.6..5531.3            19870.8                                        
quadrature   4 wf-b                 5452.3   3.7 5106.8..5990.6            17844.6 1.065 [0.95-1.21]  0.862   1/5     992 
quadrature   4 rayon-join-left      5497.1   2.2 4955.1..5920.2            21497.3                                        
quadrature   4 static               6072.3   6.9 5185.3..6489.2            18348.7                                        excursions retained
quadrature   4 tbb                  6487.3   5.6 6102.8..7041.1            25937.5                                        
quadrature   4 parlay               7128.0   5.2 6759.2..8504.1            21716.3                                        
quadrature   4 parlay-left         10542.7   5.6 8997.6..11196.5           27303.1                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 1.086 [1.02-1.21]  lower 0/5  cpu 1.029
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf                   5129.5   2.3 5013.2..5839.3            17213.1 0.970 [0.89-1.17]  0.846   4/5    1178 
quadrature   8 wf-b                 5182.6   2.7 4648.0..5449.1            18101.8 0.971 [0.84-1.07]  0.873   3/5    1199 
quadrature   8 parlay-left          5509.6   0.8 5462.9..5990.7            22435.1                                        
quadrature   8 rayon-join           5526.2   3.4 4986.7..5750.2            21135.2                                        
quadrature   8 parlay               6355.4   8.9 5290.1..6922.4            23544.4                                        
quadrature   8 tbb                  6421.5   1.7 6294.1..7101.2            25215.2                                        
quadrature   8 rayon-join-left      7041.5   1.7 6925.0..7372.8            27018.3                                        
quadrature   8 static             519984.9   1.5 511971.5..539986.7      2054770.2                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 1.020 [0.86-1.06]  lower 2/5  cpu 1.054
  wf: compiler-chosen
  wf-b: compiler-chosen
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15698.0   1.2 15377.2..16373.2          15695.3                                        
quadrature   1 wf-b                16181.7   0.2 16088.6..16372.2          16180.5                                        
quadrature   1 wf-seq              16288.2   0.6 16078.9..16829.0          16284.5                                        
quadrature   1 wf                  16294.8   1.7 15959.8..16633.9          16293.4                                        
quadrature   1 A/B  wf-b/wf  wall 0.997 [0.97-1.02]  lower 3/5  cpu 0.997
  serial: none: one thread, a loop over all callbacks
  wf-b: compiler-chosen
  wf-seq: control
  wf: compiler-chosen

records      2 tbb                 16479.0   1.6 16171.1..16945.3          32449.7                                        
records      2 static              16526.8   0.7 15954.8..17166.0          32526.7                                        excursions retained
records      2 rayon-join          16604.8   2.3 16216.6..17697.8          32962.9                                        
records      2 parlay              17016.0   3.1 16335.1..17787.1          31658.2                                        
records      2 rayon-iter          17300.0   2.0 16609.2..17652.8          33352.7                                        
records      2 wf-b                18145.3   0.8 17958.0..19264.7          36033.3 1.122 [1.11-1.17]  1.132   0/5       3 512 chunks
records      2 wf                  18664.6   0.5 18577.1..19502.9          36457.5 1.161 [1.14-1.18]  1.153   0/5       1 32 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
records      2 A/B  wf-b/wf  wall 0.967 [0.94-1.00]  lower 4/5  cpu 0.977
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen

records      4 tbb                  8493.3   2.1 8313.6..9650.7            32443.7                                        
records      4 rayon-iter           8577.6   2.8 8327.3..12309.0           33036.8                                        
records      4 rayon-join           8621.1   1.1 8384.6..9031.9            33175.0                                        
records      4 static               8820.6   5.8 8306.7..16256.4           44723.8                                        excursions retained
records      4 parlay               9195.0   3.0 8552.6..10582.1           28905.1                                        
records      4 wf-b                 9562.6   4.1 9166.6..13612.7           34387.0 1.148 [1.05-1.64]  0.980   0/5      22 1024 chunks
records      4 wf                   9847.7   1.4 9580.2..12689.4           36904.5 1.166 [1.10-1.53]  1.122   0/5       7 64 chunks
records      4 BEST REFERENCE = rayon-join   FASTEST = tbb          WF fastest: no
records      4 A/B  wf-b/wf  wall 0.985 [0.96-1.07]  lower 3/5  cpu 0.930
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen

records      8 parlay               8598.9   0.9 8523.3..9157.9            33993.1                                        
records      8 rayon-iter           8771.1   2.6 8442.5..9012.4            33532.1                                        
records      8 tbb                  8798.2   0.5 8590.1..11508.1           33674.3                                        
records      8 rayon-join           8954.0   1.0 8471.5..12044.3           34709.4                                        
records      8 wf-b                 9376.8   0.6 9257.0..10366.0           36690.8 1.100 [1.08-1.21]  1.119   0/5      39 1024 chunks
records      8 wf                   9828.0   1.7 9657.2..10849.9           36594.9 1.164 [1.14-1.26]  1.115   0/5      19 64 chunks
records      8 static              19566.8   2.6 15544.5..20080.5          79559.3                                        excursions retained
records      8 BEST REFERENCE = rayon-iter   FASTEST = parlay       WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 0.963 [0.85-1.00]  lower 4/5  cpu 1.007
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              33664.4   1.5 32656.1..34163.5          33662.0                                        
records      1 wf-b                35706.4   0.5 35541.8..37739.4          35702.0                                        
records      1 wf                  36142.9   1.6 35547.2..38044.5          36134.1                                        
records      1 wf-seq              36347.5   0.6 35754.5..37560.2          36342.8                                        
records      1 A/B  wf-b/wf  wall 1.003 [0.95-1.04]  lower 2/5  cpu 1.003
  serial: none: one thread, a loop over all callbacks
  wf-b: compiler-chosen
  wf: compiler-chosen
  wf-seq: control

fir          2 wf-b                13835.2   0.3 13603.5..14088.1          26778.7 0.864 [0.84-0.88]  0.850   5/5       3 512 chunks
fir          2 wf                  13982.9   1.7 13427.8..14456.5          27020.4 0.875 [0.85-0.90]  0.863   5/5       1 32 chunks
fir          2 tbb                 16306.0   3.6 15656.6..17454.5          31829.8                                        
fir          2 static              16519.1   1.7 16074.5..16927.2          32511.8                                        excursions retained
fir          2 parlay              16584.3   3.3 15604.6..20337.0          32209.5                                        
fir          2 rayon-join          16650.3   1.6 15986.7..17735.1          32446.0                                        
fir          2 rayon-iter          16885.5   1.6 16288.2..17879.3          32377.4                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 0.987 [0.96-1.01]  lower 3/5  cpu 0.991
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          4 wf-b                 7113.8   2.0 6972.6..7970.4            22387.4 0.855 [0.81-0.96]  0.728   5/5      19 1024 chunks
fir          4 wf                   7728.9   6.1 7201.7..8714.0            27645.3 0.940 [0.84-1.05]  0.899   4/5       9 64 chunks
fir          4 tbb                  8458.5   1.5 8159.0..8787.8            30747.1                                        
fir          4 static               8610.8   1.1 8253.7..10038.4           32686.4                                        excursions retained
fir          4 parlay               8805.2   2.9 8553.4..9580.5            33585.9                                        
fir          4 rayon-iter           9135.0   4.3 8265.2..9529.3            32686.0                                        
fir          4 rayon-join           9900.7   1.9 8283.5..10087.8           37581.0                                        
fir          4 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: yes
fir          4 A/B  wf-b/wf  wall 0.968 [0.90-1.02]  lower 4/5  cpu 0.810
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

fir          8 wf-b                 7523.3   0.9 7328.8..7659.2            28154.0 0.827 [0.82-0.92]  0.860   5/5      38 1024 chunks
fir          8 wf                   8272.7   5.5 7817.8..9799.5            30242.9 0.986 [0.90-1.06]  0.944   4/5      18 64 chunks
fir          8 tbb                  8862.1   5.8 8350.7..9789.2            32765.2                                        
fir          8 rayon-join           8945.1   7.8 8092.8..9928.5            33453.8                                        
fir          8 rayon-iter           9125.5   1.7 8318.5..9285.0            33720.5                                        
fir          8 parlay               9255.6   9.9 8341.8..10451.1           36134.7                                        
fir          8 static              15858.0   9.9 14211.7..17917.0          63864.3                                        excursions retained
fir          8 BEST REFERENCE = rayon-iter   FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 0.909 [0.78-0.95]  lower 5/5  cpu 0.922
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27812.5   1.2 26678.0..28133.1          27807.8                                        
fir          1 wf                  27845.4   2.0 27296.5..28837.4          27791.5                                        
fir          1 wf-b                28000.4   1.0 27075.1..28271.9          27996.2                                        
fir          1 serial              32616.5   3.8 30313.3..37190.9          32611.9                                        
fir          1 A/B  wf-b/wf  wall 0.982 [0.94-1.03]  lower 3/5  cpu 0.982
  wf-seq: control
  wf: compiler-chosen
  wf-b: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

The third arm, **cap 1,024 with work unit 20,000**:

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 20739's current affinity list: 0-3  date=2026-09-11T15:07:34Z
run=local  compiler=641520ddfe50512a540cdfa1741d693273ae1f1b  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PAR_SPLIT_OVERSUBSCRIBE=1024 -DWF_PAR_SPLIT_WORK_UNIT=20000
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 parlay              13604.9   1.0 13468.7..13963.2          26887.8                                        
mandelbrot   2 tbb                 13629.4   1.1 13390.5..13919.0          27068.8                                        
mandelbrot   2 wf                  13753.0   0.4 13696.0..14231.7          27028.7 1.021 [0.99-1.05]  1.004   1/5       3 16 chunks
mandelbrot   2 wf-b                13772.9   0.8 13608.1..14035.6          27278.9 1.019 [0.98-1.03]  1.003   1/5       5 1024 chunks
mandelbrot   2 rayon-join          13780.2   1.2 13432.3..13939.3          27259.7                                        
mandelbrot   2 rayon-iter          13790.3   1.9 13528.8..14662.2          26989.6                                        
mandelbrot   2 static              26826.3   1.7 26367.1..27522.4          54602.6                                        excursions retained
mandelbrot   2 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 0.997 [0.99-1.00]  lower 4/5  cpu 0.999
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 rayon-iter           7040.4   0.5 7002.7..7682.3            27256.1                                        
mandelbrot   4 tbb                  7061.8   1.6 6743.1..7638.7            27383.2                                        
mandelbrot   4 wf-b                 7141.1   1.8 6986.5..7293.2            27384.0 1.019 [0.99-1.04]  1.007   2/5      14 1024 chunks
mandelbrot   4 rayon-join           7152.6   1.7 7032.2..7481.3            27457.7                                        
mandelbrot   4 parlay               7339.4   2.1 7041.0..7581.6            28069.1                                        
mandelbrot   4 wf                   7429.6   3.9 7046.7..10752.2           27313.0 1.045 [1.02-1.55]  1.000   0/5       7 16 chunks
mandelbrot   4 static              27055.5   0.7 26353.0..27741.4         110553.3                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = rayon-iter   WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.970 [0.66-0.99]  lower 5/5  cpu 1.007
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  7031.6   0.5 6925.1..7237.9            27913.3                                        
mandelbrot   8 rayon-join           7259.8   1.6 6965.6..7378.1            28140.8                                        
mandelbrot   8 rayon-iter           7350.9   0.3 7247.4..7601.1            28056.9                                        
mandelbrot   8 parlay               7461.0   2.3 7209.1..7692.4            29037.2                                        
mandelbrot   8 wf-b                 7804.6   8.9 7113.7..8885.6            28060.6 1.108 [1.02-1.27]  1.007   0/5      36 1024 chunks
mandelbrot   8 wf                   7926.2   4.9 7041.1..12621.2           27780.8 1.125 [1.01-1.80]  1.008   0/5       8 16 chunks
mandelbrot   8 static              31523.0   0.5 31274.1..33538.8         126061.7                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.995 [0.70-1.05]  lower 3/5  cpu 1.011
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26715.1   0.6 26410.8..27005.2          26711.0                                        
mandelbrot   1 wf                  27043.5   1.2 26607.3..27473.8          27031.2                                        
mandelbrot   1 wf-b                27208.6   0.8 26874.7..27473.7          27205.9                                        
mandelbrot   1 wf-seq              27314.5   1.3 26758.5..27667.6          27261.5                                        
mandelbrot   1 A/B  wf-b/wf  wall 1.010 [0.99-1.02]  lower 1/5  cpu 1.010
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control

quadrature   2 rayon-join           8601.5   1.3 8478.1..8715.4            17200.8                                        
quadrature   2 rayon-join-left      8620.1   1.2 8427.3..9280.3            17238.6                                        
quadrature   2 wf                   8708.6   1.3 8539.2..8834.0            16951.9 1.009 [0.98-1.04]  0.997   1/5     411 
quadrature   2 wf-b                 8899.5   2.9 8637.9..12836.4           16982.0 1.056 [0.99-1.48]  1.008   1/5     373 
quadrature   2 static               8952.4   3.8 8613.5..10524.0           16953.2                                        excursions retained
quadrature   2 parlay               9743.4   3.4 9384.1..11156.1           18046.4                                        
quadrature   2 tbb                 10078.4   1.0 9751.7..10687.1           19395.2                                        
quadrature   2 parlay-left         13101.0  11.3 11624.0..15514.9          20269.6                                        
quadrature   2 BEST REFERENCE = rayon-join-left FASTEST = rayon-join   WF fastest: no
quadrature   2 A/B  wf-b/wf  wall 1.012 [0.98-1.47]  lower 1/5  cpu 1.010
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf-b                 4927.4   0.8 4886.7..5217.9            17073.0 0.929 [0.81-0.98]  0.819   5/5    1016 
quadrature   4 wf                   5205.7   3.5 5011.3..5549.1            17299.6 0.948 [0.87-1.05]  0.818   3/5    1024 
quadrature   4 rayon-join           5301.5   6.2 4972.3..6751.9            20852.4                                        
quadrature   4 rayon-join-left      6146.1   7.5 5318.6..7004.1            22569.0                                        
quadrature   4 tbb                  6459.8   2.1 5996.4..6608.0            25762.9                                        
quadrature   4 static               6489.0   1.4 5761.4..6577.8            29780.4                                        excursions retained
quadrature   4 parlay               7386.7   0.7 7331.6..8231.9            23396.0                                        
quadrature   4 parlay-left         10369.1   3.4 10011.6..13261.9          27968.5                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 0.940 [0.93-0.99]  lower 5/5  cpu 1.001
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf-b                 5109.0   2.3 4981.2..5633.1            17235.1 0.963 [0.89-0.99]  0.832   5/5    1197 
quadrature   8 rayon-join           5718.6  10.1 5040.1..6312.1            22563.4                                        
quadrature   8 wf                   5806.0   5.7 5058.7..6137.5            17373.2 1.013 [0.88-1.19]  0.762   2/5    1183 
quadrature   8 parlay               6008.2   7.1 5524.2..7299.3            22955.3                                        
quadrature   8 rayon-join-left      6328.7  18.1 5174.2..9097.1            24826.6                                        
quadrature   8 tbb                  6545.5   1.7 6431.6..7016.7            25949.9                                        
quadrature   8 parlay-left          7559.9  28.0 5254.7..9748.1            27524.2                                        
quadrature   8 static             515987.6   0.8 511991.2..523979.4      2053654.5                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 0.970 [0.81-1.01]  lower 4/5  cpu 1.015
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15397.4   1.7 15048.0..16322.1          15396.0                                        
quadrature   1 wf                  16631.6   1.3 14648.1..16842.8          16619.7                                        
quadrature   1 wf-seq              16728.4   2.0 16255.3..17226.8          16726.9                                        
quadrature   1 wf-b                16844.7   4.3 15197.7..17615.4          16843.1                                        
quadrature   1 A/B  wf-b/wf  wall 1.055 [0.93-1.12]  lower 1/5  cpu 1.055
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen

records      2 tbb                 16421.0   0.2 16390.7..17537.9          32468.0                                        
records      2 rayon-iter          16918.1   2.7 16467.1..18375.4          32846.9                                        
records      2 static              16954.8   3.6 16141.6..18450.8          32936.8                                        excursions retained
records      2 rayon-join          17078.7   5.0 16227.3..19768.2          33402.2                                        
records      2 parlay              17333.9   5.3 16285.9..20737.6          32705.9                                        
records      2 wf                  18750.2   2.0 18382.3..22021.9          36463.1 1.153 [1.13-1.30]  1.123   0/5       1 32 chunks
records      2 wf-b                19202.3   3.6 18405.0..20689.6          35895.0 1.158 [1.14-1.22]  1.106   0/5       5 2048 chunks
records      2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
records      2 A/B  wf-b/wf  wall 1.004 [0.94-1.03]  lower 2/5  cpu 0.984
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen

records      4 parlay               8724.7   1.1 8310.7..8819.1            26541.3                                        
records      4 tbb                  8730.8   2.8 8329.9..9964.5            33430.3                                        
records      4 rayon-join           8962.9   2.3 8748.1..10310.4           34356.7                                        
records      4 rayon-iter           9218.6   2.6 8931.9..9477.6            35205.8                                        
records      4 wf-b                10215.4   7.1 9336.6..11753.4           33496.9 1.197 [1.10-1.35]  1.163   0/5      25 4096 chunks
records      4 wf                  10269.1   1.9 9961.3..10528.3           38946.7 1.200 [1.19-1.24]  1.375   0/5      11 64 chunks
records      4 static              13002.7  15.5 8615.5..15018.5           48986.7                                        excursions retained
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
records      4 A/B  wf-b/wf  wall 0.992 [0.92-1.12]  lower 3/5  cpu 0.852
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      8 rayon-iter           9251.8   4.0 8812.2..10091.1           35380.1                                        
records      8 parlay               9261.5   1.8 8622.6..10384.6           35760.5                                        
records      8 rayon-join           9596.2  10.9 8545.7..11197.6           36206.1                                        
records      8 tbb                  9660.2   8.1 8827.9..10803.7           37235.6                                        
records      8 wf                  10078.0   1.4 9750.9..10978.6           37450.6 1.141 [1.08-1.27]  1.108   0/5      18 64 chunks
records      8 wf-b                10235.3   4.7 9633.0..11331.1           39580.1 1.142 [1.11-1.28]  1.140   0/5      53 4096 chunks
records      8 static              19804.8   0.6 19477.2..20740.8          79482.6                                        excursions retained
records      8 BEST REFERENCE = parlay       FASTEST = rayon-iter   WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 1.002 [0.88-1.12]  lower 1/5  cpu 1.057
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32978.7   2.3 31885.8..33739.4          32973.8                                        
records      1 wf                  36064.0   0.4 35928.7..37367.7          36059.4                                        
records      1 wf-seq              36580.8   3.2 35285.7..38911.8          36526.0                                        
records      1 wf-b                37032.0   1.5 35675.6..48960.2          37026.4                                        
records      1 A/B  wf-b/wf  wall 0.996 [0.98-1.36]  lower 3/5  cpu 0.996
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen

fir          2 wf                  14362.6   3.4 13685.0..19042.4          27618.2 0.894 [0.83-1.17]  0.901   4/5       1 32 chunks
fir          2 wf-b                14709.8   0.8 14269.9..15122.8          27052.7 0.921 [0.88-0.94]  0.848   5/5       4 2048 chunks
fir          2 tbb                 16065.2   1.2 15871.1..17420.2          31271.6                                        
fir          2 static              16231.7   1.7 15707.8..16523.7          32226.2                                        excursions retained
fir          2 rayon-join          16393.0   1.7 15674.2..16764.4          32194.7                                        
fir          2 rayon-iter          16763.5   0.6 16623.1..20898.2          32437.6                                        
fir          2 parlay              16811.1   1.2 16579.8..17533.7          32750.3                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 1.053 [0.75-1.08]  lower 1/5  cpu 0.965
  wf: compiler-chosen
  wf-b: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf-b                 8352.1   1.8 8029.0..8531.9            24417.9 1.008 [0.95-1.05]  0.762   2/5      25 2048 chunks
fir          4 parlay               8456.8   0.4 8068.8..8523.8            31912.5                                        
fir          4 wf                   8503.2   1.9 7492.4..8660.8            30587.3 1.052 [0.88-1.06]  0.955   2/5      13 64 chunks
fir          4 static               8527.4   4.4 8156.4..16964.4           32637.7                                        excursions retained
fir          4 rayon-join           8596.6   1.9 8123.6..8830.3            32787.7                                        
fir          4 rayon-iter           8640.1   3.4 8122.5..9504.4            32396.0                                        
fir          4 tbb                  9603.5   4.4 8390.4..10022.2           34675.9                                        
fir          4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
fir          4 A/B  wf-b/wf  wall 1.000 [0.95-1.11]  lower 3/5  cpu 0.855
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

fir          8 wf                   7735.5   2.4 7547.6..9029.9            28129.1 0.926 [0.90-1.11]  0.872   4/5      19 64 chunks
fir          8 wf-b                 7808.1   2.7 7518.3..8112.2            29146.6 0.939 [0.90-1.00]  0.899   5/5      49 2048 chunks
fir          8 parlay               8451.6   1.6 8114.2..9338.4            32946.3                                        
fir          8 tbb                  8558.8   0.9 8355.7..9732.3            31942.0                                        
fir          8 rayon-iter           8967.7   5.3 8496.8..10843.3           33793.1                                        
fir          8 rayon-join           9028.6   5.7 8510.7..10333.1           34002.1                                        
fir          8 static              13721.2   6.9 12631.1..22204.6          60693.6                                        excursions retained
fir          8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 1.009 [0.90-1.03]  lower 2/5  cpu 1.032
  wf: compiler-chosen
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              26932.5   1.0 26667.8..28316.2          26929.1                                        
fir          1 wf-b                27252.7   1.3 26445.7..28689.0          27248.9                                        
fir          1 wf                  27330.7   1.2 26860.5..27683.0          27327.1                                        
fir          1 serial              32424.4   1.7 31629.8..32985.0          32421.5                                        
fir          1 A/B  wf-b/wf  wall 0.999 [0.97-1.05]  lower 3/5  cpu 0.999
  wf-seq: control
  wf-b: compiler-chosen
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks

```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), what code placement alone is worth, with a shifted null arm

**Every remaining gap on this scoreboard is about five percent, and this run
says the instrument cannot resolve five percent whenever the two arms differ in
placement.** Mandelbrot reads 1.02–1.06 against oneTBB at W=4, records 1.10 on
one hosted runner class, fir 1.02; the A/B twin was built to select effects of
that size and, until now, every candidate it measured differed in bytes as well
as in behaviour, so no reading could be attributed to one rather than the other.
The three sections here separate the two for the first time.

**The arm.** `compiler/src/backend/sched/core.c` now carries a never-called,
non-inlined 587-byte function under `#ifdef WF_PLACEMENT_PAD`, which nothing
that ships defines and `whitefootc` never passes; the bundle reaches it through
the runtime handle it already had. Defined, it sits ahead of every function in
that file, so the twin's `sched_core-b.o` and the two runtime objects the link
places after it — `sched_prim_host-b.o`, `sched_entry-b.o` — move by 587 bytes
and **the two images run identical code**. Nothing else differs: the emitted
modules are byte-identical (the hashes below), the plain image is byte-identical
to every section since `53359d73`, and the harness, every reference and the
sequential control object are one build shared by both arms.

**Quadrature is the cleanest reading.** It emits no `wf__par_split_budget` call
at all, so no runtime constant and no grain rule can reach it; with the pad
defined its two images do identical work through an identically built scheduler
that merely sits at a different offset.

**The twin lines, all four kernels at every width.** `wall` is the median of the
five within-pass paired ratios with its `[min-max]`, `lower` the number of those
pairs with the shifted arm faster, `cpu` the same pairing over process CPU:

```text
mandelbrot   2 A/B  wf-b/wf  wall 0.996 [0.92-1.01]  lower 4/5  cpu 0.998
mandelbrot   4 A/B  wf-b/wf  wall 0.992 [0.97-1.01]  lower 4/5  cpu 1.001
mandelbrot   8 A/B  wf-b/wf  wall 0.985 [0.91-1.07]  lower 3/5  cpu 0.993
mandelbrot   1 A/B  wf-b/wf  wall 1.008 [1.00-1.01]  lower 1/5  cpu 1.008
quadrature   2 A/B  wf-b/wf  wall 1.004 [0.94-1.03]  lower 2/5  cpu 0.997
quadrature   4 A/B  wf-b/wf  wall 1.004 [0.88-1.20]  lower 2/5  cpu 1.020
quadrature   8 A/B  wf-b/wf  wall 1.012 [0.78-1.05]  lower 2/5  cpu 0.969
quadrature   1 A/B  wf-b/wf  wall 1.005 [0.68-1.30]  lower 1/5  cpu 1.005
records      2 A/B  wf-b/wf  wall 1.008 [0.97-1.04]  lower 2/5  cpu 1.013
records      4 A/B  wf-b/wf  wall 0.961 [0.84-1.07]  lower 3/5  cpu 0.989
records      8 A/B  wf-b/wf  wall 1.007 [0.95-2.36]  lower 2/5  cpu 1.001
records      1 A/B  wf-b/wf  wall 0.995 [0.97-1.02]  lower 3/5  cpu 0.995
fir          2 A/B  wf-b/wf  wall 1.017 [0.97-1.14]  lower 2/5  cpu 1.002
fir          4 A/B  wf-b/wf  wall 1.006 [0.99-1.16]  lower 2/5  cpu 1.013
fir          8 A/B  wf-b/wf  wall 0.883 [0.83-1.12]  lower 3/5  cpu 0.948
fir          1 A/B  wf-b/wf  wall 1.008 [0.99-1.03]  lower 2/5  cpu 1.008
```

**What this establishes.** A placement change and nothing else moves a block
median by up to **11.7 percent** (fir W=8, 0.883) and moves five of the sixteen
lines by more than one percent, over two images that execute the same
instructions in the same order. The paired *range* is the louder half: over
identical work, quadrature's five pairs span **0.68 to 1.30** at W=1 and 0.88 to
1.20 at W=4, and records W=8 carries a pair at 2.36. The medians themselves are
well behaved — quadrature reads 1.005, 1.004, 1.004 and 1.012, all inside 1.2
percent — so the instrument is unbiased by placement and is not *precise* under
it: what a single A/B line can say on this host is bounded by several percent
whenever the arms are not byte-identical, which is exactly the size of every gap
the scoreboard is trying to close.

**What it does not establish.** The pad moves the scheduler, not the kernel. The
emitted module object is linked ahead of the four runtime objects, so the
kernel's own code sits at the same offsets in both arms and nothing here
measures what moving a kernel body does; the bundle's own six-image measurement,
where byte-identical kernel bodies at 0/16/48 mod 64 split a width-one median
7.3 ms against 10.7 ms, remains the reading for that. Nor does one run separate
placement from ordinary run-to-run noise line by line: it bounds what the two
together do to a paired line, which is the quantity an A/B verdict is read
against.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- compiler revision: `e69592dc41da7db3c9bff6b81826e822f939d0fb`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes: unchanged — mandelbrot 98,304 points at limit 256, shape `trailing`;
  quadrature M=64 at tolerance `0x1p-54`, depth 24; records 131,072 at
  `max_length` 255; fir K=64 over N=524,288. Emitted chunk counts are unchanged
  from every section since `53359d73` and identical in both arms — mandelbrot
  16/16/16, records and fir 32/64/64, quadrature `chunks=na` — because no
  control in these three runs reaches the splitter.
- module identity: the four emitted `--par` modules are byte-identical to every
  section since `53359d73` — `mandelbrot-par.ll` `f07c190e3a396fc9…`,
  `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`,
  `fir-par.ll` `e0af1ed03ef2bc52…` — and each twin `-par-b.ll` matches its plain
  module exactly, so nothing the compiler emitted differs between the arms in
  any of the three runs. `WF_PAR_CONTROL_FLAGS` was empty in all three.
- passes: 5, calls: 5
- runtime control flags: `WF_RUNTIME_CONTROL_FLAGS=-DWF_PLACEMENT_PAD=1`,
  reaching the **twin only**; `WF_FLAGS` unchanged; `WF_MODULE_CONTROL_FLAGS`
  empty. `WF_AB_TWIN: yes` in the manifest and the disclosure line in the table
  header. **This table carries a `wf-b` row and is not a candidate record.**
- image identity: plain images `mandelbrot` `e546b37823fa5472…`, `quadrature`
  `efabfb8d95808395…`, `records` `2d468d594e628e59…`, `fir` `505fd6772508ee12…`,
  the same four as every section since `53359d73`; the shifted twins are
  `5f4d2ac43f8ef178…`, `9f972d73c728a722…`, `fc5baaeff1f6135a…` and
  `6c603966c475eb0a…`.
- recorded block: none. W=4 is this host's highest non-oversubscribed block and
  W=8 is oversubscribed, as in every section above.
- workflow run: `local`
- sizing window: every `wf-seq` median at W=1 inside [5 ms, 60 ms] — mandelbrot
  26.515, quadrature 16.299, records 36.070, fir 27.327 ms; every `wf` median at
  W=4 above 1 ms; `steals > 0` on every `wf` and every `wf-b` row at every
  parallel width, so no row carries `no-lanes`.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 2799's current affinity list: 0-3  date=2026-09-11T15:47:53Z
run=local  compiler=e69592dc41da7db3c9bff6b81826e822f939d0fb  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PLACEMENT_PAD=1
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13306.8   0.6 13185.9..13424.3          26374.9                                        
mandelbrot   2 rayon-iter          13365.1   0.1 13351.8..14166.9          26533.1                                        
mandelbrot   2 parlay              13461.5   0.1 13444.6..13779.4          26607.2                                        
mandelbrot   2 rayon-join          13513.0   0.1 13439.5..13575.1          26779.9                                        
mandelbrot   2 wf-b                13640.7   0.8 13526.8..14301.2          26990.6 1.034 [1.02-1.07]  1.015   0/5       3 16 chunks
mandelbrot   2 wf                  13883.6   1.8 13582.1..14680.7          26916.9 1.038 [1.03-1.10]  1.019   0/5       3 16 chunks
mandelbrot   2 static              26346.8   0.4 26180.9..26848.6          54335.9                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 0.996 [0.92-1.01]  lower 4/5  cpu 0.998
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6770.2   1.3 6684.2..7523.7            25495.7                                        
mandelbrot   4 wf-b                 6932.5   0.5 6898.0..7069.3            26866.0 1.024 [0.96-1.05]  0.995   2/5       6 16 chunks
mandelbrot   4 wf                   6998.6   1.6 6855.8..7127.5            26787.6 1.047 [0.96-1.06]  1.007   2/5       6 16 chunks
mandelbrot   4 parlay               7005.4   1.6 6894.3..7360.1            26965.1                                        
mandelbrot   4 rayon-iter           7087.0   1.9 6950.1..8050.2            27190.9                                        
mandelbrot   4 rayon-join           7397.0   3.3 7149.3..7817.9            28340.3                                        
mandelbrot   4 static              26399.3   0.5 26243.8..26542.0         110036.1                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.992 [0.97-1.01]  lower 4/5  cpu 1.001
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6852.7   0.5 6772.0..7106.7            27062.6                                        
mandelbrot   8 rayon-join           7136.2   2.0 6887.8..7380.8            27630.9                                        
mandelbrot   8 parlay               7151.2   1.7 6918.6..7309.7            28001.5                                        
mandelbrot   8 rayon-iter           7181.2   1.1 7101.4..7714.9            27686.8                                        
mandelbrot   8 wf-b                 7213.3   3.4 6967.4..7754.2            27082.3 1.053 [1.02-1.12]  0.996   0/5       7 16 chunks
mandelbrot   8 wf                   7326.0   1.5 7216.8..7772.7            27234.6 1.069 [1.02-1.15]  1.007   0/5      10 16 chunks
mandelbrot   8 static              31513.5   0.0 31503.7..31608.2         127077.4                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.985 [0.91-1.07]  lower 3/5  cpu 0.993
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf                  26475.1   0.3 26408.3..26749.8          26472.1                                        
mandelbrot   1 wf-seq              26514.6   0.2 26463.8..26950.2          26511.0                                        
mandelbrot   1 serial              26560.5   0.5 26388.9..26741.6          26557.6                                        
mandelbrot   1 wf-b                26673.8   0.0 26396.9..27040.3          26658.6                                        
mandelbrot   1 A/B  wf-b/wf  wall 1.008 [1.00-1.01]  lower 1/5  cpu 1.008
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
  wf-b: compiler-chosen

quadrature   2 wf-b                 8738.0   2.2 8545.0..9330.0            16891.2 1.025 [0.95-1.08]  0.970   2/5     412 
quadrature   2 rayon-join           9011.3   5.4 8524.6..10096.0           17928.1                                        
quadrature   2 rayon-join-left      9094.2   3.8 8649.5..10027.7           18149.5                                        
quadrature   2 wf                   9158.7   4.2 8327.6..9543.4            17227.5 1.021 [0.92-1.11]  0.999   1/5     424 
quadrature   2 static               9464.0   7.8 8726.8..10267.3           17844.5                                        excursions retained
quadrature   2 tbb                 10196.1   1.4 10050.1..11635.4          20389.7                                        
quadrature   2 parlay              10300.8   0.6 9922.3..10888.0           18044.2                                        
quadrature   2 parlay-left         13755.6  12.0 11943.6..16278.9          20675.6                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
quadrature   2 A/B  wf-b/wf  wall 1.004 [0.94-1.03]  lower 2/5  cpu 0.997
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf                   5158.3   9.0 4694.7..5975.5            23064.4 1.049 [0.89-1.13]  1.180   2/5    1021 
quadrature   4 wf-b                 5258.1   6.8 4899.4..5998.5            18820.0 1.031 [0.96-1.14]  0.963   2/5    1034 
quadrature   4 rayon-join           5272.9   3.4 4917.6..5686.6            20474.3                                        
quadrature   4 rayon-join-left      5585.3   8.1 4948.7..6064.8            22135.6                                        
quadrature   4 static               6086.5   7.6 5621.1..8159.5            29996.1                                        excursions retained
quadrature   4 tbb                  6898.7   3.4 6493.8..7130.8            25757.7                                        
quadrature   4 parlay               8086.9   8.5 7402.1..8867.0            23774.5                                        
quadrature   4 parlay-left         10638.6   7.6 9215.6..12640.7           27521.0                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 1.004 [0.88-1.20]  lower 2/5  cpu 1.020
  wf: compiler-chosen
  wf-b: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf-b                 4979.3   3.1 4755.7..5814.6            16930.3 0.937 [0.79-1.10]  0.789   4/5    1205 
quadrature   8 wf                   5517.0  12.9 4698.4..6376.6            17764.6 0.997 [0.78-1.20]  0.839   3/5    1172 
quadrature   8 rayon-join           5545.0   4.2 5303.7..6006.8            21834.2                                        
quadrature   8 tbb                  6694.8   2.1 6137.0..6999.4            26523.0                                        
quadrature   8 parlay               6711.9  11.1 5168.9..7827.3            21613.4                                        
quadrature   8 parlay-left          7106.1  10.2 6180.7..9448.4            24459.5                                        
quadrature   8 rayon-join-left      7453.0   3.2 6391.4..7877.6            27915.5                                        
quadrature   8 static             511986.6   0.0 507991.9..519970.6      2040609.2                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 1.012 [0.78-1.05]  lower 2/5  cpu 0.969
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15540.5   1.5 15256.5..15770.5          15538.7                                        
quadrature   1 wf-seq              16299.1   2.6 14994.6..16734.6          16297.8                                        
quadrature   1 wf                  16580.9   0.6 16474.4..23837.3          16579.5                                        
quadrature   1 wf-b                16675.5   1.3 16274.5..21481.8          16673.8                                        
quadrature   1 A/B  wf-b/wf  wall 1.005 [0.68-1.30]  lower 1/5  cpu 1.005
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen
  wf-b: compiler-chosen

records      2 rayon-join          16465.5   0.6 16165.5..16897.5          32700.7                                        
records      2 static              16539.2   1.0 16311.3..17234.8          32536.8                                        excursions retained
records      2 tbb                 16569.5   1.4 16315.4..17630.2          32050.5                                        
records      2 rayon-iter          16633.8   2.1 16281.7..17611.4          32398.4                                        
records      2 parlay              16784.8   2.2 16172.0..18517.3          31856.5                                        
records      2 wf-b                18339.4   0.2 18308.8..19847.2          36192.2 1.134 [1.11-1.22]  1.128   0/5       1 32 chunks
records      2 wf                  18952.6   0.4 18045.7..19024.1          36861.9 1.166 [1.10-1.17]  1.143   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
records      2 A/B  wf-b/wf  wall 1.008 [0.97-1.04]  lower 2/5  cpu 1.013
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen

records      4 tbb                  8559.1   1.0 8472.1..11104.7           32413.7                                        
records      4 rayon-join           8790.9   3.1 8515.9..11789.0           33669.4                                        
records      4 rayon-iter           9048.8   3.2 8762.5..9674.8            34364.2                                        
records      4 parlay               9119.6   1.8 8953.2..10973.7           31262.1                                        
records      4 static               9431.8   6.6 8124.6..11760.8           33375.5                                        excursions retained
records      4 wf                   9801.2   2.4 9565.0..14369.5           36746.5 1.157 [1.09-1.77]  1.225   0/5       7 64 chunks
records      4 wf-b                10168.8   2.9 9400.1..12130.7           36940.4 1.160 [1.10-1.49]  1.115   0/5      11 64 chunks
records      4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
records      4 A/B  wf-b/wf  wall 0.961 [0.84-1.07]  lower 3/5  cpu 0.989
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  wf: compiler-chosen
  wf-b: compiler-chosen

records      8 tbb                  8774.5   1.3 8664.4..9314.1            33885.2                                        
records      8 parlay               8791.2   3.0 8524.4..9422.0            34729.1                                        
records      8 rayon-join           8851.0   1.0 8505.6..9372.5            33948.2                                        
records      8 rayon-iter           9129.1   2.4 8569.6..9522.6            33969.4                                        
records      8 wf                  10211.1   4.7 9554.8..11462.5           38274.1 1.168 [1.09-1.26]  1.136   0/5      21 64 chunks
records      8 wf-b                10351.7   6.6 9620.9..27021.2           37993.5 1.214 [1.10-2.90]  1.124   0/5      20 64 chunks
records      8 static              18847.8   5.4 14733.7..19874.8          76673.3                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 1.007 [0.95-2.36]  lower 2/5  cpu 1.001
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32910.2   1.8 32111.2..34120.4          32906.6                                        
records      1 wf                  35808.2   1.2 35364.2..38082.0          35805.7                                        
records      1 wf-b                35995.4   0.4 35460.8..37900.3          35990.3                                        
records      1 wf-seq              36069.7   0.8 35788.9..38632.0          36065.1                                        
records      1 A/B  wf-b/wf  wall 0.995 [0.97-1.02]  lower 3/5  cpu 0.995
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control

fir          2 wf                  14467.8   3.1 13912.6..15034.3          27928.7 0.888 [0.85-0.91]  0.865   5/5       1 32 chunks
fir          2 wf-b                14642.6   1.5 13487.2..16556.6          27835.1 0.883 [0.85-1.02]  0.860   4/5       1 32 chunks
fir          2 static              16515.1   2.3 15850.5..17338.6          32509.0                                        excursions retained
fir          2 rayon-join          16577.4   0.9 16174.5..23889.0          32380.3                                        
fir          2 rayon-iter          16676.4   1.8 15962.8..19285.1          32228.2                                        
fir          2 parlay              16870.5   1.6 16603.0..17713.1          33016.5                                        
fir          2 tbb                 17122.0   2.1 16233.5..17487.4          32199.3                                        
fir          2 BEST REFERENCE = static       FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 1.017 [0.97-1.14]  lower 2/5  cpu 1.002
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1

fir          4 wf                   7469.1   3.3 7219.7..8282.5            27025.5 0.892 [0.86-0.99]  0.867   5/5       6 64 chunks
fir          4 wf-b                 8167.4   2.4 7248.5..8375.4            29559.8 0.976 [0.85-1.02]  0.946   4/5       9 64 chunks
fir          4 parlay               8618.3   1.8 8372.9..8912.3            32561.4                                        
fir          4 rayon-join           8773.5   3.1 8176.3..9877.5            33083.7                                        
fir          4 tbb                  8836.7   3.0 8377.6..9099.0            31246.4                                        
fir          4 static               8979.2   3.5 8547.8..10624.0           32958.1                                        excursions retained
fir          4 rayon-iter           9152.7  10.6 8186.4..10875.1           34309.9                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          4 A/B  wf-b/wf  wall 1.006 [0.99-1.16]  lower 2/5  cpu 1.013
  wf: compiler-chosen
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting

fir          8 wf-b                 8475.1   4.0 7508.2..8816.5            29960.4 0.976 [0.87-1.09]  0.934   3/5      20 64 chunks
fir          8 wf                   8502.1   7.3 7881.4..10212.3           30257.7 0.987 [0.97-1.15]  0.960   3/5      19 64 chunks
fir          8 rayon-join           8610.9   5.8 8109.3..9269.9            31916.8                                        
fir          8 parlay               8663.1   5.1 8160.8..9462.9            33251.6                                        
fir          8 tbb                  8750.9   4.1 8390.7..9676.9            32651.2                                        
fir          8 rayon-iter           8981.0   1.1 8333.5..9375.1            32465.1                                        
fir          8 static              12561.8   0.8 12466.7..17948.7          48578.4                                        excursions retained
fir          8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 0.883 [0.83-1.12]  lower 3/5  cpu 0.948
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf                  27102.1   0.7 26725.4..27565.2          27097.2                                        
fir          1 wf-seq              27326.7   0.5 27198.0..28117.2          27322.9                                        
fir          1 wf-b                27356.3   1.2 26418.2..28308.0          27351.7                                        
fir          1 serial              31594.4   2.4 29314.1..32851.9          31590.7                                        
fir          1 A/B  wf-b/wf  wall 1.008 [0.99-1.03]  lower 2/5  cpu 1.008
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), the same shifted null arm with both arms aligned

**Alignment does not remove the placement sensitivity, and the candidate remedy
is refused on this reading.** The criterion was written before the run: if
`-falign-functions=64 -falign-loops=32` is what the previous section's readings
are made of, then compiling **both** arms with it and shifting one by the same
587-byte pad should bring the `wf-b/wf` lines back near 1.000 with mixed `lower`
counts.

**The method.** One `compare PASSES=5 CALLS=5` on the same idle host minutes
after the previous one, with `WF_FLAGS` overridden for this run only so the
alignment flags reach **both** arms' Whitefoot side — the emitted module object,
the four runtime units and the shared sequential control object — and the same
`-DWF_PLACEMENT_PAD=1` on the twin. The references are untouched: they already
carry `-falign-loops=32` in `CFLAGS`.

```text
mandelbrot   2 A/B  wf-b/wf  wall 1.006 [0.98-1.05]  lower 2/5  cpu 1.002
mandelbrot   4 A/B  wf-b/wf  wall 0.989 [0.95-1.12]  lower 3/5  cpu 0.994
mandelbrot   8 A/B  wf-b/wf  wall 0.980 [0.95-1.00]  lower 5/5  cpu 1.010
mandelbrot   1 A/B  wf-b/wf  wall 0.999 [0.99-1.02]  lower 3/5  cpu 1.000
quadrature   2 A/B  wf-b/wf  wall 1.026 [0.93-1.26]  lower 2/5  cpu 1.048
quadrature   4 A/B  wf-b/wf  wall 1.057 [1.02-1.09]  lower 0/5  cpu 0.929
quadrature   8 A/B  wf-b/wf  wall 0.968 [0.95-1.21]  lower 4/5  cpu 0.978
quadrature   1 A/B  wf-b/wf  wall 1.007 [0.97-1.02]  lower 2/5  cpu 1.007
records      2 A/B  wf-b/wf  wall 0.986 [0.95-1.17]  lower 3/5  cpu 0.973
records      4 A/B  wf-b/wf  wall 1.068 [0.79-1.10]  lower 1/5  cpu 1.033
records      8 A/B  wf-b/wf  wall 0.944 [0.91-1.03]  lower 4/5  cpu 0.970
records      1 A/B  wf-b/wf  wall 0.995 [0.99-1.02]  lower 3/5  cpu 0.995
fir          2 A/B  wf-b/wf  wall 0.999 [0.80-1.04]  lower 3/5  cpu 0.990
fir          4 A/B  wf-b/wf  wall 0.956 [0.94-1.03]  lower 3/5  cpu 0.982
fir          8 A/B  wf-b/wf  wall 0.969 [0.85-1.05]  lower 4/5  cpu 0.949
fir          1 A/B  wf-b/wf  wall 1.004 [0.98-1.03]  lower 2/5  cpu 1.004
```

**It reads worse, not better.** Taking the sixteen lines' distance from 1.000:
the median line moves **1.7 percent** here against **0.8 percent** in the
unaligned run, and the mean 2.4 against 1.7 percent. Two lines are one-sided —
quadrature W=4 at **1.057 with none of five pairs lower**, mandelbrot W=8 at
0.980 with five of five — where the unaligned run had none, and quadrature's
1.057 is the exact shape an accepted candidate would have to be distinguished
from. Records W=4 reads 1.068 (1/5). Only the tail is shorter: the widest line
is 6.8 percent here against 11.7 percent unaligned, which is one excursion's
worth of difference and not a claim either way.

**And the mechanism rules alignment out rather than merely failing to select
it.** Under `-falign-functions=64` every function in the twin starts at a
64-byte boundary whether the pad is there or not, so the pad cannot change any
function's alignment modulo 64 and cannot change any loop's; the flags make the
arms' *internal* alignment identical by construction. What still moves the
timings is therefore not function or loop alignment — it is the rest of what
placement means on this machine, which an alignment flag does not reach. The
previous section's spread is not an alignment artifact and a compiler flag that
aligns functions and loops cannot be its remedy.

**One row moved a great deal and it is not a ratio row.** Records' `wf-seq`
median at W=1 — the `--no-overlap` control object, which this run compiled with
alignment and the other two did not — reads **28.180 ms here against 36.070 and
35.777 ms in the two plain runs**, with MADs of 0.7, 0.8 and 0.6 percent. That
is a 21 percent gain from alignment on Whitefoot's *sequential* lowering of one
kernel, while the same kernel's `--par` module at width one is unmoved
(35.932 ms here, 35.808 and 35.685 there). It is unpaired, one run, and on a row
that never enters the ratio column, so it decides nothing here; it is recorded
because it is the only reading in the three runs that suggests the alignment
question is worth reopening with an instrument that can resolve it.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- compiler revision: `e69592dc41da7db3c9bff6b81826e822f939d0fb`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes: unchanged — mandelbrot 98,304 points at limit 256, shape `trailing`;
  quadrature M=64 at tolerance `0x1p-54`, depth 24; records 131,072 at
  `max_length` 255; fir K=64 over N=524,288. Emitted chunk counts are unchanged
  from every section since `53359d73` and identical in both arms — mandelbrot
  16/16/16, records and fir 32/64/64, quadrature `chunks=na` — because no
  control in these three runs reaches the splitter.
- module identity: the four emitted `--par` modules are byte-identical to every
  section since `53359d73` — `mandelbrot-par.ll` `f07c190e3a396fc9…`,
  `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`,
  `fir-par.ll` `e0af1ed03ef2bc52…` — and each twin `-par-b.ll` matches its plain
  module exactly, so nothing the compiler emitted differs between the arms in
  any of the three runs. `WF_PAR_CONTROL_FLAGS` was empty in all three.
- passes: 5, calls: 5
- runtime control flags: `WF_RUNTIME_CONTROL_FLAGS=-DWF_PLACEMENT_PAD=1`,
  reaching the **twin only**. `WF_FLAGS` overridden **for this run only** to
  `-std=c11 -pthread -O2 -Wno-override-module -falign-functions=64
  -falign-loops=32`, reaching **both** arms, as the manifest and the table
  header record. `WF_MODULE_CONTROL_FLAGS` empty. **This table carries a `wf-b`
  row and is built at flags `whitefootc` does not pass; it is not a candidate
  record and its `wf` row is not the shipped program.**
- image identity: every image in this run differs from the plain images of the
  other two, as the flag override requires — `mandelbrot` `8a4d2b03f749327d…`,
  `quadrature` `1ce696413135c434…`, `records` `2b7ddecf29a848f6…`, `fir`
  `07bbd983e9db58cd…`, with twins `c3acfd159f7d978d…`, `71e8d7ae84855c03…`,
  `7606b88443063b04…` and `1fd239b4aa0ebf81…`.
- recorded block: none, for two reasons: the `wf-b` row and the flag override.
- workflow run: `local`
- sizing window: every `wf-seq` median at W=1 inside [5 ms, 60 ms] — mandelbrot
  27.101, quadrature 16.639, records 28.180, fir 27.482 ms; every `wf` median at
  W=4 above 1 ms; `steals > 0` on every `wf` and every `wf-b` row at every
  parallel width.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 6880's current affinity list: 0-3  date=2026-09-11T15:50:23Z
run=local  compiler=e69592dc41da7db3c9bff6b81826e822f939d0fb  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module -falign-functions=64 -falign-loops=32
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF runtime control flags=-DWF_PLACEMENT_PAD=1
      (appended to the compile of the `wf-b` TWIN's Whitefoot runtime only:
      `wf` above is still the runtime this tree ships. A table with a wf-b row
      is an A/B instrument and must not be recorded as a plain table -- see
      README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13462.3   0.7 13373.0..13677.1          26782.6                                        
mandelbrot   2 parlay              13711.6   0.6 13556.6..13988.6          27093.7                                        
mandelbrot   2 rayon-join          13752.8   0.8 13640.0..14184.9          27306.3                                        
mandelbrot   2 wf                  13853.6   0.5 13759.3..14596.9          27042.6 1.030 [1.02-1.07]  1.016   0/5       3 16 chunks
mandelbrot   2 wf-b                13866.9   2.2 13559.9..15253.3          27060.7 1.037 [1.00-1.12]  1.012   0/5       3 16 chunks
mandelbrot   2 rayon-iter          14010.1   2.4 13509.8..14497.3          27279.8                                        
mandelbrot   2 static              26651.1   0.7 26368.0..26884.6          54457.2                                        excursions retained
mandelbrot   2 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 1.006 [0.98-1.05]  lower 2/5  cpu 1.002
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6833.5   0.7 6788.8..7313.6            26179.9                                        
mandelbrot   4 parlay               7079.0   1.4 6807.3..7403.9            27656.6                                        
mandelbrot   4 wf-b                 7165.1   4.0 6875.0..8045.4            27380.5 1.031 [0.99-1.18]  1.019   1/5       6 16 chunks
mandelbrot   4 wf                   7211.3   3.6 6950.5..7838.2            27165.4 1.059 [0.98-1.15]  1.038   1/5       6 16 chunks
mandelbrot   4 rayon-iter           7215.8   1.4 7073.3..8245.8            27607.9                                        
mandelbrot   4 rayon-join           7216.0   1.9 7076.8..7778.0            27880.0                                        
mandelbrot   4 static              26561.1   0.7 26139.6..26984.3         108723.7                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.989 [0.95-1.12]  lower 3/5  cpu 0.994
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6932.0   1.4 6831.2..7189.0            27453.0                                        
mandelbrot   8 parlay               7120.3   3.2 6895.0..7449.7            27891.9                                        
mandelbrot   8 rayon-join           7228.3   1.3 6975.4..7325.5            27580.1                                        
mandelbrot   8 rayon-iter           7275.7   2.0 7127.1..7725.6            28038.5                                        
mandelbrot   8 wf-b                 7300.9   1.5 6895.3..7415.9            27173.9 1.032 [1.01-1.09]  0.992   0/5       8 16 chunks
mandelbrot   8 wf                   7557.2   0.1 6967.7..7566.5            26912.2 1.086 [1.02-1.11]  0.983   0/5       9 16 chunks
mandelbrot   8 static              31515.2   0.1 27687.8..33184.6         127451.3                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.980 [0.95-1.00]  lower 5/5  cpu 1.010
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 wf-b                26933.4   0.6 26559.6..27463.3          26917.1                                        
mandelbrot   1 wf                  26942.1   1.3 26425.6..27318.4          26938.5                                        
mandelbrot   1 wf-seq              27101.0   0.9 26504.0..27488.2          27054.4                                        
mandelbrot   1 serial              27413.3   0.3 26606.6..27503.2          27409.3                                        
mandelbrot   1 A/B  wf-b/wf  wall 0.999 [0.99-1.02]  lower 3/5  cpu 1.000
  wf-b: compiler-chosen
  wf: compiler-chosen
  wf-seq: control
  serial: none: one thread, a loop over all callbacks

quadrature   2 rayon-join           8699.3   1.2 8242.4..13690.6           17397.1                                        
quadrature   2 rayon-join-left      8740.3   1.3 8628.1..14129.9           17479.1                                        
quadrature   2 wf-b                 8781.5   3.5 8474.7..11433.4           16745.3 1.028 [0.99-1.11]  0.987   2/5     397 
quadrature   2 wf                   8893.5   2.6 8337.9..9126.8            16936.9 0.968 [0.87-1.11]  0.938   3/5     412 
quadrature   2 static               8913.3   1.8 8752.1..12289.7           16943.9                                        excursions retained
quadrature   2 tbb                  9875.3   3.1 9571.0..10473.7           18893.9                                        
quadrature   2 parlay              10343.5   2.4 9507.6..10592.1           17981.4                                        
quadrature   2 parlay-left         12244.8   5.0 11634.8..14385.9          19718.8                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
quadrature   2 A/B  wf-b/wf  wall 1.026 [0.93-1.26]  lower 2/5  cpu 1.048
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 rayon-join-left      5271.5   9.1 4653.8..9721.4            20505.1                                        
quadrature   4 wf                   5271.6   9.0 4799.3..6581.4            19903.1 1.092 [0.94-1.37]  1.069   1/5    1012 
quadrature   4 wf-b                 5377.1   9.1 4886.3..7180.8            17906.9 1.154 [0.96-1.50]  0.933   1/5     995 
quadrature   4 static               5689.6   5.7 5107.9..6100.3            18697.8                                        excursions retained
quadrature   4 rayon-join           5801.3  10.7 4829.8..9699.8            19373.8                                        
quadrature   4 tbb                  6380.2   3.2 6053.4..6738.7            25503.9                                        
quadrature   4 parlay               7246.8   0.4 7214.2..8327.5            22736.1                                        
quadrature   4 parlay-left         10530.2   2.2 9920.5..11729.0           27770.8                                        
quadrature   4 BEST REFERENCE = rayon-join-left FASTEST = rayon-join-left WF fastest: no
quadrature   4 A/B  wf-b/wf  wall 1.057 [1.02-1.09]  lower 0/5  cpu 0.929
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf-b                 4850.9   5.8 4410.7..6043.0            16777.9 0.843 [0.74-0.96]  0.973   5/5    1184 
quadrature   8 wf                   4991.1   3.7 4555.5..5418.6            17325.4 0.808 [0.76-0.93]  0.815   5/5    1192 
quadrature   8 parlay               5952.7   1.3 5849.6..6265.8            19886.3                                        
quadrature   8 rayon-join           6138.2   3.2 5752.5..6510.8            23728.0                                        
quadrature   8 tbb                  6734.7   3.0 6474.9..8022.4            25959.6                                        
quadrature   8 parlay-left          7203.0  15.1 6103.1..8310.6            26581.7                                        
quadrature   8 rayon-join-left      7470.1  10.5 6553.1..8329.5            28648.9                                        
quadrature   8 static             528014.3   1.5 519979.4..596063.2      2090663.5                                        excursions retained
quadrature   8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 0.968 [0.95-1.21]  lower 4/5  cpu 0.978
  wf-b: compiler-chosen
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15824.8   2.0 15504.7..16713.3          15822.3                                        
quadrature   1 wf                  16390.4   2.3 14662.4..17115.9          16388.8                                        
quadrature   1 wf-b                16472.4   1.0 14896.4..17034.0          16470.6                                        
quadrature   1 wf-seq              16638.9   1.2 16433.1..17028.7          16637.5                                        
quadrature   1 A/B  wf-b/wf  wall 1.007 [0.97-1.02]  lower 2/5  cpu 1.007
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-b: compiler-chosen
  wf-seq: control

records      2 static              16613.6   0.8 16485.0..17425.7          33172.2                                        excursions retained
records      2 rayon-join          16662.6   0.6 16566.2..17575.1          32960.3                                        
records      2 rayon-iter          16765.5   1.4 16424.5..17360.7          32748.8                                        
records      2 parlay              16885.8   3.1 16313.3..17555.3          31671.0                                        
records      2 tbb                 16996.5   0.9 16302.5..17902.1          32863.8                                        
records      2 wf                  18800.3   1.5 18519.9..19825.9          37327.9 1.145 [1.12-1.17]  1.148   0/5       1 32 chunks
records      2 wf-b                18882.4   0.5 18535.9..21731.7          36740.8 1.137 [1.11-1.33]  1.122   0/5       1 32 chunks
records      2 BEST REFERENCE = rayon-iter   FASTEST = static       WF fastest: no
records      2 A/B  wf-b/wf  wall 0.986 [0.95-1.17]  lower 3/5  cpu 0.973
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf: compiler-chosen
  wf-b: compiler-chosen

records      4 rayon-iter           8582.4   0.8 8513.6..9016.5            33177.7                                        
records      4 static               8658.0   1.2 8554.6..19773.2           32824.3                                        excursions retained
records      4 rayon-join           8955.3   1.9 8692.3..9353.0            33622.4                                        
records      4 tbb                  9290.6   8.9 8468.2..11993.6           33499.1                                        
records      4 parlay               9353.3   7.7 8385.5..12422.8           29711.3                                        
records      4 wf                   9760.9   2.3 9539.7..12472.6           37266.7 1.125 [1.11-1.47]  1.139   0/5      10 64 chunks
records      4 wf-b                10248.3   2.7 9825.6..11076.3           37463.7 1.189 [1.15-1.23]  1.145   0/5       9 64 chunks
records      4 BEST REFERENCE = rayon-iter   FASTEST = rayon-iter   WF fastest: no
records      4 A/B  wf-b/wf  wall 1.068 [0.79-1.10]  lower 1/5  cpu 1.033
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen

records      8 parlay               8819.2   4.5 8425.7..9707.7            34513.4                                        
records      8 rayon-join           8828.9   2.8 8585.4..9385.3            34527.4                                        
records      8 tbb                  8933.3   2.8 8349.2..9519.9            33095.9                                        
records      8 rayon-iter           9021.9   2.4 8801.1..9946.2            34164.7                                        
records      8 wf-b                10152.7   4.7 9677.9..12442.8           38536.9 1.183 [1.15-1.41]  1.116   0/5      20 64 chunks
records      8 wf                  10932.9  10.7 9683.1..13182.3           40614.9 1.238 [1.15-1.50]  1.176   0/5      19 64 chunks
records      8 static              19029.9   6.3 12562.9..20230.0          77575.4                                        excursions retained
records      8 BEST REFERENCE = rayon-join   FASTEST = parlay       WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 0.944 [0.91-1.03]  lower 4/5  cpu 0.970
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 wf-seq              28180.0   0.7 27971.2..32710.1          28164.1                                        
records      1 serial              32476.2   2.1 31792.5..34918.7          32469.1                                        
records      1 wf-b                35835.4   1.1 35308.4..36845.1          35814.5                                        
records      1 wf                  35931.8   1.0 35504.9..36820.9          35915.0                                        
records      1 A/B  wf-b/wf  wall 0.995 [0.99-1.02]  lower 3/5  cpu 0.995
  wf-seq: control
  serial: none: one thread, a loop over all callbacks
  wf-b: compiler-chosen
  wf: compiler-chosen

fir          2 wf-b                14120.4   1.3 13937.1..14425.1          27113.2 0.891 [0.87-0.91]  0.868   5/5       1 32 chunks
fir          2 wf                  14305.8   3.0 13629.9..17975.9          27422.8 0.880 [0.86-1.14]  0.859   4/5       1 32 chunks
fir          2 tbb                 16217.1   1.8 15848.3..17161.2          31238.5                                        
fir          2 static              16442.2   1.7 15916.3..16957.6          32696.6                                        excursions retained
fir          2 rayon-join          16566.6   0.8 15783.1..16698.7          32281.2                                        
fir          2 rayon-iter          16969.9   1.7 16545.2..17280.2          32582.9                                        
fir          2 parlay              17304.0   5.2 15927.1..18201.6          32491.1                                        
fir          2 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 0.999 [0.80-1.04]  lower 3/5  cpu 0.990
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf-b                 7851.3   1.3 7589.8..8115.1            28504.6 0.932 [0.90-0.98]  0.908   5/5      10 64 chunks
fir          4 wf                   7937.0   4.6 7570.3..8541.6            28147.6 0.966 [0.91-1.02]  0.914   4/5       9 64 chunks
fir          4 rayon-join           8432.9   3.5 8012.2..8846.2            31774.7                                        
fir          4 tbb                  8666.8   5.1 8124.8..9245.2            31755.4                                        
fir          4 rayon-iter           8793.1   3.6 8474.2..10176.5           32052.2                                        
fir          4 parlay               8926.9   1.1 8336.7..9021.2            33955.0                                        
fir          4 static               9174.7   9.0 8347.7..10792.9           33434.4                                        excursions retained
fir          4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
fir          4 A/B  wf-b/wf  wall 0.956 [0.94-1.03]  lower 3/5  cpu 0.982
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          8 wf-b                 7953.3   5.1 7289.0..8355.7            29054.5 0.958 [0.85-0.99]  0.921   5/5      21 64 chunks
fir          8 wf                   8211.3   3.7 7878.4..8748.6            30199.4 0.989 [0.94-1.02]  0.955   3/5      17 64 chunks
fir          8 tbb                  8573.8   2.6 8308.3..8907.2            32127.1                                        
fir          8 rayon-join           8629.3   2.2 8439.6..9963.6            32947.0                                        
fir          8 parlay               8898.7   2.3 8697.6..9605.1            34333.1                                        
fir          8 rayon-iter           8972.7   7.5 8302.7..10111.1           34092.4                                        
fir          8 static              13717.1   6.0 12250.2..22288.5          57595.3                                        excursions retained
fir          8 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 0.969 [0.85-1.05]  lower 4/5  cpu 0.949
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-seq              27482.1   0.3 27008.0..27982.5          27478.0                                        
fir          1 wf-b                27622.8   0.7 27275.5..28013.1          27616.8                                        
fir          1 wf                  27675.0   1.9 26886.9..28257.1          27660.0                                        
fir          1 serial              31784.4   1.0 31446.5..35003.8          31780.5                                        
fir          1 A/B  wf-b/wf  wall 1.004 [0.98-1.03]  lower 2/5  cpu 1.004
  wf-seq: control
  wf-b: compiler-chosen
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```

## 2026-09-11 — local host (Linux x86_64, 4 logical CPUs), loop and function alignment on the Whitefoot side, A/B

**The remedy's own A/B: it costs nothing and buys nothing this instrument can
resolve, and the compiler driver is left alone.** The `wf` row is built at the
flags `whitefootc` passes clang — `-O2`, no `-march`, no alignment — while every
reference is built at `-O3` with `-march=x86-64-v3` and `-falign-loops=32`, an
asymmetry the bundle discloses and which cuts against the WF row. This run asks
what closing half of it is worth, through the bundle's new third handle: the
twin's emitted module object and its four Whitefoot runtime units are compiled
with `-falign-functions=64 -falign-loops=32` and everything else is the plain
build.

```text
mandelbrot   2 A/B  wf-b/wf  wall 0.997 [0.99-1.06]  lower 3/5  cpu 1.000
mandelbrot   4 A/B  wf-b/wf  wall 0.978 [0.93-1.37]  lower 4/5  cpu 0.980
mandelbrot   8 A/B  wf-b/wf  wall 0.954 [0.55-1.07]  lower 3/5  cpu 0.964
mandelbrot   1 A/B  wf-b/wf  wall 0.989 [0.98-1.00]  lower 3/5  cpu 0.990
quadrature   2 A/B  wf-b/wf  wall 1.000 [0.95-1.06]  lower 2/5  cpu 0.985
quadrature   4 A/B  wf-b/wf  wall 0.997 [0.88-1.12]  lower 3/5  cpu 1.014
quadrature   8 A/B  wf-b/wf  wall 0.993 [0.93-1.02]  lower 3/5  cpu 0.983
quadrature   1 A/B  wf-b/wf  wall 1.010 [0.92-1.14]  lower 2/5  cpu 1.011
records      2 A/B  wf-b/wf  wall 0.992 [0.97-1.04]  lower 4/5  cpu 0.992
records      4 A/B  wf-b/wf  wall 0.990 [0.96-1.07]  lower 4/5  cpu 0.990
records      8 A/B  wf-b/wf  wall 0.996 [0.80-1.05]  lower 4/5  cpu 0.994
records      1 A/B  wf-b/wf  wall 1.001 [0.98-1.01]  lower 2/5  cpu 1.006
fir          2 A/B  wf-b/wf  wall 0.991 [0.95-1.02]  lower 3/5  cpu 1.001
fir          4 A/B  wf-b/wf  wall 0.983 [0.74-1.03]  lower 4/5  cpu 0.991
fir          8 A/B  wf-b/wf  wall 0.990 [0.87-1.06]  lower 3/5  cpu 1.013
fir          1 A/B  wf-b/wf  wall 0.984 [0.97-1.03]  lower 4/5  cpu 0.983
```

**All sixteen lines sit in [0.954, 1.010].** No kernel is worse than one percent
at any width, thirteen lines are within 1.6 percent of 1.000, and the `lower`
counts lean gently toward the aligned arm (four of five on six lines, three of
five on seven). The largest movements are mandelbrot W=8 0.954 and W=4 0.978,
both at the oversubscribed or highest parallel width, and both inside what the
first two sections show this instrument does to arms that execute identical
code.

**So the driver is unchanged, and the reason is the second section rather than
this one.** The condition for putting these flags into `whitefootc`'s own
compilation of the emitted module and the runtime sources was that alignment
remove or clearly reduce the placement sensitivity; it does not, and the
mechanical argument there says it cannot. What is left is this section's one to
two percent, which is smaller than the several percent the same instrument
assigns to moving code that does not change — so selecting the flags on it would
be selecting on a difference the measurement cannot support. The cost of being
wrong is not zero: `WF_FLAGS` in the bundle exists to be exactly what
`whitefootc` passes clang, and every recorded table afterwards would be timing a
program built at flags chosen on an unresolved reading.

**Reopen** on a host whose shifted null arm holds inside one percent, where a
one-to-two-percent paired line is selectable; or on a paired measurement of the
sequential lowering, where the previous section's unpaired 21 percent on records
`wf-seq` is the largest single alignment effect anywhere in the three runs and
is not explained by anything measured here.

- host: `Linux vm 6.18.44-fc-v24 #1 SMP PREEMPT_DYNAMIC @0 x86_64 x86_64 x86_64 GNU/Linux`
- logical CPUs: 4 (four cores, one thread per core)   inherited mask: `0-3`
  (as recorded; `cgroup_cpu_max` and `cpuset.cpus.effective` both absent and
  recorded as unqualified)
- compiler revision: `e69592dc41da7db3c9bff6b81826e822f939d0fb`
- clang: Ubuntu clang version 18.1.3 (1ubuntu1)   clang++: Ubuntu clang version
  18.1.3 (1ubuntu1)   rustc: rustc 1.94.1 (e408947bf 2026-03-25)   cargo: cargo
  1.94.1 (29ea6fb6a 2026-03-24)   cmake: cmake version 3.28.3
- pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
- BENCH_ARCH: `-march=x86-64-v3`
- sizes: unchanged — mandelbrot 98,304 points at limit 256, shape `trailing`;
  quadrature M=64 at tolerance `0x1p-54`, depth 24; records 131,072 at
  `max_length` 255; fir K=64 over N=524,288. Emitted chunk counts are unchanged
  from every section since `53359d73` and identical in both arms — mandelbrot
  16/16/16, records and fir 32/64/64, quadrature `chunks=na` — because no
  control in these three runs reaches the splitter.
- module identity: the four emitted `--par` modules are byte-identical to every
  section since `53359d73` — `mandelbrot-par.ll` `f07c190e3a396fc9…`,
  `quadrature-par.ll` `dc6aeaf3fda497ba…`, `records-par.ll` `9ceebaedf90e4a60…`,
  `fir-par.ll` `e0af1ed03ef2bc52…` — and each twin `-par-b.ll` matches its plain
  module exactly, so nothing the compiler emitted differs between the arms in
  any of the three runs. `WF_PAR_CONTROL_FLAGS` was empty in all three.
- passes: 5, calls: 5
- module control flags: `WF_MODULE_CONTROL_FLAGS='-falign-functions=64
  -falign-loops=32'`, reaching the **twin's** emitted module object and its four
  Whitefoot runtime units only; `WF_FLAGS` unchanged, so the `wf` row is built
  the way `whitefootc` builds it; `WF_RUNTIME_CONTROL_FLAGS` empty and no pad in
  this run. `WF_AB_TWIN: yes` in the manifest and the disclosure line in the
  table header. **This table carries a `wf-b` row and is not a candidate
  record.**
- image identity: the plain images are `e546b37823fa5472…`, `efabfb8d95808395…`,
  `2d468d594e628e59…` and `505fd6772508ee12…`, byte-identical to the first
  section's and to every section since `53359d73`, so its `wf` rows and this
  one's are two readings of one build; the aligned twins are
  `0c52d8100f2ccc9f…`, `1ce696413135c434…`, `34bf41fd3954e878…` and
  `9d539c6f6d9d3e1b…`. The quadrature twin here is byte-identical to the
  previous section's plain quadrature image, which is the cross-check that this
  run's B arm really is the aligned build.
- recorded block: none. W=4 is this host's highest non-oversubscribed block.
- workflow run: `local`
- sizing window: every `wf-seq` median at W=1 inside [5 ms, 60 ms] — mandelbrot
  26.598, quadrature 16.191, records 35.777, fir 26.952 ms; every `wf` median at
  W=4 above 1 ms; `steals > 0` on every `wf` and every `wf-b` row at every
  parallel width.

```text
compute-bench  host=Linux x86_64  cpus=4  mask=pid 10932's current affinity list: 0-3  date=2026-09-11T15:52:32Z
run=local  compiler=e69592dc41da7db3c9bff6b81826e822f939d0fb  clang=Ubuntu clang version 18.1.3 (1ubuntu1)  rustc=rustc 1.94.1 (e408947bf 2026-03-25)
reference flags=-std=c11 -O3 -g -Wall -Wextra -Werror -Wpedantic -pthread -falign-loops=32 -fno-fast-math -ffp-contract=off -fno-vectorize -fno-slp-vectorize -fno-lto -march=x86-64-v3
      (identical for every reference implementation of every kernel)
WF flags=-std=c11 -pthread -O2 -Wno-override-module
      (module and runtime, as whitefootc links them: no -march, no loop
      alignment -- see README)
WF module control flags=-falign-functions=64 -falign-loops=32
      (appended to the compile of the `wf-b` TWIN's emitted module object and
      its Whitefoot runtime only: `wf` above is still built at the WF flags
      above. A table with a wf-b row is an A/B instrument and must not be
      recorded as a plain table -- see README)
pins: oneTBB 3046c8b0 (v2023.1.0)  ParlayLib 51017699  rayon =1.12.0
sizes: mandelbrot   points=98304 limit=256 shape=trailing seed=828219
sizes: quadrature   integrations=64 tolerance=0x1p-54 depth=24
sizes: records      records=131072 max_length=255 shape=unicode seed=812381
sizes: fir          taps=64 outputs=524288 seed=92821
passes=5 calls=5

kernel       w form              median_us  mad% p10..p90_us                cpu_us ratio              cpu_r lower  steals note
mandelbrot   2 tbb                 13287.1   0.2 13264.8..13947.9          26549.4                                        
mandelbrot   2 rayon-iter          13428.2   1.3 13258.1..14403.1          26599.1                                        
mandelbrot   2 rayon-join          13479.4   0.7 13389.8..14314.7          26741.9                                        
mandelbrot   2 parlay              13511.8   0.7 13390.7..14209.2          26753.2                                        
mandelbrot   2 wf-b                13657.8   1.1 13513.4..14907.5          26788.7 1.030 [0.99-1.12]  1.017   1/5       3 16 chunks
mandelbrot   2 wf                  13732.7   1.0 13601.6..14187.1          26812.2 1.035 [1.00-1.06]  1.017   0/5       3 16 chunks
mandelbrot   2 static              26384.4   0.3 26184.0..26619.9          54302.6                                        excursions retained
mandelbrot   2 BEST REFERENCE = rayon-iter   FASTEST = tbb          WF fastest: no
mandelbrot   2 A/B  wf-b/wf  wall 0.997 [0.99-1.06]  lower 3/5  cpu 1.000
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   4 tbb                  6782.4   1.2 6682.1..6936.4            26786.2                                        
mandelbrot   4 wf-b                 6911.9   0.8 6853.9..11730.4           26787.4 1.027 [1.01-1.69]  1.040   0/5       6 16 chunks
mandelbrot   4 parlay               7029.7   1.0 6960.6..7252.2            27170.5                                        
mandelbrot   4 wf                   7064.0   0.1 7055.2..8592.3            27128.2 1.057 [1.04-1.24]  1.057   0/5       7 16 chunks
mandelbrot   4 rayon-join           7072.9   0.5 6935.8..7469.2            27272.1                                        
mandelbrot   4 rayon-iter           7104.3   1.5 6993.6..7411.7            27193.0                                        
mandelbrot   4 static              26582.7   0.8 26221.5..27508.5         109475.4                                        excursions retained
mandelbrot   4 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: no
mandelbrot   4 A/B  wf-b/wf  wall 0.978 [0.93-1.37]  lower 4/5  cpu 0.980
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   8 tbb                  6817.1   0.3 6749.1..7279.7            27004.9                                        
mandelbrot   8 parlay               7066.3   1.8 6925.8..7223.1            27690.2                                        
mandelbrot   8 rayon-iter           7203.7   2.3 7034.6..8424.8            27612.1                                        
mandelbrot   8 rayon-join           7346.9   2.5 6930.8..7531.5            27397.3                                        
mandelbrot   8 wf                   7362.3   5.8 6938.1..13517.0           27399.7 1.081 [1.03-1.98]  1.002   0/5       8 16 chunks
mandelbrot   8 wf-b                 7492.1   3.8 6951.4..7803.9            27104.8 1.099 [1.02-1.14]  0.980   0/5       9 16 chunks
mandelbrot   8 static              31483.2   0.1 31360.4..35495.5         127421.4                                        excursions retained
mandelbrot   8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
mandelbrot   8 A/B  wf-b/wf  wall 0.954 [0.55-1.07]  lower 3/5  cpu 0.964
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  wf: compiler-chosen
  wf-b: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

mandelbrot   1 serial              26412.3   0.0 26248.6..26479.3          26398.7                                        
mandelbrot   1 wf-seq              26598.0   0.4 26458.1..26986.6          26594.9                                        
mandelbrot   1 wf-b                26598.7   0.1 26456.2..26634.6          26595.6                                        
mandelbrot   1 wf                  26888.4   0.1 26465.4..26924.2          26876.5                                        
mandelbrot   1 A/B  wf-b/wf  wall 0.989 [0.98-1.00]  lower 3/5  cpu 0.990
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf-b: compiler-chosen
  wf: compiler-chosen

quadrature   2 rayon-join           8231.8   2.7 7975.9..8571.4            16421.6                                        
quadrature   2 rayon-join-left      8591.3   0.8 8145.8..9466.9            17180.9                                        
quadrature   2 wf-b                 8662.1   0.6 8531.3..8744.6            16683.7 1.046 [1.02-1.09]  1.029   0/5     402 
quadrature   2 wf                   8682.8   3.8 8255.5..9096.2            16853.3 1.038 [0.96-1.14]  1.026   1/5     399 
quadrature   2 static               8812.4   0.9 8730.9..9039.7            17225.5                                        excursions retained
quadrature   2 parlay              10553.5   4.5 9527.6..11031.5           18255.8                                        
quadrature   2 tbb                 10614.9   8.1 9673.6..12517.2           21085.7                                        
quadrature   2 parlay-left         11862.1   1.5 10329.9..15533.0          19042.5                                        
quadrature   2 BEST REFERENCE = rayon-join   FASTEST = rayon-join   WF fastest: no
quadrature   2 A/B  wf-b/wf  wall 1.000 [0.95-1.06]  lower 2/5  cpu 0.985
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   4 wf-b                 4863.6   5.1 4613.1..6059.6            18037.2 0.977 [0.90-1.09]  0.887   4/5    1021 
quadrature   4 wf                   4876.2   8.1 4479.2..5858.4            16858.2 0.987 [0.93-1.03]  0.865   4/5    1033 
quadrature   4 rayon-join           4925.0   2.6 4798.4..6172.9            19098.8                                        
quadrature   4 rayon-join-left      5483.6   4.0 4831.5..5701.4            21313.4                                        
quadrature   4 static               5735.0   1.5 5161.9..5818.7            18080.9                                        excursions retained
quadrature   4 tbb                  6378.7   6.7 5858.3..6808.5            24952.3                                        
quadrature   4 parlay               7596.1   3.5 6517.2..8832.4            23171.3                                        
quadrature   4 parlay-left         10120.6   0.5 9416.8..10175.9           26178.1                                        
quadrature   4 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: yes
quadrature   4 A/B  wf-b/wf  wall 0.997 [0.88-1.12]  lower 3/5  cpu 1.014
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork

quadrature   8 wf-b                 4753.8   2.6 4629.9..5211.1            16719.4 0.951 [0.91-0.97]  0.849   5/5    1187 
quadrature   8 wf                   4918.4   2.6 4662.6..5155.5            17077.3 0.957 [0.92-1.02]  0.836   4/5    1196 
quadrature   8 rayon-join           5139.8   4.2 4852.0..5478.6            20135.2                                        
quadrature   8 tbb                  6181.9   3.1 5908.6..6735.8            24670.4                                        
quadrature   8 parlay-left          6229.7   2.6 5268.3..6392.1            19758.7                                        
quadrature   8 rayon-join-left      6305.4   5.3 5973.9..6813.6            24518.0                                        
quadrature   8 parlay               6357.8   4.5 5202.9..6643.9            22576.2                                        
quadrature   8 static             519982.9   1.5 511988.7..563974.6      2065704.5                                        excursions retained
quadrature   8 BEST REFERENCE = rayon-join   FASTEST = wf           WF fastest: n/a (oversubscribed)
quadrature   8 A/B  wf-b/wf  wall 0.993 [0.93-1.02]  lower 3/5  cpu 0.983
  wf-b: compiler-chosen
  wf: compiler-chosen
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay-left: ParlayLib native scheduler, parallel_for granularity 1, left-offer fork
  rayon-join-left: rayon 1.12.0 join, bisect the chunk range to one callback, left-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

quadrature   1 serial              15458.2   0.4 14249.4..15849.1          15456.6                                        
quadrature   1 wf-seq              16191.3   3.9 15045.3..16818.2          16190.2                                        
quadrature   1 wf                  16272.5   1.4 14590.0..16508.5          16261.0                                        
quadrature   1 wf-b                16440.5   0.8 14904.6..16662.9          16437.6                                        
quadrature   1 A/B  wf-b/wf  wall 1.010 [0.92-1.14]  lower 2/5  cpu 1.011
  serial: none: one thread, a loop over all callbacks
  wf-seq: control
  wf: compiler-chosen
  wf-b: compiler-chosen

records      2 tbb                 16629.0   0.8 16456.9..17375.8          32491.0                                        
records      2 rayon-join          16630.7   1.4 16345.6..17072.5          33038.4                                        
records      2 parlay              16785.2   3.0 16285.9..22250.6          32612.9                                        
records      2 static              16896.2   2.1 16442.5..17563.9          32893.5                                        excursions retained
records      2 rayon-iter          17055.8   1.4 16450.1..17298.6          33058.5                                        
records      2 wf-b                18266.1   1.4 18001.3..19188.3          35919.6 1.106 [1.07-1.17]  1.101   0/5       1 32 chunks
records      2 wf                  18513.8   1.5 18155.3..19209.6          36443.5 1.126 [1.07-1.18]  1.135   0/5       1 32 chunks
records      2 BEST REFERENCE = parlay       FASTEST = tbb          WF fastest: no
records      2 A/B  wf-b/wf  wall 0.992 [0.97-1.04]  lower 4/5  cpu 0.992
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen

records      4 parlay               8608.4   3.0 8263.1..8947.9            27077.8                                        
records      4 rayon-join           8758.6   5.9 8200.8..11305.9           34223.9                                        
records      4 static               8787.7   1.0 8456.9..8877.2            32733.6                                        excursions retained
records      4 rayon-iter           8935.2   6.0 8089.4..12178.8           34047.3                                        
records      4 tbb                  9072.0   3.8 8370.3..9595.1            33140.8                                        
records      4 wf-b                 9547.7   1.9 9366.6..9874.3            36179.5 1.145 [1.11-1.21]  1.145   0/5       7 64 chunks
records      4 wf                   9771.2   1.9 9195.8..9954.1            36614.1 1.155 [1.12-1.22]  1.157   0/5       6 64 chunks
records      4 BEST REFERENCE = parlay       FASTEST = parlay       WF fastest: no
records      4 A/B  wf-b/wf  wall 0.990 [0.96-1.07]  lower 4/5  cpu 0.990
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  wf-b: compiler-chosen
  wf: compiler-chosen

records      8 tbb                  8448.9   1.4 8331.0..11827.0           33112.5                                        
records      8 parlay               8777.9   3.2 8499.4..12752.7           34437.4                                        
records      8 rayon-join           8853.6   3.4 8286.2..9158.9            32770.7                                        
records      8 rayon-iter           9450.3   9.6 8412.3..11003.1           36268.3                                        
records      8 wf-b                 9920.3   2.1 9712.0..10417.5           37202.9 1.149 [1.13-1.26]  1.109   0/5      19 64 chunks
records      8 wf                   9945.8   0.1 9750.6..12746.3           37132.9 1.195 [1.15-1.41]  1.140   0/5      20 64 chunks
records      8 static              16335.9  21.7 12067.2..20233.5          75799.3                                        excursions retained
records      8 BEST REFERENCE = tbb          FASTEST = tbb          WF fastest: n/a (oversubscribed)
records      8 A/B  wf-b/wf  wall 0.996 [0.80-1.05]  lower 4/5  cpu 0.994
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  wf-b: compiler-chosen
  wf: compiler-chosen
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

records      1 serial              32651.0   0.7 32048.8..32869.7          32646.8                                        
records      1 wf                  35685.0   0.7 35437.3..36791.4          35680.7                                        
records      1 wf-seq              35776.5   0.6 35289.4..36098.8          35772.5                                        
records      1 wf-b                36039.5   0.2 35675.3..36222.3          36036.2                                        
records      1 A/B  wf-b/wf  wall 1.001 [0.98-1.01]  lower 2/5  cpu 1.006
  serial: none: one thread, a loop over all callbacks
  wf: compiler-chosen
  wf-seq: control
  wf-b: compiler-chosen

fir          2 wf-b                13725.1   1.5 13329.1..14531.1          26681.5 0.863 [0.84-0.92]  0.901   5/5       1 32 chunks
fir          2 wf                  13757.9   2.6 13404.7..14664.2          26974.0 0.867 [0.84-0.93]  0.896   5/5       1 32 chunks
fir          2 tbb                 15915.2   0.2 15416.3..16064.1          29678.8                                        
fir          2 rayon-join          16186.0   1.9 15817.8..16887.5          31719.3                                        
fir          2 static              16190.3   1.0 15909.7..16610.6          32179.2                                        excursions retained
fir          2 rayon-iter          16373.6   0.5 15715.3..16469.6          31791.1                                        
fir          2 parlay              16443.6   1.3 15855.0..16651.3          32297.0                                        
fir          2 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          2 A/B  wf-b/wf  wall 0.991 [0.95-1.02]  lower 3/5  cpu 1.001
  wf-b: compiler-chosen
  wf: compiler-chosen
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork

fir          4 wf-b                 7316.1   1.9 7067.3..7624.6            27043.4 0.916 [0.86-0.94]  0.896   5/5      10 64 chunks
fir          4 wf                   7484.4   1.0 7407.1..9729.7            27527.7 0.929 [0.92-1.25]  0.932   3/5       9 64 chunks
fir          4 parlay               8069.7   0.7 8013.6..8933.2            31100.0                                        
fir          4 tbb                  8174.8   1.2 7764.8..8303.3            29619.9                                        
fir          4 static               8264.7   1.9 7903.3..10266.6           32259.2                                        excursions retained
fir          4 rayon-iter           8454.3   5.4 7958.9..8907.2            31342.8                                        
fir          4 rayon-join           8536.4   2.2 8063.5..8724.9            32252.5                                        
fir          4 BEST REFERENCE = tbb          FASTEST = wf           WF fastest: yes
fir          4 A/B  wf-b/wf  wall 0.983 [0.74-1.03]  lower 4/5  cpu 0.991
  wf-b: compiler-chosen
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork

fir          8 wf-b                 7375.3   4.6 7032.5..7969.4            27621.2 0.913 [0.85-0.95]  0.866   5/5      20 64 chunks
fir          8 wf                   7450.4   2.2 7284.7..8259.3            27367.0 0.887 [0.88-1.05]  0.857   4/5      19 64 chunks
fir          8 parlay               8403.7   2.3 7874.5..8628.0            32932.3                                        
fir          8 rayon-join           8422.9   2.1 8174.1..8681.9            32036.0                                        
fir          8 tbb                  8454.9   1.6 8223.2..9289.6            32204.5                                        
fir          8 rayon-iter           8786.6   2.5 8280.2..9009.4            32689.0                                        
fir          8 static              12545.9   1.5 12270.0..12737.8          48542.1                                        excursions retained
fir          8 BEST REFERENCE = parlay       FASTEST = wf           WF fastest: n/a (oversubscribed)
fir          8 A/B  wf-b/wf  wall 0.990 [0.87-1.06]  lower 3/5  cpu 1.013
  wf-b: compiler-chosen
  wf: compiler-chosen
  parlay: ParlayLib native scheduler, parallel_for granularity 1, right-offer fork
  rayon-join: rayon 1.12.0 join, bisect the chunk range to one callback, right-offer fork
  tbb: oneTBB v2023.1.0 parallel_for, auto_partitioner, range grain 1
  rayon-iter: rayon 1.12.0 parallel iterator, its own adaptive splitting
  static: equal contiguous partition, persistent helpers, no stealing: a regular-work reference, not a dynamic-scheduling ceiling for skew

fir          1 wf-b                26851.2   0.5 26721.8..27132.9          26847.3                                        
fir          1 wf-seq              26952.2   1.3 26603.8..27823.3          26949.0                                        
fir          1 wf                  27279.4   1.1 26256.8..27619.4          27275.3                                        
fir          1 serial              31536.3   0.1 31394.1..32096.6          31532.7                                        
fir          1 A/B  wf-b/wf  wall 0.984 [0.97-1.03]  lower 4/5  cpu 0.983
  wf-b: compiler-chosen
  wf-seq: control
  wf: compiler-chosen
  serial: none: one thread, a loop over all callbacks
```
