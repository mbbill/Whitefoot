from hashlib import sha256
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


sys.path.insert(0, str(Path(__file__).parent))

from model import RouteError  # noqa: E402
from receipt import decode_receipt, encode_receipt  # noqa: E402
import run as route_cli  # noqa: E402


class RunTests(unittest.TestCase):
    def _generate(self, root: Path) -> tuple[Path, Path, str]:
        route = Path(__file__).parent
        producer = route.parent / "workloads.py"
        source = root / "source.wf"
        manifest = root / "manifest.json"
        subprocess.run(
            (
                sys.executable,
                "-B",
                str(producer),
                "--family",
                "compiler",
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
        logical_path = json.loads(manifest.read_bytes())["sources"][0]["logical_path"]
        return source, manifest, logical_path

    def _command(
        self, source: Path, manifest: Path, logical_path: str, output: Path
    ) -> tuple[str, ...]:
        return (
            sys.executable,
            "-B",
            str(Path(__file__).parent / "run.py"),
            "--logical-path",
            logical_path,
            "--source-file",
            str(source),
            "--manifest",
            str(manifest),
            "--output",
            str(output),
        )

    def test_cli_publishes_only_a_strict_canonical_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, manifest, logical_path = self._generate(root)
            output = root / "receipt.json"
            output.write_bytes(b"old incomplete output")
            completed = subprocess.run(
                self._command(source, manifest, logical_path, output),
                check=True,
                capture_output=True,
                text=True,
            )
            encoded = output.read_bytes()
            self.assertEqual(encode_receipt(decode_receipt(encoded)), encoded)
            self.assertEqual(
                completed.stdout,
                f"bytes={len(encoded)} sha256={sha256(encoded).hexdigest()}\n",
            )

    def test_cli_rejects_missing_manifest_symlink_and_input_alias(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, manifest, logical_path = self._generate(root)
            missing = subprocess.run(
                tuple(
                    argument
                    for argument in self._command(
                        source, manifest, logical_path, root / "receipt.json"
                    )
                    if argument not in ("--manifest", str(manifest))
                ),
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn("--manifest", missing.stderr)

            link = root / "manifest-link.json"
            os.symlink(manifest, link)
            symlink = subprocess.run(
                self._command(source, link, logical_path, root / "receipt.json"),
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(symlink.returncode, 0)
            self.assertIn("nonsymlink", symlink.stderr)

            original = manifest.read_bytes()
            alias = subprocess.run(
                self._command(source, manifest, logical_path, manifest),
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(alias.returncode, 0)
            self.assertIn("must not alias", alias.stderr)
            self.assertEqual(manifest.read_bytes(), original)

            hardlink = root / "hardlink-receipt.json"
            os.link(manifest, hardlink)
            hardlink_alias = subprocess.run(
                self._command(source, manifest, logical_path, hardlink),
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(hardlink_alias.returncode, 0)
            self.assertIn("must not alias", hardlink_alias.stderr)
            self.assertEqual(hardlink.read_bytes(), original)

    def test_bounded_read_detects_metadata_race(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input"
            path.write_bytes(b"stable bytes")
            real_fstat = os.fstat
            calls = 0

            def changed_after_read(descriptor: int) -> object:
                nonlocal calls
                calls += 1
                observed = real_fstat(descriptor)
                if calls == 1:
                    return observed
                values = {
                    name: getattr(observed, name)
                    for name in (
                        "st_dev",
                        "st_ino",
                        "st_mode",
                        "st_size",
                        "st_mtime_ns",
                        "st_ctime_ns",
                    )
                }
                values["st_ctime_ns"] += 1
                return type("ChangedStat", (), values)()

            with mock.patch.object(
                route_cli.os, "fstat", side_effect=changed_after_read
            ):
                with self.assertRaisesRegex(RouteError, "changed during"):
                    route_cli._read_bounded(path, 1024, "test input")

    def test_atomic_writer_enforces_its_own_bound(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "receipt.json"
            output.write_bytes(b"preserved")
            with self.assertRaisesRegex(RouteError, "output cap"):
                route_cli._write_atomic(output, b"too large", 4, ())
            self.assertEqual(output.read_bytes(), b"preserved")

    def test_decoder_rejects_duplicate_keys_and_forged_source_count(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, manifest, logical_path = self._generate(root)
            output = root / "receipt.json"
            subprocess.run(
                self._command(source, manifest, logical_path, output),
                check=True,
                capture_output=True,
            )
            encoded = output.read_bytes()
            duplicate = encoded.replace(b'{"agreement_derived_counts"', b'{"schema":"duplicate","agreement_derived_counts"', 1)
            with self.assertRaises(RouteError):
                decode_receipt(duplicate)
            value = json.loads(encoded)
            value["counts"][2]["value"] += 1
            forged = (
                json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
                + "\n"
            ).encode("ascii")
            with self.assertRaises(RouteError):
                decode_receipt(forged)


if __name__ == "__main__":
    unittest.main()
