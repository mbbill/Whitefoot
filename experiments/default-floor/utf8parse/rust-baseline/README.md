# Frozen Rust baseline

This package is the ordinary safe adapter around the unmodified public API of
`utf8parse = 0.2.2` with its ordinary default features. This release's default
feature set is empty. The adapter does not reimplement the state machine:
`parse_into` creates a `Parser`, advances it once per input byte, and writes
`Receiver` events into a caller-owned `u32` slice.

Valid codepoints are stored as their Unicode scalar values. Invalid-sequence
events are stored as `0x00110000`, the first integer above the Unicode scalar
range. Output capacity is checked before any write, and unused output elements
remain unchanged.
