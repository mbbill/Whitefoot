from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from support_common import inputs
from support_oracle import oracle_report
from support_static import static_report
from runner_inputs import Inputs, RunnerError, SourceRevision
from runner_package import validate_published_package, write_evidence
from runner_report import parse_report


class ReportPackageTests(unittest.TestCase):
    def test_installed_report_has_explicit_v2_installation_binding(self) -> None:
        base = inputs()
        installation = {
            "mode": "installed-v0.9",
            "relation": "byte-identical",
        }
        value = Inputs(base.sections, base.expectations, base.limits, installation)
        reports = (
            parse_report(static_report(value), "static", value),
            parse_report(oracle_report(value), "oracle", value),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            output = root / "evidence"
            write_evidence(output, root, value, self._revisions(), reports)
            report = json.loads((output / "report.json").read_text(encoding="ascii"))
            self.assertEqual(report["schema"], "whitefoot.grammar-evidence.v2")
            self.assertEqual(report["installation"], installation)

    def test_common_disagreement_blocks_packaging(self) -> None:
        value = inputs()
        static = parse_report(static_report(value), "static", value)
        oracle = parse_report(oracle_report(value), "oracle", value)
        altered = type(oracle)(
            oracle.engine,
            oracle.raw,
            oracle.common + b"x\n",
            oracle.specific,
            None,
            oracle.domains,
            oracle.observations,
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            with self.assertRaisesRegex(RunnerError, "disagree"):
                write_evidence(root / "evidence", root, value, self._revisions(), (static, altered))

    def test_generated_domain_disagreement_blocks_packaging(self) -> None:
        value = inputs()
        static = parse_report(static_report(value), "static", value)
        oracle = parse_report(oracle_report(value), "oracle", value)
        changed_domains = dict(oracle.domains)
        key = next(iter(changed_domains))
        claim = list(changed_domains[key])
        claim[2] = b"2"
        changed_domains[key] = tuple(claim)
        altered = type(oracle)(
            oracle.engine,
            oracle.raw,
            oracle.common,
            oracle.specific,
            None,
            changed_domains,
            oracle.observations,
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            with self.assertRaisesRegex(RunnerError, "generated-domain"):
                write_evidence(root / "evidence", root, value, self._revisions(), (static, altered))

    def test_package_is_deterministic_and_binds_every_component(self) -> None:
        value = inputs()
        reports = (
            parse_report(static_report(value), "static", value),
            parse_report(oracle_report(value), "oracle", value),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            first = root / "first"
            second = root / "second"
            first_report = write_evidence(first, root, value, self._revisions(), reports)
            second_report = write_evidence(second, root, value, self._revisions(), reports)
            self.assertEqual(first_report, second_report)
            for name in (
                "static.raw",
                "oracle.raw",
                "report.json",
                "report.sha256",
                "proposal-delta.md",
                "protected-surface-census.json",
                "package.json",
                "package.sha256",
            ):
                self.assertEqual((first / name).read_bytes(), (second / name).read_bytes())
            package = json.loads((first / "package.json").read_text(encoding="ascii"))
            for name, binding in package["components"].items():
                data = (first / name).read_bytes()
                self.assertEqual(binding["byte_length"], len(data))
                self.assertEqual(binding["sha256"], hashlib.sha256(data).hexdigest())
            report = json.loads((first / "report.json").read_text(encoding="ascii"))
            observations = report["observations"]
            self.assertEqual(observations["cases"]["deref-x"]["current"], {"class": "many", "trace_count": 2})
            self.assertEqual(
                observations["case_delta_status_counts"]["deref-x"],
                {"introduced": 0, "removed": 1, "retained": 1},
            )
            self.assertEqual(
                observations["static_transitions"]["fixed-ident-partition"],
                {
                    "current_count": 1,
                    "proposal_count": 0,
                    "status": "removes-call-through-ident",
                    "witness_hex": "6465726566287829",
                },
            )
            self.assertEqual(observations["static_delta_status_counts"]["conflict"]["removed"], 1)
            self.assertEqual(observations["domains"][0]["stream_count"], 1)
            self.assertEqual(len(observations["domains"][0]["stream_sha256"]), 64)
            validate_published_package(first)

    def test_package_marker_is_fail_closed_over_every_component(self) -> None:
        value = inputs()
        reports = (
            parse_report(static_report(value), "static", value),
            parse_report(oracle_report(value), "oracle", value),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            output = root / "evidence"
            write_evidence(output, root, value, self._revisions(), reports)
            (output / "static.raw").write_bytes(b"changed\n")
            with self.assertRaisesRegex(RunnerError, "does not match"):
                validate_published_package(output)
            (output / "package.sha256").unlink()
            with self.assertRaisesRegex(RunnerError, "unavailable"):
                validate_published_package(output)

    def test_report_digest_must_truthfully_bind_the_report(self) -> None:
        value = inputs()
        reports = (
            parse_report(static_report(value), "static", value),
            parse_report(oracle_report(value), "oracle", value),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = self._proposal_root(Path(directory))
            output = root / "evidence"
            write_evidence(output, root, value, self._revisions(), reports)
            false_sidecar = ("0" * 64 + "  report.json\n").encode("ascii")
            (output / "report.sha256").write_bytes(false_sidecar)
            package = json.loads((output / "package.json").read_text(encoding="ascii"))
            package["components"]["report.sha256"] = {
                "byte_length": len(false_sidecar),
                "sha256": hashlib.sha256(false_sidecar).hexdigest(),
            }
            package_bytes = (
                json.dumps(
                    package,
                    allow_nan=False,
                    ensure_ascii=True,
                    indent=2,
                    separators=(",", ": "),
                    sort_keys=True,
                )
                + "\n"
            ).encode("ascii")
            (output / "package.json").write_bytes(package_bytes)
            marker = f"{hashlib.sha256(package_bytes).hexdigest()}  package.json\n"
            (output / "package.sha256").write_text(marker, encoding="ascii")
            with self.assertRaisesRegex(RunnerError, "does not bind"):
                validate_published_package(output)

    @staticmethod
    def _proposal_root(root: Path) -> Path:
        proposal = root / "proposal"
        proposal.mkdir()
        (proposal / "DELTA.md").write_text("delta\n", encoding="ascii")
        (proposal / "protected-surface-census.json").write_text("{}\n", encoding="ascii")
        return root

    @staticmethod
    def _revisions() -> dict[str, SourceRevision]:
        revision = SourceRevision(1, 1, "0" * 64)
        return {"runner": revision, "static": revision, "oracle": revision}


if __name__ == "__main__":
    unittest.main()
