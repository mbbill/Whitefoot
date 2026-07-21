from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import run  # noqa: E402
from installed_policy import REVIEW_PACKET_BINDINGS, validate_review_packet  # noqa: E402
from runner_inputs import SUCCESSOR_SHA256  # noqa: E402


@unittest.skipUnless(
    (ROOT / "static-auditor" / "Cargo.toml").is_file()
    and (ROOT / "static-auditor" / "SOURCES").is_file()
    and (ROOT / "oracle" / "main.py").is_file()
    and (ROOT / "oracle" / "SOURCES").is_file(),
    "both independent engines must exist before the fresh-run differential",
)
class FreshRunTests(unittest.TestCase):
    def test_two_fresh_runs_are_byte_identical(self) -> None:
        validate_review_packet(ROOT)
        review_before = {
            name: (ROOT / "evidence" / name).read_bytes()
            for name in REVIEW_PACKET_BINDINGS
        }
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
            report = json.loads((first / "report.json").read_text(encoding="ascii"))
            self.assertEqual(report["schema"], "whitefoot.grammar-evidence.v2")
            self.assertEqual(report["installation"]["mode"], "installed-v0.9")
            self.assertEqual(report["installation"]["relation"], "byte-identical")
            self.assertEqual(
                report["installation"]["candidate"]["sha256"],
                SUCCESSOR_SHA256,
            )
            self.assertEqual(
                report["installation"]["installed_specification"]["sha256"],
                SUCCESSOR_SHA256,
            )
            self.assertEqual(report["inputs"]["proposal"]["sha256"], SUCCESSOR_SHA256)
        for name, before in review_before.items():
            with self.subTest(review_artifact=name):
                after = (ROOT / "evidence" / name).read_bytes()
                self.assertEqual(after, before)
                self.assertEqual(hashlib.sha256(after).hexdigest(), REVIEW_PACKET_BINDINGS[name])


if __name__ == "__main__":
    unittest.main()
