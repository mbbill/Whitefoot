# Task: Checked Integer Parser

Write a complete program in the target language.

Requirements:

- Parse an ASCII byte sequence representing a non-negative decimal `u64`.
- Reject an empty input.
- Reject any non-digit byte.
- Reject overflow before it can silently wrap.
- Return a recoverable error value, not a process abort, for invalid input.
- In `main`, verify these cases:
  - `"0"` parses as `0`.
  - `"42"` parses as `42`.
  - `"18446744073709551615"` parses as `u64::MAX`.
  - `"18446744073709551616"` is rejected as overflow.
  - `"12x"` is rejected as invalid digit.
- Print `ok` on success.

Do not use unchecked overflow, undefined behavior, or an unsafe escape hatch.
