#!/usr/bin/env python3
"""Hostile tests for isolated Cargo execution."""

from __future__ import annotations

import importlib.util
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock


MODULE_PATH = Path(__file__).with_name("cargo_policy.py")
SPEC = importlib.util.spec_from_file_location("cargo_policy", MODULE_PATH)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import machinery failure
    raise RuntimeError("cannot load cargo_policy.py")
cargo_policy = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(cargo_policy)


class CargoIsolationTests(unittest.TestCase):
    def test_make_variables_cannot_bypass_cargo_policy(self) -> None:
        result = subprocess.run(
            (
                "make",
                "-n",
                "-C",
                str(cargo_policy.ROOT),
                "format",
                "CARGO=cargo",
                "PYTHON=false",
            ),
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("python3 -B tools/cargo_policy.py fmt", result.stdout)

    def test_ambient_home_rustfmt_configuration_is_ignored(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            workspace = base / "workspace"
            source = workspace / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                "pub fn value() {\n    let _answer = 42;\n}\n",
                encoding="utf-8",
            )
            (workspace / "Cargo.toml").write_text(
                '[package]\nname = "rustfmt-probe"\nversion = "0.0.0"\n'
                'edition = "2024"\n',
                encoding="utf-8",
            )
            ambient_home = base / "ambient-home"
            ambient_home.mkdir()
            (ambient_home / ".rustfmt.toml").write_text(
                "hard_tabs = true\n",
                encoding="utf-8",
            )
            with mock.patch.dict(os.environ, {"HOME": str(ambient_home)}, clear=False):
                result = cargo_policy.run_cargo(
                    ("fmt", "--all", "--", "--check"),
                    workspace=workspace,
                    capture_output=True,
                )
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_doctest_and_configuration_injection_commands_are_rejected(self) -> None:
        rejected = (
            ("test", "--doc"),
            ("check", "--config", "build.rustc-wrapper='outside'"),
            ("check", "--config=build.rustflags=['--cfg=outside']"),
            ("check", "--manifest-path", "outside/Cargo.toml"),
            ("check", "--manifest-path=outside/Cargo.toml"),
            ("check", "--target", "outside/target.json"),
            ("check", "--target=outside/target.json"),
            ("check", "--target-dir", "outside-target"),
            ("check", "--target-dir=outside-target"),
            ("check", "-Zconfig-include"),
        )
        for arguments in rejected:
            with self.subTest(arguments=arguments):
                with self.assertRaises(ValueError):
                    cargo_policy.cargo_command(arguments)

    def test_workspace_and_ambient_configs_cannot_install_a_wrapper(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            workspace = base / "workspace"
            source = workspace / "src" / "lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub const VALUE: u8 = 1;\n", encoding="utf-8")
            (workspace / "Cargo.toml").write_text(
                '[package]\nname = "isolation-probe"\nversion = "0.0.0"\n'
                'edition = "2024"\n',
                encoding="utf-8",
            )

            marker = base / "wrapper-ran"
            wrapper = base / "wrapper.sh"
            wrapper.write_text(
                "#!/bin/sh\n"
                f"touch {marker}\n"
                'exec "$@"\n',
                encoding="utf-8",
            )
            wrapper.chmod(0o755)
            configuration = f'[build]\nrustc-wrapper = "{wrapper}"\n'
            workspace_config = workspace / ".cargo" / "config.toml"
            workspace_config.parent.mkdir()
            workspace_config.write_text(configuration, encoding="utf-8")
            ambient_home = base / "ambient-cargo-home"
            ambient_home.mkdir()
            (ambient_home / "config.toml").write_text(configuration, encoding="utf-8")

            seed_home = base / "seed-home"
            seed_temporary = base / "seed-tmp"
            seed_home.mkdir()
            seed_temporary.mkdir()
            seed_environment = {
                "CARGO_HOME": str(ambient_home),
                "CARGO_INCREMENTAL": "0",
                "CARGO_NET_OFFLINE": "true",
                "CARGO_TARGET_DIR": str(workspace / "target"),
                "CARGO_TERM_COLOR": "never",
                "HOME": str(seed_home),
                "LANG": "C",
                "LC_ALL": "C",
                "PATH": os.environ.get("PATH", ""),
                "RUSTUP_HOME": str(cargo_policy.RUSTUP_HOME),
                "RUSTUP_TOOLCHAIN": cargo_policy.TOOLCHAIN_CHANNEL,
                "SOURCE_DATE_EPOCH": "0",
                "TMPDIR": str(seed_temporary),
                "ZERO_AR_DATE": "1",
            }
            seeded = subprocess.run(
                ("cargo", "check", "--offline", "--verbose"),
                cwd=workspace,
                env=seed_environment,
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            self.assertEqual(seeded.returncode, 0, seeded.stderr)
            self.assertTrue(marker.exists())
            marker.unlink()

            workspace_config.unlink()
            (ambient_home / "config.toml").unlink()
            reused = subprocess.run(
                ("cargo", "check", "--offline", "--verbose"),
                cwd=workspace,
                env=seed_environment,
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            self.assertEqual(reused.returncode, 0, reused.stderr)
            self.assertIn("Fresh isolation-probe", reused.stderr)
            self.assertFalse(marker.exists())
            workspace_config.write_text(configuration, encoding="utf-8")
            (ambient_home / "config.toml").write_text(configuration, encoding="utf-8")

            with mock.patch.dict(
                os.environ,
                {
                    "CARGO_HOME": str(ambient_home),
                    "RUSTC_WRAPPER": str(wrapper),
                    "RUSTFLAGS": "--cfg hostile_ambient_configuration",
                },
                clear=False,
            ):
                result = cargo_policy.run_cargo(
                    ("check", "--offline", "--verbose"),
                    workspace=workspace,
                    capture_output=True,
                )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(marker.exists())
            self.assertIn("Checking isolation-probe", result.stderr)

    def test_isolated_working_directory_configuration_is_detected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            working = base / "work"
            cargo_home = base / "cargo-home"
            configuration = working / ".cargo" / "config"
            configuration.parent.mkdir(parents=True)
            cargo_home.mkdir()
            configuration.write_text("[build]\n", encoding="utf-8")
            self.assertEqual(
                cargo_policy.ambient_tool_configuration_files(
                    working,
                    cargo_home,
                    working,
                ),
                (configuration.resolve(),),
            )

    def test_source_ancestor_tool_configuration_is_detected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            workspace = base / "nested" / "workspace"
            working = base / "cargo-work"
            cargo_home = base / "cargo-home"
            workspace.mkdir(parents=True)
            working.mkdir()
            cargo_home.mkdir()
            configuration = base / "rustfmt.toml"
            configuration.write_text("hard_tabs = true\n", encoding="utf-8")
            self.assertIn(
                configuration.resolve(),
                cargo_policy.ambient_tool_configuration_files(
                    working,
                    cargo_home,
                    workspace,
                ),
            )

    def test_non_gate_command_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "allowlisted Cargo command"):
            cargo_policy.run_cargo(("run",))


if __name__ == "__main__":
    unittest.main()
