Node: compiler/parallel-lowering/two-worlds

Decision: After needed-capture selection, rescue a synthesized range split whose conservative estimate still exceeds the lane bound when the shared selected-target calculation proves its transported signature fits, because a used small aggregate can otherwise be charged the entire slot despite its fitting representation in the [target-fitting investigation](../../research/investigations/compute-model/DESIGN.md#selected-target-loop-frame-fitting), instead of duplicating layout arithmetic or enlarging every lane. Preserve the existing path for conservatively fitting frames and reuse the completed ordinary CFG once on a real refusal, because exact fitting must not change their capture interfaces or add splitter overhead to fallback.

Rejected:
- Accepting every candidate and leaving refusal to the emitter: rejected because a truly oversized loop would retain recursive splitting despite being unable to hand out work.
- Deciding exact fit before needed-capture selection: rejected because that changes the existing rescue policy's transport and helper order without serving the demonstrated post-pruning obstruction.
