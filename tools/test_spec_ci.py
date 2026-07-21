#!/usr/bin/env python3
"""Hostile tests for the exact active-specification binding in spec_ci.py."""

import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "tools" / "spec_ci.py"
SPEC = ROOT / "spec" / "kernel-spec-v0.9.md"
LEDGER = ROOT / "spec" / "derivation-ledger.md"


class ActiveSpecificationTests(unittest.TestCase):
    def make_repository(self, directory: Path) -> Path:
        tools = directory / "tools"
        spec = directory / "spec"
        tools.mkdir()
        spec.mkdir()
        (tools / "spec_ci.py").write_bytes(SCRIPT.read_bytes())
        (spec / "kernel-spec-v0.9.md").write_bytes(SPEC.read_bytes())
        (spec / "derivation-ledger.md").write_bytes(LEDGER.read_bytes())
        return tools / "spec_ci.py"

    def run_script(self, script: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", "-B", str(script)],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_higher_version_lookalike_cannot_change_active_specification(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            script = self.make_repository(directory)
            (directory / "spec" / "kernel-spec-v99.0.md").write_text(
                "[FAKE-1] not authority\n"
            )

            result = self.run_script(script)

            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("kernel-spec-v0.9.md", result.stdout)
            self.assertNotIn("kernel-spec-v99.0.md", result.stdout)

    def test_active_specification_digest_is_exact(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            script = self.make_repository(directory)
            active = directory / "spec" / "kernel-spec-v0.9.md"
            active.write_bytes(active.read_bytes() + b"\n")

            result = self.run_script(script)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("active specification digest mismatch", result.stdout)


if __name__ == "__main__":
    unittest.main()
