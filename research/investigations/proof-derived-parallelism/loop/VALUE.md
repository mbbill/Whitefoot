# The map-permission value falsifier (2026-08-23)

Question: is loop-shaped MAP permission worth spec bytes at today's place
granularity? Method: the most favorable legal proxy — K distinct
destination buffers of N/K elements, K adjacent calls, judged eligible by
the in-tree compiler (an upper bound on any [PAR-2] map lowering). One
distinct checksum per probe across 4 forms x 6 worker settings. Interleaved
min-of-7 plus a confirmation pass agreeing within 3%; load average 1.7-2.6
(shared machine, annotated).

| probe | shape | ns/elem | loop s | best twin s | twin/loop | verdict |
|---|---|---|---|---|---|---|
| c8_64m | byte copy, 64 MiB | 0.12 | 0.2441 | 0.1893 (k8,w8) | 0.775 | 1.29x, below band |
| c8_4m | byte copy, 4 MiB | 0.12 | 0.2458 | 0.2037 (k8) | 0.829 | 1.21x, band edge |
| c8_4k | byte copy, 4 KiB | 0.12 | 0.0331 | 0.0561 (k4,w4) | 1.695 | split LOSES 1.70x |
| c64_64m | u64 copy, 64 MiB | 0.47 | 0.0580 | 0.0459 (k4) | 0.791 | 1.26x, below band |
| heavy_2m | 256-round mix/elem | 302 | 0.6373 | 0.1042 (k8,w8) | 0.163 | 6.12x |

Grain sweep at fixed size, varying per-element arithmetic: twin/loop ratios
0.662 / 0.374 / 0.261 / 0.190 / 0.177 / 0.166 at 0.18 / 0.69 / 4.9 / 49 /
127 / 303 ns per element. The discriminator is arithmetic intensity, not
size: the band is left below ~1 ns/element of real work.

Physics anchors: a C memcpy control gains 1.41x from 1 to 8 threads
(77.9 -> 110.2 GB/s); c64_64m's loop runs at 94% of the single-thread
memcpy rate and gains 1.26x — as bandwidth predicts. CPU cost of the
memory-shaped wins: c8_64m spends 0.23 s -> 1.17 s CPU for its 1.29x.

Open anomalies, recorded not chased: c8_64m's loop runs at only 22% of the
single-thread bandwidth ceiling yet still gains only 1.29x at 5.1x CPU; and
every memory-shaped probe is ~2x SLOWER at WF_WORKERS=2 than at 1 (w2
ratios 2.26-2.34) while the compute-heavy probe scales cleanly (0.53).
Both are runtime or lowering properties, not the memory system.

Verdict: map permission does not pay on real shapes at today's
granularity; the reduction (compute-heavy) shape pays decisively. The
design ruling derived from this table is in DESIGN.md.
