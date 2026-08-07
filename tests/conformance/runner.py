#!/usr/bin/env python3
"""whitefoot conformance test system — spec-anchored, rule-keyed, toolchain-agnostic.

Each case is a canonical `.wf` source (tests/conformance/cases/<id>.wf) plus a
manifest entry (tests/conformance/manifest.jsonl) declaring the rule id(s) it
exercises and the expected verdict. Cases are driven through a named toolchain
adapter. The active adapter is native, not Python: `compiler/tests/conformance.rs`
compiles each case through the ordinary compiler path, realizes the ARRANGE
below as a real invocation, and reduces the outcome to one verdict below
(`make conformance-run`). This file stays on the other side of that boundary —
identity, corpus structure, declared rule coverage, and schema validity — so
the corpus keeps outliving any one compiler and no compiler behaviour is
reimplemented here. `ADAPTER` below is the hook for a future non-native
toolchain; with none installed, a Python-side semantic run fails explicitly
rather than reporting an empty success.

Because it tests the LANGUAGE (source -> verdict), not a compiler's internals, this
suite outlives compiler implementations.

Verdict = ("accept",) | ("reject", rule) | ("run", exit) | ("trap",) | ("unsupported", why)

Manifest line (JSON):
  {"id": str, "rules": [rule_id...], "expect": EXPECT, "arrange": ARRANGE?,
   "status": "runnable"|"pending"|"xfail", "reason": str?, "doc": str}
  EXPECT = {"kind":"accept"} | {"kind":"reject","rule":R} | {"kind":"run","exit":N}
         | {"kind":"trap"} | {"kind":"unsupported","why":str}

  status: runnable = must match expect;  pending = toolchain can't run it yet (skip);
          xfail = expect is the CORRECT spec behavior but the current toolchain does
                  not yet produce it (a tracked gap) — reported, non-failing; if it
                  starts matching, it is flagged XPASS (fix landed; drop the xfail).

`expect` is the verdict the SPECIFICATION requires; `status` is the separate
toolchain-readiness axis. An unimplemented compiler capability is a `status`
fact and never rewrites `expect`.

The `unsupported` expectation is for the stops the specification itself fixes
as non-rejections citing no language rule — target qualification failure and
pre-entry startup refusal ([QUAL-1], [QUAL-2], [PROG-3]). It is not a place to
record that this compiler has not implemented something yet.

ARRANGE describes the invocation a `run`/`trap` case needs; a case that is
never executed must not carry one. Every byte string is lowercase hex so the
corpus can express non-UTF-8 argument and path bytes exactly and diff them
readably:

  ARRANGE = {"argv": ["<hex>"...]?,          the complete native argument vector
             "stdin": "<hex>"?,              standard input body
             "files": [FIXTURE...]?,         fixtures under the initial directory
             "redirect": {"stdout": NAME?, "stderr": NAME?}?}
  FIXTURE = {"path":"<hex>", "bytes":"<hex>"} | {"path":"<hex>", "directory":true}

`argv` is the COMPLETE native argument vector, position 0 included, exactly as
the language's own argument value delivers it: `argv[i]` is what the program
reads at position i, and the vector's length is the count the program reads.
This is the only lossless reading. `Args` carries the complete native vector
[HOST-1, SYS-9], so a schema that listed only the arguments after the invoked
name would leave the vector's first element — an element a program can count
and read — unstated by the case that claims to fix its invocation. Position 0
is therefore a case's own datum rather than the harness's incidental choice of
where it built the executable, and an adapter sets it explicitly. An absent
`argv` means the adapter's own default vector, which supplies position 0 and
nothing after it.

A redirection NAME is an opaque sink label: two entries naming the same sink
are the same destination, which is how "stdout and stderr redirected to one
target" is stated. An adapter realizes one sink label as one object under the
initial directory, so a case may also name that label as a path and read the
combined bytes back. Absent keys mean the default arrangement — the default
argument vector, an empty standard input, no fixtures, and separate sinks.
"""
import hashlib, json, re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
CASES = HERE / "cases"
MANIFEST = HERE / "manifest.jsonl"
ACTIVE_SPEC = Path("spec/kernel-spec-v0.20.md")
ACTIVE_SPEC_SHA256 = "b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1"
# The named native adapter is compiler/tests/conformance.rs, reached through
# `make conformance-run`; this hook stays open for a future non-native
# toolchain. Keeping it explicit prevents a missing compiler, crash, or broad
# exception from becoming `Unsupported`.
ADAPTER = None


def matches(v, expect):
    k = expect["kind"]
    return ((k == "accept" and v[0] == "accept")
            or (k == "reject" and v[0] == "reject" and v[1] == expect["rule"])
            or (k == "run" and v[0] == "run" and v[1] == expect["exit"])
            or (k == "trap" and v[0] == "trap")
            # `why` is the adapter's prose; the corpus asserts the verdict kind,
            # never a diagnostic's wording.
            or (k == "unsupported" and v[0] == "unsupported"))


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
        elif v[0] == "unsupported" and c["expect"]["kind"] != "unsupported":
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


HEX = re.compile(r"\A(?:[0-9a-f]{2})*\Z")


def hex_errors(label, value, allow_empty=True):
    """A byte string is lowercase hex so the corpus states non-UTF-8 bytes exactly."""
    if not isinstance(value, str) or not HEX.match(value):
        return [f"{label} must be a lowercase even-length hex byte string"]
    if not allow_empty and not value:
        return [f"{label} must not be empty"]
    return []


def arrange_errors(label, arrange):
    """Check one invocation arrangement. Keys are a closed set: an unknown key is a
    silently ignored intention, which is worse in a corpus than a rejected line."""
    if not isinstance(arrange, dict):
        return [f"{label}: arrange must be an object"]
    errors = []
    unknown = sorted(set(arrange) - {"argv", "stdin", "files", "redirect"})
    if unknown:
        errors.append(f"{label}: unknown arrange keys: {' '.join(unknown)}")

    if "argv" in arrange:
        argv = arrange["argv"]
        if not isinstance(argv, list) or not argv:
            errors.append(f"{label}: argv must be a nonempty list")
        else:
            for position, argument in enumerate(argv):
                errors += hex_errors(f"{label}: argv[{position}]", argument)

    if "stdin" in arrange:
        errors += hex_errors(f"{label}: stdin", arrange["stdin"])

    if "files" in arrange:
        files = arrange["files"]
        if not isinstance(files, list) or not files:
            errors.append(f"{label}: files must be a nonempty list")
        else:
            paths = []
            for position, fixture in enumerate(files):
                where = f"{label}: files[{position}]"
                if not isinstance(fixture, dict):
                    errors.append(f"{where} must be an object")
                    continue
                unknown = sorted(set(fixture) - {"path", "bytes", "directory"})
                if unknown:
                    errors.append(f"{where}: unknown fixture keys: {' '.join(unknown)}")
                errors += hex_errors(f"{where}.path", fixture.get("path"), allow_empty=False)
                paths.append(fixture.get("path"))
                is_file, is_directory = "bytes" in fixture, "directory" in fixture
                if is_file == is_directory:
                    errors.append(f"{where} must carry exactly one of bytes, directory")
                elif is_file:
                    errors += hex_errors(f"{where}.bytes", fixture["bytes"])
                elif fixture["directory"] is not True:
                    errors.append(f"{where}.directory must be true")
            duplicate = sorted({path for path in paths if paths.count(path) > 1})
            if duplicate:
                errors.append(f"{label}: duplicate fixture paths: {' '.join(duplicate)}")

    if "redirect" in arrange:
        redirect = arrange["redirect"]
        if not isinstance(redirect, dict) or not redirect:
            errors.append(f"{label}: redirect must be a nonempty object")
        else:
            unknown = sorted(set(redirect) - {"stdout", "stderr"})
            if unknown:
                errors.append(f"{label}: unknown redirect streams: {' '.join(unknown)}")
            for stream, sink in redirect.items():
                if not isinstance(sink, str) or not sink:
                    errors.append(f"{label}: redirect {stream} needs a nonempty sink name")

    return errors


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
        "unsupported": {"kind", "why"},
    }
    executed_kinds = {"run", "trap"}
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
        elif kind == "unsupported" and not expect.get("why"):
            errors.append(f"{case_id}: unsupported expectation requires a why")

        if "arrange" in case:
            if kind not in executed_kinds:
                errors.append(f"{case_id}: only a run or trap case may carry an arrange")
            else:
                errors += arrange_errors(case_id, case["arrange"])

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
        kind = c["expect"]["kind"]
        # Coverage measures the corpus against the specification, so every case
        # counts whatever its toolchain-readiness status is. An `unsupported`
        # expectation is a qualification stop: neither a positive exercise of
        # the rule nor a rejection citing it.
        if kind == "reject":
            neg.add(c["expect"]["rule"])
        elif kind != "unsupported":
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
