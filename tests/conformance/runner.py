#!/usr/bin/env python3
"""whitefoot conformance test system — spec-anchored, rule-keyed, toolchain-agnostic.

Each case is a canonical `.wf` source (tests/conformance/cases/<id>.wf) plus a
manifest entry (tests/conformance/manifest.jsonl) declaring the rule id(s) it
exercises and the expected verdict. Cases are driven through a named toolchain
adapter. No active adapter exists yet; identity, corpus structure, declared rule
coverage, and expectations remain checkable, while a semantic run fails
explicitly.

Because it tests the LANGUAGE (source -> verdict), not a compiler's internals, this
suite outlives compiler implementations.

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
import hashlib, json, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CASES = HERE / "cases"
MANIFEST = HERE / "manifest.jsonl"
ACTIVE_SPEC = Path("spec/kernel-spec-v0.14.md")
ACTIVE_SPEC_SHA256 = "31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f"
# A later entrance-gated integration may install a named Rust adapter. Keeping
# this explicit prevents a missing compiler, crash, or broad exception from
# becoming `Unsupported`.
ADAPTER = None


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
    if ADAPTER is None:
        raise RuntimeError(
            "no active compiler adapter; use `coverage` until an entrance-gated "
            "integration installs one"
        )
    results = []
    for c in cases:
        status = c.get("status", "runnable")
        if status == "pending":
            results.append((c, "SKIP", ("pending",)))
            continue
        src = (CASES / f"{c['id']}.wf").read_text()
        v = ADAPTER(src, c["expect"]["kind"] in ("run", "trap"))
        m = matches(v, c["expect"])
        if status == "xfail":
            outcome = "XPASS" if m else "XFAIL"
        elif v[0] == "unsupported":
            outcome = "FAIL"                     # runnable means supported; gaps belong in pending
        else:
            outcome = "PASS" if m else "FAIL"
        results.append((c, outcome, v))
    return results


def spec_rule_ids(root=ROOT):
    spec = root / ACTIVE_SPEC
    raw = spec.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != ACTIVE_SPEC_SHA256:
        raise ValueError(f"active specification digest mismatch: {digest}")
    text = raw.decode("utf-8")
    return set(re.findall(r"^\[([A-Z]+-\d+[a-z]?)\]", text, re.M)), spec.name


def validate_manifest(cases, annots, root=ROOT, cases_dir=CASES):
    """Reject malformed or stale corpus structure before reporting coverage."""
    rules, _ = spec_rule_ids(root)
    errors = []

    ids = [case.get("id") for case in cases]
    duplicate_ids = sorted({case_id for case_id in ids if ids.count(case_id) > 1})
    if duplicate_ids:
        errors.append("duplicate case ids: " + " ".join(duplicate_ids))

    expected_sources = {f"{case_id}.wf" for case_id in ids if isinstance(case_id, str)}
    actual_sources = {path.name for path in cases_dir.glob("*.wf")}
    missing_sources = sorted(expected_sources - actual_sources)
    orphan_sources = sorted(actual_sources - expected_sources)
    if missing_sources:
        errors.append("missing case sources: " + " ".join(missing_sources))
    if orphan_sources:
        errors.append("orphan case sources: " + " ".join(orphan_sources))

    valid_statuses = {"runnable", "pending", "xfail"}
    expectation_fields = {
        "accept": {"kind"},
        "reject": {"kind", "rule"},
        "run": {"exit", "kind"},
        "trap": {"kind"},
    }
    for case in cases:
        case_id = case.get("id", "<missing-id>")
        case_rules = case.get("rules")
        if not isinstance(case_rules, list) or not case_rules:
            errors.append(f"{case_id}: rules must be a nonempty list")
            case_rules = []
        unknown = sorted(set(case_rules) - rules)
        if unknown:
            errors.append(f"{case_id}: unknown rules: {' '.join(unknown)}")

        status = case.get("status", "runnable")
        if status not in valid_statuses:
            errors.append(f"{case_id}: invalid status {status!r}")
        if status in {"pending", "xfail"} and not case.get("reason"):
            errors.append(f"{case_id}: {status} case requires a reason")

        expect = case.get("expect")
        kind = expect.get("kind") if isinstance(expect, dict) else None
        if kind not in expectation_fields:
            errors.append(f"{case_id}: invalid expectation {kind!r}")
        elif set(expect) != expectation_fields[kind]:
            fields = " ".join(sorted(expectation_fields[kind]))
            errors.append(
                f"{case_id}: {kind} expectation fields must be exactly: {fields}"
            )
        elif kind == "reject":
            reject_rule = expect.get("rule")
            if reject_rule not in case_rules:
                errors.append(f"{case_id}: reject rule must appear in case rules")
        elif kind == "run" and type(expect.get("exit")) is not int:
            errors.append(f"{case_id}: run expectation requires an integer exit")

        if not case.get("doc"):
            errors.append(f"{case_id}: missing documentation")

    annotation_rules = [annotation.get("rule") for annotation in annots]
    duplicate_annotations = sorted(
        {rule for rule in annotation_rules if annotation_rules.count(rule) > 1}
    )
    if duplicate_annotations:
        errors.append("duplicate annotations: " + " ".join(duplicate_annotations))
    for annotation in annots:
        rule = annotation.get("rule", "<missing-rule>")
        if rule not in rules:
            errors.append(f"annotation {rule}: unknown rule")
        if not annotation.get("covered_by"):
            errors.append(f"annotation {rule}: missing covered_by")
        if not annotation.get("reason"):
            errors.append(f"annotation {rule}: missing reason")

    if errors:
        raise ValueError("invalid conformance manifest:\n  " + "\n  ".join(errors))


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
    try:
        validate_manifest(cases, annots)
    except ValueError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1) from error
    fail = 0
    if cmd in ("run", "all"):
        if ADAPTER is None:
            print(
                "conformance run: UNAVAILABLE — no active compiler adapter",
                file=sys.stderr,
            )
            fail += 1
            if cmd == "run":
                raise SystemExit(fail)
        else:
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
        if uncovered:
            fail += 1
        if verbose and uncovered:
            print("  uncovered: " + " ".join(uncovered))
    sys.exit(1 if fail else 0)


if __name__ == "__main__":
    main()
