"""Subprocess-only differential gate for the two independent evidence routes."""

from __future__ import annotations

import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from agreement_validation import (
    ANALYTIC_CODE_FILES,
    ANALYTIC_DERIVED_NAMES,
    AgreementError,
    EXPECTED_AUTHORITY_IDENTITIES,
    FIELD_NAMES,
    TRACE_FIELDS,
    _analytic_code_identity,
    _analytic_fields,
    _compare_profile_claims,
    _neutral_bundle_identity,
    _source_code_identity,
    _source_fields,
    _validate_case_bindings,
    _validate_manifest,
    _validate_route_identities,
    _validate_trace_gaps,
)


ROOT = Path(__file__).resolve().parent
WORKLOADS = ROOT / "workloads.py"
SOURCE_ROUTE = ROOT / "source-route" / "run.py"
ANALYTIC_ROUTE = ROOT / "analytic-route" / "run.py"
FAMILIES = ("compiler", "codec")
SCALE_POINTS = (1, 2, 17)


def _pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise AgreementError(f"comparison JSON contains duplicate key: {key}")
        result[key] = value
    return result


def _invoke(arguments: tuple[str, ...]) -> None:
    environment = {
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": os.environ.get("PATH", ""),
        "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONHASHSEED": "0",
    }
    completed = subprocess.run(
        arguments,
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AgreementError(
            f"subprocess failed ({completed.returncode}): "
            f"{' '.join(arguments)}\n{completed.stderr}"
        )


def _read_json(path: Path, label: str) -> dict[str, object]:
    try:
        encoded = path.read_bytes()
        value = json.loads(encoded, object_pairs_hook=_pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise AgreementError(f"{label} is not readable JSON") from error
    if not isinstance(value, dict):
        raise AgreementError(f"{label} is not a JSON object")
    try:
        canonical = (
            json.dumps(
                value,
                ensure_ascii=True,
                allow_nan=False,
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n"
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise AgreementError(f"{label} contains a noncanonical value") from error
    if canonical != encoded:
        raise AgreementError(f"{label} is not canonical JSON")
    return value


def run_case(family: str, units: int, directory: Path) -> dict[str, object]:
    if family not in FAMILIES or units not in SCALE_POINTS:
        raise AgreementError("agreement case is outside the named smoke matrix")
    source = directory / f"{family}-{units}.wf"
    manifest = directory / f"{family}-{units}.manifest.json"
    source_receipt = directory / f"{family}-{units}.source.json"
    analytic_receipt = directory / f"{family}-{units}.analytic.bin"
    analytic_summary = directory / f"{family}-{units}.analytic.json"
    _invoke(
        (
            sys.executable,
            "-B",
            str(WORKLOADS),
            "--family",
            family,
            "--units",
            str(units),
            "--output",
            str(source),
            "--manifest-output",
            str(manifest),
        )
    )
    manifest_value = _read_json(manifest, "neutral manifest")
    try:
        manifest_bytes = manifest.read_bytes()
        source_bytes = source.read_bytes()
    except OSError as error:
        raise AgreementError("generated agreement inputs are unreadable") from error
    descriptor = _validate_manifest(manifest_value, family, units, source_bytes)
    _invoke(
        (
            sys.executable,
            "-B",
            str(SOURCE_ROUTE),
            "--logical-path",
            descriptor["logical_path"],
            "--source-file",
            str(source),
            "--manifest",
            str(manifest),
            "--output",
            str(source_receipt),
        )
    )
    _invoke(
        (
            sys.executable,
            "-B",
            str(ANALYTIC_ROUTE),
            "--manifest",
            str(manifest),
            "--source-file",
            str(source),
            "--output",
            str(analytic_receipt),
            "--summary-output",
            str(analytic_summary),
        )
    )
    source_value = _read_json(source_receipt, "source-route receipt")
    analytic_value = _read_json(analytic_summary, "analytic-route summary")
    try:
        analytic_receipt_bytes = analytic_receipt.read_bytes()
    except OSError as error:
        raise AgreementError("analytic receipt is unreadable") from error
    source_fields = _compare_profile_claims(source_value, analytic_value)
    if source_value.get("selected_diagnostic") is not None:
        raise AgreementError("source route did not select Complete")
    if analytic_value.get("expected_diagnostic") != ["Complete"]:
        raise AgreementError("analytic route did not select Complete")
    _validate_trace_gaps(source_value, analytic_value)
    source_hashes, neutral_bundle_sha256 = _validate_case_bindings(
        manifest_value,
        source_value,
        analytic_value,
        family=family,
        units=units,
        manifest_bytes=manifest_bytes,
        source_bytes=source_bytes,
        analytic_receipt_bytes=analytic_receipt_bytes,
    )
    workload = source_value["workload"]
    if not isinstance(workload, dict):
        raise AgreementError("validated workload identity was lost")
    canonical = {
        "available_fields": sum(value is not None for value in source_fields.values()),
        "family": family,
        "manifest_sha256": workload["manifest_sha256"],
        "neutral_bundle_sha256": neutral_bundle_sha256,
        "source_sha256": source_hashes,
        "trace_fields": list(TRACE_FIELDS),
        "units": units,
    }
    encoded = (json.dumps(canonical, separators=(",", ":"), sort_keys=True) + "\n").encode(
        "ascii"
    )
    return {**canonical, "agreement_sha256": sha256(encoded).hexdigest()}


def run_suite() -> tuple[dict[str, object], ...]:
    with tempfile.TemporaryDirectory(prefix="whitefoot-route-agreement-") as raw:
        directory = Path(raw)
        return tuple(
            run_case(family, units, directory)
            for family in FAMILIES
            for units in SCALE_POINTS
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.parse_args()
    result = {
        "cases": list(run_suite()),
        "schema": "whitefoot-resource-route-agreement-v1",
        "status": "trace-incomplete",
    }
    print(json.dumps(result, separators=(",", ":"), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
