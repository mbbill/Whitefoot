#!/usr/bin/env python3
import subprocess, time, sys, statistics

def bench(label, path, runs=7, iters=None):
    # warm up
    subprocess.run([path], capture_output=True)
    ts = []
    for _ in range(runs):
        t0 = time.perf_counter()
        subprocess.run([path], capture_output=True)
        ts.append(time.perf_counter() - t0)
    med = statistics.median(ts)
    lo, hi = min(ts), max(ts)
    extra = ""
    if iters:
        extra = f"  {med/iters*1e9:8.3f} ns/iter"
    print(f"{label:16s} median {med*1000:9.3f} ms  (min {lo*1000:8.3f}, max {hi*1000:8.3f}){extra}")
    return med

if __name__ == "__main__":
    iters = int(sys.argv[1]) if len(sys.argv) > 1 else None
    binmap = sys.argv[2:] if len(sys.argv) > 2 else []
    # pairs: label=path
    for spec in binmap:
        label, path = spec.split("=", 1)
        bench(label, path, iters=iters)
