# CSR graph (frozen adjacency scan) — non-normative worked card (DEMOTED, PROVISIONAL) [M5R3-PART2]

Status: NON-NORMATIVE. Demoted out of the always-loaded catalog set per D18
decision 5 (option 2); the always-loaded stub for this card is the one-line
promise + pointer in `cards.md`. Demotion is PROVISIONAL and re-decided on
round-4 data. This file holds the full worked text when authored.

Promise: two-phase: mutable build as `seq` of adjacency `seq`s over u32 node ids, then freeze to compressed sparse row (one offsets `seq` + one packed-edge `seq`) for stride-1 neighbour scans. u32 ids, SoA node payloads. Taught pattern over F1/F3, no new form.

Full worked example: TO BE AUTHORED (this card had no full text at the split; it
is a reserved non-normative slot, not lost content).
