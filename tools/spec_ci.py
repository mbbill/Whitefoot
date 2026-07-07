#!/usr/bin/env python3
"""Spec-CI: machine checks for kernel-spec META invariants. Exit 1 on violation."""
import re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
def _ver(f):
    import re as _re
    m=_re.search(r'v(\d+)(?:\.(\d+))?', f.name); return (int(m.group(1)), int(m.group(2) or 0))
SPEC = max((ROOT/'spec').glob('kernel-spec-v*.md'), key=_ver)
LEDGER = ROOT/'spec/derivation-ledger.md'
spec = SPEC.read_text(); ledger = LEDGER.read_text()
fail = []
# defined rule IDs = [XXX-n] at line starts; referenced = [XXX-n] anywhere
defined = re.findall(r'^\[([A-Z]+-\d+[a-z]?)\]', spec, re.M)
dups = {r for r in defined if defined.count(r) > 1}
if dups: fail.append(f"META-1/4: duplicate rule definitions: {sorted(dups)}")
refs = set(re.findall(r'\[([A-Z]+-\d+[a-z]?)\]', spec)) | set(re.findall(r'\b([A-Z]{2,6}-\d+[a-z]?)\b', spec))
unknown = {r for r in refs if r not in set(defined)
           and re.match(r'^(SCOPE|FORM|LEX|GRAM|TYPE|CONST|OWN|STOR|OP|FN|EFF|ERR|PROG|DIAG|CAP|GATE|LEDGER|PRE|EX|META)-', r)}
if unknown: fail.append(f"cross-reference to undefined rule IDs: {sorted(unknown)}")
# META-6: every defined rule has a ledger row
missing = [r for r in set(defined) if f'| {r} |' not in ledger]
if missing: fail.append(f"META-6: rules missing derivation-ledger entries: {sorted(missing)}")
# META-3: exception-clause smell scan (words inside normative rule lines)
for line in spec.splitlines():
    if re.match(r'^\[[A-Z]+-\d+', line) and re.search(r'\bexcept that\b|\bunless\b', line):
        rid = line.split(']')[0][1:]
        if rid not in ('OWN-4',):   # OWN-4 'only if' phrasing reviewed as total rule
            fail.append(f"META-3 smell (review; total-rule rewording?): {rid}: ...{line[:90]}")
print(f"spec-ci: {SPEC.name} — {len(set(defined))} rules; ledger rows checked against {LEDGER.name}")
if fail:
    print("VIOLATIONS:"); [print(' -', f) for f in fail]; sys.exit(1)
print("OK: META-1 uniqueness, cross-ref integrity, META-6 ledger coverage, META-3 scan")
