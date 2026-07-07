#!/usr/bin/env python3
"""Regression pin: the noalias load-elimination win must not silently vanish.
Compiles twice_read with/without ownership facts; asserts 1 vs 2 loads at -O2."""
import subprocess, sys, re
from pathlib import Path
def loads(extra):
    subprocess.run([sys.executable, "democ.py", "examples/twice_read.xl", "--asm"] + extra, check=True, capture_output=True)
    return Path("examples/twice_read.s").read_text().count("ldr")
w = loads([]); wo = loads(["--no-facts"])
print(f"loads with facts={w} without={wo}")
assert w == 1 and wo == 2, "PERF REGRESSION: noalias win changed"
print("OK: ownership-fact load elimination intact")
