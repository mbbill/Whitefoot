from hashlib import sha256
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class RunTests(unittest.TestCase):
    def test_cli_consumes_actual_producer_outputs_in_separate_process(self) -> None:
        route = Path(__file__).parent
        generator = route.parent / "workloads.py"
        cli = route / "run.py"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.wf"
            manifest = root / "manifest.json"
            receipt = root / "receipt.bin"
            summary = root / "summary.json"
            subprocess.run(
                (
                    sys.executable,
                    str(generator),
                    "--family",
                    "codec",
                    "--units",
                    "2",
                    "--output",
                    str(source),
                    "--manifest-output",
                    str(manifest),
                ),
                check=True,
                capture_output=True,
            )
            completed = subprocess.run(
                (
                    sys.executable,
                    str(cli),
                    "--manifest",
                    str(manifest),
                    "--source-file",
                    str(source),
                    "--output",
                    str(receipt),
                    "--summary-output",
                    str(summary),
                ),
                check=True,
                capture_output=True,
                text=True,
            )
            encoded = receipt.read_bytes()
            self.assertEqual(
                completed.stdout,
                f"bytes={len(encoded)} sha256={sha256(encoded).hexdigest()}\n",
            )
            summary_bytes = summary.read_bytes()
            parsed = json.loads(summary_bytes)
            self.assertEqual(parsed["receipt_sha256"], sha256(encoded).hexdigest())
            self.assertEqual(parsed["family"], "codec")
            self.assertEqual(parsed["units"], 2)
            self.assertEqual(
                parsed["generator_revision"],
                "4ecae9410fb82b62c2e8da595d944d29b1de8ae9f9c57983c1eaa393a1bc07d3",
            )
            source_bytes = source.read_bytes()
            self.assertEqual(
                parsed["sources"],
                [{
                    "byte_length": len(source_bytes),
                    "logical_path": "demand/codec-000002.wf",
                    "sha256": sha256(source_bytes).hexdigest(),
                }],
            )
            self.assertEqual(len(parsed["analytic_code_sha256"]), 64)
            self.assertEqual(
                parsed["analytic_code_files"],
                [
                    "dependency_audit.py", "manifest.py", "measure.py",
                    "receipt.py", "relation.py", "run.py", "selection.py",
                ],
            )
            self.assertEqual(len(parsed["fields"]), 33)
            self.assertEqual(
                parsed["fields"][8],
                {
                    "name": "max_lexical_scan_work",
                    "state": "unavailable",
                    "value": None,
                },
            )
            self.assertEqual(len(parsed["trace_gaps"]), 6)
            self.assertEqual(parsed["status"], "trace-incomplete")
            self.assertEqual(
                parsed["work_sha256"],
                "2d085436e8d9288a982ef83a13554c2310cead38892e8223d7f2661b60b3c7e7",
            )
            self.assertEqual(
                parsed["storage_sha256"],
                "6d624da13ddd48d6dd46f3a2feaac38b83b51e4154e0e70e08a73524e9e7505a",
            )
            self.assertEqual(
                summary_bytes,
                (json.dumps(parsed, ensure_ascii=True, allow_nan=False,
                            separators=(",", ":"), sort_keys=True) + "\n").encode("ascii"),
            )

    def test_cli_rejects_symlink_and_oversized_manifest_reads(self) -> None:
        route = Path(__file__).parent
        cli = route / "run.py"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target.json"
            target.write_bytes(b"{}\n")
            link = root / "link.json"
            os.symlink(target, link)
            source = root / "source.wf"
            source.write_bytes(b"x")
            output = root / "receipt.bin"
            symlink = subprocess.run(
                (
                    sys.executable,
                    str(cli),
                    "--manifest",
                    str(link),
                    "--source-file",
                    str(source),
                    "--output",
                    str(output),
                ),
                capture_output=True,
                text=True,
            )
            self.assertEqual(symlink.returncode, 1)
            self.assertIn("nonsymlink", symlink.stderr)

            oversized = root / "oversized.json"
            with oversized.open("wb") as stream:
                stream.truncate((1 << 20) + 1)
            too_large = subprocess.run(
                (
                    sys.executable,
                    str(cli),
                    "--manifest",
                    str(oversized),
                    "--source-file",
                    str(source),
                    "--output",
                    str(output),
                ),
                capture_output=True,
                text=True,
            )
            self.assertEqual(too_large.returncode, 1)
            self.assertIn("operational byte cap", too_large.stderr)
            self.assertFalse(output.exists())

    def test_cli_never_overwrites_an_input_with_an_output(self) -> None:
        route = Path(__file__).parent
        cli = route / "run.py"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "manifest.json"
            source = root / "source.wf"
            manifest.write_bytes(b"{}\n")
            source.write_bytes(b"x")
            original = manifest.read_bytes()
            completed = subprocess.run(
                (
                    sys.executable,
                    str(cli),
                    "--manifest",
                    str(manifest),
                    "--source-file",
                    str(source),
                    "--output",
                    str(manifest),
                ),
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 1)
            self.assertIn("must not alias an input", completed.stderr)
            self.assertEqual(manifest.read_bytes(), original)

    def test_selection_mode_fails_closed_before_publication(self) -> None:
        route = Path(__file__).parent
        generator = route.parent / "workloads.py"
        cli = route / "run.py"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.wf"
            manifest = root / "manifest.json"
            output = root / "receipt.bin"
            subprocess.run(
                (
                    sys.executable, str(generator), "--family", "compiler",
                    "--units", "1", "--output", str(source),
                    "--manifest-output", str(manifest),
                ),
                check=True,
                capture_output=True,
            )
            completed = subprocess.run(
                (
                    sys.executable, str(cli), "--manifest", str(manifest),
                    "--source-file", str(source), "--output", str(output),
                    "--selection-mode",
                ),
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 1)
            self.assertIn("exact algorithm trace is unavailable", completed.stderr)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
