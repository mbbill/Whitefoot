#!/usr/bin/env python3
"""Tests for compiler-independent conformance coverage plumbing."""

import tempfile
import unittest
from pathlib import Path

import runner


ROOT = Path(__file__).resolve().parent.parent.parent
SPEC = ROOT / "spec" / "kernel-spec-v0.10.md"


class ActiveSpecificationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> None:
        spec = directory / "spec"
        spec.mkdir()
        (spec / "kernel-spec-v0.10.md").write_bytes(SPEC.read_bytes())

    def test_higher_version_lookalike_cannot_change_coverage_authority(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            (directory / "spec" / "kernel-spec-v99.0.md").write_text(
                "[FAKE-1] not authority\n"
            )

            rules, name = runner.spec_rule_ids(directory)

            self.assertEqual(name, "kernel-spec-v0.10.md")
            self.assertIn("PROG-2", rules)
            self.assertNotIn("FAKE-1", rules)

    def test_active_specification_digest_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.make_repository(directory)
            active = directory / "spec" / "kernel-spec-v0.10.md"
            active.write_bytes(active.read_bytes() + b"\n")

            with self.assertRaisesRegex(
                ValueError, "active specification digest mismatch"
            ):
                runner.spec_rule_ids(directory)


if __name__ == "__main__":
    unittest.main()
