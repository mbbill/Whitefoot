Decision: Struct-of-arrays is the taught default layout for bulk data while array-of-structs stays a genuine need with a named path through a copy-struct tier, because a struct of copy fields can be declared copy and stored in a buffer without importing the affine-element problems, instead of forbidding array-of-structs.

Decision: Append-only stable identity and recyclable stable identity are separate observable contracts, and the append-only path pays no generation or recycling cost, because a well-typed slot-recycling use-after-free is unrepresentable when indices never recycle and access is then a bare bounds check, instead of one pool contract that pays for recycling everywhere.

Decision: Rehash, compaction, drain repair, insertion and removal shifts, and encoded-string removal are ownership relocations in which every retained owner reaches exactly one destination and moved-from slots are never read or dropped, because treating them as metadata updates or copies would duplicate or lose owners, instead of metadata-only updates.

Decision: Current tests and small programs are capability and cost witnesses, never evidence of a production workload distribution, because the project has no substantial real application yet and prevalence claims need representative runtime evidence, instead of reading the corpus as usage data.
