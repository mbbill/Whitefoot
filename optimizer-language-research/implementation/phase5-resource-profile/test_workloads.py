from hashlib import sha256
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from workloads import MAX_WORKLOAD_UNITS, WorkloadError, build, manifest


REPOSITORY = Path(__file__).resolve().parents[3]
MANIFEST = (
    REPOSITORY
    / "optimizer-language-research"
    / "implementation"
    / "phase5-resource-profile"
    / "frontend-observer"
    / "Cargo.toml"
)


def observe(source: bytes) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="whitefoot-resource-workload-") as directory:
        path = Path(directory) / "workload.wf"
        path.write_bytes(source)
        completed = subprocess.run(
            (
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "--offline",
                "--manifest-path",
                str(MANIFEST),
                "--",
                str(path),
            ),
            cwd=REPOSITORY,
            check=True,
            capture_output=True,
            text=True,
        )
        return json.loads(completed.stdout)


class WorkloadTests(unittest.TestCase):
    def test_families_are_deterministic_and_canonical(self) -> None:
        expected = {
            "compiler": "0dbf10a13b9f0832b67c4cdd8d0736fa54218d651eaa3a92c7789f5c720c16f6",
            "codec": "e1f49227b64c29eb7725845d732513cfbf8de06bd9d7fb8b075dc5aa12324fcd",
        }
        for family, digest in expected.items():
            source = build(family, 2)
            self.assertEqual(sha256(source).hexdigest(), digest)
            report = observe(source)
            self.assertEqual(report["sources"], 1)
            self.assertEqual(report["source_bytes"], len(source))
            self.assertEqual(
                report["projected_mixed_elements"],
                int(report["production_nodes"]) - 1 + int(report["terminals"]),
            )

    def test_scale_adds_structure(self) -> None:
        for family in ("compiler", "codec"):
            small = observe(build(family, 1))
            large = observe(build(family, 3))
            self.assertGreater(int(large["source_bytes"]), int(small["source_bytes"]))
            self.assertGreater(
                int(large["production_nodes"]), int(small["production_nodes"])
            )
            self.assertGreater(
                int(large["projected_mixed_elements"]),
                int(small["projected_mixed_elements"]),
            )

    def test_manifest_is_neutral_canonical_and_source_bound(self) -> None:
        source = build("compiler", 2)
        encoded = manifest("compiler", 2, source)
        self.assertEqual(encoded[-1:], b"\n")
        decoded = json.loads(encoded)
        self.assertEqual(decoded["schema"], "whitefoot-resource-workload-v1")
        self.assertEqual(decoded["family"], "compiler")
        self.assertEqual(decoded["units"], 2)
        self.assertEqual(
            decoded["parameters"],
            [
                {"name": "name_decimal_width", "value": 6},
                {"name": "source_records", "value": 1},
                {"name": "unit_count", "value": 2},
            ],
        )
        self.assertEqual(decoded["sources"][0]["byte_length"], len(source))
        self.assertEqual(
            decoded["sources"][0]["sha256"], sha256(source).hexdigest()
        )
        self.assertEqual(
            set(decoded),
            {
                "schema",
                "family",
                "units",
                "generator_revision",
                "parameters",
                "sources",
            },
        )

    def test_invalid_family_and_zero_units_are_rejected(self) -> None:
        with self.assertRaises(WorkloadError):
            build("unknown", 1)
        with self.assertRaises(WorkloadError):
            build("compiler", 0)
        with self.assertRaises(WorkloadError):
            build("codec", 0)
        with self.assertRaises(WorkloadError):
            build("codec", MAX_WORKLOAD_UNITS + 1)


if __name__ == "__main__":
    unittest.main()
