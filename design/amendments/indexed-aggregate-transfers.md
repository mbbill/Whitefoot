Node: design/compiler/storage-placement.md

Decision: Buffer and slice aggregate element reads and writes use the existing target-sized snapshot-copy path, preserving the old element before replacement and retaining ordinary storage interference, because expanding indexed owning transfers into SSA fields exposes the entire inline payload to scalarization even when the next operation only moves it, instead of treating indexed transfers as a separate aggregate load/store path. The fixed-source scatter control and its retention criterion are recorded in [the investigation](../../research/investigations/compute-model/DESIGN.md#scatter-utilization-attribution); performance selection remains pending that control.

Rejected:
- Borrowing the chunk directly or replacing its representation as this repair: rejected because either changes the source ownership or algorithm control and cannot isolate the indexed-transfer lowering cost.
