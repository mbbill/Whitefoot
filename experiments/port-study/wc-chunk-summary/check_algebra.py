#!/usr/bin/env python3
"""Exhaustively check the ordered wc-summary monoid on a small byte alphabet."""

from itertools import product

SPACE = {9, 10, 11, 12, 13, 32}
ALPHABET = (ord("a"), ord("b"), ord(" "), ord("\n"))


def summarize(data: bytes) -> tuple[int, int, int, int, int]:
    if not data:
        return (0, 0, 0, 1, 1)
    lines = data.count(b"\n")
    words = 0
    prev_space = True
    for byte in data:
        space = byte in SPACE
        if not space and prev_space:
            words += 1
        prev_space = space
    return (lines, words, len(data), int(data[0] in SPACE), int(prev_space))


def combine(a, b):
    if a[2] == 0:
        return b
    if b[2] == 0:
        return a
    joined = int(not a[4] and not b[3])
    return (a[0] + b[0], a[1] + b[1] - joined, a[2] + b[2], a[3], b[4])


def samples(max_len=4):
    yield b""
    for n in range(1, max_len + 1):
        for values in product(ALPHABET, repeat=n):
            yield bytes(values)


def main():
    cases = list(samples())
    summaries = [summarize(case) for case in cases]
    identity = summarize(b"")
    for data, summary in zip(cases, summaries):
        assert combine(identity, summary) == summary
        assert combine(summary, identity) == summary
        for cut in range(len(data) + 1):
            assert combine(summarize(data[:cut]), summarize(data[cut:])) == summary
    checked = 0
    for a in summaries:
        for b in summaries:
            for c in summaries:
                assert combine(combine(a, b), c) == combine(a, combine(b, c))
                checked += 1
    print(f"OK: identity/split equivalence and {checked} associativity triples")


if __name__ == "__main__":
    main()
