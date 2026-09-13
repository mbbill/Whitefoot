- Direct slices carry finite static ultimate-storage origin sets.

- Each runtime slice descriptor selects one actual member of its static origin set.

- Inside a function, an origin is one resolved source place, `immutable-const`, or a formal-slice term naming one direct slice parameter. Formation creates a singleton; binding, movement, borrowing the descriptor, passing, and returning preserve the complete set.

- An `own slice<'r, T>` signature ceiling contains `immutable-const` and exactly the own direct-slice parameters with the same formal result region and element type. Each body return must be a subset of that ceiling.

- Calls compute result-origin sets by signature-only simultaneous supplier substitution and deduplicated union.

- The selected boundary is direct-only: region-bearing generic arguments and stored content reject under FN-2/STOR-5. Borrow-mode direct-slice results, returned arena/raw-storage suppliers, stored slice leaves, and slice-valued control-flow joins require separate designs.

## Facts

- 2026-07-23 owner selection: exact v0.17 SHA-256 `19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93` selects signature-derived finite origins for direct returned slices. The selection preserves caller-visible soundness without adding syntax or making callable meaning body-sensitive. (sourced)
- 2026-07-23 rejected alternative: an explicit return-origin annotation can be more precise for a same-region function that always returns only one input, but adds writer syntax plus body validation. A writer can instead expose that distinction with separate formal regions, so the extra surface was not selected for this capability. (sourced)
- 2026-07-23 rejected alternative: a body-derived exact summary would make callable contracts depend on implementation bodies and require finite fixed-point handling for recursive call groups. That conflicts with FN-1's signature-complete caller boundary and was not selected. (sourced)
- 2026-07-23 rejected alternative: per-leaf metadata inside generic or stored values would retain more programs, but imports unresolved retained-state, cleanup, and stored-borrow obligations. v0.17 rejects those hiding positions rather than approximating them. (sourced)
- 2026-07-23 rejected alternative: a fresh call or arena token cannot soundly authorize returned arena storage until the backing allocation and cleanup obligation are proved to survive the callee. Callee-created arena suppliers remain deferred rather than represented by an invented origin. (sourced)
- 2026-07-23 implementation evidence: one safe-Rust semantic path now carries the same origin data through checked expressions and signatures, checks every returned set against its ceiling, substitutes actual sets at calls, and applies all origins to alias and effect judgments. A concrete source claim is attached to its source owner rather than a descriptor binding, unions across continuing control paths, and is removed only when its named data region ends; focused nested-scope and branch regressions prevent descriptor cleanup from shortening the claim. Host execution confirms pass-through, same-region choice, immutable-const returns, and borrowed descriptor reads without changing the runtime descriptor ABI. (code)
