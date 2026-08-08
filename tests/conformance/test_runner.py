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
        active = directory / runner.ACTIVE_SPEC
        active.parent.mkdir(parents=True, exist_ok=True)
        active.write_bytes(SPEC.read_bytes())
        (directory / "spec").mkdir(exist_ok=True)

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
        active = directory / runner.ACTIVE_SPEC
        active.parent.mkdir(parents=True, exist_ok=True)
        active.write_bytes(SPEC.read_bytes())
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

    def test_unsupported_expectation_requires_a_why(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")
            case = self.case()
            case["expect"] = {"kind": "unsupported", "why": ""}

            with self.assertRaisesRegex(ValueError, "unsupported expectation requires"):
                runner.validate_manifest([case], [], directory, cases)


class ArrangementTests(unittest.TestCase):
    """The invocation arrangement a run/trap case needs (fixtures, argv, stdin,
    redirection). Byte strings are hex so non-UTF-8 argument and path values are
    expressible exactly."""

    def make_repository(self, directory: Path) -> Path:
        active = directory / runner.ACTIVE_SPEC
        active.parent.mkdir(parents=True, exist_ok=True)
        active.write_bytes(SPEC.read_bytes())
        cases = directory / "cases"
        cases.mkdir()
        return cases

    def case(self, arrange):
        return {
            "id": "sample",
            "rules": ["PROG-2"],
            "expect": {"kind": "run", "exit": 0},
            "arrange": arrange,
            "status": "runnable",
            "doc": "Sample arranged case.",
        }

    def validate(self, case):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cases = self.make_repository(directory)
            (cases / "sample.wf").write_text("")
            runner.validate_manifest([case], [], directory, cases)

    def test_complete_arrangement_is_valid(self):
        self.validate(
            self.case(
                {
                    "argv": ["77f0", ""],
                    "stdin": "0a",
                    "files": [
                        {"path": "61ff", "bytes": ""},
                        {"path": "62", "directory": True},
                    ],
                    "redirect": {"stdout": "combined", "stderr": "combined"},
                }
            )
        )

    def test_argument_bytes_must_be_lowercase_hex(self):
        with self.assertRaisesRegex(ValueError, "argv\\[0\\] must be a lowercase"):
            self.validate(self.case({"argv": ["FF"]}))

    def test_odd_length_byte_string_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "stdin must be a lowercase"):
            self.validate(self.case({"stdin": "abc"}))

    def test_unknown_arrangement_key_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown arrange keys: spawn"):
            self.validate(self.case({"spawn": "reader"}))

    def test_fixture_is_either_a_file_or_a_directory(self):
        with self.assertRaisesRegex(ValueError, "exactly one of bytes, directory"):
            self.validate(self.case({"files": [{"path": "61", "bytes": "", "directory": True}]}))

    def test_duplicate_fixture_paths_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate fixture paths"):
            self.validate(
                self.case({"files": [{"path": "61", "bytes": ""}, {"path": "61", "directory": True}]})
            )

    def test_a_case_that_never_executes_must_not_arrange_an_invocation(self):
        case = self.case({"argv": ["61"]})
        case["expect"] = {"kind": "accept"}

        with self.assertRaisesRegex(ValueError, "only a run or trap case"):
            self.validate(case)


class VerdictMatchingTests(unittest.TestCase):
    def test_unsupported_verdict_matches_an_unsupported_expectation(self):
        expect = {"kind": "unsupported", "why": "target qualification"}

        self.assertTrue(runner.matches(("unsupported", "no approved target"), expect))
        self.assertFalse(runner.matches(("accept",), expect))

    def test_unsupported_verdict_still_fails_every_other_expectation(self):
        self.assertFalse(runner.matches(("unsupported", "gap"), {"kind": "accept"}))


class CoverageTests(unittest.TestCase):
    def test_a_pending_case_still_supplies_rule_coverage(self):
        """Coverage measures the corpus against the specification, not what the
        current toolchain can run, so toolchain readiness must not silently
        remove a rule from the covered set."""
        cases, annotations = runner.load_manifest()
        pending = [case for case in cases if case.get("status") == "pending"]
        self.assertTrue(pending)

        _, _, covered, by_case, _, _, _, _ = runner.coverage(cases, annotations)

        pending_only = set()
        for case in pending:
            pending_only |= set(case["rules"])
        for case in cases:
            if case.get("status") != "pending":
                pending_only -= set(case["rules"])
        self.assertTrue(pending_only)
        self.assertTrue(pending_only <= by_case)
        self.assertTrue(pending_only <= covered)


if __name__ == "__main__":
    unittest.main()
