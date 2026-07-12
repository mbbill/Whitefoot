#!/usr/bin/env python3
"""Summarize M3 harness results against decision-readiness requirements."""

import argparse
import json
import statistics
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
M3 = HERE.parent
TASKS = M3 / "tasks.jsonl"


def load_tasks():
    tasks = {}
    for raw in TASKS.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        task = json.loads(line)
        tasks[task["id"]] = task
    return tasks


def load_records(paths):
    out = []
    for path in paths:
        for raw in Path(path).read_text().splitlines():
            line = raw.strip()
            if line:
                out.append(json.loads(line))
    return out


def median(values):
    return statistics.median(values) if values else None


def summarize(records):
    by_suite_lang = {}
    for r in records:
        key = (r["suite"], r["language"])
        by_suite_lang.setdefault(key, []).append(r)
    rows = []
    for (suite, language), items in sorted(by_suite_lang.items()):
        runnable = [r for r in items if r.get("pass") is not None]
        passed = [r for r in runnable if r.get("pass") is True]
        failed = [r for r in runnable if r.get("pass") is False]
        pending = [r for r in items if r.get("verdict") == "pending"]
        trials = {r.get("trial") for r in runnable if r.get("trial") is not None}
        med = median([r.get("median_run_ms") for r in passed
                      if isinstance(r.get("median_run_ms"), (int, float))])
        rows.append({
            "suite": suite,
            "language": language,
            "total_records": len(items),
            "runnable": len(runnable),
            "passed": len(passed),
            "failed": len(failed),
            "pending": len(pending),
            "trial_count": len(trials),
            "correct_rate": (len(passed) / len(runnable)) if runnable else None,
            "median_run_ms": med,
        })
    return rows


def decision_readiness(records, tasks, required_suites, min_trials_per_task):
    issues = []
    pending = []
    for t in tasks.values():
        for language, reason in t.get("pending_languages", {}).items():
            pending.append({"id": t["id"], "language": language, "reason": reason})
    if pending:
        issues.append({
            "kind": "pending_language_tasks",
            "count": len(pending),
            "tasks": pending,
        })
    for suite in required_suites:
        for language in ("rust", "xlang"):
            seen = {(r["task"], r["language"], r["suite"]) for r in records}
            missing = [t["id"] for t in tasks.values()
                       if language not in t.get("pending_languages", {})
                       if (t["id"], language, suite) not in seen]
            if missing:
                issues.append({
                    "kind": "missing_required_records",
                    "suite": suite,
                    "language": language,
                    "tasks": missing,
                })
            if min_trials_per_task:
                missing_ids = set(missing)
                insufficient = []
                for t in tasks.values():
                    if language in t.get("pending_languages", {}):
                        continue
                    if t["id"] in missing_ids:
                        continue
                    trials = {r.get("trial") for r in records
                              if r["suite"] == suite
                              and r["language"] == language
                              and r["task"] == t["id"]
                              and r.get("pass") is not None}
                    if len(trials) < min_trials_per_task:
                        insufficient.append({
                            "task": t["id"],
                            "have": len(trials),
                            "need": min_trials_per_task,
                        })
                if insufficient:
                    issues.append({
                        "kind": "insufficient_required_trials",
                        "suite": suite,
                        "language": language,
                        "tasks": insufficient,
                    })
    for suite in required_suites:
        for language in ("rust", "xlang"):
            failures = [r for r in records
                        if r["suite"] == suite
                        and r["language"] == language
                        and r.get("pass") is False]
            if failures:
                issues.append({
                    "kind": "failed_required_records",
                    "suite": suite,
                    "language": language,
                    "tasks": [r["task"] for r in failures],
                })
    return {
        "decision_ready": not issues,
        "issues": issues,
    }


def recommendation(readiness):
    kinds = {i["kind"] for i in readiness["issues"]}
    if "pending_language_tasks" in kinds:
        return {
            "status": "hold_self_hosting",
            "next": "implement_or_explicitly_abandon_the_minimum_xlang_M3_unblocks",
        }
    if "missing_required_records" in kinds or "insufficient_required_trials" in kinds:
        return {
            "status": "not_decision_ready",
            "next": "run_the_required_model_tier_trials_with_fixed_prompt_and_repair_budgets",
        }
    if "failed_required_records" in kinds:
        return {
            "status": "evidence_contains_failures",
            "next": "apply_DECISION_SPRINT_pass_fail_criteria_to_the_full_result_set",
        }
    return {
        "status": "decision_evidence_complete",
        "next": "apply_DECISION_SPRINT_pass_fail_criteria",
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results", nargs="+")
    ap.add_argument("--required-suite", action="append", default=[])
    ap.add_argument("--min-trials-per-task", type=int, default=0,
                    help="For required suites, require at least this many runnable trials per task/language")
    ap.add_argument("--require-decision-ready", action="store_true")
    args = ap.parse_args()

    tasks = load_tasks()
    records = load_records(args.results)
    readiness = decision_readiness(records, tasks, args.required_suite,
                                   args.min_trials_per_task)
    report = {
        "summary": summarize(records),
        "readiness": readiness,
        "recommendation": recommendation(readiness),
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    if args.require_decision_ready and not report["readiness"]["decision_ready"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
