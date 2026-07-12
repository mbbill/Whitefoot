#!/usr/bin/env python3
"""M3 Phase-C trial runner: generation + fixed repair budget.

Drives one model tier over the task set. The model is an EXTERNAL command
(--gen-cmd), invoked once per attempt with the full prompt on stdin and
expected to print the program (a single fenced code block, or raw source).
Feedback between attempts is machine output only: the compiler diagnostic or
the failing run record. Nothing here tunes prompts per language beyond the
fixed per-language excerpt file (DECISION_SPRINT.md protocol).

Usage:
  python3 m3/harness/trial.py --language xlang --tier weak \
      --gen-cmd 'your-model-cli --flags' [--task NAME]... \
      [--trials 3] [--repairs 3] --out /path/results.jsonl

Records one JSON line per attempt and one summary line per trial.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run as harness                     # compile/verdict machinery, task table

M3 = HERE.parent
EXCERPT = {
    "xlang": M3 / "prompts" / "xlang-spec-excerpt.md",
    "rust": M3 / "prompts" / "rust-guardrails.md",
}
EXT = {"xlang": "xl", "rust": "rs"}

FENCE = re.compile(r"```[a-zA-Z]*\n(.*?)```", re.S)


def extract_source(text):
    """The last fenced block if any, else the raw text."""
    blocks = FENCE.findall(text)
    return (blocks[-1] if blocks else text).strip() + "\n"


def build_prompt(task, language, prompt_md, excerpt_md):
    return (
        f"# Language: {language}\n\n{excerpt_md}\n\n---\n\n{prompt_md}\n\n"
        "Output the complete program and nothing else, in a single fenced "
        "code block. Do not explain."
    )


def repair_prompt(prev_source, machine_output):
    return (
        "Your previous program:\n\n```\n" + prev_source + "```\n\n"
        "The toolchain reported:\n\n```\n" + machine_output[:4000] + "\n```\n\n"
        "Output the corrected complete program and nothing else, in a single "
        "fenced code block. Do not explain."
    )


def generate(gen_cmd, prompt, timeout):
    p = subprocess.run(gen_cmd, shell=True, input=prompt, text=True,
                       capture_output=True, timeout=timeout)
    if p.returncode != 0:
        raise RuntimeError(f"gen-cmd failed ({p.returncode}): {p.stderr[:400]}")
    return p.stdout


def attempt(task, language, source_path):
    """Compile + run one submission; returns (passed, machine_output, record)."""
    rec = {}
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        if language == "rust":
            exe, cres = harness.compile_rust(source_path, task, tmp)
        else:
            exe, cres = harness.compile_xlang(source_path, task, tmp)
        rec["compile"] = cres
        if exe is None:
            return False, cres.get("stderr", "") or cres.get("status", "compile failed"), rec
        runs = [harness.run_cmd([str(exe)], task["timeout_sec"])
                for _ in range(task.get("runs", 1))]
        rec["runs"] = runs
        ok, reason = harness.verdict_from_runs(task, runs, language)
        rec["verdict"] = "pass" if ok else "fail"
        rec["reason"] = reason
        if ok:
            return True, "", rec
        last = runs[-1]
        fb = (f"verdict: {reason}\nexit code: {last['returncode']}\n"
              f"stdout: {last['stdout'][:500]}\nstderr: {last['stderr'][:1500]}")
        return False, fb, rec


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--language", required=True, choices=("xlang", "rust"))
    ap.add_argument("--tier", required=True)
    ap.add_argument("--gen-cmd", required=True,
                    help="shell command; prompt on stdin, program on stdout")
    ap.add_argument("--task", action="append", default=None)
    ap.add_argument("--trials", type=int, default=3)
    ap.add_argument("--repairs", type=int, default=3)
    ap.add_argument("--gen-timeout", type=int, default=600)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    excerpt = EXCERPT[args.language].read_text()
    tasks = [t for t in harness.load_tasks()
             if (args.task is None or t["id"] in args.task)
             and args.language not in t.get("pending_languages", {})]
    outdir = M3 / "submissions" / "generated" / args.tier / args.language
    outdir.mkdir(parents=True, exist_ok=True)
    outf = open(args.out, "a")

    for task in tasks:
        prompt_md = (M3 / task["prompt"]).read_text()
        for trial in range(args.trials):
            base = build_prompt(task, args.language, prompt_md, excerpt)
            prompt, source = base, None
            first_shot = None
            for k in range(args.repairs + 1):
                t0 = time.time()
                try:
                    reply = generate(args.gen_cmd, prompt, args.gen_timeout)
                except Exception as e:
                    outf.write(json.dumps({"task": task["id"], "tier": args.tier,
                                           "language": args.language, "trial": trial,
                                           "attempt": k, "error": str(e)}) + "\n")
                    outf.flush()
                    break
                source = extract_source(reply)
                sp = outdir / f"{task['id']}-t{trial}-a{k}.{EXT[args.language]}"
                sp.write_text(source)
                ok, fb, rec = attempt(task, args.language, sp)
                rec.update({"task": task["id"], "tier": args.tier,
                            "language": args.language, "trial": trial,
                            "attempt": k, "source": str(sp),
                            "source_bytes": len(source),
                            "gen_seconds": round(time.time() - t0, 1),
                            "pass": ok})
                outf.write(json.dumps(rec) + "\n")
                outf.flush()
                if first_shot is None:
                    first_shot = ok
                if ok:
                    break
                prompt = base + "\n\n---\n\n" + repair_prompt(source, fb)
            outf.write(json.dumps({"summary": True, "task": task["id"],
                                   "tier": args.tier, "language": args.language,
                                   "trial": trial, "first_shot": bool(first_shot),
                                   "passed": ok, "repairs_used": k}) + "\n")
            outf.flush()
            print(f"{args.tier}/{args.language}/{task['id']} trial {trial}: "
                  f"{'PASS' if ok else 'FAIL'} (first_shot={first_shot}, repairs={k})")
    outf.close()


if __name__ == "__main__":
    main()
