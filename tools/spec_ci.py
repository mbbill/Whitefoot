#!/usr/bin/env python3
"""Spec-CI: machine checks for kernel-spec META invariants. Exit 1 on violation."""
import hashlib, re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT/'spec/kernel-spec-v0.9.md'
SPEC_SHA256 = 'bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68'
LEDGER = ROOT/'spec/derivation-ledger.md'
spec_bytes = SPEC.read_bytes()
spec_digest = hashlib.sha256(spec_bytes).hexdigest()
if spec_digest != SPEC_SHA256:
    print(f"VIOLATIONS:\n - active specification digest mismatch: {spec_digest}")
    sys.exit(1)
spec = spec_bytes.decode('utf-8'); ledger = LEDGER.read_text()
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
# D1 guard: the FORM-3 OPNAME lexer class must not swallow a field-access place [GRAM-5]
_m = re.search(r'OPNAME `([^`]+)`', spec)
if _m:
    try:
        _opn = re.compile('^(?:' + _m.group(1) + ')$')
        for _s in ('s.field', 'p.x', 'result.count', 'node.left'):
            if _opn.match(_s):
                fail.append(f"D1: FORM-3 OPNAME swallows field-access place {_s!r} (GRAM-5 collision)")
        for _op in ('iadd.wrap', 'isub.trap', 'imul.checked', 'fadd.strict'):
            if not _opn.match(_op):
                fail.append(f"D1: FORM-3 OPNAME fails to lex operation {_op!r}")
    except re.error as _e:
        fail.append(f"D1: FORM-3 OPNAME pattern uncompilable: {_e}")
print(f"spec-ci: {SPEC.name} — {len(set(defined))} rules; ledger rows checked against {LEDGER.name}")
if fail:
    print("VIOLATIONS:"); [print(' -', f) for f in fail]; sys.exit(1)
print("OK: META-1 uniqueness, cross-ref integrity, META-6 ledger coverage, META-3 scan")
