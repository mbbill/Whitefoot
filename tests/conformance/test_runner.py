#!/usr/bin/env python3
"""Tests for compiler-independent conformance coverage plumbing."""

import hashlib
import tempfile
import unittest
from pathlib import Path

import runner


ROOT = Path(__file__).resolve().parent.parent.parent
SPEC = ROOT / runner.ACTIVE_SPEC
APPROVALS = ROOT / runner.APPROVALS


def copy_authorities(directory: Path) -> None:
    """Copy the active specification and the approval ledger it is pinned by."""
    active = directory / runner.ACTIVE_SPEC
    active.parent.mkdir(parents=True, exist_ok=True)
    active.write_bytes(SPEC.read_bytes())
    approvals = directory / runner.APPROVALS
    approvals.parent.mkdir(parents=True, exist_ok=True)
    approvals.write_bytes(APPROVALS.read_bytes())


class ActiveSpecificationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> None:
        copy_authorities(directory)
        (directory / "spec").mkdir(exist_ok=True)

    def make_active_repository(self, directory: Path) -> tuple[str, str]:
        """Normalize the copied fixture to a synthetic ACTIVE state.

        The real stable file may legitimately be a declared candidate — that
        is the point of candidate mode — so a test that needs an ACTIVE
        fixture must build one instead of assuming the working tree's state:
        rewrite whatever status line the copy carries to a fresh synthetic
        ACTIVE version and append a chain record naming the rewritten bytes,
        the same accepted-extension pattern
        test_expected_identity_is_the_approval_chain_tail exercises.
        Returns the synthetic (version, digest).
        """
        active = directory / runner.ACTIVE_SPEC
        text = active.read_text()
        lines = text.split("\n")
        status_indexes = [
            i for i, line in enumerate(lines) if line.startswith("Status: ")
        ]
        self.assertTrue(status_indexes, "fixture spec has no status line")
        version = "v99.8"
        lines[status_indexes[0]] = f"Status: ACTIVE {version}"
        rewritten = "\n".join(lines).encode()
        active.write_bytes(rewritten)
        _, old_digest = runner.activation_chain_tail(directory)
        digest = hashlib.sha256(rewritten).hexdigest()
        approvals = directory / runner.APPROVALS
        approvals.write_bytes(
            approvals.read_bytes()
            + f"ACTIVE-SPEC: {version} {digest} {old_digest}\n".encode()
        )
        return version, digest

    def test_versioned_archive_cannot_change_coverage_authority(self):
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

    def test_missing_stable_active_specification_fails_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            (directory / runner.ACTIVE_SPEC).unlink()

            with self.assertRaises(FileNotFoundError):
                runner.spec_rule_ids(directory)

    def test_active_specification_digest_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            self.make_active_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            active.write_bytes(active.read_bytes() + b"\n")

            with self.assertRaisesRegex(
                ValueError, "active specification digest mismatch"
            ):
                runner.spec_rule_ids(directory)

    def test_expected_identity_is_the_approval_chain_tail(self):
        # The pin follows the ledger: appending a new activation record whose
        # digest names the (modified) spec bytes is accepted with no runner
        # edit, which is the point of reading the pin from the chain.
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            _, old_digest = runner.activation_chain_tail(directory)
            modified = active.read_bytes() + b"\n[ZZZ-1] appended rule.\n"
            active.write_bytes(modified)
            new_digest = hashlib.sha256(modified).hexdigest()
            approvals = directory / runner.APPROVALS
            approvals.write_bytes(
                approvals.read_bytes()
                + f"ACTIVE-SPEC: v99.0 {new_digest} {old_digest}\n".encode()
            )

            rules, _ = runner.spec_rule_ids(directory)

            self.assertIn("ZZZ-1", rules)

    def test_declared_candidate_superseding_the_chain_tail_is_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            version, digest = self.make_active_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            text = active.read_text()
            marker = f"Status: ACTIVE {version}"
            self.assertIn(marker, text)
            active.write_text(
                text.replace(
                    marker,
                    f"Status: CANDIDATE v99.9 supersedes {version} {digest}",
                    1,
                )
            )

            rules, name = runner.spec_rule_ids(directory)

            self.assertEqual(name, runner.ACTIVE_SPEC.name)
            self.assertIn("PROG-2", rules)

    def test_declared_candidate_with_wrong_supersedes_digest_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            version, _ = self.make_active_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            text = active.read_text()
            marker = f"Status: ACTIVE {version}"
            self.assertIn(marker, text)
            active.write_text(
                text.replace(
                    marker,
                    f"Status: CANDIDATE v99.9 supersedes {version} {'0' * 64}",
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "active specification digest mismatch"
            ):
                runner.spec_rule_ids(directory)

    def test_sub_rule_ids_are_recognized(self):
        # Forward compatibility for the migration that introduces `[FAM-N.Sk]`
        # sub-ids: the regex already reads them, so the corpus can cite them
        # the release they exist.
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            active = directory / runner.ACTIVE_SPEC
            _, old_digest = runner.activation_chain_tail(directory)
            modified = active.read_bytes() + b"\n[ENT-3.S10] sub-rule body.\n"
            active.write_bytes(modified)
            new_digest = hashlib.sha256(modified).hexdigest()
            approvals = directory / runner.APPROVALS
            approvals.write_bytes(
                approvals.read_bytes()
                + f"ACTIVE-SPEC: v99.0 {new_digest} {old_digest}\n".encode()
            )

            rules, _ = runner.spec_rule_ids(directory)

            self.assertIn("ENT-3.S10", rules)


class ManifestValidationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> Path:
        copy_authorities(directory)
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
        copy_authorities(directory)
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


class DeclaredVerdictDiffTests(unittest.TestCase):
    """The one population the adapter cannot see: a verdict that moves while
    its manifest row is edited to follow it, leaving the adapter green."""

    BEFORE = (
        '{"id": "a", "rules": ["OP-1"], "expect": {"kind": "reject", "rule": "OP-1"}, '
        '"status": "runnable", "doc": "d"}\n'
        '{"id": "b", "rules": ["FN-2"], "expect": {"kind": "run", "exit": 0}, '
        '"status": "runnable", "doc": "d"}\n'
        '{"id": "c", "rules": ["FN-8"], "expect": {"kind": "accept"}, '
        '"status": "runnable", "doc": "d"}\n'
        '{"rule": "GRAM-6", "covered_by": "policy", "reason": "r"}\n'
    )

    def test_a_moved_citation_is_reported_with_both_sides(self):
        after = self.BEFORE.replace('"rule": "OP-1"}', '"rule": "TYPE-5"}')
        before_map = runner.declared_verdicts(self.BEFORE)
        after_map = runner.declared_verdicts(after)
        moved = [k for k in before_map.keys() & after_map.keys()
                 if before_map[k] != after_map[k]]
        self.assertEqual(moved, ["a"])
        self.assertEqual(before_map["a"]["rule"], "OP-1")
        self.assertEqual(after_map["a"]["rule"], "TYPE-5")

    def test_an_annotation_row_is_not_a_case(self):
        # The GRAM-6 policy row states a rule and no id; reading it as a case
        # would invent a verdict that no case declares.
        self.assertEqual(sorted(runner.declared_verdicts(self.BEFORE)), ["a", "b", "c"])

    def test_a_changed_exit_status_counts_as_a_moved_verdict(self):
        # The whole expectation is compared, not just its rule, so a run case
        # whose expected exit moves is caught with the reject cases.
        after = self.BEFORE.replace('"exit": 0}', '"exit": 1}')
        before_map = runner.declared_verdicts(self.BEFORE)
        after_map = runner.declared_verdicts(after)
        self.assertNotEqual(before_map["b"], after_map["b"])

    def test_an_emptied_case_is_invisible_here_and_that_is_the_limit(self):
        # `b` keeps `run 0` whatever happens to its source. This check compares
        # DECLARED verdicts, so a positive whose subject was deleted -- the
        # emptied class -- produces nothing at all. Asserted rather than
        # documented, so the limit cannot quietly stop being true.
        before_map = runner.declared_verdicts(self.BEFORE)
        after_map = runner.declared_verdicts(self.BEFORE)
        moved = [k for k in before_map.keys() & after_map.keys()
                 if before_map[k] != after_map[k]]
        self.assertEqual(moved, [])
if __name__ == "__main__":
    unittest.main()
