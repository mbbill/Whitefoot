- The range split descends to a leaf of exactly one iteration, making every iteration a schedulable unit and leaving grain entirely to the runtime.
- The loop body reaches the lowering as a function of the index rather than as a loop; the emitted leaf carries no loop at all.
- Weighed during the batch that landed the split lowering; never landed.

## Moves

- 2026-08-23 (ddf1d139) replaced by [[loop-permission]]: a leaf of one iteration destroys the body's own optimization, measured at a 3.6-7.6x penalty on light bodies against the 3.1x a subrange leaf preserves; a leaf that is still a loop keeps the sequential world's code byte-for-byte and costs the split nothing to decline. (sourced)
