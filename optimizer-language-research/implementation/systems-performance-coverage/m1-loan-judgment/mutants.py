"""Mutation harness (kept separate from run.py). Each mutation flips one
deliberate bit of the machine spec in checker.py; the FULL extended corpus
(programs_ast.PROGRAMS) is then run against the mutant. A mutation that flips no
program's verdict is a corpus HOLE at that spec clause.

The repair round added W-D / W-D2 / W-F to the corpus specifically so mutations
D (drop R12 liveness), D2 (erase R12 holder names) and F (drop the R5 issue/
capture preconditions) -- all previously uncaught (NO FLIPS) -- are now caught.

Usage: python3 mutants.py
"""
import os, sys, shutil, subprocess, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = open(os.path.join(HERE, "checker.py")).read()

EVAL = '''import sys
from checker import check_program
from programs_ast import PROGRAMS
flips = 0
for pid, prog, expected, canonical in PROGRAMS:
    try:
        r = check_program(prog)
        got = 'ACCEPT' if r == 'ACCEPT' else 'REJECT'
    except Exception as ex:
        got = f'CRASH({type(ex).__name__})'
    if got != expected:
        flips += 1
        print(f"  FLIP {pid}: expected {expected}, got {got}")
print(f"__FLIPS__ {flips}")
'''

# (name, list of (old, new) text replacements). Each old must occur exactly once.
MUTATIONS = {
 "A-skip-R10a-place-comparison": [(
    """                if hb.src[hreg] != R:
                    raise Reject('R10a', f"brand mismatch {root}: src {hb.src[hreg]} != tied {R}")
                if Entry(R, hb.kind, root) not in self.loans:
                    raise Reject('R10a', f"no live entry ({R},{hb.kind},{root})")""",
    """                if not any(e.holder == root for e in self.loans):
                    raise Reject('R10a', f"no live entry held by {root}")""")],

 "B-forward-order-scope-end": [(
    "        for name in reversed(names):",
    "        for name in list(names):")],

 "C-issues-before-legality": [(
    """        params, args = sig.params, s.args

        consumed = set()""",
    """        params, args = sig.params, s.args

        issued_early = False
        if s.name is not None and sig.result is not None and sig.result.confined:
            self._issue_result(s.name, sig, args)
            issued_early = True

        consumed = set()"""),
   ("""        if s.name is not None:
            if sig.result is not None and sig.result.confined:
                self._issue_result(s.name, sig, args)""",
    """        if s.name is not None:
            if sig.result is not None and sig.result.confined:
                if not issued_early:
                    self._issue_result(s.name, sig, args)""")],

 "D-projection-drops-liveness": [(
    """        lv = frozenset((n, b.status) for n, b in self.bindings.items()
                       if b.decl_index < cutoff)""",
    """        lv = frozenset()""")],

 "D2-projection-erases-holder-names": [(
    """        ents = frozenset((e.place, e.kind, e.holder) for e in self.loans
                         if self.bindings[e.holder].decl_index < cutoff)""",
    """        ents = frozenset((e.place, e.kind) for e in self.loans
                         if self.bindings[e.holder].decl_index < cutoff)""")],

 "E-EXC-intersection-not-sole": [(
    "        return len(holders) == 1 and holders <= consumed_holders",
    "        return bool(holders & consumed_holders)")],

 "F-drop-R5-issue-preconditions": [(
    """            if res.kind == 'uniq':
                if any(overlap(e.place, S) for e in self.loans):
                    raise Reject('R5', f"uniq issue on {S} with an overlapping entry")
            else:
                if any(overlap(e.place, S) and e.kind == 'uniq' for e in self.loans):
                    raise Reject('R5', f"shr issue on {S} with an overlapping uniq entry")""",
    """            pass  # MUT: R5 issue preconditions removed"""),
   ("""                if td.kind == 'uniq':
                    if any(overlap(e.place, S) for e in self.loans):
                        raise Reject('R5', f"uniq capture on {S} overlaps an entry")
                else:
                    if any(overlap(e.place, S) and e.kind == 'uniq' for e in self.loans):
                        raise Reject('R5', f"shr capture on {S} overlaps a uniq entry")""",
    """                pass  # MUT: R5 capture preconditions removed""")],

 "G-drop-R5-unbound-region-check": [(
    """        missing = set(td.region_params) - bound
        if missing:
            raise Reject('R5', f"construction of {td.name} leaves region(s) {missing} unbound")""",
    """        missing = set(td.region_params) - bound
        if False and missing:
            raise Reject('R5', f"construction of {td.name} leaves region(s) {missing} unbound")""")],

 # H makes the frozen R9 verbatim (no is_form carve-out): the form table cannot
 # be typed and the module fails to import -> caught as a load-time crash.
 "H-spec-verbatim-R9": [(
    """            if sig.is_form:
                # form-table token self-ties (its source place is recorded, not
                # passed) even by own; the clause carries the real effect.
                ties[rp] = SELF
            elif borrow_conf and not own_conf and not result_uses:""",
    """            if borrow_conf and not own_conf and not result_uses:""")],
}


def main():
    root = tempfile.mkdtemp(prefix="m1mut_")
    caught, holes = [], []
    for name, repls in MUTATIONS.items():
        d = os.path.join(root, name)
        os.makedirs(d)
        src = SRC
        ok = True
        for old, new in repls:
            if src.count(old) != 1:
                print(f"== {name}: PATTERN MISMATCH (count={src.count(old)}) -- skipped")
                ok = False
                break
            src = src.replace(old, new)
        if not ok:
            continue
        open(os.path.join(d, "checker.py"), "w").write(src)
        shutil.copy(os.path.join(HERE, "programs_ast.py"), d)
        open(os.path.join(d, "eval_corpus.py"), "w").write(EVAL)
        out = subprocess.run([sys.executable, "eval_corpus.py"], cwd=d,
                             capture_output=True, text=True, timeout=180)
        flips = None
        for line in out.stdout.splitlines():
            if line.startswith("__FLIPS__"):
                flips = int(line.split()[1])
        crash = out.returncode != 0 and flips is None
        n = flips if flips is not None else 0
        status = "CAUGHT" if (n > 0 or crash) else "HOLE"
        (caught if status == "CAUGHT" else holes).append(name)
        detail = "import/run crash" if crash else f"{n} program(s) flipped"
        print(f"== {name:34} {status:6} ({detail})")
        if flips:
            for line in out.stdout.splitlines():
                if line.startswith("  FLIP"):
                    print("   " + line.strip())
        if crash and out.stderr:
            print("   " + out.stderr.strip().splitlines()[-1])
    shutil.rmtree(root, ignore_errors=True)
    print(f"\ncaught {len(caught)}/{len(MUTATIONS)}: {caught}")
    if holes:
        print(f"HOLES ({len(holes)}): {holes}")
    else:
        print("no corpus holes across the mutation set")


if __name__ == '__main__':
    main()
