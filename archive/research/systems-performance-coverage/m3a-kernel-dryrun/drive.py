#!/usr/bin/env python3
"""Interleave the C-kernel and Rust-baseline binaries R times to share thermal
state, parse each benchmark's per-run median, and report the median-across-runs
and the ratio C/Rust. Indicative (Apple M4, macOS arm64)."""
import os, subprocess, re, statistics, sys

R = int(sys.argv[1]) if len(sys.argv) > 1 else 9
HERE = os.environ.get("WHITEFOOT_M3A_WORK_ROOT")
if not HERE:
    raise SystemExit("WHITEFOOT_M3A_WORK_ROOT is required")
C_SEQ = f"{HERE}/bench_cseq_lto"
C_TBL = f"{HERE}/bench_ctable"
RUST  = f"{HERE}/rustbench/target/release/rustbench"

# label -> (regex capturing the ns number)
PATS = {
    "B1-combined": r"combined= ([\d.]+)",
    "B2-build":    r"insert \(ns/op.*?: ([\d.]+)",
    "B2-lookup":   r"hit-lookup \(ns/op.*?: ([\d.]+)",
    "B3-miss":     r"miss-lookup \(ns/op.*?: ([\d.]+)",
    "B4-iter":     r"iterate-sum \(ns/elem.*?: ([\d.]+)",
}

def grab(text, key):
    m = re.search(PATS[key], text)
    return float(m.group(1)) if m else None

c = {k: [] for k in PATS}
rs = {k: [] for k in PATS}
for i in range(R):
    ct = subprocess.run([C_SEQ], capture_output=True, text=True).stdout
    tb = subprocess.run([C_TBL], capture_output=True, text=True).stdout
    ru = subprocess.run([RUST], capture_output=True, text=True).stdout
    cc = ct + tb
    for k in PATS:
        v = grab(cc, k)
        if v is not None: c[k].append(v)
        v = grab(ru, k)
        if v is not None: rs[k].append(v)

print(f"{'benchmark':<13}{'C (ns)':>10}{'Rust (ns)':>12}{'ratio C/Rust':>14}   {'band<=1.25':>10}")
order = ["B1-combined", "B2-build", "B2-lookup", "B3-miss", "B4-iter"]
for k in order:
    cm = statistics.median(c[k]); rm = statistics.median(rs[k])
    ratio = cm / rm
    tag = "OK" if ratio <= 1.25 else "OVER"
    print(f"{k:<13}{cm:>10.3f}{rm:>12.3f}{ratio:>13.3f}x   {tag:>10}")
print(f"(medians across {R} interleaved runs)")
