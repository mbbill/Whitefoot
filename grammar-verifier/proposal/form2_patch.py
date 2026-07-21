from __future__ import annotations

import difflib


class PatchError(RuntimeError):
    pass


def unified_patch(files: list[tuple[str, bytes, bytes]]) -> bytes:
    output = bytearray()
    for relative, before, after in files:
        if before == after:
            continue
        rows = difflib.diff_bytes(
            difflib.unified_diff,
            before.splitlines(keepends=True),
            after.splitlines(keepends=True),
            fromfile=f"a/{relative}".encode("utf-8"),
            tofile=f"b/{relative}".encode("utf-8"),
            lineterm=b"\n",
        )
        for row in rows:
            if row[:1] in (b" ", b"+", b"-") and not row.endswith(b"\n"):
                output.extend(row)
                output.extend(b"\n\\ No newline at end of file\n")
            else:
                output.extend(row)
    raw = bytes(output)
    if raw and not raw.endswith(b"\n"):
        raise PatchError("generated patch does not end in LF")
    return raw
