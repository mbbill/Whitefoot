- System access appears as ambient free functions — args, open, stdin, spawn — callable from any function with small call sites and familiar names.
- No signature reveals which system facilities a function or its callees use.

## Moves

- 2026-08-05 (8f7055fc) replaced by [[system-interface]]: ambient system functions hide access and create inter-function channels against FN-7's no-global rationale; system use invisible in signatures cannot be narrowed, tested, or parallelized by ownership (sourced)
