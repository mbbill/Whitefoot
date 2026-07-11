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

# Second pin [F003 channel]: scoped-alias metadata from ownership provenance.
# The &uniq-struct-of-buffers kernel must (a) carry !alias.scope facts, and
# (b) vectorize at -O2 with ZERO runtime alias guards; the no-facts control
# must do neither. Guards the un-Rust-able delta measured in
# experiments/scoped-alias-channel/.
def asmtext(extra):
    subprocess.run([sys.executable, "democ.py", "examples/soa_kernel.xl", "--asm"] + extra, check=True, capture_output=True)
    return Path("examples/soa_kernel.ll").read_text(), Path("examples/soa_kernel.s").read_text()
fll, fs = asmtext([]); nll, ns_ = asmtext(["--no-facts"])
assert "!alias.scope" in fll and "!alias.scope" not in nll, "PERF REGRESSION: scoped-alias metadata channel changed"
fvec = fs.count("add.2d") + fs.count(".2d,")
nvec = ns_.count("add.2d") + ns_.count(".2d,")
print(f"soa kernel vector adds with facts={fvec} without={nvec}")
assert fvec > 0 and nvec == 0, "PERF REGRESSION: scoped-alias vectorization changed"
print("OK: scoped-alias (F003) vectorization intact")

# Third pin [FN-4 channel]: checked-law reassociation. The proved-law reduction
# must be rewritten to 4 independent accumulators (>= 8 sat-add sites in the
# facts IR); the control keeps the single serial op. Guards the 3.3x delta
# measured in experiments/checked-law-channel/.
sys.path.insert(0, ".")
import democ as _d
_src = Path("examples/sat_reduce.xl").read_text()
_f = _d.compile_program(_src).count("uadd.sat")
_n = _d.compile_program(_src, alias=False).count("uadd.sat")
print(f"sat-reduce uadd.sat sites with facts={_f} without={_n}")
assert _f >= 8 and _n <= 3, "PERF REGRESSION: checked-law reassociation changed"
print("OK: checked-law (FN-4) reassociation intact")

# Fourth pin: willreturn tier soundness. The give-match counterexamples from
# the adversarial review (a loop hidden in a give-match arm; a call to that fn
# from another give-match arm) must NEVER earn willreturn.
_tp = _d.compile_program(Path("examples/totality_pins.xl").read_text())
assert "willreturn" not in _tp, "SOUNDNESS REGRESSION: willreturn on a divergent give-match fn"
print("OK: willreturn tier soundness pins intact")
