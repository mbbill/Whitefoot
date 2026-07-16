# Timer wheel (O(1) insert/cancel timeouts) — non-normative worked card (DEMOTED, PROVISIONAL) [M5R3-PART2]

Status: NON-NORMATIVE. Demoted out of the always-loaded catalog set per D18
decision 5 (option 2); the always-loaded stub for this card is the one-line
promise + pointer in `cards.md`. Demotion is PROVISIONAL and re-decided on
round-4 data. This file holds the full worked text when authored.

Promise: array of buckets (`seq` or `buffer`) + embedded-link lists (intrusive-links card) + masked bucket index; O(1) bucket append and O(1) cancel via embedded-link removal; hierarchical for wide deadline ranges. Composition of F3/F4, not a primitive.

Full worked example: TO BE AUTHORED (this card had no full text at the split; it
is a reserved non-normative slot, not lost content).
