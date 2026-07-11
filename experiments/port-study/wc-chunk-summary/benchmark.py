#!/usr/bin/env python3
"""Report median kernel time for the ordered chunk-summary ceiling study."""

import re
import statistics
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TIME = re.compile(r"best_ns=(\d+)")
VARIANTS = (
    ("xlang-facts", HERE / "xchunkwc_facts"),
    ("xlang-no-facts", HERE / "xchunkwc_nofacts"),
    ("C-control", HERE / "c_chunk_wc"),
    ("Rust-safe", HERE / "rust_chunk_wc"),
)


def measure(executable: Path, corpus: Path, threads: int, samples: int) -> float:
    values = []
    for _ in range(samples):
        output = subprocess.check_output(
            [str(executable), str(corpus), str(threads), "1"], text=True
        )
        match = TIME.search(output)
        if not match:
            raise RuntimeError(output)
        values.append(int(match.group(1)) / 1e6)
    return statistics.median(values)


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit(f"usage: {sys.argv[0]} CORPUS [SAMPLES]")
    corpus = Path(sys.argv[1]).resolve()
    samples = int(sys.argv[2]) if len(sys.argv) == 3 else 7
    print("| variant | 1 thread | 2 threads | 4 threads | 8 threads |")
    print("|---|---:|---:|---:|---:|")
    for name, executable in VARIANTS:
        values = [measure(executable, corpus, threads, samples) for threads in (1, 2, 4, 8)]
        print(f"| {name} | " + " | ".join(f"{value:.1f} ms" for value in values) + " |")


if __name__ == "__main__":
    main()
