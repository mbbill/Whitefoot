# Task: Arena AST Builder

Write a complete program in the target language.

Requirements:

- Define a tiny expression AST with integer literals and binary addition.
- Allocate at least seven nodes in an arena or index pool.
- Use handles/indices instead of per-node heap ownership.
- Evaluate the expression tree.
- Drop/free the whole arena or pool in bulk.
- In `main`, build an expression equivalent to `(1 + 2) + (3 + 4)`.
- Verify the result is `10`.
- Print `ok` on success.

Do not use reference counting, garbage collection, unchecked aliasing, or an
unsafe escape hatch.
