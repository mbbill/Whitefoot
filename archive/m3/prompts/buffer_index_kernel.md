# Task: Buffer Index Kernel

Write a complete program in the target language.

Requirements:

- Allocate a runtime-length buffer of numeric values.
- Fill each element with its index.
- Sum the elements through checked indexing.
- Verify the sum of `0..1024` is `523776`.
- Include one out-of-bounds access path that is rejected or trapped according to
  the language's bounds-checking rules, without corrupting memory.
- Print `ok` on success.

Do not use unchecked indexing, undefined behavior, or an unsafe escape hatch.
