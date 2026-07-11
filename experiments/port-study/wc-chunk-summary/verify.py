#!/usr/bin/env python3
"""Differentially verify xlang and Rust chunk summaries across arbitrary splits."""

import random
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SPACE = {9, 10, 11, 12, 13, 32}


def reference(data: bytes) -> tuple[int, int, int]:
    lines = data.count(b"\n")
    words = 0
    previous_space = True
    for byte in data:
        space = byte in SPACE
        if not space and previous_space:
            words += 1
        previous_space = space
    return lines, words, len(data)


def run(executable: Path, path: Path, threads: int) -> tuple[int, int, int]:
    output = subprocess.check_output(
        [str(executable), str(path), str(threads), "1"], text=True
    )
    fields = output.split()
    return tuple(map(int, fields[:3]))


def main():
    rng = random.Random(0x584C414E47)
    cases = [
        b"",
        b"a",
        b" ",
        b"ab",
        b"a b",
        b"a\nb\n",
        bytes(range(256)),
    ]
    for _ in range(100):
        cases.append(bytes(rng.randrange(256) for _ in range(rng.randrange(8193))))
    with tempfile.TemporaryDirectory(prefix="xlang-wc-summary-") as tmp:
        path = Path(tmp) / "case.bin"
        checked = 0
        for data in cases:
            path.write_bytes(data)
            want = reference(data)
            for threads in (1, 2, 3, 4, 7, 16):
                for executable in (HERE / "xchunkwc", HERE / "rust_chunk_wc"):
                    got = run(executable, path, threads)
                    assert got == want, (executable.name, len(data), threads, got, want)
                    checked += 1
    print(f"OK: {len(cases)} byte cases, {checked} implementation/split verdicts")


if __name__ == "__main__":
    main()
