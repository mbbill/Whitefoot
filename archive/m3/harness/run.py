#!/usr/bin/env python3
"""M3 decision-sprint harness.

Runs Rust and xlang submissions for each task in m3/tasks.jsonl and emits one
JSON record per task/language pair. The harness intentionally does not call an
LLM; generated code is an input, not hidden inside the scorer.
"""

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
M3 = HERE.parent
ROOT = M3.parent
TASKS = M3 / "tasks.jsonl"
CLANG = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"

sys.path[:0] = [str(ROOT / "prototype" / "democ"),
                str(ROOT / "prototype" / "checker")]


def load_tasks():
    tasks = []
    for raw in TASKS.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        tasks.append(json.loads(line))
    return tasks


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_metadata(src):
    """Optional sidecar for generated submissions.

    Supported names:
      foo.rs.meta.json / foo.xl.meta.json
      foo.meta.json
    """
    candidates = [src.with_suffix(src.suffix + ".meta.json"),
                  src.with_suffix(".meta.json")]
    for path in candidates:
        if path.exists():
            try:
                return json.loads(path.read_text()), None
            except json.JSONDecodeError as e:
                return None, f"{path}: {e}"
    return None, None


def submission_sources(task, suite, language, trials=None):
    """Return [(trial_name, source_path)].

    Backward-compatible single-file layout:
      submissions/<suite>/<language>/<task>.{rs,xl}

    Multi-trial model layout:
      submissions/<suite>/<language>/<task>/<trial>.{rs,xl}
    """
    ext = {"rust": "rs", "xlang": "xl"}[language]
    root = M3 / "submissions" / suite / language
    legacy = root / f"{task['id']}.{ext}"
    trial_dir = root / task["id"]
    selected = set(trials or [])

    out = []
    if trial_dir.is_dir():
        for src in sorted(trial_dir.glob(f"*.{ext}")):
            if selected and src.stem not in selected:
                continue
            out.append((src.stem, src))
    if legacy.exists() and (not selected or "reference" in selected or "default" in selected):
        out.append(("reference" if suite == "reference" else "default", legacy))
    if out:
        return out
    if selected:
        return [(trial, trial_dir / f"{trial}.{ext}") for trial in sorted(selected)]
    return [(None, legacy)]


def run_cmd(argv, timeout_sec, cwd=ROOT):
    t0 = time.perf_counter()
    try:
        p = subprocess.run(argv, cwd=str(cwd), capture_output=True, text=True,
                           timeout=timeout_sec)
        return {
            "argv": [str(a) for a in argv],
            "returncode": p.returncode,
            "stdout": p.stdout[:4000],
            "stderr": p.stderr[:4000],
            "ms": round((time.perf_counter() - t0) * 1000.0, 3),
        }
    except subprocess.TimeoutExpired as e:
        return {
            "argv": [str(a) for a in argv],
            "returncode": None,
            "stdout": (e.stdout or "")[:4000],
            "stderr": (e.stderr or "")[:4000],
            "ms": round((time.perf_counter() - t0) * 1000.0, 3),
            "timeout": True,
        }


def verdict_from_runs(task, runs, language=None):
    expected = task.get("expected", {})
    if language is not None:
        expected = task.get(f"expected_{language}", expected)
    if not runs:
        return False, "no-run"
    last = runs[-1]
    if "exit" in expected and last["returncode"] != expected["exit"]:
        return False, f"exit {last['returncode']} != {expected['exit']}"
    if "stdout" in expected and last["stdout"].strip() != expected["stdout"]:
        return False, f"stdout {last['stdout'].strip()!r} != {expected['stdout']!r}"
    return True, "ok"


def compile_xlang(src, task, tmp):
    import democ
    from checker import CheckError

    ll = tmp / "program.ll"
    exe = tmp / "program"
    try:
        ir = democ.compile_program(src.read_text())
    except CheckError as e:
        return None, {
            "status": "reject",
            "rule": e.rule,
            "stderr": str(e)[:4000],
            "ms": 0.0,
        }
    except (AssertionError, SystemExit, KeyError, IndexError, AttributeError, ValueError) as e:
        return None, {
            "status": "unsupported",
            "stderr": f"{type(e).__name__}: {str(e)[:4000]}",
            "ms": 0.0,
        }
    ll.write_text(ir)
    argv = [CLANG, "-O2", str(ll), "-o", str(exe)]
    if task["kind"] == "ffi_accumulate":
        argv = [CLANG, "-O2", str(ll), str(ROOT / task["driver_c"]),
                f"-DNITER={task['niter']}", "-o", str(exe)]
    result = run_cmd(argv, task["timeout_sec"])
    result["status"] = "ok" if result["returncode"] == 0 else "compile-fail"
    return exe if result["returncode"] == 0 else None, result


def compile_rust(src, task, tmp):
    exe = tmp / "program"
    if task["kind"] == "ffi_accumulate":
        obj = tmp / "program.o"
        c1 = run_cmd(["rustc", "-C", "opt-level=3", "--emit", "obj",
                      str(src), "-o", str(obj), "--crate-type", "lib"],
                     task["timeout_sec"])
        if c1["returncode"] != 0:
            c1["status"] = "compile-fail"
            return None, c1
        c2 = run_cmd([CLANG, "-O2", str(obj), str(ROOT / task["driver_c"]),
                      f"-DNITER={task['niter']}", "-o", str(exe)],
                     task["timeout_sec"])
        c2["status"] = "ok" if c2["returncode"] == 0 else "compile-fail"
        c2["rustc_ms"] = c1["ms"]
        c2["link_ms"] = c2["ms"]
        c2["ms"] = round(c1["ms"] + c2["ms"], 3)
        return exe if c2["returncode"] == 0 else None, c2
    result = run_cmd(["rustc", "-C", "opt-level=3", str(src), "-o", str(exe)],
                     task["timeout_sec"])
    result["status"] = "ok" if result["returncode"] == 0 else "compile-fail"
    return exe if result["returncode"] == 0 else None, result


def run_one(task, suite, language, trial, src):
    record = {
        "task": task["id"],
        "language": language,
        "suite": suite,
        "trial": trial,
        "kind": task["kind"],
        "source": str(src.relative_to(ROOT)) if src.exists() else str(src),
        "exists": src.exists(),
    }
    pending_languages = task.get("pending_languages", {})
    if language in pending_languages:
        record.update({
            "verdict": "pending",
            "pass": None,
            "reason": pending_languages[language],
        })
        return record
    if not src.exists():
        record.update({"verdict": "missing", "pass": False})
        return record
    record["source_sha256"] = sha256(src)
    metadata, metadata_error = read_metadata(src)
    if metadata is not None:
        record["metadata"] = metadata
    if metadata_error is not None:
        record["metadata_error"] = metadata_error
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        if language == "rust":
            exe, compile_result = compile_rust(src, task, tmp)
        else:
            exe, compile_result = compile_xlang(src, task, tmp)
        record["compile"] = compile_result
        if exe is None:
            record.update({"verdict": compile_result["status"], "pass": False})
            return record
        runs = []
        for _ in range(task.get("runs", 1)):
            runs.append(run_cmd([str(exe)], task["timeout_sec"]))
        record["runs"] = runs
        ok, reason = verdict_from_runs(task, runs, language)
        record["pass"] = ok
        record["verdict"] = "pass" if ok else "fail"
        record["reason"] = reason
        record["run_ms"] = [r["ms"] for r in runs]
        if runs:
            ordered = sorted(record["run_ms"])
            record["median_run_ms"] = ordered[len(ordered) // 2]
        return record


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", default="reference")
    ap.add_argument("--language", choices=["rust", "xlang"], action="append")
    ap.add_argument("--task", action="append")
    ap.add_argument("--trial", action="append",
                    help="Run one named trial from submissions/<suite>/<language>/<task>/<trial>.{rs,xl}")
    ap.add_argument("--out")
    args = ap.parse_args()

    languages = args.language or ["rust", "xlang"]
    want_tasks = set(args.task or [])
    records = []
    for task in load_tasks():
        if want_tasks and task["id"] not in want_tasks:
            continue
        for language in languages:
            for trial, src in submission_sources(task, args.suite, language, args.trial):
                records.append(run_one(task, args.suite, language, trial, src))

    lines = [json.dumps(r, sort_keys=True) for r in records]
    if args.out:
        Path(args.out).write_text("\n".join(lines) + "\n")
    for line in lines:
        print(line)

    failed = [r for r in records if r.get("pass") is False]
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
