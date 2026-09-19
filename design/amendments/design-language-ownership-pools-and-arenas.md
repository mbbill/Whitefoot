Node: language/ownership/pools-and-arenas

Decision: A pool and an arena are usages of ordinary storage rather than types of their own: a pool is a block of slots plus integer handles into it, and an arena is the same block used with an append as allocation and a truncation as reset, because once storage can be indexed and its length is ordinary readable data neither usage needs a distinct type, a confinement rule or an origin discipline to be safe, instead of arena and pool types whose content the checker keeps apart from ordinary values.

Decision: A handle a program considers stale, while still within the block's length, names the current occupant of that slot and is a logic error rather than a memory error, and a program that must detect reuse keeps a generation number beside the element as data, because the storage it reaches still exists and is still initialized, so no memory-safety property depends on the handle's age, and the same holds for a reference into a user-level pool whose bookkeeping calls the slot free, instead of compiler-maintained slot identity or a generation check built into the language.

Rejected:
- Arena and pool as language types carrying confined content: rejected because their content is ordinary owned storage, and the confinement existed to protect an origin discipline that references no longer carry.
- Compiler-maintained per-slot occupancy or generation tags: rejected because occupancy that is decided by data belongs in data, and a tag the program does not need would be paid for by every program that reuses a slot.
