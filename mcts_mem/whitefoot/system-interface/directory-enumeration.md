- Directory enumeration is a one-attempt transfer into the caller's validated buffer range, in the same family as the file read: one host call reports a bounded batch of entries and advances the directory's own position. The handle owns a cursor and no storage.
- The batch is delivered as portable records — one kind byte, one little-endian `u16` name length, then exactly that many name bytes — rewritten in place within the caller's authorized range. The returned endpoint delimits the portable prefix, while the native transfer may also have changed its unused tail; no intermediate staging buffer or second host call exists.
- An enumerated name reaches source only as those bytes. No operation turns one into a host string or a path value, and the child-open operation takes a caller-owned single-component name range rather than a path.
- A short batch is not the end of the directory; only the distinguished end outcome states that. A range too small for the next record is a recoverable failure that does not advance the cursor.
- Self and parent entries are reported unfiltered, and no enumeration order is fixed. A program that needs determinism sorts what it collected, and a program that descends excludes those entries itself.

## Facts

- 2026-08-18 (f8c81dfc) measurement: the target binding was selected against the host rather than assumed — the portable-inode enumeration entry point is the only one that links with 64-bit inodes, and the open-plus-read pair costs two calls and an allocation. (code)
- 2026-08-18 (f8c81dfc) measurement: a too-small range returns the target's own invalid-argument failure and advances nothing, verified at 8- and 40-byte ranges with a following full-size call reporting every entry. (code)
- 2026-08-18 (f8c81dfc) pitfall: every bound derived from an enumeration record is external data under the provenance gate, so a claim over it is rejected and the walk must discharge each bound with a real value branch. (code)
- 2026-08-18 (f8c81dfc) statement: that rejection is the gate working as designed, and it is why the first traversal program was rewritten before it ran. (code)
- 2026-08-18 statement: the portable record header is smaller than any native one, so the in-place rewrite only ever moves bytes forward and cannot overwrite unread data. (code)

## Moves

- 2026-08-18 (f8c81dfc) replaced [[enumeration-cursor-window]]: a handle owning its own window needs storage the system layer does not allocate, a second call to drain it, and a lifetime rule for entries it still holds, while the caller-range transfer reuses the existing one-attempt contract whole (sourced)
