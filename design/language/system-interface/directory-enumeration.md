Decision: Directory enumeration is a one-attempt transfer of a bounded batch of portable records into the caller's validated buffer range, advancing the directory's own position, with the handle owning a cursor and no storage, because a handle owning its own window needs storage the system layer does not allocate, a second call to drain it, and a lifetime rule for entries it still holds, while the caller-range transfer reuses the existing one-attempt file-read contract whole, instead of a handle-owned window yielding entries one at a time.

Decision: An enumerated name reaches source only as bytes, self and parent entries are reported unfiltered, and no order is fixed, because turning a name into a host string or path value would reintroduce a path model and a fixed order would promise what hosts do not provide, instead of filtered, ordered, path-valued entries.

Rejected:
- An enumeration handle owning an internal window of host entries yielded one at a time: rejected because it needs storage the system layer does not allocate, a second call to drain it, and a lifetime rule for held entries.
