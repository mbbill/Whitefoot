# Intrusive links (O(1) unlink from within an element) — non-normative worked card (DEMOTED, PROVISIONAL) [M5R3-PART2]

Status: NON-NORMATIVE. Demoted out of the always-loaded catalog set per D18
decision 5 (option 2); the always-loaded stub for this card is the one-line
promise + pointer in `cards.md`. Demotion is PROVISIONAL and re-decided on
round-4 data. This file holds the full worked text when authored.

Promise: elements in a `pool` slab, membership as u32 prev/next index fields inside each element; O(1) unlink given only the element, O(1) splice between lists, zero per-membership allocation. The mimalloc/list_head shape in checked index form.

Full worked example: TO BE AUTHORED (this card had no full text at the split; it
is a reserved non-normative slot, not lost content).
