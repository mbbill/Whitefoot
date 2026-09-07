- Recoverable failures are ordinary `Result` values; Whitefoot has no exception, throw, catch, unwinding, or exception-handling region.
- The forwarding form is `let value = propagate expression;`, with its type inferred under the ordinary binding rule.
- `propagate` is the fixed terminal for this form; `try` is an ordinary IDENT and there is no compatibility alias.
- `Ok` binds its payload and continues; `Err` returns through the enclosing function's Result type and receives the checked auto-derived context record.

## Facts

- 2026-07-22 (c95bda9b) owner selection: `propagate` replaced `try` one-for-one because the construct forwards a Result value and does not enter an exception-handling region. (sourced)
- 2026-07-22 (d5c95b72) implementation: exact v0.11, the compiler frontend and resolver, conformance source, and the focused reference model all use `propagate`; `try` lexes and resolves as an ordinary identifier. (code)
- 2026-07-22 implementation: the v0.12 checker and lowerer implement exact same-E Result forwarding with a checked `(function, node_path)` context, an Ok payload continuation, and an Err return edge carrying derived cleanup. A non-place Result expression and an explicitly moved affine Result place are supported through the one control-flow path. (code)
- 2026-07-22 discrepancy: current ERR-3 does not say that `propagate` implicitly consumes a bare own Result place, while several existing cases assume that match-like exception to OWN-1. Whether to add that exception in a successor specification or require written `move` remains an owner decision; the active compiler does not infer the exception from tests. (code)
- 2026-07-22 owner-approved specification: exact v0.13 resolves the discrepancy by making a direct bare affine own-rooted Result place a consuming operand while retaining explicit `move`; Ok/Err control flow, same-E checking, cleanup, borrow restrictions, effects, and runtime behavior are unchanged. (sourced)
- 2026-07-22 implementation: the semantic checker uses one consuming-expression context shared by `match` and `propagate`; it consumes the whole affine root exactly once and rejects later reuse under OWN-1. (code)

## Moves

- 2026-07-22 (c95bda9b) replaced [[try-spelling]]: `try` commonly suggests entering exception-handling control flow, but Whitefoot only forwards an ordinary Result value; `propagate` names that exact action without implying throw, catch, or unwinding semantics (sourced)
