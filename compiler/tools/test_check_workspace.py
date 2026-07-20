#!/usr/bin/env python3
"""Focused tests for the production/static-capability boundary."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_workspace.py")
SPEC = importlib.util.spec_from_file_location("check_workspace", MODULE_PATH)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import machinery failure
    raise RuntimeError("cannot load check_workspace.py")
check_workspace = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check_workspace)


class CapabilityMetadataNonConsumptionTests(unittest.TestCase):
    def test_every_forbidden_reference_is_detected(self) -> None:
        cases = {
            'const ID: &str = "facet:TYPE-7/deref-consumption-own1-binding";':
                "direct facet ID",
        }
        for source, expected in cases.items():
            with self.subTest(source=source):
                violation = check_workspace.facet_metadata_reference(source)
                self.assertIsNotNone(violation)
                self.assertIn(expected, violation)

    def test_safe_near_misses_and_catalog_hash_type_are_allowed(self) -> None:
        safe_sources = (
            "pub struct CatalogSha256([u8; 32]);",
            "pub struct CatalogHash([u8; 32]);",
            'const NAME: &str = "facet-TYPE-7/deref-consumption";',
            'const NAME: &str = "facet:TYPE-7";',
        )
        for source in safe_sources:
            with self.subTest(source=source):
                self.assertIsNone(check_workspace.facet_metadata_reference(source))

    def test_scanner_is_limited_to_crate_rust_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub struct CatalogHash([u8; 32]);\n", encoding="utf-8")
            (root / "outside.rs").write_text(
                'const ID: &str = "facet:TYPE-1/primitive-types";\n',
                encoding="utf-8",
            )
            check_workspace.check_no_production_facet_ids(root)

    def test_scanner_reports_a_controlled_policy_error(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                'const ID: &str = "facet:TYPE-1/primitive-types";\n',
                encoding="utf-8",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_no_production_facet_ids(root)
            self.assertIn("workspace policy:", diagnostic.getvalue())
            self.assertIn("crates/sample/src/lib.rs", diagnostic.getvalue())
            self.assertNotIn("Traceback", diagnostic.getvalue())

    def test_invalid_utf8_is_a_controlled_policy_error(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"\xff")
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_no_production_facet_ids(root)
            self.assertIn("cannot inspect production Rust source", diagnostic.getvalue())
            self.assertNotIn("Traceback", diagnostic.getvalue())

    def test_compile_time_input_cannot_reach_external_capability_overlay(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            root = parent / "compiler"
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            capability = (
                parent
                / "capabilities"
                / "whitefoot-rust"
                / "v0.8"
                / "foundation.json"
            )
            capability.parent.mkdir(parents=True)
            capability.write_text("{}\n", encoding="ascii")
            source.write_text(
                'const DATA: &str = include_str!('
                '"../../../../capabilities/whitefoot-rust/v0.8/foundation.json"'
                ");\n",
                encoding="utf-8",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_compile_time_inputs(root)
            self.assertIn("path escapes compiler/", diagnostic.getvalue())
            self.assertNotIn("Traceback", diagnostic.getvalue())


if __name__ == "__main__":
    unittest.main()
