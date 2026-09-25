Node: language/effects

Decision: A row lists no entry at or below the path of another of its `writes` entries, so `reads(p)`, `reads(p.x)` and `writes(p.x)` beside `writes(p)` are each an EFF-1 rejection at that entry, carrying the `writes` entry that covers it, because `writes(p)` already states every access at or below `p`, so the entry is a second spelling of that statement, and once a call no longer compares two entries one argument supplies that overlap whatever their positions are, only the declaration can refuse the redundant row, instead of refusing only `reads(p)` beside `writes(p)` or admitting the redundant entry.

Rejected:
- Refusing only the same-path pair `reads(p), writes(p)`: rejected because `reads(p.x)` and `writes(p.x)` beside `writes(p)` are just as redundant, and after the call-site exemption for one argument's always-overlapping entries nothing else refuses them, so one body would have several admitted rows.
- Placing the rejection at the covering `writes` entry or at the whole row: rejected because the redundant entry is the one the writer deletes, so its own position together with the entry that covers it locates the repair in one step.
