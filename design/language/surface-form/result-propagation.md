Decision: Recoverable failure is an ordinary `Result` value forwarded by `let value = propagate expression;`, with no exception, throw, catch, or unwinding, because `try` commonly suggests entering exception-handling control flow while the language only forwards an ordinary value, and `propagate` names that exact action, instead of a `try` spelling.

Decision: A bare affine `Result` place used by `propagate` consumes its storage root exactly once, with explicit `move` still valid, because forwarding must consume one affine operand and the ordinary consuming rule lets the canonical bare form do so without weakening ownership, while requiring `propagate move p` contradicted the approved writer form, instead of requiring an explicit move operand.

Rejected:
- `let value = try expression;`: rejected because `try` suggests exception-handling control flow that the language does not have.
- Requiring `propagate move result` for a named affine Result: rejected because forwarding must consume one affine operand and the bare form can do so under the ordinary consuming rule without weakening ownership.
