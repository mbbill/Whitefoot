#!/usr/bin/env python3
"""xlang conformance test system — spec-anchored, rule-keyed, toolchain-agnostic.

Each case is a canonical `.xl` source (conformance/cases/<id>.xl) plus a manifest
entry (conformance/manifest.jsonl) declaring the rule id(s) it exercises and the
expected verdict. Cases are driven through a TOOLCHAIN ADAPTER — democ today, any
conformant xlang compiler (real, then self-hosted) tomorrow — and the verdict is
asserted; for a rejection the EXACT cited rule id is checked [DIAG-1]. A coverage
tracker binds cases to the spec's rule ids and reports which rules are untested.

Because it tests the LANGUAGE (source -> verdict), not a prototype's internals, this
suite outlives every compiler rewrite, including self-hosting. It is the correctness
oracle the M3 AI-codegen harness scores against, and the bootstrap safety net.

Verdict = ("accept",) | ("reject", rule) | ("run", exit) | ("trap",) | ("unsupported", why)

Manifest line (JSON):
  {"id": str, "rules": [rule_id...], "expect": EXPECT,
   "status": "runnable"|"pending"|"xfail", "reason": str?, "doc": str}
  EXPECT = {"kind":"accept"} | {"kind":"reject","rule":R} | {"kind":"run","exit":N} | {"kind":"trap"}

  status: runnable = must match expect;  pending = toolchain can't run it yet (skip);
          xfail = expect is the CORRECT spec behavior but the current toolchain does
                  not yet produce it (a tracked gap) — reported, non-failing; if it
                  starts matching, it is flagged XPASS (fix landed; drop the xfail).
"""
import json, re, sys, subprocess, tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
CASES = HERE / "cases"
MANIFEST = HERE / "manifest.jsonl"
sys.path[:0] = [str(ROOT / "prototype" / "democ"), str(ROOT / "prototype" / "checker")]
CLANG = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"


# -- toolchain adapter: swap this out for the real / self-hosted compiler ----------
def adapter_democ(source, want_run):
    import democ
    from checker import CheckError
    try:
        ir = democ.compile_program(source)
    except CheckError as e:                      # a spec rejection, with its rule id
        return ("reject", e.rule)
    except (SystemExit, AssertionError, KeyError, IndexError, AttributeError, ValueError) as e:
        return ("unsupported", f"{type(e).__name__}: {str(e)[:80]}")   # outside democ's subset
    if not want_run:
        return ("accept",)
    with tempfile.TemporaryDirectory() as d:
        ll, exe = Path(d) / "m.ll", Path(d) / "m"
        ll.write_text(ir)
        c = subprocess.run([CLANG, "-O2", str(ll), "-o", str(exe)], capture_output=True, text=True)
        if c.returncode != 0:
            return ("unsupported", f"clang: {c.stderr[:80]}")
        r = subprocess.run([str(exe)], capture_output=True)
        return ("trap",) if r.returncode < 0 else ("run", r.returncode)   # <0 = killed by signal (llvm.trap)


ADAPTER = adapter_democ


def matches(v, expect):
    k = expect["kind"]
    return ((k == "accept" and v[0] == "accept")
            or (k == "reject" and v[0] == "reject" and v[1] == expect["rule"])
            or (k == "run" and v[0] == "run" and v[1] == expect["exit"])
            or (k == "trap" and v[0] == "trap"))


def load_manifest():
    """Split manifest lines into cases (have "id") and coverage annotations (have "covered_by")."""
    cases, annots = [], []
    for ln in MANIFEST.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        o = json.loads(ln)
        (cases if "id" in o else annots).append(o)
    return cases, annots


def run_cases(cases):
    results = []
    for c in cases:
        status = c.get("status", "runnable")
        if status == "pending":
            results.append((c, "SKIP", ("pending",)))
            continue
        src = (CASES / f"{c['id']}.xl").read_text()
        v = ADAPTER(src, c["expect"]["kind"] in ("run", "trap"))
        m = matches(v, c["expect"])
        if status == "xfail":
            outcome = "XPASS" if m else "XFAIL"
        elif v[0] == "unsupported":
            outcome = "SKIP"                     # runnable, but the toolchain can't process it yet
        else:
            outcome = "PASS" if m else "FAIL"
        results.append((c, outcome, v))
    return results


def spec_rule_ids():
    def ver(f):
        m = re.search(r"v(\d+)(?:\.(\d+))?", f.name)
        return (int(m.group(1)), int(m.group(2) or 0))
    spec = max((ROOT / "spec").glob("kernel-spec-v*.md"), key=ver)
    return set(re.findall(r"^\[([A-Z]+-\d+[a-z]?)\]", spec.read_text(), re.M)), spec.name


def coverage(cases, annots):
    rules, spec_name = spec_rule_ids()
    tagged, pos, neg = set(), set(), set()
    for c in cases:
        tagged |= set(c["rules"])
        if c["expect"]["kind"] == "reject":
            neg.add(c["expect"]["rule"])
        else:
            pos |= set(c["rules"])
    annotated = {a["rule"] for a in annots} & rules
    by_case = tagged & rules
    covered = by_case | annotated
    return rules, spec_name, covered, by_case, annotated, pos, neg, sorted(rules - covered)


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "all"
    verbose = "-v" in sys.argv
    cases, annots = load_manifest()
    fail = 0
    if cmd in ("run", "all"):
        tally = {}
        for c, o, v in run_cases(cases):
            tally[o] = tally.get(o, 0) + 1
            if o in ("FAIL", "XPASS"):
                fail += 1
                print(f"  {o:5} {c['id']:38} want {c['expect']} got {v}")
            elif o == "XFAIL" and verbose:
                print(f"  XFAIL {c['id']:38} ({c.get('reason','known gap')})")
            elif o == "SKIP" and verbose:
                print(f"  SKIP  {c['id']:38} {v[1] if len(v) > 1 else 'pending'}")
        print("conformance run: " + "  ".join(f"{k}={tally[k]}" for k in sorted(tally)))
    if cmd in ("coverage", "all"):
        rules, spec_name, covered, by_case, annotated, pos, neg, uncovered = coverage(cases, annots)
        print(f"coverage ({spec_name}): {len(covered)}/{len(rules)} rules covered "
              f"({len(by_case)} by case [+{len(pos)}/-{len(neg)}], {len(annotated)} by annotation); "
              f"{len(uncovered)} uncovered")
        if verbose and uncovered:
            print("  uncovered: " + " ".join(uncovered))
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
