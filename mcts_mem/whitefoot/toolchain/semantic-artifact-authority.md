- The numbered specification alone defines source-language behavior.
- The compiler's checked in-memory representation will become lowering authority when its real checker and lowerer exist; it need not first serialize and replay itself.
- Conformance cases and focused reference models are independent evidence, not production acceptance authorities.
- Serialization, cache validation, or third-party artifact trust must be designed only when a real consumer requires them.

## Facts

- 2026-07-22 owner correction: stable artifact interchange and mandatory replay are not required for the current research compiler. (sourced)
- 2026-07-22 implementation state: no semantic checker, checked IR, lowerer, cache, or external artifact consumer exists yet. (code)

## Moves

- 2026-07-22 replaced [[mandatory-artifact-replay]]: same-kernel serialization and replay added no independent semantic evidence and imposed a protocol before any real artifact consumer existed; direct checked in-memory state plus independent behavioral tests is the smallest correct route to execution (sourced)
