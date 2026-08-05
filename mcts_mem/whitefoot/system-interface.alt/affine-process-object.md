- The entry receives one permanently retained affine `Process` object, and every system operation takes a unique borrow of it.
- All system state — arguments, files, streams, clocks, network — is reached through that single holder.

## Moves

- 2026-08-05 (8f7055fc) replaced by [[system-interface]]: one permanently retained affine Process object makes every operation contend for the same unique holder, falsely serializing files, output, networking, clocks, and workers; making it shared would need a central lock or hidden aliasing (sourced)
