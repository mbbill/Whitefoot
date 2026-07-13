# Generation task

Return only a complete xlang source file, with no Markdown fences or prose.

Implement a function named `parse` with this exact public declaration shape:

```text
fn parse ['r] (out: &uniq 'r buffer<u32>, src: own buffer<u8>) -> own u64 reads('r), writes('r), traps requires { ... } { ... }
```

`src` is one complete arbitrary byte buffer.  Start in the UTF-8 ground state
for every call and process its bytes from left to right.  Produce a sequence of
`u32` events:

- a completed valid UTF-8 sequence produces its Unicode scalar value;
- detection of an invalid sequence produces `0x00110000` (decimal `1114112`),
  which is one greater than the largest Unicode scalar value. In xlang source,
  write this value as `1114112_u32`; hexadecimal integer literal syntax is
  unavailable;
- a valid-looking but incomplete sequence at the end of `src` produces no
  event.  There is no end-of-input flush event.

The accepted UTF-8 forms are exactly:

- `00` through `7F`, producing that one-byte scalar immediately;
- `C2` through `DF`, followed by one byte in `80` through `BF`;
- `E0`, followed by `A0` through `BF`, then `80` through `BF`;
- `E1` through `EC` or `EE` through `EF`, followed by two bytes in `80`
  through `BF`;
- `ED`, followed by `80` through `9F`, then `80` through `BF`;
- `F0`, followed by `90` through `BF`, then two bytes in `80` through `BF`;
- `F1` through `F3`, followed by three bytes in `80` through `BF`;
- `F4`, followed by `80` through `8F`, then two bytes in `80` through `BF`.

Every other byte in the ground state produces one invalid event.  While a
multi-byte sequence is pending, a byte outside the range required at that
position produces one invalid event and returns the parser to the ground
state.  That detecting byte is consumed by the invalid event and must not be
processed a second time from the ground state.  For example, `C2 41` produces
only one invalid event, not an additional event for `41`.

The required entry condition is that the visible output length is at least the
source length, even if this particular input will produce fewer events.
Violation of the entry condition must trap before writing any output element.
On success, write the events starting at output index zero, return the event
count as `u64`, leave every output element at or after the returned count
unchanged, and leave the source unchanged. Source and output are distinct live
buffers.

The file may contain helper functions, but it must contain the required
`parse` function and must not contain `main`.
