Decision: Bool and user tag-only enums with at most two variants lower to a one-bit value and larger tag-only enums to a 32-bit value, consistently across values, calls, equality, and match dispatch, because word-sized tags kept a loop's carried state at word width where the vectorizer could not pack it one bit wide, a measured 34 percent penalty that blocked the sixteen-wide vectorization the one-bit lowering restores to parity with C and Rust, and equivalent shapes must not have unequal speed, instead of word-sized tags everywhere.

Rejected:
- Every enum tag lowered at 32 bits including Bool: rejected because it kept a loop's carried state at word width where the vectorizer could not pack it one bit wide, a measured 34 percent penalty that blocked sixteen-wide vectorization.
