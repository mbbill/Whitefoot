#!/usr/bin/env python3
"""Focused tests for the production/static-capability boundary."""

from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock


MODULE_PATH = Path(__file__).with_name("check_workspace.py")
SPEC = importlib.util.spec_from_file_location("check_workspace", MODULE_PATH)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import machinery failure
    raise RuntimeError("cannot load check_workspace.py")
check_workspace = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check_workspace)


class CapabilityMetadataNonConsumptionTests(unittest.TestCase):
    def test_cargo_configuration_is_forbidden_in_workspace_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            root = repository / "compiler"
            configuration = root / ".cargo" / "config.toml"
            configuration.parent.mkdir(parents=True)
            configuration.write_text("[build]\nrustflags = []\n", encoding="utf-8")
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_no_workspace_build_tool_configuration(root, repository)
            self.assertIn("build-tool configuration is forbidden", diagnostic.getvalue())

    def test_doctests_are_disabled_in_exact_target_topology(self) -> None:
        live = check_workspace.metadata()
        self.assertTrue(
            all(
                target["doctest"] is False
                for package in live["packages"]
                for target in package["targets"]
            )
        )
        mutated = copy.deepcopy(live)
        mutated["packages"][0]["targets"][0]["doctest"] = True
        diagnostic = io.StringIO()
        with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
            check_workspace.check_workspace_topology(mutated)
        self.assertIn("target topology drifted", diagnostic.getvalue())

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
            'const NAME: &str = concat!("facet:", "TYPE-7/deref-consumption");',
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
                / "v0.9"
                / "foundation.json"
            )
            capability.parent.mkdir(parents=True)
            capability.write_text("{}\n", encoding="ascii")
            source.write_text(
                'const DATA: &str = include_str!('
                '"../../../../capabilities/whitefoot-rust/v0.9/foundation.json"'
                ");\n",
                encoding="utf-8",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_compile_time_inputs(root)
            self.assertIn("path escapes compiler/", diagnostic.getvalue())
            self.assertNotIn("Traceback", diagnostic.getvalue())

    def test_path_attributes_are_forbidden(self) -> None:
        cases = (
            '#[path = "../hidden.rs"]\nmod hidden;\n',
            '#[path/**/=/**/"../hidden.rs"]\nmod hidden;\n',
            'mod inline { #[path = "payload.wf"] mod hidden; }\n',
        )
        for crate_source in cases:
            with self.subTest(crate_source=crate_source):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary) / "compiler"
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(crate_source, encoding="utf-8")
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.check_compile_time_inputs(root)
                    self.assertIn("#[path] is forbidden", diagnostic.getvalue())

    def test_nested_path_cannot_hide_arbitrary_extension_source(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "compiler"
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                'mod inline { #[path = "payload.wf"] mod hidden; }\n',
                encoding="utf-8",
            )
            (source.parent / "payload.wf").write_text(
                "// benign decoy\n",
                encoding="utf-8",
            )
            hidden = source.parent / "inline" / "payload.wf"
            hidden.parent.mkdir()
            hidden.write_text(
                'const ID: &str = "facet:TYPE-1/primitive-types";\n',
                encoding="utf-8",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic), self.assertRaises(SystemExit):
                check_workspace.check_compile_time_inputs(root)
            self.assertIn("#[path] is forbidden", diagnostic.getvalue())

    def test_include_macro_spellings_are_forbidden(self) -> None:
        cases = (
            'include!("../../../hidden.rs");\n',
            'include ! ("../../../hidden.rs");\n',
            'include/**/!/**/("../../../hidden.rs");\n',
        )
        for crate_source in cases:
            with self.subTest(crate_source=crate_source):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(crate_source, encoding="utf-8")
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.check_compile_time_inputs(root)
                    self.assertIn("include! is forbidden", diagnostic.getvalue())

    def test_path_source_cannot_reach_external_overlay(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            root = parent / "compiler"
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                '#[path = "../../../hidden.rs"]\nmod hidden;\n',
                encoding="utf-8",
            )
            capability = (
                parent
                / "capabilities"
                / "whitefoot-rust"
                / "v0.9"
                / "foundation.json"
            )
            capability.parent.mkdir(parents=True)
            capability.write_text("{}\n", encoding="ascii")
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic):
                with self.assertRaises(SystemExit):
                    check_workspace.check_compile_time_inputs(root)
            self.assertIn("#[path] is forbidden", diagnostic.getvalue())

    def test_noncanonical_path_attribute_forms_are_rejected(self) -> None:
        cases = (
            (
                '#[path = r"../../../hidden.rs"]\nmod hidden;\n',
                "#[path] is forbidden",
            ),
            (
                '#[cfg_attr(any(), path = "../../../hidden.rs")]\nmod hidden;\n',
                "canonical #[cfg(test)]",
            ),
        )
        for crate_source, message in cases:
            with self.subTest(crate_source=crate_source, message=message):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(crate_source, encoding="utf-8")
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.check_compile_time_inputs(root)
                    self.assertIn(message, diagnostic.getvalue())

    def test_copied_internal_capability_data_is_not_an_approved_input(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                'const DATA: &str = include_str!("../../../copied.json");\n',
                encoding="utf-8",
            )
            (root / "copied.json").write_text(
                '{"format":"whitefoot-capability-fragment-v1"}\n',
                encoding="ascii",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic):
                with self.assertRaises(SystemExit):
                    check_workspace.check_compile_time_inputs(root)
            self.assertIn("unapproved compile-time data input", diagnostic.getvalue())

    def test_local_macro_generation_is_forbidden(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "macro_rules/**/! hidden { () => {}; }\n",
                encoding="utf-8",
            )
            diagnostic = io.StringIO()
            with contextlib.redirect_stderr(diagnostic):
                with self.assertRaises(SystemExit):
                    check_workspace.check_compile_time_inputs(root)
            self.assertIn("macro_rules! is forbidden", diagnostic.getvalue())

    def test_comment_and_literal_tokens_do_not_trigger_directive_policy(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                '// include ! ("outside");\n'
                '/* outer /* include! */ macro_rules! */\n'
                'const A: &str = "include! macro_rules!";\n'
                'const B: &str = r#"#[path = outside]"#;\n'
                "const C: char = '/';\n",
                encoding="utf-8",
            )
            check_workspace.check_compile_time_inputs(root)

    def test_spaced_commented_data_include_uses_exact_allowlist(self) -> None:
        for lock_name in (
            "kernel-spec-v0.8.sha256",
            "kernel-spec-v0.9.sha256",
            "static-semantic-catalog-v0.8.sha256",
            "static-semantic-catalog-v0.9.sha256",
        ):
            with self.subTest(lock_name=lock_name):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(
                        'const LOCK: &str = include_str/**/ ! /**/('
                        f'"../../../{lock_name}" /**/);\n',
                        encoding="utf-8",
                    )
                    (root / lock_name).write_text(
                        ("0" * 64) + "\n",
                        encoding="ascii",
                    )
                    check_workspace.check_compile_time_inputs(root)

    def test_static_catalog_lock_is_exact_and_pinned(self) -> None:
        check_workspace.check_static_catalog_identity()
        valid = (
            check_workspace.EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256 + "\n"
        ).encode("ascii")
        self.assertEqual(
            check_workspace.parse_sha256_lock(valid, "probe"),
            check_workspace.EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256,
        )
        malformed = (
            valid[:-1],
            valid[:-1] + b"\r\n",
            valid.upper(),
            valid + b"\n",
            b"g" + valid[1:],
        )
        for raw in malformed:
            with self.subTest(raw=raw):
                diagnostic = io.StringIO()
                with contextlib.redirect_stderr(diagnostic):
                    with self.assertRaises(SystemExit):
                        check_workspace.parse_sha256_lock(raw, "probe")
                self.assertIn("workspace policy:", diagnostic.getvalue())

    def test_identity_lock_reader_rejects_indirection_and_special_files(self) -> None:
        valid = (
            check_workspace.EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256 + "\n"
        ).encode("ascii")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / "target"
            target.write_bytes(valid)
            symlink = root / "symlink"
            symlink.symlink_to(target)
            fifo = root / "fifo"
            os.mkfifo(fifo)
            oversized = root / "oversized"
            oversized.write_bytes(valid + b"\n")

            for path in (symlink, fifo, oversized):
                with self.subTest(path=path.name):
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.read_exact_lock(path, "probe")
                    self.assertIn("workspace policy:", diagnostic.getvalue())

    def test_compile_time_data_macro_aliases_are_rejected(self) -> None:
        for macro in ("include_str", "include_bytes"):
            with self.subTest(macro=macro):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(
                        f"use std::{macro} as data;\n"
                        'const DATA: &str = data!("../../../copied.json");\n',
                        encoding="utf-8",
                    )
                    (root / "copied.json").write_text("{}\n", encoding="ascii")
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.check_compile_time_inputs(root)
                    self.assertIn("must be direct invocations", diagnostic.getvalue())

    def test_compile_time_environment_channels_are_always_rejected(self) -> None:
        cases = (
            'const DATA: &str = env!("WHITEFOOT_CAPABILITY_OVERLAY");\n',
            'const DATA: Option<&str> = '
            'option_env!("WHITEFOOT_CAPABILITY_OVERLAY");\n',
            "use std::option_env as data;\n",
        )
        for crate_source in cases:
            for present in (False, True):
                with self.subTest(crate_source=crate_source, present=present):
                    with tempfile.TemporaryDirectory() as temporary:
                        root = Path(temporary)
                        source = root / "crates" / "sample" / "src" / "lib.rs"
                        source.parent.mkdir(parents=True)
                        source.write_text(crate_source, encoding="utf-8")
                        environment = (
                            {"WHITEFOOT_CAPABILITY_OVERLAY": "forged"}
                            if present
                            else {}
                        )
                        diagnostic = io.StringIO()
                        with mock.patch.dict(
                            os.environ,
                            environment,
                            clear=not present,
                        ):
                            with contextlib.redirect_stderr(diagnostic):
                                with self.assertRaises(SystemExit):
                                    check_workspace.check_compile_time_inputs(root)
                        self.assertIn(
                            "compile-time environment channels are forbidden",
                            diagnostic.getvalue(),
                        )

    def test_only_canonical_test_conditional_compilation_is_allowed(self) -> None:
        rejected = (
            '#[cfg(whitefoot_overlay)]\nconst MODE: &str = "overlay";\n',
            '#[cfg_attr(test, allow(dead_code))]\nconst MODE: u8 = 0;\n',
            "const MODE: bool = cfg!(whitefoot_overlay);\n",
            "use std::cfg as conditional;\n",
            "#![cfg(test)]\n",
        )
        for crate_source in rejected:
            with self.subTest(crate_source=crate_source):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    source = root / "crates" / "sample" / "src" / "lib.rs"
                    source.parent.mkdir(parents=True)
                    source.write_text(crate_source, encoding="utf-8")
                    diagnostic = io.StringIO()
                    with contextlib.redirect_stderr(diagnostic):
                        with self.assertRaises(SystemExit):
                            check_workspace.check_compile_time_inputs(root)
                    self.assertIn("canonical #[cfg(test)]", diagnostic.getvalue())

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates" / "sample" / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "# /**/ [ /**/ cfg /**/ ( /**/ test /**/ ) /**/ ]\n"
                "const TEST_ONLY: u8 = 0;\n",
                encoding="utf-8",
            )
            check_workspace.check_compile_time_inputs(root)


if __name__ == "__main__":
    unittest.main()
