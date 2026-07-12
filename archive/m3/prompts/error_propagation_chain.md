# Task: Error Propagation Chain

Write a complete program in the target language.

Requirements:

- Define at least two functions that return recoverable error values.
- Build a third function that calls them in sequence and propagates errors.
- Preserve the original error type; do not convert to strings or trap.
- In `main`, verify:
  - the all-good path returns the expected value
  - the first failing path returns the first error
  - the second failing path returns the second error
- Print `ok` on success.

Do not use exceptions, panics as normal recoverable control flow, unchecked
sentinels, or an unsafe escape hatch.
