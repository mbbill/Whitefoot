from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import run  # noqa: E402


@unittest.skipUnless(
    (ROOT / "static-auditor" / "Cargo.toml").is_file()
    and (ROOT / "static-auditor" / "SOURCES").is_file()
    and (ROOT / "oracle" / "main.py").is_file()
    and (ROOT / "oracle" / "SOURCES").is_file(),
    "both independent engines must exist before the fresh-run differential",
)
class FreshRunTests(unittest.TestCase):
    def test_two_fresh_runs_are_byte_identical(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "first"
            second = Path(directory) / "second"
            self.assertEqual(run.run_repository(first), run.run_repository(second))
            names = (
                "static.raw",
                "oracle.raw",
                "report.json",
                "report.sha256",
                "proposal-delta.md",
                "protected-surface-census.json",
                "package.json",
                "package.sha256",
            )
            for name in names:
                with self.subTest(name=name):
                    self.assertEqual((first / name).read_bytes(), (second / name).read_bytes())


if __name__ == "__main__":
    unittest.main()
