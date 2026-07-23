#!/usr/bin/env python3
"""Tests for compiler-independent conformance coverage plumbing."""

import tempfile
import unittest
from pathlib import Path

import runner


ROOT = Path(__file__).resolve().parent.parent.parent
SPEC = ROOT / runner.ACTIVE_SPEC


class ActiveSpecificationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> None:
        spec = directory / "spec"
        spec.mkdir()
        (spec / runner.ACTIVE_SPEC.name).write_bytes(SPEC.read_bytes())

    def test_higher_version_lookalike_cannot_change_coverage_authority(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            (directory / "spec" / "kernel-spec-v99.0.md").write_text(
                "[FAKE-1] not authority\n"
            )

            rules, name = runner.spec_rule_ids(directory)

            self.assertEqual(name, runner.ACTIVE_SPEC.name)
            self.assertIn("PROG-2", rules)
            self.assertNotIn("FAKE-1", rules)

    def test_active_specification_digest_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            active.write_bytes(active.read_bytes() + b"\n")

            with self.assertRaisesRegex(
                ValueError, "active specification digest mismatch"
            ):
                runner.spec_rule_ids(directory)


class ManifestValidationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> Path:
        spec = directory / "spec"
        spec.mkdir()
        (spec / runner.ACTIVE_SPEC.name).write_bytes(SPEC.read_bytes())
        cases = directory / "cases"
        cases.mkdir()
        return cases

    def case(self):
        return {
            "id": "sample",
            "rules": ["PROG-2"],
            "expect": {"kind": "accept"},
            "status": "runnable",
            "doc": "Sample structural case.",
        }

    def test_repository_manifest_and_sources_are_consistent(self):
        cases, annotations = runner.load_manifest()
        runner.validate_manifest(cases, annotations)

    def test_paired_source_is_valid(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")

            runner.validate_manifest([self.case()], [], directory, cases)

    def test_orphan_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")
            (cases / "orphan.wf").write_text("")

            with self.assertRaisesRegex(ValueError, "orphan case sources"):
                runner.validate_manifest([self.case()], [], directory, cases)

    def test_reject_rule_must_be_declared_by_case(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")
            case = self.case()
            case["expect"] = {"kind": "reject", "rule": "TYPE-6"}

            with self.assertRaisesRegex(ValueError, "reject rule"):
                runner.validate_manifest([case], [], directory, cases)

    def test_expectation_fields_must_match_the_declared_kind(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")
            case = self.case()
            case["expect"] = {"kind": "accept", "run": {"exit": 0}}

            with self.assertRaisesRegex(
                ValueError, "accept expectation fields must be exactly"
            ):
                runner.validate_manifest([case], [], directory, cases)


if __name__ == "__main__":
    unittest.main()
