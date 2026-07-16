# Interner (value -> stable u32 id) — non-normative worked card (DEMOTED, PROVISIONAL) [M5R3-PART2]

Status: NON-NORMATIVE. Demoted out of the always-loaded catalog set per D18
decision 5 (option 2); the always-loaded stub for this card is the one-line
promise + pointer in `cards.md`. Demotion is PROVISIONAL and re-decided on
round-4 data. This file holds the full worked text when authored.

Promise: bump arena + span-keyed `table<slice<'a,u8>, u32>` + id vector; `intern(bytes) -> Id` idempotent, `resolve(Id) -> &[u8]` O(1); append-only. Composition of catalog shapes (arena, table with borrowed-key lookup, seq); no new sealed form.

Full worked example: TO BE AUTHORED (this card had no full text at the split; it
is a reserved non-normative slot, not lost content).
