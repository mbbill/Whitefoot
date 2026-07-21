from __future__ import annotations

import hashlib
from pathlib import Path
import tempfile
import unittest

from support_common import ROOT, inputs
from runner_inputs import (
    CURRENT_SHA256,
    SUCCESSOR_SHA256,
    Inputs,
    MAGIC,
    RunnerError,
    load_inputs,
    make_frame,
    parse_limits,
    runner_sources,
    source_revision,
)
from run import source_revisions
from runner_report import parse_report
import runner_inputs


class InputTests(unittest.TestCase):
    def test_repository_inputs_are_canonical_and_pinned(self) -> None:
        value = load_inputs(ROOT)
        self.assertEqual(value.section("current").binding["sha256"], CURRENT_SHA256)
        self.assertEqual(value.section("proposal").binding["sha256"], SUCCESSOR_SHA256)
        self.assertIsNone(value.installation)

    def test_installed_inputs_require_the_reviewed_candidate_bytes(self) -> None:
        value = load_inputs(ROOT, installed=True)
        self.assertEqual(value.section("proposal").binding["sha256"], SUCCESSOR_SHA256)
        self.assertEqual(value.installation["mode"], "installed-v0.9")
        self.assertEqual(value.installation["relation"], "byte-identical")
        self.assertEqual(
            value.installation["candidate"],
            {
                "byte_length": 98044,
                "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
                "sha256": SUCCESSOR_SHA256,
            },
        )
        self.assertEqual(
            value.installation["installed_specification"],
            {
                "byte_length": 98044,
                "path": "spec/kernel-spec-v0.9.md",
                "sha256": SUCCESSOR_SHA256,
            },
        )

    def test_wrong_current_hash_stops(self) -> None:
        with self.assertRaisesRegex(RunnerError, "pinned v0.8"):
            load_inputs(ROOT, "0" * 64)

    def test_frame_has_exact_magic_lengths_and_excludes_expectations(self) -> None:
        value = inputs()
        frame = make_frame(value)
        self.assertEqual(frame[:8], MAGIC)
        offset = 8
        lengths = []
        for _ in value.sections:
            lengths.append(int.from_bytes(frame[offset : offset + 8], "big"))
            offset += 8
        self.assertEqual(lengths, [len(section.data) for section in value.sections])
        self.assertEqual(frame[offset:], b"".join(section.data for section in value.sections))
        self.assertNotIn(value.expectations.data, frame)

    def test_report_validator_repeats_the_engine_output_byte_bound(self) -> None:
        value = inputs()
        reduced_limits = dict(value.limits)
        reduced_limits["max_engine_output_bytes"] = 1
        reduced = Inputs(value.sections, value.expectations, reduced_limits)
        with self.assertRaisesRegex(RunnerError, "declared byte bound"):
            parse_report(b"WFGRREPORT1\nENGINE\tstatic\nFAIL\tinput\tx\nEND\n", "static", reduced)

    def test_limits_reject_noncanonical_order(self) -> None:
        raw = (ROOT / "limits.txt").read_bytes()
        lines = raw.splitlines(keepends=True)
        with self.assertRaisesRegex(RunnerError, "missing, extra, duplicate, or reordered"):
            parse_limits(b"".join(reversed(lines)))

    def test_limits_accept_reductions_but_reject_values_above_v1_maxima(self) -> None:
        raw = (ROOT / "limits.txt").read_bytes()
        reduced = raw.replace(b"max_symbol_bytes=256\n", b"max_symbol_bytes=255\n", 1)
        self.assertEqual(parse_limits(reduced)["max_symbol_bytes"], 255)
        excessive = raw.replace(b"max_symbol_bytes=256\n", b"max_symbol_bytes=257\n", 1)
        with self.assertRaisesRegex(RunnerError, "hard maximum"):
            parse_limits(excessive)
        huge = raw.replace(b"max_symbol_bytes=256\n", b"max_symbol_bytes=" + b"9" * 5000 + b"\n", 1)
        with self.assertRaisesRegex(RunnerError, "hard maximum"):
            parse_limits(huge)

    def test_source_revision_is_exhaustive_and_content_bound(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "SOURCES").write_text("SOURCES\na.py\n", encoding="ascii")
            (root / "a.py").write_text("first\n", encoding="ascii")
            expected = {"SOURCES", "a.py"}
            first = source_revision(root / "SOURCES", expected)
            (root / "a.py").write_text("second\n", encoding="ascii")
            second = source_revision(root / "SOURCES", expected)
            self.assertNotEqual(first.sha256, second.sha256)
            with self.assertRaisesRegex(RunnerError, "exhaustive"):
                source_revision(root / "SOURCES", expected | {"missing.py"})

    def test_source_rediscovery_rejects_a_helper_added_between_observations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("FORMAT.md", "Makefile", "base.py"):
                (root / name).write_text(f"{name}\n", encoding="ascii")
            evidence = root / "evidence"
            evidence.mkdir()
            for name in ("v0.9-manifest-metadata.patch", "v0.9-post-form2-case-intent.patch"):
                (evidence / name).write_text(f"{name}\n", encoding="ascii")
            runner_manifest = (
                "FORMAT.md",
                "Makefile",
                "RUNNER_SOURCES",
                "base.py",
                "evidence/v0.9-manifest-metadata.patch",
                "evidence/v0.9-post-form2-case-intent.patch",
            )
            (root / "RUNNER_SOURCES").write_text("\n".join(runner_manifest) + "\n", encoding="ascii")
            static = root / "static-auditor"
            (static / "src").mkdir(parents=True)
            for name in ("Cargo.lock", "Cargo.toml", "rust-toolchain.toml"):
                (static / name).write_text(f"{name}\n", encoding="ascii")
            (static / "src" / "lib.rs").write_text("// source\n", encoding="ascii")
            static_manifest = ("Cargo.lock", "Cargo.toml", "SOURCES", "rust-toolchain.toml", "src/lib.rs")
            (static / "SOURCES").write_text("\n".join(static_manifest) + "\n", encoding="ascii")
            oracle = root / "oracle"
            oracle.mkdir()
            (oracle / "main.py").write_text("# source\n", encoding="ascii")
            (oracle / "SOURCES").write_text("SOURCES\nmain.py\n", encoding="ascii")
            source_revisions(root)
            (root / "runner_extra.py").write_text("# late source\n", encoding="ascii")
            self.assertIn("runner_extra.py", runner_sources(root))
            with self.assertRaisesRegex(RunnerError, "exhaustive"):
                source_revisions(root)

    def test_nested_runner_helper_is_discovered(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            helper = root / "runner_helpers" / "formatting.py"
            helper.parent.mkdir()
            helper.write_text("# nested runner source\n", encoding="ascii")
            (root / "oracle" / "nested").mkdir(parents=True)
            (root / "oracle" / "nested" / "engine.py").write_text(
                "# independently manifested engine source\n",
                encoding="ascii",
            )
            self.assertIn("runner_helpers/formatting.py", runner_sources(root))
            self.assertNotIn("oracle/nested/engine.py", runner_sources(root))

    def test_current_pin_is_exact_repository_bytes(self) -> None:
        actual = hashlib.sha256((ROOT.parent / "spec" / "kernel-spec-v0.8.md").read_bytes()).hexdigest()
        self.assertEqual(actual, CURRENT_SHA256)

    def test_tables_reject_duplicate_logical_ids_with_different_payloads(self) -> None:
        mutants = (
            (
                "cases",
                b"whitefoot.grammar-cases.v1\n"
                b"case\tsame\texpr\t61\n"
                b"case\tsame\tstmt\t62\n",
            ),
            (
                "domains",
                b"whitefoot.grammar-domains.v1\n"
                b"domain\tsame\tfixed-lowerword-call\texpr\t61\n"
                b"domain\tsame\tfixed-lowerword-call\tstmt\t62\n",
            ),
            (
                "expectations",
                b"whitefoot.grammar-expectations.v2\n"
                b"case\tsame\tcurrent\tone\n"
                b"case\tsame\tcurrent\tzero\n",
            ),
            (
                "expectations",
                b"whitefoot.grammar-expectations.v2\n"
                b"transition\tsame\tintroduced\n"
                b"transition\tsame\tremoved\n",
            ),
        )
        for kind, raw in mutants:
            with self.subTest(kind=kind, raw=raw):
                with self.assertRaisesRegex(RunnerError, "logical record id"):
                    runner_inputs._validate_table(raw, kind, 16)

    def test_manifest_ids_and_start_symbols_obey_the_symbol_byte_limit(self) -> None:
        exact = b"a" * 8
        accepted = (
            b"whitefoot.grammar-cases.v1\ncase\t" + exact + b"\t" + exact + b"\t61\n"
        )
        runner_inputs._validate_table(accepted, "cases", 1, 8)
        oversized_id = accepted.replace(exact + b"\t", exact + b"a\t", 1)
        with self.assertRaisesRegex(RunnerError, "oversized"):
            runner_inputs._validate_table(oversized_id, "cases", 1, 8)
        oversized_start = accepted.replace(b"\t" + exact + b"\t61", b"\t" + exact + b"a\t61", 1)
        with self.assertRaisesRegex(RunnerError, "oversized"):
            runner_inputs._validate_table(oversized_start, "cases", 1, 8)


if __name__ == "__main__":
    unittest.main()
