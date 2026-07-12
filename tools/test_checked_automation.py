#!/usr/bin/env python3
"""Unit oracles for the bounds-v1 checked-automation promotion policy."""

from __future__ import annotations

import argparse
import copy
import json
import sys
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import codegen_parity as parity  # noqa: E402

PROMOTED_SOURCE = "experiments/port-study/base64/b64.xl"
PROMOTED_DIGEST = "e3abbce3c8d1b24eba3471c5c0ea6800418d38c3d3d97ce64b7f15d2abfb9764"


def site(**changes):
    record = {
        "function": "probe",
        "site": 0,
        "status": "retained",
        "proof": None,
        "kind": "buffer",
        "target": "out",
        "index": "i",
        "obligation_analysis": {
            "scope": parity.CHECKED_AUTOMATION_SCOPE,
            "complete": True,
            "analyzers": list(parity.CHECKED_AUTOMATION_ANALYZERS),
        },
        "obligation": None,
        "obligation_status": "not-applicable",
        "obligation_exactness": None,
        "requirement_relation": "not-applicable",
        "first_missing_fact": None,
        "first_failed_premise": None,
    }
    record.update(changes)
    return record


def manifest(*, mode="gate", facts=True, elide_bounds=False, approvals=None):
    return {
        "schema": 1,
        "checked_automation": {
            "schema": 1,
            "policy": parity.CHECKED_AUTOMATION_SCOPE,
            "roots": [
                {
                    "case": "root",
                    "variant": "facts",
                    "function": "encode",
                    "source": PROMOTED_SOURCE,
                    "source_sha256": PROMOTED_DIGEST,
                    "scope": "closed-unit",
                }
            ],
            "approvals": [] if approvals is None else approvals,
        },
        "cases": [
            {
                "id": "root",
                "mode": mode,
                "variants": [
                    {
                        "name": "facts",
                        "kind": "xlang",
                        "source": PROMOTED_SOURCE,
                        "function": "encode",
                        "facts": facts,
                        "elide_bounds": elide_bounds,
                    }
                ],
                "checks": [{"left": "facts.proof.proved", "value": 0}],
            }
        ],
    }


class CheckedAutomationClassifierTests(unittest.TestCase):
    def disposition(self, record):
        summary = parity.checked_automation_summary([record])
        nonzero = [
            name for name, count in summary["disposition_counts"].items() if count
        ]
        self.assertEqual(len(nonzero), 1)
        return nonzero[0], summary

    def test_proved_site_is_automatically_accounted(self):
        disposition, summary = self.disposition(
            site(status="proved", proof="dominating-guard")
        )
        self.assertEqual(disposition, "automatically-accounted")
        self.assertTrue(summary["ready"])

    def test_retained_affirmative_not_applicable_is_intrinsic_dynamic(self):
        disposition, summary = self.disposition(site())
        self.assertEqual(disposition, "intrinsic-dynamic")
        self.assertTrue(summary["ready"])

    def test_missing_and_mismatched_derived_facts_are_hard_findings(self):
        for relation in ("missing", "mismatch"):
            with self.subTest(relation=relation):
                disposition, summary = self.disposition(site(
                    obligation="output-capacity-lockstep",
                    obligation_status="derived",
                    obligation_exactness="exact",
                    requirement_relation=relation,
                    first_missing_fact={"family": "output-capacity-lockstep"},
                    first_failed_premise=(
                        None if relation == "missing" else {"reason": "wrong-relation"}
                    ),
                ))
                self.assertEqual(disposition, "hard-finding")
                self.assertFalse(summary["ready"])

    def test_indeterminate_states_fail_closed(self):
        records = {
            "incomplete": site(
                obligation_analysis={"scope": None, "complete": False, "analyzers": []}
            ),
            "incomplete-proved": site(
                status="proved",
                proof="dominating-guard",
                obligation_analysis={"scope": None, "complete": False, "analyzers": []},
                obligation_status="unknown",
                obligation_exactness="unknown",
                requirement_relation="unknown",
            ),
            "failed-premise": site(
                obligation="output-capacity-lockstep",
                obligation_status="failed-premise",
                obligation_exactness="unknown",
                requirement_relation="unknown",
                first_failed_premise={"reason": "unsupported-body"},
            ),
            "unknown": site(
                obligation_status="unknown",
                obligation_exactness="unknown",
                requirement_relation="unknown",
            ),
            "matched-but-retained": site(
                obligation="output-capacity-lockstep",
                obligation_status="derived",
                obligation_exactness="exact",
                requirement_relation="equivalent",
            ),
            "ceiling": site(status="ceiling"),
        }
        for name, record in records.items():
            with self.subTest(name=name):
                disposition, summary = self.disposition(record)
                self.assertEqual(disposition, "unaccounted")
                self.assertFalse(summary["ready"])


class CheckedAutomationManifestTests(unittest.TestCase):
    def test_default_manifest_matches_separately_review_pinned_roots(self):
        candidate = json.loads((ROOT / "codegen-parity.json").read_text())
        parity.validate_promotion_authority(candidate)

        deleted = copy.deepcopy(candidate)
        del deleted["checked_automation"]
        with self.assertRaises(parity.HarnessError):
            parity.validate_promotion_authority(deleted)

        repointed = copy.deepcopy(candidate)
        repointed["checked_automation"]["roots"][0]["case"] = "scalar-backend-parity"
        with self.assertRaises(parity.HarnessError):
            parity.validate_promotion_authority(repointed)

    def test_policy_oracle_sources_and_expectations_match_pinned_digest(self):
        parity.validate_policy_oracle_authority()
        self.assertEqual(
            len(parity.policy_oracle_descriptors()),
            parity.REVIEW_PINNED_POLICY_ORACLE_COUNT,
        )

    def test_valid_root_acquires_only_internal_marker(self):
        cases = parity.validate_manifest(manifest(), activate_promotion=True)
        self.assertEqual(
            cases[0]["_checked_automation_root"],
            {
                "case": "root",
                "variant": "facts",
                "function": "encode",
                "source": PROMOTED_SOURCE,
                "source_sha256": PROMOTED_DIGEST,
                "scope": "closed-unit",
            },
        )

    def test_policy_is_diagnostic_until_authenticated_promotion_activation(self):
        cases = parity.validate_manifest(manifest())
        self.assertNotIn("_checked_automation_root", cases[0])

    def test_promotion_invocation_rejects_every_partial_run(self):
        base = argparse.Namespace(
            promotion=True,
            manifest=parity.DEFAULT_MANIFEST,
            corpus=True,
            cases=None,
            tags=None,
            gate_only=False,
            audit_only=False,
        )
        parity.validate_promotion_invocation(base)
        mutations = {
            "external-manifest": {"manifest": ROOT / "other.json"},
            "missing-corpus": {"corpus": False},
            "case-filter": {"cases": ["root"]},
            "tag-filter": {"tags": ["bounds"]},
            "gate-filter": {"gate_only": True},
            "audit-filter": {"audit_only": True},
        }
        for name, changes in mutations.items():
            with self.subTest(name=name):
                candidate = copy.copy(base)
                for field, value in changes.items():
                    setattr(candidate, field, value)
                with self.assertRaises(parity.HarnessError):
                    parity.validate_promotion_invocation(candidate)

    def test_no_policy_does_not_infer_promotion_from_gate_mode(self):
        candidate = manifest()
        del candidate["checked_automation"]
        cases = parity.validate_manifest(candidate)
        self.assertNotIn("_checked_automation_root", cases[0])

    def test_writer_cannot_supply_internal_marker(self):
        candidate = manifest()
        candidate["cases"][0]["_checked_automation_root"] = {}
        with self.assertRaises(parity.HarnessError):
            parity.validate_manifest(candidate)

    def test_root_rejects_facts_off_ceiling_audit_and_approval_shortcuts(self):
        candidates = {
            "facts-off": manifest(facts=False),
            "ceiling": manifest(elide_bounds=True),
            "audit": manifest(mode="audit"),
            "approval": manifest(approvals=[{"allow": "all"}]),
            "non-string-digest": manifest(),
        }
        candidates["non-string-digest"]["checked_automation"]["roots"][0][
            "source_sha256"
        ] = 7
        for name, candidate in candidates.items():
            with self.subTest(name=name), self.assertRaises(parity.HarnessError):
                parity.validate_manifest(copy.deepcopy(candidate))

    def test_root_verdict_is_automatic_not_a_writer_expected_value(self):
        case = parity.validate_manifest(manifest(), activate_promotion=True)[0]
        compiled_metrics = {
            "proof.proved": 0,
            "proof.checked_automation_scope": parity.CHECKED_AUTOMATION_SCOPE,
            "proof.checked_automation_module_ready": False,
            "proof.checked_automation_module_disposition_counts": {
                "automatically-accounted": 0,
                "intrinsic-dynamic": 0,
                "hard-finding": 1,
                "unaccounted": 0,
            },
            "proof.checked_automation_module_findings_by_site": {
                "probe:0": {"reason": "missing-requirement"}
            },
        }
        with mock.patch.object(
                parity, "compile_variant", return_value=compiled_metrics):
            result = parity.run_case(case, ROOT, object())
        self.assertTrue(result["checks"][0]["passed"])
        self.assertFalse(result["passed"])
        self.assertEqual(parity.result_exit_code([result]), 1)
        self.assertEqual(
            [check["passed"] for check in result["checks"][-3:]],
            [True, False, False],
        )

    def test_root_compile_error_is_not_a_policy_finding(self):
        case = parity.validate_manifest(manifest(), activate_promotion=True)[0]
        for error in (RuntimeError("broken compiler"), SystemExit("unsupported")):
            with self.subTest(error=type(error).__name__), mock.patch.object(
                    parity, "compile_variant", side_effect=error):
                result = parity.run_case(case, ROOT, object())
                self.assertEqual(
                    result["error_class"], "checked-automation-evaluation-error"
                )
                self.assertNotIn("checked_automation", result)
                self.assertEqual(parity.result_exit_code([result]), 2)

    def test_non_root_compiler_error_is_also_exit_two(self):
        result = {"id": "corpus", "mode": "gate", "passed": False, "error": "bad"}
        self.assertEqual(parity.result_exit_code([result]), 2)


class CheckedAutomationReportTests(unittest.TestCase):
    def test_capacity_family_cannot_claim_not_applicable(self):
        invalid = site(obligation="output-capacity-lockstep")
        with self.assertRaises(parity.HarnessError):
            parity.validate_proof_report([invalid])

    def test_unknown_duplicate_and_unsorted_analyzer_sets_are_rejected(self):
        analyzer_sets = [
            ["unknown-v1"],
            ["output-capacity-lockstep-v1", "output-capacity-lockstep-v1"],
            ["z-v1", "a-v1"],
        ]
        for analyzers in analyzer_sets:
            with self.subTest(analyzers=analyzers):
                invalid = site(obligation_analysis={
                    "scope": parity.CHECKED_AUTOMATION_SCOPE,
                    "complete": True,
                    "analyzers": analyzers,
                })
                with self.assertRaises(parity.HarnessError):
                    parity.validate_proof_report([invalid])

    def test_whole_module_closure_catches_helper_debt(self):
        root_site = site(status="proved", proof="dominating-guard")
        helper_site = site(
            function="helper",
            obligation="output-capacity-lockstep",
            obligation_status="derived",
            obligation_exactness="exact",
            requirement_relation="missing",
            first_missing_fact={"family": "output-capacity-lockstep"},
        )
        llvm = "define void @probe() {\n  ret void\n}\n"
        assembly = "probe:\n  ret\n"
        measured = parity.metrics(
            llvm, llvm, assembly, "", "probe", [root_site, helper_site]
        )
        self.assertTrue(measured["proof.checked_automation_ready"])
        self.assertFalse(measured["proof.checked_automation_module_ready"])

    def test_fn4_generated_indexes_do_not_inherit_complete_analysis(self):
        democ = parity.load_democ()
        report = []
        source = (ROOT / "prototype/democ/examples/sat_reduce.xl").read_text()
        democ.compile_program(source, alias=True, proof_report=report)
        parity.validate_proof_report(report)
        generated = report[:4]
        self.assertEqual(
            [entry["index"] for entry in generated], ["i", "ra0i1", "ra0i2", "ra0i3"]
        )
        self.assertTrue(all(
            not entry["obligation_analysis"]["complete"] for entry in generated
        ))
        summary = parity.checked_automation_summary(report)
        self.assertFalse(summary["ready"])
        self.assertGreaterEqual(summary["disposition_counts"]["unaccounted"], 4)

    def test_indexed_borrow_is_lowered_checked_and_reported(self):
        democ = parity.load_democ()
        report = []
        source = (ROOT / (
            "codegen-corpus/cases/bounds/output-capacity-lockstep/"
            "n33-indexed-borrow-output-escape.xl"
        )).read_text()
        llvm = democ.compile_program(source, alias=True, proof_report=report)
        parity.validate_proof_report(report)
        probe_sites = [entry for entry in report if entry["function"] == "probe"]
        self.assertEqual(len(probe_sites), 1)
        self.assertEqual(probe_sites[0]["target"], "out")
        self.assertEqual(probe_sites[0]["index"], "o")
        self.assertIn("icmp ult i64", llvm)
        self.assertIn("getelementptr i8", llvm)
        self.assertIn("call void @llvm.trap", llvm)

    def test_unreachable_index_is_not_a_lowered_report_origin(self):
        democ = parity.load_democ()
        source = """
fn probe (b: own buffer<u8>) -> own u8 traps {
  return 0_u8;
  let dead: own u8 = index<u8>(b, 0_u64);
}
"""
        report = []
        with_report = democ.compile_program(source, alias=True, proof_report=report)
        without_report = democ.compile_program(source, alias=True)
        self.assertEqual(report, [])
        self.assertEqual(with_report, without_report)

        nested = """fn probe (b: own buffer<u8>, flag: own Bool) -> own u8 traps {
  loop @once {
    match flag {
      True() => {
        break @once;
        let dead_break: own u8 = index<u8>(b, 0_u64);
      }
      False() => {
        let live: own u8 = index<u8>(b, 0_u64);
        break @once;
      }
    }
    let dead_after_match: own u8 = index<u8>(b, 0_u64);
  }
  return 0_u8;
  let dead_return: own u8 = index<u8>(b, 0_u64);
}
"""
        nested_report = []
        nested_with_report = democ.compile_program(
            nested, alias=True, proof_report=nested_report
        )
        nested_without_report = democ.compile_program(nested, alias=True)
        parity.validate_proof_report(nested_report)
        self.assertEqual(len(nested_report), 1)
        self.assertEqual(
            (nested_report[0]["function"], nested_report[0]["site"],
             nested_report[0]["target"]),
            ("probe", 0, "b"),
        )
        self.assertEqual(nested_with_report, nested_without_report)


if __name__ == "__main__":
    unittest.main(verbosity=2)
