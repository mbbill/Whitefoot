# Task: Value Match Result

Write a complete program in the target language.

Requirements:

- Define a three-case sign enum: negative, zero, positive.
- Implement `sign_of(x)` using explicit branch or match logic.
- In `main`, compute `40 + 2` using a recoverable checked arithmetic operation.
- If the arithmetic succeeds, bind the value and verify it is `42`.
- If it fails, return normally without a crash.
- Call `sign_of(42)` and verify the result is positive.
- Exit successfully only if all checks pass.

Do not use unchecked overflow, exceptions, panics as normal recoverable control
flow, or an unsafe escape hatch.
