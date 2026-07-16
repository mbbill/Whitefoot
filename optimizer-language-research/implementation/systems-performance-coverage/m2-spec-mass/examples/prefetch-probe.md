# Prefetch-probe (memory-level parallelism) — non-normative worked card (DEMOTED, PROVISIONAL) [M5R3-PART2]

Status: NON-NORMATIVE. Demoted out of the always-loaded catalog set per D18
decision 5 (option 2); the always-loaded stub for this card is the one-line
promise + pointer in `cards.md`. Demotion is PROVISIONAL and re-decided on
round-4 data. This file holds the full worked text when authored.

Promise: compute N probe addresses, issue prefetch hints for all, then process N — keeping 8-16 misses in flight over a DRAM-resident table. Prefetch is a safe-by-construction hint; the pattern is a scheduling idiom over the blessed loop/recursion forms.

Full worked example: TO BE AUTHORED (this card had no full text at the split; it
is a reserved non-normative slot, not lost content).
