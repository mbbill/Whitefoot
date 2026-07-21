"""Source-level mutation gates for both independent grammar engines."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Iterator
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from installed_policy import HISTORICAL_V08_BINDINGS  # noqa: E402

CURRENT_SHA256 = "d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8"
SUBPROCESS_TIMEOUT_SECONDS = 180


@contextmanager
def mutant_copy() -> Iterator[Path]:
    with tempfile.TemporaryDirectory(prefix="whitefoot-source-mutant-") as directory:
        temporary = Path(directory)
        verifier = temporary / "grammar-verifier"
        shutil.copytree(
            ROOT,
            verifier,
            ignore=shutil.ignore_patterns(
                "target",
                "evidence",
                "installed-v0.9-evidence",
                "__pycache__",
                ".pytest_cache",
                ".mypy_cache",
                ".ruff_cache",
                "*.pyc",
            ),
        )
        shutil.copytree(ROOT / "evidence", verifier / "evidence")
        current = (REPOSITORY / "spec" / "kernel-spec-v0.8.md").read_bytes()
        if hashlib.sha256(current).hexdigest() != CURRENT_SHA256:
            raise AssertionError("the mutation gate requires exact v0.8 specification bytes")
        specification = temporary / "spec"
        specification.mkdir()
        (specification / "kernel-spec-v0.8.md").write_bytes(current)
        installed = (REPOSITORY / "spec" / "kernel-spec-v0.9.md").read_bytes()
        candidate = (ROOT / "proposal" / "kernel-spec-successor-candidate.md").read_bytes()
        if installed != candidate:
            raise AssertionError("the mutation gate requires exact installed v0.9 bytes")
        (specification / "kernel-spec-v0.9.md").write_bytes(installed)
        shutil.copyfile(
            REPOSITORY / "spec" / "derivation-ledger.md",
            specification / "derivation-ledger.md",
        )
        for relative in HISTORICAL_V08_BINDINGS:
            destination = temporary / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPOSITORY / relative, destination)
        yield verifier


def replace_unique(path: Path, original: bytes, replacement: bytes) -> None:
    source = path.read_bytes()
    count = source.count(original)
    if count != 1:
        raise AssertionError(
            f"mutation anchor in {path.name} occurs {count} times instead of once"
        )
    path.write_bytes(source.replace(original, replacement, 1))


class SourceMutantTests(unittest.TestCase):
    def assert_full_run_rejects(
        self,
        relative: str,
        original: bytes,
        replacement: bytes,
        error_code: bytes,
    ) -> None:
        with mutant_copy() as verifier:
            replace_unique(verifier / relative, original, replacement)
            completed = subprocess.run(
                (sys.executable, "-I", "-S", "-B", str(verifier / "run.py")),
                cwd=verifier,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=SUBPROCESS_TIMEOUT_SECONDS,
            )
        self.assertNotEqual(completed.returncode, 0)
        self.assertEqual(completed.stdout, b"")
        self.assertIn(b"grammar verifier: " + error_code + b":", completed.stderr)

    def test_oracle_current_hash_pin_is_observed_by_the_full_run(self) -> None:
        self.assert_full_run_rejects(
            "oracle/core.py",
            b'CURRENT_SHA256 = "d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8"\n',
            b'CURRENT_SHA256 = "004336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8"\n',
            b"engine_outcome",
        )

    def test_oracle_rule_ledger_omission_causes_engine_disagreement(self) -> None:
        self.assert_full_run_rejects(
            "oracle/report.py",
            b"    for rule in grammar.rules:\n",
            b"    for rule in grammar.rules[1:]:\n",
            b"engine_disagreement",
        )

    def test_oracle_fixed_lowerword_exclusion_is_required_by_cases(self) -> None:
        self.assert_full_run_rejects(
            "oracle/lexical.py",
            b"            if self.exclude_fixed_lowerwords and spelling in fixed_lowerwords:\n",
            b"            if False and self.exclude_fixed_lowerwords and spelling in fixed_lowerwords:\n",
            b"oracle_expectations",
        )

    def test_oracle_two_tree_saturation_is_required_by_expectations(self) -> None:
        self.assert_full_run_rejects(
            "oracle/generalized.py",
            b"            ) or len(cell) >= 2:\n",
            b"            ) or len(cell) >= 1:\n",
            b"oracle_expectations",
        )

    def test_static_intersection_must_inspect_the_second_predicate(self) -> None:
        with mutant_copy() as verifier:
            source = verifier / "static-auditor" / "src" / "ll2.rs"
            replace_unique(
                source,
                b"    for (lhs, rhs) in left.0.iter().zip(&right.0) {\n",
                b"    for (lhs, rhs) in left.0.iter().zip(&right.0).take(1) {\n",
            )
            cargo = shutil.which("cargo")
            self.assertIsNotNone(cargo, "the pinned Cargo launcher must be installed")
            environment = os.environ.copy()
            environment.update(
                {
                    "CARGO_NET_OFFLINE": "true",
                    "CARGO_TARGET_DIR": str(verifier / "cargo-target"),
                    "CARGO_TERM_COLOR": "never",
                    "RUSTUP_TOOLCHAIN": "1.91.1",
                }
            )
            completed = subprocess.run(
                (
                    cargo or "cargo",
                    "test",
                    "--locked",
                    "--offline",
                    "--manifest-path",
                    str(verifier / "static-auditor" / "Cargo.toml"),
                    "--test",
                    "protocol",
                    "ll2_relations_distinguish_the_second_lookahead_token",
                    "--",
                    "--exact",
                ),
                cwd=verifier / "static-auditor",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=SUBPROCESS_TIMEOUT_SECONDS,
                env=environment,
            )
        output = completed.stdout + completed.stderr
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn(b"different second tokens must separate the alternatives", output)


if __name__ == "__main__":
    unittest.main()
