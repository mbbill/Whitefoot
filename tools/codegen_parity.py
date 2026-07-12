#!/usr/bin/env python3
"""Deterministic code-generation parity gate for xlang.

The runner compiles every variant in a temporary directory, extracts stable
IR/assembly properties, and evaluates the relations in codegen-parity.json.
Cases marked "audit" are reported but never make the command fail.
The dedicated --promotion mode additionally requires every separately pinned
checked-automation root and forbids partial-run filters.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import operator
import platform
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


sys.dont_write_bytecode = True  # keep parity runs from polluting the workspace with caches
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = ROOT / "codegen-parity.json"
CORPUS_ROOT = ROOT / "codegen-corpus/cases"
CLANG = Path("/usr/bin/clang") if Path("/usr/bin/clang").exists() else Path("clang")
VECTOR_REMARK = re.compile(
    r"vectorized loop \(vectorization width: (\d+), interleaved count: (\d+)\)"
)
OPS = {
    "eq": operator.eq,
    "ne": operator.ne,
    "lt": operator.lt,
    "le": operator.le,
    "gt": operator.gt,
    "ge": operator.ge,
}
NULL_METRIC_KEY = "<null>"
OBLIGATION_STATUSES = {"derived", "failed-premise", "not-applicable", "unknown"}
OBLIGATION_EXACTNESSES = {"exact", "sufficient", "unknown"}
REQUIREMENT_RELATIONS = {
    "equivalent", "missing", "mismatch", "unknown", "not-applicable",
}
FAILED_PREMISE_KEYS = {"reason", "expected", "observed", "path"}
CHECKED_AUTOMATION_SCOPE = "frontend-implicit-bounds-v1"
CHECKED_AUTOMATION_ANALYZERS = ("output-capacity-lockstep-v1",)
CHECKED_AUTOMATION_DISPOSITIONS = (
    "automatically-accounted",
    "intrinsic-dynamic",
    "hard-finding",
    "unaccounted",
)
REVIEW_PINNED_PROMOTION_ROOTS = (
    {
        "case": "base64-local-proof-accounting",
        "variant": "facts",
        "function": "encode",
        "source": "experiments/port-study/base64/b64.xl",
        "source_sha256": (
            "e3abbce3c8d1b24eba3471c5c0ea6800418d38c3d3d97ce64b7f15d2abfb9764"
        ),
        "scope": "closed-unit",
    },
)
REVIEW_PINNED_POLICY_ORACLE_COUNT = 15
REVIEW_PINNED_POLICY_ORACLE_SHA256 = (
    "c5e7640bb3c3e7c933d4b8c33d6ca84f9ab2492f03faedbad65ae1647b75ec57"
)


class HarnessError(RuntimeError):
    pass


def validate_promotion_authority(manifest: dict[str, Any]) -> None:
    """Require the separately code-pinned review root set for a promotion run.

    This dual repository pin prevents a manifest-only edit from deleting or
    repointing a root. Protected external review must still govern coordinated
    changes to both pins; this function cannot establish GATE-1 authority.
    """
    policy = manifest.get("checked_automation")
    if not isinstance(policy, dict):
        raise HarnessError("--promotion requires checked_automation roots")
    if policy.get("roots") != list(REVIEW_PINNED_PROMOTION_ROOTS):
        raise HarnessError("checked-automation roots differ from the review-pinned set")


def policy_oracle_descriptors() -> list[dict[str, Any]]:
    descriptors = []
    for manifest_path in sorted(CORPUS_ROOT.rglob("cases.json")):
        family = json.loads(manifest_path.read_text())
        for entry in family.get("cases", []):
            expected = entry.get("expected", {})
            if "checked_automation_ready" not in expected:
                continue
            source_path = (manifest_path.parent / entry["source"]).resolve()
            descriptors.append({
                "family": family["family"],
                "id": entry.get("id", source_path.stem),
                "source": str(source_path.relative_to(ROOT)),
                "source_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
                "maturity": entry["maturity"],
                "ready": expected["checked_automation_ready"],
                "disposition_counts": expected[
                    "checked_automation_disposition_counts"
                ],
            })
    return sorted(descriptors, key=lambda item: (item["family"], item["id"]))


def validate_policy_oracle_authority() -> None:
    descriptors = policy_oracle_descriptors()
    encoded = json.dumps(
        descriptors, sort_keys=True, separators=(",", ":")
    ).encode()
    digest = hashlib.sha256(encoded).hexdigest()
    if len(descriptors) != REVIEW_PINNED_POLICY_ORACLE_COUNT \
            or digest != REVIEW_PINNED_POLICY_ORACLE_SHA256:
        raise HarnessError("checked-automation policy oracles differ from the pinned set")


def validate_promotion_invocation(args: argparse.Namespace) -> None:
    if not args.promotion:
        return
    if args.manifest.resolve() != DEFAULT_MANIFEST.resolve():
        raise HarnessError("--promotion requires the default review-pinned manifest")
    if not args.corpus:
        raise HarnessError("--promotion requires --corpus policy oracles")
    if args.cases or args.tags or args.gate_only or args.audit_only:
        raise HarnessError("--promotion does not permit case, tag, or mode filters")


def result_exit_code(results: list[dict[str, Any]]) -> int:
    if any("error" in item for item in results):
        return 2
    return int(any(not item["passed"] and item["mode"] == "gate" for item in results))


def load_democ():
    path = ROOT / "prototype/democ/democ.py"
    spec = importlib.util.spec_from_file_location("xlang_democ", path)
    if spec is None or spec.loader is None:
        raise HarnessError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        rendered = " ".join(command)
        detail = result.stderr.strip() or result.stdout.strip()
        raise HarnessError(f"command failed ({rendered}):\n{detail}")
    return result


def llvm_function(text: str, function: str | None) -> str:
    if not function:
        return text
    start = re.compile(rf'^define\b.*@"?{re.escape(function)}"?\(', re.MULTILINE).search(text)
    if not start:
        raise HarnessError(f"LLVM function @{function} not found")
    pos = start.start()
    end = text.find("\n}", pos)
    if end < 0:
        raise HarnessError(f"unterminated LLVM function @{function}")
    return text[pos : end + 2]


def assembly_function(text: str, function: str | None) -> str:
    if not function:
        return text
    label = re.compile(rf"^_?{re.escape(function)}:\s*(?:[#;/].*)?$", re.MULTILINE)
    match = label.search(text)
    if not match:
        raise HarnessError(f"assembly function {function} not found")
    lines = text[match.start() :].splitlines()
    out = []
    for index, line in enumerate(lines):
        if index and re.match(r"\s*\.globl\s+", line):
            break
        out.append(line)
        if index and re.match(r"\s*\.size\s+_?" + re.escape(function) + r"\b", line):
            break
        if index and line.strip() == ".cfi_endproc":
            break
    return "\n".join(out)


def assembly_opcodes(body: str) -> list[str]:
    opcodes: list[str] = []
    for raw in body.splitlines():
        line = raw.split(";", 1)[0].split("//", 1)[0].strip()
        if not line or line.startswith((".", "#")) or line.endswith(":"):
            continue
        token = line.split(None, 1)[0]
        if re.match(r"^[A-Za-z][A-Za-z0-9_.]*$", token):
            opcodes.append(token.lower())
    return opcodes


def checked_automation_summary(bounds: list[dict[str, Any]]) -> dict[str, Any]:
    """Apply review B2 to frontend implicit-bounds sites only.

    This is a promotion policy, never a source-language acceptance rule.
    It grants no backend-elimination credit and has no source suppression or
    authorization path; unresolved accounting therefore fails closed.
    """
    counts = {name: 0 for name in CHECKED_AUTOMATION_DISPOSITIONS}
    findings: dict[str, dict[str, Any]] = {}
    for site in bounds:
        key = f'{site["function"]}:{site["site"]}'
        analysis = site["obligation_analysis"]
        analysis_complete = (
            analysis["scope"] == CHECKED_AUTOMATION_SCOPE
            and analysis["complete"] is True
            and analysis["analyzers"] == list(CHECKED_AUTOMATION_ANALYZERS)
        )
        if not analysis_complete:
            disposition, reason = "unaccounted", "incomplete-obligation-analysis"
        elif site["status"] == "proved":
            disposition, reason = "automatically-accounted", None
        elif site["status"] == "retained" \
                and site["obligation"] is None \
                and site["obligation_status"] == "not-applicable":
            disposition, reason = "intrinsic-dynamic", None
        elif site["status"] == "retained" \
                and site["obligation_status"] == "derived" \
                and site["requirement_relation"] == "missing":
            disposition, reason = "hard-finding", "missing-requirement"
        elif site["status"] == "retained" \
                and site["obligation_status"] == "derived" \
                and site["requirement_relation"] == "mismatch":
            disposition, reason = "hard-finding", "mismatched-requirement"
        elif site["status"] == "ceiling":
            disposition, reason = "unaccounted", "experiment-only-ceiling"
        elif site["obligation_status"] == "failed-premise":
            disposition, reason = "unaccounted", "failed-proof-premise"
        elif site["obligation_status"] == "unknown":
            disposition, reason = "unaccounted", "indeterminate-obligation"
        elif site["obligation_status"] == "derived" \
                and site["requirement_relation"] == "equivalent":
            disposition, reason = "unaccounted", "matched-obligation-retained"
        else:
            disposition, reason = "unaccounted", "unclassified-retained-site"
        counts[disposition] += 1
        if reason is not None:
            findings[key] = {
                "disposition": disposition,
                "reason": reason,
                "function": site["function"],
                "site": site["site"],
                "target": site["target"],
                "index": site["index"],
                "status": site["status"],
                "proof": site["proof"],
                "obligation_analysis": analysis,
                "obligation": site["obligation"],
                "obligation_status": site["obligation_status"],
                "requirement_relation": site["requirement_relation"],
                "first_missing_fact": site["first_missing_fact"],
                "first_failed_premise": site["first_failed_premise"],
            }
    return {
        "scope": CHECKED_AUTOMATION_SCOPE,
        "ready": not findings,
        "disposition_counts": counts,
        "findings_by_site": dict(sorted(findings.items())),
    }


def metrics(raw_ir: str, optimized_ir: str, assembly: str, remarks: str,
            function: str | None, proof_report: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    raw_body = llvm_function(raw_ir, function)
    opt_body = llvm_function(optimized_ir, function)
    asm_body = assembly_function(assembly, function)
    opcodes = assembly_opcodes(asm_body)
    vectors = [(int(width), int(interleave)) for width, interleave in VECTOR_REMARK.findall(remarks)]
    ir_vector_widths = [int(width) for width in re.findall(r"<(\d+)\s+x\s+[^>]+>", opt_body)]
    all_bounds = proof_report
    bounds = None if proof_report is None else [
        site for site in proof_report if function is None or site["function"] == function
    ]
    if bounds is None:
        proof_metrics = {
            "proof.bounds_sites": None,
            "proof.eligible": None,
            "proof.proved": None,
            "proof.retained": None,
            "proof.ceiling": None,
            "proof.partition_ok": None,
            "proof.dominating_guard": None,
            "proof.masked_index": None,
            "proof.remainder_guard": None,
            "proof.remainder_tail": None,
            "proof.output_capacity_lockstep": None,
            "proof.proved_targets": None,
            "proof.retained_targets": None,
            "proof.obligation_family_counts": None,
            "proof.obligation_status_counts": None,
            "proof.obligation_exactness_counts": None,
            "proof.requirement_relation_counts": None,
            "proof.first_missing_fact_count": None,
            "proof.first_failed_premise_count": None,
            "proof.first_failed_premise_reason_counts": None,
            "proof.obligation_by_site": None,
            "proof.obligation_status_by_site": None,
            "proof.obligation_exactness_by_site": None,
            "proof.requirement_relation_by_site": None,
            "proof.first_missing_fact_by_site": None,
            "proof.first_failed_premise_by_site": None,
            "proof.diagnostics_by_site": None,
            "proof.obligation_diagnostics_by_site": None,
            "proof.checked_automation_scope": None,
            "proof.checked_automation_ready": None,
            "proof.checked_automation_disposition_counts": None,
            "proof.checked_automation_findings_by_site": None,
            "proof.checked_automation_module_ready": None,
            "proof.checked_automation_module_disposition_counts": None,
            "proof.checked_automation_module_findings_by_site": None,
            "proof.sites": None,
        }
    else:
        def target_counts(status: str) -> dict[str, int]:
            counts: dict[str, int] = {}
            for site in bounds:
                if site["status"] == status:
                    target = site["target"]
                    counts[target] = counts.get(target, 0) + 1
            return dict(sorted(counts.items()))

        def value_counts(field: str) -> dict[str, int]:
            counts: dict[str, int] = {}
            for site in bounds:
                value = site[field]
                key = NULL_METRIC_KEY if value is None else value
                counts[key] = counts.get(key, 0) + 1
            return dict(sorted(counts.items()))

        def site_key(site: dict[str, Any]) -> str:
            return f'{site["function"]}:{site["site"]}'

        def site_map(field: str) -> dict[str, Any]:
            return dict(sorted(
                (site_key(site), site[field]) for site in bounds
            ))

        def failed_reason(site: dict[str, Any]) -> str | None:
            premise = site["first_failed_premise"]
            return None if premise is None else premise["reason"]

        obligation_family_counts: dict[str, int] = {}
        failed_reason_counts: dict[str, int] = {}
        diagnostics_by_site: dict[str, dict[str, Any]] = {}
        obligation_diagnostics_by_site: dict[str, dict[str, Any]] = {}
        obligation_ordinals: dict[str, int] = {}
        for site in bounds:
            family = site["obligation"]
            family_key = NULL_METRIC_KEY if family is None else family
            obligation_family_counts[family_key] = (
                obligation_family_counts.get(family_key, 0) + 1
            )
            reason = failed_reason(site)
            reason_key = NULL_METRIC_KEY if reason is None else reason
            failed_reason_counts[reason_key] = failed_reason_counts.get(reason_key, 0) + 1
            diagnostics_by_site[site_key(site)] = {
                "obligation_analysis": site["obligation_analysis"],
                "obligation": site["obligation"],
                "obligation_status": site["obligation_status"],
                "obligation_exactness": site["obligation_exactness"],
                "requirement_relation": site["requirement_relation"],
                "first_missing_fact": site["first_missing_fact"],
                "first_failed_premise": site["first_failed_premise"],
            }
            if site["obligation"] is not None:
                function_name = site["function"]
                ordinal = obligation_ordinals.get(function_name, 0)
                obligation_ordinals[function_name] = ordinal + 1
                obligation_diagnostics_by_site[f"{function_name}:{ordinal}"] = (
                    diagnostics_by_site[site_key(site)]
                )

        automation = checked_automation_summary(bounds)
        module_automation = checked_automation_summary(all_bounds or [])
        proof_metrics = {
            "proof.bounds_sites": len(bounds),
            "proof.eligible": len(bounds),
            "proof.proved": sum(site["status"] == "proved" for site in bounds),
            "proof.retained": sum(site["status"] == "retained" for site in bounds),
            "proof.ceiling": sum(site["status"] == "ceiling" for site in bounds),
            "proof.partition_ok": len(bounds) == sum(
                site["status"] in {"proved", "retained", "ceiling"} for site in bounds
            ),
            "proof.dominating_guard": sum(
                site["proof"] == "dominating-guard" for site in bounds
            ),
            "proof.masked_index": sum(site["proof"] == "masked-index" for site in bounds),
            "proof.remainder_guard": sum(site["proof"] == "remainder-guard" for site in bounds),
            "proof.remainder_tail": sum(site["proof"] == "remainder-tail" for site in bounds),
            "proof.output_capacity_lockstep": sum(
                site["proof"] == "output-capacity-lockstep" for site in bounds
            ),
            "proof.proved_targets": target_counts("proved"),
            "proof.retained_targets": target_counts("retained"),
            "proof.obligation_family_counts": dict(sorted(obligation_family_counts.items())),
            "proof.obligation_status_counts": value_counts("obligation_status"),
            "proof.obligation_exactness_counts": value_counts("obligation_exactness"),
            "proof.requirement_relation_counts": value_counts("requirement_relation"),
            "proof.first_missing_fact_count": sum(
                site["first_missing_fact"] is not None for site in bounds
            ),
            "proof.first_failed_premise_count": sum(
                site["first_failed_premise"] is not None for site in bounds
            ),
            "proof.first_failed_premise_reason_counts": dict(sorted(failed_reason_counts.items())),
            "proof.obligation_by_site": site_map("obligation"),
            "proof.obligation_status_by_site": site_map("obligation_status"),
            "proof.obligation_exactness_by_site": site_map("obligation_exactness"),
            "proof.requirement_relation_by_site": site_map("requirement_relation"),
            "proof.first_missing_fact_by_site": site_map("first_missing_fact"),
            "proof.first_failed_premise_by_site": site_map("first_failed_premise"),
            "proof.diagnostics_by_site": dict(sorted(diagnostics_by_site.items())),
            "proof.obligation_diagnostics_by_site": dict(
                sorted(obligation_diagnostics_by_site.items())
            ),
            "proof.checked_automation_scope": automation["scope"],
            "proof.checked_automation_ready": automation["ready"],
            "proof.checked_automation_disposition_counts": (
                automation["disposition_counts"]
            ),
            "proof.checked_automation_findings_by_site": automation["findings_by_site"],
            "proof.checked_automation_module_ready": module_automation["ready"],
            "proof.checked_automation_module_disposition_counts": (
                module_automation["disposition_counts"]
            ),
            "proof.checked_automation_module_findings_by_site": (
                module_automation["findings_by_site"]
            ),
            "proof.sites": bounds,
        }
    return {
        "raw_ir.alias_scope_uses": sum(
            "!alias.scope" in line for line in raw_body.splitlines() if not line.lstrip().startswith("!")
        ),
        "raw_ir.noalias_uses": sum(
            "!noalias" in line for line in raw_body.splitlines() if not line.lstrip().startswith("!")
        ),
        "raw_ir.saturating_add_mentions": raw_body.count("@llvm.uadd.sat"),
        "raw_ir.trap_calls": raw_body.count("call void @llvm.trap"),
        "opt_ir.loads": len(re.findall(r"(?m)^\s*%[^\n=]+?=\s*load\b", opt_body)),
        "opt_ir.vector_loads": len(re.findall(r"(?m)=\s*load\s+<\d+\s+x\s+", opt_body)),
        "opt_ir.vector_ops": sum(
            bool(re.search(r"<\d+\s+x\s+[^>]+>", line)) for line in opt_body.splitlines()
        ),
        "opt_ir.max_vector_width": max(ir_vector_widths, default=0),
        "opt_ir.saturating_add_mentions": opt_body.count("@llvm.uadd.sat"),
        "opt_ir.trap_calls": opt_body.count("call void @llvm.trap"),
        "asm.instructions": len(opcodes),
        "asm.opcodes": opcodes,
        "asm.traps": sum(opcode in {"brk", "ud2", "udf", "trap"} for opcode in opcodes),
        "remarks.vectorized_loops": len(vectors),
        "remarks.max_vector_width": max((width for width, _ in vectors), default=0),
        "remarks.max_interleave": max((interleave for _, interleave in vectors), default=0),
        **proof_metrics,
    }


def tool_version(command: list[str]) -> str | None:
    try:
        result = run(command)
    except HarnessError:
        return None
    return (result.stdout.strip() or result.stderr.strip()).splitlines()[0]


def environment_report() -> dict[str, str | None]:
    return {
        "architecture": platform.machine(),
        "system": platform.system(),
        "clang": tool_version([str(CLANG), "--version"]),
        "rustc": tool_version([shutil.which("rustc") or "rustc", "--version"]),
        "python": platform.python_version(),
    }


def validate_proof_report(report: list[dict[str, Any]]) -> None:
    by_function: dict[str, list[dict[str, Any]]] = {}
    for site in report:
        if not isinstance(site.get("function"), str) or not site["function"]:
            raise HarnessError(f"invalid proof-site function: {site!r}")
        if not isinstance(site.get("site"), int) or site["site"] < 0:
            raise HarnessError(f"invalid proof-site ordinal: {site!r}")
        if site.get("kind") not in {"const", "buffer", "buffer-field"}:
            raise HarnessError(f"invalid proof-site target kind: {site!r}")
        if not isinstance(site.get("target"), str) or not site["target"]:
            raise HarnessError(f"invalid proof-site target: {site!r}")
        if site.get("index") is not None and not isinstance(site["index"], str):
            raise HarnessError(f"invalid proof-site index: {site!r}")
        by_function.setdefault(site.get("function"), []).append(site)
        if site.get("status") not in {"proved", "retained", "ceiling"}:
            raise HarnessError(f"unknown proof-site status: {site!r}")
        if site.get("proof") not in {
                None, "dominating-guard", "masked-index", "remainder-guard", "remainder-tail",
                "output-capacity-lockstep"}:
            raise HarnessError(f"unknown bounds proof reason: {site!r}")
        if site["status"] == "proved" and site["proof"] is None:
            raise HarnessError(f"proved bounds site lacks a proof reason: {site!r}")
        if site["status"] == "retained" and site["proof"] is not None:
            raise HarnessError(f"retained bounds site carries an elision proof: {site!r}")
        analysis = site.get("obligation_analysis")
        if not isinstance(analysis, dict) or set(analysis) != {
                "scope", "complete", "analyzers"}:
            raise HarnessError(f"invalid proof-site obligation analysis: {site!r}")
        analyzers = analysis.get("analyzers")
        if not isinstance(analysis.get("complete"), bool) \
                or not isinstance(analyzers, list) \
                or any(not isinstance(name, str) or not name for name in analyzers) \
                or analyzers != sorted(set(analyzers)):
            raise HarnessError(f"invalid proof-site obligation analyzer set: {site!r}")
        if analysis["complete"]:
            if analysis["scope"] != CHECKED_AUTOMATION_SCOPE \
                    or analyzers != list(CHECKED_AUTOMATION_ANALYZERS):
                raise HarnessError(f"unknown complete obligation analysis: {site!r}")
        elif analysis["scope"] is not None or analyzers:
            raise HarnessError(f"incomplete obligation analysis claims coverage: {site!r}")
        if "obligation" not in site or site["obligation"] not in {
                None, "output-capacity-lockstep"}:
            raise HarnessError(f"unknown proof-site obligation: {site!r}")
        if site.get("obligation_status") not in OBLIGATION_STATUSES:
            raise HarnessError(f"unknown proof-site obligation status: {site!r}")
        if "obligation_exactness" not in site or (
                site["obligation_exactness"] is not None
                and site["obligation_exactness"] not in OBLIGATION_EXACTNESSES):
            raise HarnessError(f"unknown proof-site obligation exactness: {site!r}")
        if "requirement_relation" not in site or (
                site["requirement_relation"] is not None
                and site["requirement_relation"] not in REQUIREMENT_RELATIONS):
            raise HarnessError(f"unknown proof-site requirement relation: {site!r}")
        if "first_missing_fact" not in site or (
                site["first_missing_fact"] is not None
                and not isinstance(site["first_missing_fact"], dict)):
            raise HarnessError(f"invalid proof-site first missing fact: {site!r}")
        if "first_failed_premise" not in site:
            raise HarnessError(f"missing proof-site first failed premise: {site!r}")
        premise = site["first_failed_premise"]
        if premise is not None:
            if not isinstance(premise, dict) or set(premise) - FAILED_PREMISE_KEYS:
                raise HarnessError(f"invalid proof-site first failed premise: {site!r}")
            if not isinstance(premise.get("reason"), str) or not premise["reason"]:
                raise HarnessError(f"proof-site failed premise lacks a reason: {site!r}")
            if any(not isinstance(value, str) for value in premise.values()):
                raise HarnessError(f"proof-site failed premise fields must be strings: {site!r}")
        if site["obligation"] is None:
            if analysis["complete"]:
                valid_no_obligation = (
                    site["obligation_status"] == "not-applicable"
                    and site["obligation_exactness"] is None
                    and site["requirement_relation"] == "not-applicable"
                    and site["first_missing_fact"] is None
                    and site["first_failed_premise"] is None
                )
            else:
                valid_no_obligation = (
                    site["obligation_status"] == "unknown"
                    and site["obligation_exactness"] == "unknown"
                    and site["requirement_relation"] == "unknown"
                    and site["first_missing_fact"] is None
                    and site["first_failed_premise"] is None
                )
            if not valid_no_obligation:
                raise HarnessError(f"non-obligation site carries invalid state: {site!r}")
        if site["obligation"] == "output-capacity-lockstep":
            if not analysis["complete"]:
                raise HarnessError(f"capacity obligation lacks complete analysis: {site!r}")
            if site["obligation_status"] not in {"derived", "failed-premise"}:
                raise HarnessError(f"invalid capacity obligation status: {site!r}")
            if site["obligation_status"] == "failed-premise" and not (
                    site["obligation_exactness"] == "unknown"
                    and site["requirement_relation"] == "unknown"
                    and site["first_missing_fact"] is None
                    and site["first_failed_premise"] is not None):
                raise HarnessError(f"invalid failed capacity obligation: {site!r}")
            if site["obligation_status"] == "derived" and not (
                    site["obligation_exactness"] in {"exact", "sufficient"}
                    and site["requirement_relation"] in {
                        "equivalent", "missing", "mismatch"
                    }):
                raise HarnessError(f"invalid derived capacity obligation: {site!r}")
            if site["requirement_relation"] == "equivalent" and (
                    site["first_missing_fact"] is not None
                    or site["first_failed_premise"] is not None):
                raise HarnessError(f"equivalent capacity obligation carries debt: {site!r}")
            if site["requirement_relation"] in {"missing", "mismatch"} \
                    and site["first_missing_fact"] is None:
                raise HarnessError(f"unmatched capacity obligation lacks repair fact: {site!r}")
            if site["requirement_relation"] == "mismatch" \
                    and site["first_failed_premise"] is None:
                raise HarnessError(f"mismatched capacity obligation lacks premise: {site!r}")
        if site["proof"] == "output-capacity-lockstep" and not (
                site["obligation"] == "output-capacity-lockstep"
                and site["obligation_status"] == "derived"
                and site["obligation_exactness"] in {"exact", "sufficient"}
                and site["requirement_relation"] == "equivalent"
                and site["first_missing_fact"] is None
                and site["first_failed_premise"] is None):
            raise HarnessError(
                f"capacity proof lacks a matching discharged obligation: {site!r}"
            )
    for function, sites in by_function.items():
        ordinals = [site.get("site") for site in sites]
        if ordinals != list(range(len(sites))):
            raise HarnessError(f"non-contiguous bounds-site ordinals in {function}: {ordinals}")


def compile_variant(case_id: str, variant: dict[str, Any], build: Path, democ: Any) -> dict[str, Any]:
    name = variant["name"]
    kind = variant["kind"]
    source = (ROOT / variant["source"]).resolve()
    if not source.is_file() or ROOT not in source.parents:
        raise HarnessError(f"invalid source path for {case_id}/{name}: {source}")
    stem = re.sub(r"[^A-Za-z0-9_.-]", "-", f"{case_id}-{name}")
    raw_ir_path = build / f"{stem}.raw.ll"
    opt_ir_path = build / f"{stem}.opt.ll"
    asm_path = build / f"{stem}.s"
    level = str(variant.get("opt", "O2"))
    if not re.fullmatch(r"O[0-3sz]", level):
        raise HarnessError(f"invalid optimization level {level!r}")

    if kind == "xlang":
        proof_report: list[dict[str, Any]] = []
        raw_ir = democ.compile_program(
            source.read_text(),
            alias=bool(variant.get("facts", True)),
            elide_bounds=bool(variant.get("elide_bounds", False)),
            proof_report=proof_report,
        )
        validate_proof_report(proof_report)
        raw_ir_path.write_text(raw_ir)
        input_path = raw_ir_path
    elif kind == "c":
        input_path = source
    elif kind == "rust":
        rustc = shutil.which("rustc")
        if not rustc:
            raise HarnessError("rustc not found")
        run([rustc, "-C", f"opt-level={level[1:]}", "--emit", "asm", str(source), "-o", str(asm_path)])
        run([rustc, "-C", f"opt-level={level[1:]}", "--emit", "llvm-ir", str(source), "-o", str(opt_ir_path)])
        optimized_ir = opt_ir_path.read_text()
        return metrics(optimized_ir, optimized_ir, asm_path.read_text(), "", variant.get("function"))
    else:
        raise HarnessError(f"unknown variant kind {kind!r}")

    remark_flags = [
        "-Rpass=loop-vectorize",
        "-Rpass-missed=loop-vectorize",
        "-Rpass-analysis=loop-vectorize",
    ]
    compiled = run([str(CLANG), f"-{level}", *remark_flags, "-S", str(input_path), "-o", str(asm_path)])
    run([str(CLANG), f"-{level}", "-S", "-emit-llvm", str(input_path), "-o", str(opt_ir_path)])
    optimized_ir = opt_ir_path.read_text()
    if kind == "c":
        raw_ir = optimized_ir
    return metrics(raw_ir, optimized_ir, asm_path.read_text(), compiled.stderr,
                   variant.get("function"), proof_report if kind == "xlang" else None)


def resolve(reference: str, variants: dict[str, dict[str, Any]]) -> Any:
    name, separator, metric = reference.partition(".")
    if not separator or name not in variants or metric not in variants[name]:
        raise HarnessError(f"unknown metric reference {reference!r}")
    return variants[name][metric]


def compact(value: Any) -> str:
    if isinstance(value, list):
        preview = ",".join(str(item) for item in value[:8])
        suffix = ",…" if len(value) > 8 else ""
        return f"[{len(value)} items: {preview}{suffix}]"
    return repr(value)


def evaluate(check: dict[str, Any], variants: dict[str, dict[str, Any]]) -> tuple[bool, Any, Any]:
    operation = check.get("op", "eq")
    if operation not in OPS:
        raise HarnessError(f"unknown comparison operator {operation!r}")
    left = resolve(check["left"], variants)
    right = resolve(check["right"], variants) if "right" in check else check["value"]
    return bool(OPS[operation](left, right)), left, right


def validate_manifest(
        manifest: dict[str, Any], *, activate_promotion: bool = False
) -> list[dict[str, Any]]:
    if manifest.get("schema") != 1:
        raise HarnessError("manifest schema must be 1")
    cases = manifest.get("cases")
    if not isinstance(cases, list) or not cases:
        raise HarnessError("manifest must contain at least one case")
    case_ids: set[str] = set()
    for case in cases:
        if "_checked_automation_root" in case:
            raise HarnessError("checked-automation runtime marker is reserved")
        case_id = case.get("id")
        if not isinstance(case_id, str) or not case_id or case_id in case_ids:
            raise HarnessError(f"invalid or duplicate case id {case_id!r}")
        case_ids.add(case_id)
        if case.get("mode") not in {"gate", "audit"}:
            raise HarnessError(f"{case_id}: mode must be 'gate' or 'audit'")
        variants = case.get("variants")
        if not isinstance(variants, list) or not variants:
            raise HarnessError(f"{case_id}: at least one variant is required")
        names = [variant.get("name") for variant in variants]
        if any(not isinstance(name, str) or not name for name in names) or len(names) != len(set(names)):
            raise HarnessError(f"{case_id}: variant names must be non-empty and unique")
        for variant in variants:
            if variant.get("kind") not in {"xlang", "c", "rust"}:
                raise HarnessError(f"{case_id}/{variant.get('name')}: invalid kind")
            if not isinstance(variant.get("source"), str):
                raise HarnessError(f"{case_id}/{variant.get('name')}: source is required")
        checks = case.get("checks")
        if not isinstance(checks, list) or not checks:
            raise HarnessError(f"{case_id}: at least one check is required")
        for check in checks:
            if not isinstance(check.get("left"), str) or (("right" in check) == ("value" in check)):
                raise HarnessError(f"{case_id}: each check needs left and exactly one of right/value")
            if check.get("op", "eq") not in OPS:
                raise HarnessError(f"{case_id}: invalid comparison operator {check.get('op')!r}")

    policy = manifest.get("checked_automation")
    if policy is not None:
        required_policy_keys = {"schema", "policy", "roots", "approvals"}
        if not isinstance(policy, dict) or set(policy) != required_policy_keys:
            raise HarnessError(
                "checked_automation must contain schema, policy, roots, and approvals"
            )
        if policy["schema"] != 1 or policy["policy"] != CHECKED_AUTOMATION_SCOPE:
            raise HarnessError("unsupported checked-automation policy")
        if policy["approvals"] != []:
            raise HarnessError(
                "checked-automation approvals are not implemented; approvals must be empty"
            )
        roots = policy["roots"]
        if not isinstance(roots, list) or not roots:
            raise HarnessError("checked_automation.roots must be a non-empty list")
        cases_by_id = {case["id"]: case for case in cases}
        seen_roots: set[tuple[str, str]] = set()
        root_keys = {
            "case", "variant", "function", "source", "source_sha256", "scope",
        }
        for root in roots:
            if not isinstance(root, dict) or set(root) != root_keys:
                raise HarnessError(
                    "each checked-automation root needs case, variant, function, source, "
                    "source_sha256, and scope"
                )
            case_id, variant_name = root["case"], root["variant"]
            identity = (case_id, variant_name)
            if not isinstance(case_id, str) or not isinstance(variant_name, str) \
                    or identity in seen_roots:
                raise HarnessError(f"invalid or duplicate checked-automation root {identity!r}")
            seen_roots.add(identity)
            if root["scope"] != "closed-unit":
                raise HarnessError(f"{case_id}/{variant_name}: scope must be 'closed-unit'")
            case = cases_by_id.get(case_id)
            if case is None:
                raise HarnessError(f"unknown checked-automation root case {case_id!r}")
            if case["mode"] != "gate":
                raise HarnessError(f"{case_id}: a checked-automation root must be a gate case")
            variant = next(
                (item for item in case["variants"] if item["name"] == variant_name), None)
            if variant is None:
                raise HarnessError(
                    f"{case_id}: unknown checked-automation variant {variant_name!r}"
                )
            if variant["kind"] != "xlang" \
                    or variant.get("facts", True) is not True \
                    or variant.get("elide_bounds", False) is not False:
                raise HarnessError(
                    f"{case_id}/{variant_name}: promotion requires facts-on, non-ceiling xlang"
                )
            if root["source"] != variant["source"] \
                    or root["function"] != variant.get("function"):
                raise HarnessError(
                    f"{case_id}/{variant_name}: promoted source/function identity changed"
                )
            source_path = (ROOT / root["source"]).resolve()
            if not source_path.is_file() or ROOT not in source_path.parents:
                raise HarnessError(f"{case_id}/{variant_name}: invalid promoted source")
            digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
            if not isinstance(root["source_sha256"], str) \
                    or not re.fullmatch(r"[0-9a-f]{64}", root["source_sha256"]) \
                    or root["source_sha256"] != digest:
                raise HarnessError(
                    f"{case_id}/{variant_name}: promoted source digest changed"
                )
            if activate_promotion:
                if "_checked_automation_root" in case:
                    raise HarnessError(
                        f"{case_id}: only one promoted variant per case is supported"
                    )
                case["_checked_automation_root"] = dict(root)
    return cases


def load_corpus_cases() -> list[dict[str, Any]]:
    """Translate compact family manifests into ordinary parity-harness cases."""
    cases: list[dict[str, Any]] = []
    if not CORPUS_ROOT.is_dir():
        return cases
    for manifest_path in sorted(CORPUS_ROOT.rglob("cases.json")):
        family_manifest = json.loads(manifest_path.read_text())
        if family_manifest.get("schema") != 1 or not isinstance(family_manifest.get("cases"), list):
            raise HarnessError(f"invalid corpus family manifest: {manifest_path}")
        family = family_manifest.get("family")
        if not isinstance(family, str) or not family:
            raise HarnessError(f"corpus family is required: {manifest_path}")
        family_tags = family_manifest.get("tags", [])
        if not isinstance(family_tags, list) or not all(isinstance(tag, str) for tag in family_tags):
            raise HarnessError(f"corpus family tags must be strings: {manifest_path}")
        measurement = family_manifest.get("measurement", {})
        function = measurement.get("function")
        if not function:
            function = "probe" if family == "bounds/dominating-guard" else None
        if not function:
            raise HarnessError(f"corpus measurement function is required: {manifest_path}")
        if measurement.get("metric") != "proof.sites":
            raise HarnessError(f"unsupported corpus proof metric in {manifest_path}")
        for entry in family_manifest["cases"]:
            source_name = entry.get("source")
            if not isinstance(source_name, str):
                raise HarnessError(f"corpus source is required: {manifest_path}")
            source_path = (manifest_path.parent / source_name).resolve()
            if not source_path.is_file() or ROOT not in source_path.parents:
                raise HarnessError(f"invalid corpus source: {source_path}")
            expected = entry.get("expected", {})
            classification = expected.get("proof_classification", expected.get("classification"))
            if classification not in {"proved", "elided", "retained", "checked", "mixed"}:
                raise HarnessError(f"invalid proof classification for {source_path}")
            is_positive = classification in {"proved", "elided"}
            expected_sites = expected.get("bounds_sites")
            if not isinstance(expected_sites, int) or expected_sites < 1:
                raise HarnessError(f"expected.bounds_sites is required for {source_path}")
            maturity = entry.get("maturity")
            if maturity not in {"explore", "audit", "gate"}:
                raise HarnessError(f"invalid corpus maturity for {source_path}")
            mode = "gate" if maturity == "gate" else "audit"
            entry_tags = entry.get("tags")
            if not isinstance(entry_tags, list) or not entry_tags \
                    or not all(isinstance(tag, str) for tag in entry_tags):
                raise HarnessError(f"corpus case tags must be non-empty strings: {source_path}")
            stem = entry.get("id", source_path.stem)
            case_id = "corpus." + family.replace("/", ".") + "." + stem
            relative_source = str(source_path.relative_to(ROOT))
            checks = []
            checks.extend([
                {
                    "label": "facts enumerate every expected bounds site",
                    "left": "facts.proof.eligible",
                    "op": "eq",
                    "value": expected_sites,
                },
                {
                    "label": "facts-off enumerates the same bounds sites",
                    "left": "nofacts.proof.eligible",
                    "op": "eq",
                    "value": expected_sites,
                },
                {
                    "label": "proof status partition is complete",
                    "left": "facts.proof.partition_ok",
                    "op": "eq",
                    "value": True,
                },
                {
                    "label": "facts and facts-off report identical obligation diagnostics",
                    "left": "facts.proof.obligation_diagnostics_by_site",
                    "op": "eq",
                    "right": "nofacts.proof.obligation_diagnostics_by_site",
                },
            ])

            diagnostic_expectations = (
                (
                    "obligation_status",
                    "proof.obligation_status_counts",
                    OBLIGATION_STATUSES,
                    False,
                ),
                (
                    "obligation_exactness",
                    "proof.obligation_exactness_counts",
                    OBLIGATION_EXACTNESSES,
                    True,
                ),
                (
                    "requirement_relation",
                    "proof.requirement_relation_counts",
                    REQUIREMENT_RELATIONS,
                    True,
                ),
            )
            for expected_field, metric, allowed, nullable in diagnostic_expectations:
                if expected_field not in expected:
                    continue
                expected_value = expected[expected_field]
                if expected_value is None:
                    if not nullable:
                        raise HarnessError(
                            f"expected.{expected_field} may not be null for {source_path}"
                        )
                    metric_key = NULL_METRIC_KEY
                elif isinstance(expected_value, str) and expected_value in allowed:
                    metric_key = expected_value
                else:
                    raise HarnessError(
                        f"invalid expected.{expected_field} for {source_path}: "
                        f"{expected_value!r}"
                    )
                expected_counts = {metric_key: expected_sites}
                for variant_name in ("facts", "nofacts"):
                    checks.append({
                        "label": (
                            f"{variant_name} reports expected {expected_field} "
                            "for every bounds site"
                        ),
                        "left": f"{variant_name}.{metric}",
                        "op": "eq",
                        "value": expected_counts,
                    })

            diagnostic_count_expectations = (
                (
                    "obligation_status_counts",
                    "proof.obligation_status_counts",
                    OBLIGATION_STATUSES,
                ),
                (
                    "obligation_exactness_counts",
                    "proof.obligation_exactness_counts",
                    OBLIGATION_EXACTNESSES | {NULL_METRIC_KEY},
                ),
                (
                    "requirement_relation_counts",
                    "proof.requirement_relation_counts",
                    REQUIREMENT_RELATIONS | {NULL_METRIC_KEY},
                ),
                (
                    "first_failed_premise_reason_counts",
                    "proof.first_failed_premise_reason_counts",
                    None,
                ),
            )
            for expected_field, metric, allowed_keys in diagnostic_count_expectations:
                if expected_field not in expected:
                    continue
                expected_counts = expected[expected_field]
                if not isinstance(expected_counts, dict) or not expected_counts:
                    raise HarnessError(
                        f"expected.{expected_field} must be a non-empty count map "
                        f"for {source_path}"
                    )
                for key, count in expected_counts.items():
                    if not isinstance(key, str) or not key or (
                            allowed_keys is not None and key not in allowed_keys):
                        raise HarnessError(
                            f"invalid expected.{expected_field} key for {source_path}: "
                            f"{key!r}"
                        )
                    if isinstance(count, bool) or not isinstance(count, int) or count < 1:
                        raise HarnessError(
                            f"invalid expected.{expected_field} count for {source_path}: "
                            f"{key!r}={count!r}"
                        )
                if sum(expected_counts.values()) != expected_sites:
                    raise HarnessError(
                        f"expected.{expected_field} must account for all {expected_sites} "
                        f"bounds sites in {source_path}"
                    )
                expected_counts = dict(sorted(expected_counts.items()))
                for variant_name in ("facts", "nofacts"):
                    checks.append({
                        "label": (
                            f"{variant_name} reports the expected {expected_field} "
                            "distribution"
                        ),
                        "left": f"{variant_name}.{metric}",
                        "op": "eq",
                        "value": expected_counts,
                    })

            automation_ready_present = "checked_automation_ready" in expected
            automation_counts_present = (
                "checked_automation_disposition_counts" in expected
            )
            if automation_ready_present != automation_counts_present:
                raise HarnessError(
                    "checked-automation readiness and disposition counts must be "
                    f"specified together for {source_path}"
                )
            if automation_ready_present:
                automation_ready = expected["checked_automation_ready"]
                automation_counts = expected["checked_automation_disposition_counts"]
                expected_keys = set(CHECKED_AUTOMATION_DISPOSITIONS)
                if not isinstance(automation_ready, bool) \
                        or not isinstance(automation_counts, dict) \
                        or set(automation_counts) != expected_keys \
                        or any(isinstance(count, bool) or not isinstance(count, int)
                               or count < 0 for count in automation_counts.values()) \
                        or sum(automation_counts.values()) != expected_sites:
                    raise HarnessError(
                        f"invalid checked-automation oracle for {source_path}"
                    )
                checks.extend([
                    {
                        "label": "facts produce the expected promotion-policy verdict",
                        "left": "facts.proof.checked_automation_ready",
                        "op": "eq",
                        "value": automation_ready,
                    },
                    {
                        "label": "facts produce the expected promotion dispositions",
                        "left": "facts.proof.checked_automation_disposition_counts",
                        "op": "eq",
                        "value": {
                            name: automation_counts[name]
                            for name in CHECKED_AUTOMATION_DISPOSITIONS
                        },
                    },
                ])

            if "first_missing_fact" in expected:
                missing_fact = expected["first_missing_fact"]
                if missing_fact is not None and not isinstance(missing_fact, dict):
                    raise HarnessError(
                        f"expected.first_missing_fact must be null or an object "
                        f"for {source_path}"
                    )
                expected_missing_facts = {
                    f"{function}:{site}": missing_fact for site in range(expected_sites)
                }
                for variant_name in ("facts", "nofacts"):
                    checks.append({
                        "label": (
                            f"{variant_name} reports the expected first missing fact "
                            "at every bounds site"
                        ),
                        "left": f"{variant_name}.proof.first_missing_fact_by_site",
                        "op": "eq",
                        "value": expected_missing_facts,
                    })

            failed_field_present = "first_failed_premise" in expected
            failed_reason_alias_present = "first_failed_premise_reason" in expected
            if failed_field_present and failed_reason_alias_present:
                raise HarnessError(
                    f"use only expected.first_failed_premise for {source_path}"
                )
            if failed_field_present or failed_reason_alias_present:
                failed_expected = expected[
                    "first_failed_premise"
                    if failed_field_present else "first_failed_premise_reason"
                ]
                if isinstance(failed_expected, dict):
                    if set(failed_expected) - FAILED_PREMISE_KEYS:
                        raise HarnessError(
                            f"invalid expected.first_failed_premise for {source_path}: "
                            f"{failed_expected!r}"
                        )
                    failed_reason = failed_expected.get("reason")
                    if not isinstance(failed_reason, str) or not failed_reason:
                        raise HarnessError(
                            f"expected.first_failed_premise needs a reason for {source_path}"
                        )
                    if any(not isinstance(value, str) for value in failed_expected.values()):
                        raise HarnessError(
                            f"expected.first_failed_premise fields must be strings for {source_path}"
                        )
                    expected_failed_premises = {
                        f"{function}:{site}": failed_expected
                        for site in range(expected_sites)
                    }
                    for variant_name in ("facts", "nofacts"):
                        checks.append({
                            "label": (
                                f"{variant_name} reports the exact first failed premise "
                                "at every bounds site"
                            ),
                            "left": f"{variant_name}.proof.first_failed_premise_by_site",
                            "op": "eq",
                            "value": expected_failed_premises,
                        })
                elif failed_expected is None:
                    failed_reason = NULL_METRIC_KEY
                elif isinstance(failed_expected, str) and failed_expected:
                    failed_reason = failed_expected
                else:
                    raise HarnessError(
                        f"invalid expected.first_failed_premise for {source_path}: "
                        f"{failed_expected!r}"
                    )
                expected_reason_counts = {failed_reason: expected_sites}
                for variant_name in ("facts", "nofacts"):
                    checks.append({
                        "label": (
                            f"{variant_name} reports the expected first failed premise reason "
                            "for every bounds site"
                        ),
                        "left": f"{variant_name}.proof.first_failed_premise_reason_counts",
                        "op": "eq",
                        "value": expected_reason_counts,
                    })
            if is_positive:
                checks.extend([
                    {
                        "label": "facts prove the eligible access sites",
                        "left": "facts.proof.proved",
                        "op": "eq",
                        "value": expected_sites,
                    },
                    {
                        "label": "facts leave no eligible site checked",
                        "left": "facts.proof.retained",
                        "op": "eq",
                        "value": 0,
                    },
                    {
                        "label": "facts-off retains exactly the proved sites",
                        "left": "nofacts.proof.retained",
                        "op": "eq",
                        "value": expected_sites,
                    },
                ])
            elif classification == "mixed":
                proved_sites = expected.get("proved_sites")
                retained_sites = expected.get("retained_sites")
                if not isinstance(proved_sites, int) or not isinstance(retained_sites, int) \
                        or proved_sites < 1 or retained_sites < 1 \
                        or proved_sites + retained_sites != expected_sites:
                    raise HarnessError(f"invalid mixed proof counts for {source_path}")
                checks.extend([
                    {
                        "label": "mixed case proves exactly its eligible subset",
                        "left": "facts.proof.proved",
                        "op": "eq",
                        "value": proved_sites,
                    },
                    {
                        "label": "mixed case retains exactly its unsafe subset",
                        "left": "facts.proof.retained",
                        "op": "eq",
                        "value": retained_sites,
                    },
                    {
                        "label": "facts-off retains every mixed-case site",
                        "left": "nofacts.proof.retained",
                        "op": "eq",
                        "value": expected_sites,
                    },
                ])
            else:
                checks.extend([
                    {
                        "label": "near-miss proves no access site",
                        "left": "facts.proof.proved",
                        "op": "eq",
                        "value": 0,
                    },
                    {
                        "label": "near-miss retains the dynamic sites",
                        "left": "facts.proof.retained",
                        "op": "eq",
                        "value": expected_sites,
                    },
                    {
                        "label": "facts do not overgeneralize beyond the control",
                        "left": "nofacts.proof.retained",
                        "op": "eq",
                        "value": expected_sites,
                    },
                ])
            cases.append({
                "id": case_id,
                "mode": mode,
                "maturity": maturity,
                "tags": sorted(set(family_tags + entry_tags)),
                "description": entry.get("hypothesis", family_manifest.get("hypothesis", "")),
                "variants": [
                    {
                        "name": "facts",
                        "kind": "xlang",
                        "source": relative_source,
                        "function": function,
                        "opt": measurement.get("opt", "O2"),
                    },
                    {
                        "name": "nofacts",
                        "kind": "xlang",
                        "source": relative_source,
                        "function": function,
                        "facts": False,
                        "opt": measurement.get("opt", "O2"),
                    },
                ],
                "checks": checks,
            })
    return cases


def run_case(case: dict[str, Any], build: Path, democ: Any) -> dict[str, Any]:
    result: dict[str, Any] = {
        "id": case["id"],
        "mode": case["mode"],
        "maturity": case.get("maturity", case["mode"]),
        "tags": case.get("tags", []),
        "description": case.get("description", ""),
        "variants": {},
        "checks": [],
    }
    root = case.get("_checked_automation_root")
    if root is not None:
        result["checked_automation_root"] = root
    try:
        for variant in case["variants"]:
            result["variants"][variant["name"]] = compile_variant(case["id"], variant, build, democ)
        checks = list(case["checks"])
        if root is not None:
            variant_name = root["variant"]
            checks.extend([
                {
                    "label": "promotion uses the registered complete bounds policy",
                    "left": f"{variant_name}.proof.checked_automation_scope",
                    "op": "eq",
                    "value": CHECKED_AUTOMATION_SCOPE,
                },
                {
                    "label": "promoted closed unit has no hard or unaccounted bounds sites",
                    "left": f"{variant_name}.proof.checked_automation_module_ready",
                    "op": "eq",
                    "value": True,
                },
                {
                    "label": "promoted closed unit has no checked-automation findings",
                    "left": f"{variant_name}.proof.checked_automation_module_findings_by_site",
                    "op": "eq",
                    "value": {},
                },
            ])
            result["checked_automation"] = {
                "scope": CHECKED_AUTOMATION_SCOPE,
                "closure": root["scope"],
                "variant": variant_name,
                "ready": result["variants"][variant_name][
                    "proof.checked_automation_module_ready"
                ],
                "disposition_counts": result["variants"][variant_name][
                    "proof.checked_automation_module_disposition_counts"
                ],
                "findings_by_site": result["variants"][variant_name][
                    "proof.checked_automation_module_findings_by_site"
                ],
            }
        for check in checks:
            passed, left, right = evaluate(check, result["variants"])
            result["checks"].append({
                "label": check.get("label", check["left"]),
                "passed": passed,
                "left": left,
                "op": check.get("op", "eq"),
                "right": right,
            })
    except (Exception, SystemExit) as error:  # structure compiler/checker failures
        result["error"] = str(error)
        result["error_class"] = (
            "checked-automation-evaluation-error"
            if root is not None else "case-evaluation-error"
        )
    result["passed"] = "error" not in result and all(check["passed"] for check in result["checks"])
    return result


def print_report(results: list[dict[str, Any]]) -> None:
    for result in results:
        label = result["mode"]
        if result["maturity"] != result["mode"]:
            label += f"/{result['maturity']}"
        print(f"\n== {result['id']} [{label}] ==")
        if result["description"]:
            print(result["description"])
        if "error" in result:
            print(f"  ERROR: {result['error']}")
            continue
        for check in result["checks"]:
            if check["passed"]:
                marker = "PASS"
            else:
                marker = "FAIL" if result["mode"] == "gate" else "DEBT"
            print(
                f"  {marker}: {check['label']} "
                f"({compact(check['left'])} {check['op']} {compact(check['right'])})"
            )
        if "checked_automation" in result:
            if result["checked_automation"]["ready"]:
                print("  PROMOTION PASS: checked-automation accounting is complete")
            else:
                print("  PERFORMANCE PROMOTION FAILED: hard or unaccounted bounds sites remain")
    gate_failures = sum(not item["passed"] and item["mode"] == "gate" for item in results)
    audit_debts = sum(not item["passed"] and item["mode"] == "audit" for item in results)
    print(f"\ncodegen parity: {gate_failures} gate failure(s), {audit_debts} known audit debt(s)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--case", action="append", dest="cases", help="run only this case id")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--gate-only", action="store_true")
    group.add_argument("--audit-only", action="store_true")
    parser.add_argument("--corpus", action="store_true", help="append discovered codegen-corpus families")
    parser.add_argument(
        "--promotion", action="store_true",
        help="run the unfiltered default manifest and require every review-pinned root",
    )
    parser.add_argument("--tag", action="append", dest="tags", help="run cases carrying this corpus tag")
    parser.add_argument("--json", action="store_true", help="emit the complete machine-readable report")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        validate_promotion_invocation(args)
        manifest = json.loads(args.manifest.read_text())
        if args.promotion:
            validate_promotion_authority(manifest)
        if args.corpus:
            manifest_cases = manifest.get("cases")
            if not isinstance(manifest_cases, list):
                raise HarnessError("manifest cases must be a list")
            manifest = dict(manifest)
            manifest["cases"] = manifest_cases + load_corpus_cases()
            if args.promotion:
                validate_policy_oracle_authority()
        # Validate exactly once so the privileged, internal promotion marker
        # cannot be supplied by an input manifest or lost during corpus merge.
        selected = validate_manifest(manifest, activate_promotion=args.promotion)
        if args.tags:
            wanted_tags = set(args.tags)
            selected = [case for case in selected if wanted_tags.issubset(set(case.get("tags", [])))]
        if args.cases:
            wanted = set(args.cases)
            selected = [case for case in selected if case["id"] in wanted]
            missing = wanted - {case["id"] for case in selected}
            if missing:
                raise HarnessError(f"unknown case(s): {', '.join(sorted(missing))}")
        if args.gate_only:
            selected = [case for case in selected if case["mode"] == "gate"]
        if args.audit_only:
            selected = [case for case in selected if case["mode"] == "audit"]
        democ = load_democ()
        with tempfile.TemporaryDirectory(prefix="xlang-codegen-parity-") as temporary:
            build = Path(temporary)
            results = [run_case(case, build, democ) for case in selected]
        if args.promotion:
            expected_roots = {
                (root["case"], root["variant"])
                for root in manifest["checked_automation"]["roots"]
            }
            evaluated_roots = {
                (result["checked_automation_root"]["case"],
                 result["checked_automation_root"]["variant"])
                for result in results if "checked_automation_root" in result
            }
            if evaluated_roots != expected_roots:
                raise HarnessError("promotion run did not evaluate every pinned root")
    except (OSError, KeyError, ValueError, TypeError, AttributeError, HarnessError) as error:
        print(f"codegen parity harness error: {error}", file=sys.stderr)
        return 2

    exit_code = result_exit_code(results)
    if args.json:
        promotion = {
            "invocation_validated": args.promotion,
            "passed": (exit_code == 0) if args.promotion else None,
            "policy": CHECKED_AUTOMATION_SCOPE if args.promotion else None,
            "roots": list(REVIEW_PINNED_PROMOTION_ROOTS) if args.promotion else [],
            "oracle_count": (
                REVIEW_PINNED_POLICY_ORACLE_COUNT if args.promotion else None
            ),
            "oracle_sha256": (
                REVIEW_PINNED_POLICY_ORACLE_SHA256 if args.promotion else None
            ),
        }
        payload = {
            "schema": 1,
            "promotion": promotion,
            "environment": environment_report(),
            "results": results,
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print_report(results)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
