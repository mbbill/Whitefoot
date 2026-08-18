- The enumeration handle owns an internal window of host-reported entries and yields them one at a time to source, refilling from the host when the window empties.
- A yielded entry is a value with its own lifetime relative to the handle's next advance.

## Facts

- 2026-08-18 statement: the shape was the reconnaissance sketch's first proposal and was weighed against the caller-range transfer. Its costs are structural rather than incidental: the window is storage the system layer must allocate although no system operation allocates, one source-visible yield no longer maps to one host call, and every yielded entry needs a stated lifetime against the refill that invalidates it. (sourced)

## Moves

- 2026-08-18 (f8c81dfc) replaced by [[directory-enumeration]]: a handle owning its own window needs storage the system layer does not allocate, a second call to drain it, and a lifetime rule for entries it still holds, while the caller-range transfer reuses the existing one-attempt contract whole (sourced)
