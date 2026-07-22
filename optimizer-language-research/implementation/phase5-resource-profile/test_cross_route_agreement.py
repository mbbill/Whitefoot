from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from cross_route_agreement import (
    ANALYTIC_CODE_FILES,
    ANALYTIC_DERIVED_NAMES,
    AgreementError,
    EXPECTED_AUTHORITY_IDENTITIES,
    FIELD_NAMES,
    SCALE_POINTS,
    TRACE_FIELDS,
    WORKLOADS,
    _analytic_code_identity,
    _analytic_fields,
    _compare_profile_claims,
    _invoke,
    _read_json,
    _source_code_identity,
    _source_fields,
    _neutral_bundle_identity,
    _validate_case_bindings,
    _validate_route_identities,
    run_suite,
)


def _claims() -> tuple[dict[str, object], dict[str, object]]:
    source_rows = []
    analytic_rows = []
    for tag, name in enumerate(FIELD_NAMES, 1):
        if name in TRACE_FIELDS:
            source_rows.append(
                {"name": name, "reason": "trace evidence required", "state": "trace-required", "tag": tag}
            )
            analytic_rows.append({"name": name, "state": "unavailable", "value": None})
        else:
            source_rows.append({"name": name, "state": "exact", "tag": tag, "value": 1})
            analytic_rows.append({"name": name, "state": "available", "value": 1})
    derived = {name: 1 for name in ANALYTIC_DERIVED_NAMES}
    return (
        {"agreement_derived_counts": derived, "counts": source_rows},
        {
            "derived": [{"name": name, "value": 1} for name in ANALYTIC_DERIVED_NAMES],
            "fields": analytic_rows,
        },
    )


def _case() -> tuple[dict[str, object], dict[str, object], dict[str, object], bytes, bytes, bytes]:
    family = "compiler"
    units = 1
    source_bytes = b"x\n"
    source_digest = sha256(source_bytes).hexdigest()
    descriptor = {
        "byte_length": len(source_bytes),
        "logical_path": "compiler-000001.wf",
        "sha256": source_digest,
    }
    manifest = {
        "family": family,
        "generator_revision": sha256(WORKLOADS.read_bytes()).hexdigest(),
        "parameters": [
            {"name": "name_decimal_width", "value": 6},
            {"name": "source_records", "value": 1},
            {"name": "unit_count", "value": units},
        ],
        "schema": "whitefoot-resource-workload-v1",
        "sources": [descriptor],
        "units": units,
    }
    manifest_bytes = (
        json.dumps(manifest, ensure_ascii=True, separators=(",", ":"), sort_keys=True) + "\n"
    ).encode("ascii")
    manifest_digest = sha256(manifest_bytes).hexdigest()
    source_claims, analytic_claims = _claims()
    source_receipt = {
        **source_claims,
        "derived_counts": {},
        "identities": {
            "candidate_sha256": EXPECTED_AUTHORITY_IDENTITIES["candidate specification"],
            "meaning_files": {
                "SCHEMA-SEMANTICS.md": "1" * 64,
                "STORAGE-MODEL.md": "2" * 64,
                "WORK-SCHEDULE.md": "3" * 64,
            },
            "meaning_sha256": {
                "semantics": EXPECTED_AUTHORITY_IDENTITIES["semantics"],
                "storage": EXPECTED_AUTHORITY_IDENTITIES["storage"],
                "work": EXPECTED_AUTHORITY_IDENTITIES["work schedule"],
            },
            "parser_audit_sha256": "4" * 64,
            "profile_schema_sha256": "5" * 64,
            "proposal_sha256": EXPECTED_AUTHORITY_IDENTITIES["proposal"],
            "route_code_sha256": _source_code_identity(),
        },
        "projection_summary": {},
        "schema": "whitefoot-resource-source-route-receipt-v1",
        "selected_diagnostic": None,
        "source_bundle": {"sha256": "6" * 64, "sources": [descriptor]},
        "spelling_components": {},
        "status": "trace-incomplete",
        "trace_gaps": [
            {
                "allowed_inputs": ["trace"],
                "field": name,
                "required_replay": "replay",
                "tag": FIELD_NAMES.index(name) + 1,
            }
            for name in TRACE_FIELDS
        ],
        "workload": {
            "family": family,
            "manifest_sha256": manifest_digest,
            "units": units,
        },
    }
    analytic_receipt_bytes = b"analytic receipt"
    analytic_summary = {
        **analytic_claims,
        "analytic_code_files": list(ANALYTIC_CODE_FILES),
        "analytic_code_sha256": _analytic_code_identity(),
        "bundle_sha256": "7" * 64,
        "expected_diagnostic": ["Complete"],
        "family": family,
        "generator_revision": manifest["generator_revision"],
        "manifest_sha256": manifest_digest,
        "profile_semantics_sha256": EXPECTED_AUTHORITY_IDENTITIES["semantics"],
        "proposal_sha256": EXPECTED_AUTHORITY_IDENTITIES["proposal"],
        "receipt_sha256": sha256(analytic_receipt_bytes).hexdigest(),
        "schema": "whitefoot-resource-analytic-summary-v1",
        "source_sha256": [source_digest],
        "sources": [descriptor],
        "specification_sha256": EXPECTED_AUTHORITY_IDENTITIES["candidate specification"],
        "status": "trace-incomplete",
        "storage_sha256": EXPECTED_AUTHORITY_IDENTITIES["storage"],
        "trace_gaps": [{"field": name, "reason": "trace"} for name in TRACE_FIELDS],
        "units": units,
        "work_sha256": EXPECTED_AUTHORITY_IDENTITIES["work schedule"],
    }
    return (
        manifest,
        source_receipt,
        analytic_summary,
        manifest_bytes,
        source_bytes,
        analytic_receipt_bytes,
    )


class CrossRouteAgreementTests(unittest.TestCase):
    def test_both_routes_agree_in_separate_processes(self) -> None:
        reports = run_suite()
        self.assertEqual(len(reports), 6)
        self.assertEqual(
            {(report["family"], report["units"]) for report in reports},
            {
                (family, units)
                for family in ("compiler", "codec")
                for units in SCALE_POINTS
            },
        )
        for report in reports:
            self.assertEqual(report["available_fields"], 27)
            self.assertEqual(tuple(report["trace_fields"]), TRACE_FIELDS)
            self.assertEqual(len(report["agreement_sha256"]), 64)

    def test_rejects_altered_count(self) -> None:
        source, analytic = _claims()
        _compare_profile_claims(source, analytic)
        analytic["fields"][0]["value"] = 2
        with self.assertRaises(AgreementError):
            _compare_profile_claims(source, analytic)

    def test_rejects_missing_reordered_or_mistagged_fields(self) -> None:
        source, analytic = _claims()
        mutations = []
        missing = deepcopy(source)
        missing["counts"].pop()
        mutations.append(lambda: _source_fields(missing))
        reordered = deepcopy(source)
        reordered["counts"][0], reordered["counts"][1] = reordered["counts"][1], reordered["counts"][0]
        mutations.append(lambda: _source_fields(reordered))
        mistagged = deepcopy(source)
        mistagged["counts"][0]["tag"] = 2
        mutations.append(lambda: _source_fields(mistagged))
        analytic_reordered = deepcopy(analytic)
        analytic_reordered["fields"][0], analytic_reordered["fields"][1] = (
            analytic_reordered["fields"][1], analytic_reordered["fields"][0]
        )
        mutations.append(lambda: _analytic_fields(analytic_reordered))
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                with self.assertRaises(AgreementError):
                    mutation()

    def test_rejects_source_or_manifest_identity_mismatch(self) -> None:
        manifest, source, analytic, manifest_raw, source_raw, receipt_raw = _case()
        arguments = {
            "family": "compiler",
            "units": 1,
            "manifest_bytes": manifest_raw,
            "source_bytes": source_raw,
            "analytic_receipt_bytes": receipt_raw,
        }
        _validate_case_bindings(manifest, source, analytic, **arguments)
        bad_source = deepcopy(source)
        bad_source["workload"]["manifest_sha256"] = "f" * 64
        with self.assertRaises(AgreementError):
            _validate_case_bindings(manifest, bad_source, analytic, **arguments)
        bad_analytic = deepcopy(analytic)
        bad_analytic["sources"][0]["logical_path"] = "different.wf"
        with self.assertRaises(AgreementError):
            _validate_case_bindings(manifest, source, bad_analytic, **arguments)

    def test_neutral_bundle_identity_binds_order_path_length_and_digest(self) -> None:
        manifest, _, _, _, _, _ = _case()
        sources = manifest["sources"]
        self.assertIsInstance(sources, list)
        baseline = _neutral_bundle_identity(sources)
        mutations = []
        for key, value in (
            ("logical_path", "different.wf"),
            ("byte_length", sources[0]["byte_length"] + 1),
            ("sha256", "f" * 64),
        ):
            changed = deepcopy(sources)
            changed[0][key] = value
            mutations.append(changed)
        duplicated = deepcopy(sources)
        duplicated.append(deepcopy(sources[0]))
        mutations.append(duplicated)
        for changed in mutations:
            with self.subTest(changed=changed):
                self.assertNotEqual(_neutral_bundle_identity(changed), baseline)

    def test_rejects_changed_shared_and_route_code_identities(self) -> None:
        _, source, analytic, _, _, _ = _case()
        keys = (
            "proposal_sha256",
            "specification_sha256",
            "profile_semantics_sha256",
            "storage_sha256",
            "work_sha256",
            "analytic_code_sha256",
        )
        for key in keys:
            changed = deepcopy(analytic)
            changed[key] = "f" * 64
            with self.subTest(key=key):
                with self.assertRaises(AgreementError):
                    _validate_route_identities(source, changed)
        changed_source = deepcopy(source)
        changed_source["identities"]["route_code_sha256"] = "f" * 64
        with self.assertRaises(AgreementError):
            _validate_route_identities(changed_source, analytic)

    def test_rejects_malformed_receipt_and_summary(self) -> None:
        with self.assertRaises(AgreementError):
            _source_fields({"counts": "not rows"})
        with self.assertRaises(AgreementError):
            _analytic_fields({"fields": {}})
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            path.write_bytes(b'{"a":1,"a":2}\n')
            with self.assertRaises(AgreementError):
                _read_json(path, "malformed")
            path.write_bytes(b'{"a": 1}\n')
            with self.assertRaises(AgreementError):
                _read_json(path, "noncanonical")

    def test_rejects_subprocess_failure(self) -> None:
        failed = subprocess.CompletedProcess(("route",), 7, "", "failed")
        with patch("cross_route_agreement.subprocess.run", return_value=failed):
            with self.assertRaisesRegex(AgreementError, "subprocess failed"):
                _invoke(("route",))


if __name__ == "__main__":
    unittest.main()
