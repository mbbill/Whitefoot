#!/usr/bin/env python3
"""Run reviewed Cargo commands without ambient Cargo configuration."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Sequence


ROOT = Path(__file__).resolve().parents[1]
ALLOWED_COMMANDS = frozenset(
    {"build", "check", "clippy", "doc", "fmt", "metadata", "test"}
)
FORBIDDEN_PATH_OPTIONS = ("--config", "--manifest-path", "--target", "--target-dir")
TOOLCHAIN_CHANNEL = json.loads(
    (ROOT / "toolchain-lock.json").read_text(encoding="utf-8")
)["channel"]
RUSTUP_HOME = Path(
    os.environ.get(
        "RUSTUP_HOME",
        str(Path(os.environ.get("HOME") or Path.home()) / ".rustup"),
    )
).resolve()


def ambient_tool_configuration_files(
    working_directory: Path,
    cargo_home: Path,
    workspace: Path,
) -> tuple[Path, ...]:
    """Return Cargo, rustfmt, or Clippy configuration discoverable by a run."""
    candidates = [cargo_home / "config", cargo_home / "config.toml"]
    working = working_directory.resolve()
    candidates.extend(
        candidate
        for directory in (working, *working.parents)
        for candidate in (
            directory / ".cargo" / "config",
            directory / ".cargo" / "config.toml",
        )
    )
    source = workspace.resolve()
    tool_directories = dict.fromkeys(
        (working, *working.parents, source, *source.parents)
    )
    candidates.extend(
        candidate
        for directory in tool_directories
        for candidate in (
            directory / ".rustfmt.toml",
            directory / "rustfmt.toml",
            directory / ".clippy.toml",
            directory / "clippy.toml",
        )
    )
    return tuple(candidate for candidate in candidates if candidate.exists())


def closed_environment(
    cargo_home: Path,
    target_directory: Path,
    home_directory: Path,
    temporary_directory: Path,
    command: str,
) -> dict[str, str]:
    """Construct the complete environment admitted to a Cargo subprocess."""
    environment = {
        "CARGO_HOME": str(cargo_home),
        "CARGO_INCREMENTAL": "0",
        "CARGO_NET_OFFLINE": "true",
        "CARGO_TARGET_DIR": str(target_directory),
        "CARGO_TERM_COLOR": "never",
        "HOME": str(home_directory),
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": os.environ.get("PATH", ""),
        "RUSTUP_HOME": str(RUSTUP_HOME),
        "RUSTUP_TOOLCHAIN": TOOLCHAIN_CHANNEL,
        "SOURCE_DATE_EPOCH": "0",
        "TMPDIR": str(temporary_directory),
        "ZERO_AR_DATE": "1",
    }
    if command == "doc":
        environment["RUSTDOCFLAGS"] = "-D warnings"
    return environment


def cargo_command(arguments: Sequence[str], workspace: Path = ROOT) -> tuple[str, ...]:
    """Construct one manifest-explicit allowlisted Cargo command."""
    if not arguments or arguments[0] not in ALLOWED_COMMANDS:
        raise ValueError("first argument must be an allowlisted Cargo command")
    forbidden_options = tuple(
        option
        for option in FORBIDDEN_PATH_OPTIONS
        if any(
            argument == option or argument.startswith(f"{option}=")
            for argument in arguments
        )
    )
    if forbidden_options:
        raise ValueError(f"Cargo path/configuration options are forbidden: {forbidden_options}")
    if any(argument.startswith("-Z") for argument in arguments):
        raise ValueError("unstable Cargo options are forbidden")
    if arguments[0] == "test" and "--doc" in arguments:
        raise ValueError("Cargo doctest execution is forbidden")
    manifest = workspace.resolve() / "Cargo.toml"
    if not manifest.is_file():
        raise ValueError(f"Cargo manifest is not a regular file: {manifest}")
    return (
        "cargo",
        arguments[0],
        "--manifest-path",
        str(manifest),
        *arguments[1:],
    )


def run_cargo(
    arguments: Sequence[str],
    *,
    workspace: Path = ROOT,
    capture_output: bool = False,
) -> subprocess.CompletedProcess[str]:
    """Run one allowlisted Cargo command from a configuration-free directory."""
    workspace = workspace.resolve()
    command = cargo_command(arguments, workspace)

    with tempfile.TemporaryDirectory(prefix="whitefoot-cargo-policy-") as temporary:
        temporary_root = Path(temporary)
        working_directory = temporary_root / "work"
        cargo_home = temporary_root / "cargo-home"
        target_directory = temporary_root / "target"
        home_directory = temporary_root / "home"
        child_temporary_directory = temporary_root / "tmp"
        working_directory.mkdir()
        cargo_home.mkdir()
        home_directory.mkdir()
        child_temporary_directory.mkdir()
        configurations = ambient_tool_configuration_files(
            working_directory,
            cargo_home,
            workspace,
        )
        if configurations:
            names = ", ".join(str(path) for path in configurations)
            raise RuntimeError(f"Cargo configuration reached isolated run: {names}")
        return subprocess.run(
            command,
            cwd=working_directory,
            env=closed_environment(
                cargo_home,
                target_directory,
                home_directory,
                child_temporary_directory,
                arguments[0],
            ),
            check=False,
            stdout=subprocess.PIPE if capture_output else None,
            stderr=subprocess.PIPE if capture_output else None,
            text=True,
        )


def main() -> None:
    """Forward command-line arguments and preserve Cargo's exit status."""
    try:
        result = run_cargo(sys.argv[1:])
    except (OSError, RuntimeError, ValueError) as error:
        raise SystemExit(f"cargo policy: {error}") from error
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
