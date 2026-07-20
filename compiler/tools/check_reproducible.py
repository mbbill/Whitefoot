#!/usr/bin/env python3
"""Compare every Cargo-declared release artifact across physical source copies."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

import cargo_policy


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_TARGETS = {
    (
        "crates/whitefoot-contract/Cargo.toml",
        "whitefoot_contract",
        ("lib",),
        ("lib",),
    ),
    (
        "crates/whitefoot-frontend/Cargo.toml",
        "whitefoot_frontend",
        ("lib",),
        ("lib",),
    ),
    (
        "crates/whitefoot-verifier/Cargo.toml",
        "whitefoot_verifier",
        ("lib",),
        ("lib",),
    ),
}


def digest(path: Path) -> str:
    """Hash one complete emitted artifact."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(message: str) -> None:
    """Stop the reproducibility gate with one direct diagnostic."""
    raise SystemExit(f"reproducibility: {message}")


def reject_symlinks(source: Path) -> None:
    """Reject indirection before copying a build input tree."""
    for directory, directory_names, file_names in os.walk(source, followlinks=False):
        base = Path(directory)
        for name in [*directory_names, *file_names]:
            candidate = base / name
            if candidate.is_symlink():
                fail(f"source tree contains symlink {candidate.relative_to(source)}")
        if base == source:
            directory_names[:] = [name for name in directory_names if name != "target"]


def copy_source(destination: Path) -> None:
    """Create a byte-for-byte physical compiler source copy without build output."""
    reject_symlinks(ROOT)

    def ignore(directory: str, names: list[str]) -> set[str]:
        ignored = {name for name in names if name == "__pycache__" or name == ".DS_Store"}
        if Path(directory).resolve() == ROOT:
            ignored.update(name for name in names if name == "target")
        return ignored

    shutil.copytree(ROOT, destination, copy_function=shutil.copy2, ignore=ignore)


def require_distinct_copies(first: Path, second: Path) -> None:
    """Prove the gate did not rebuild one checkout with two target directories."""
    if first.resolve() == second.resolve():
        fail("source copy paths are identical")
    first_files = {
        path.relative_to(first) for path in first.rglob("*") if path.is_file()
    }
    second_files = {
        path.relative_to(second) for path in second.rglob("*") if path.is_file()
    }
    if first_files != second_files:
        fail("physical source copies have different file sets")
    for relative in sorted(first_files):
        first_input = first / relative
        second_input = second / relative
        if os.path.samefile(first_input, second_input):
            fail(f"source copies share the same physical file: {relative}")
        if first_input.read_bytes() != second_input.read_bytes():
            fail(f"source copies differ before building: {relative}")


def cargo_metadata(
    source: Path,
    environment: dict[str, str],
    working_directory: Path,
) -> dict:
    """Resolve the copied workspace without network access."""
    result = subprocess.run(
        cargo_policy.cargo_command(
            ("metadata", "--format-version", "1", "--locked", "--offline"),
            source,
        ),
        cwd=working_directory,
        env=environment,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        fail(result.stderr.strip() or "cargo metadata failed")
    return json.loads(result.stdout)


def normalized_artifact_path(path: Path, target: Path) -> str:
    """Name an artifact by its exact target-relative path."""
    try:
        relative = path.resolve().relative_to(target.resolve())
    except ValueError:
        fail(f"Cargo reported artifact outside its target directory: {path}")
    return relative.as_posix()


def build_environment(
    source: Path,
    target: Path,
    cargo_home: Path,
) -> dict[str, str]:
    """Construct a closed build environment without user Cargo configuration."""
    cargo_home.mkdir(parents=True)
    process_home = cargo_home.parent / f"process-home-{cargo_home.name}"
    child_temporary = cargo_home.parent / f"tmp-{cargo_home.name}"
    process_home.mkdir()
    child_temporary.mkdir()
    environment = cargo_policy.closed_environment(
        cargo_home,
        target,
        process_home,
        child_temporary,
        "build",
    )
    environment.update(
        {
            "CARGO_ENCODED_RUSTFLAGS": "\x1f".join(
                (
                    f"--remap-path-prefix={source.resolve()}=/whitefoot/compiler",
                    f"--remap-path-prefix={target.resolve()}=/whitefoot/target",
                )
            ),
        }
    )
    return environment


def artifact_key(
    source: Path,
    target_directory: Path,
    message: dict,
    artifact: Path,
    role: str,
) -> tuple:
    """Build a collision-checked logical identity for one emitted file."""
    manifest = Path(message["manifest_path"]).resolve()
    try:
        manifest_name = manifest.relative_to(source.resolve()).as_posix()
    except ValueError:
        fail(f"workspace artifact manifest escapes its source copy: {manifest}")
    target = message["target"]
    return (
        manifest_name,
        target["name"],
        tuple(target["kind"]),
        tuple(target["crate_types"]),
        role,
        normalized_artifact_path(artifact, target_directory),
    )


def build(
    source: Path,
    target: Path,
    cargo_home: Path,
) -> dict[tuple, str]:
    """Build one physical copy and inventory every declared workspace artifact."""
    environment = build_environment(source, target, cargo_home)
    working_directory = target.parent / f"cargo-work-{target.name}"
    working_directory.mkdir()
    configurations = cargo_policy.ambient_tool_configuration_files(
        working_directory,
        cargo_home,
        source,
    )
    if configurations:
        fail(f"Cargo configuration reached reproducibility build: {configurations}")

    metadata = cargo_metadata(source, environment, working_directory)
    workspace_ids = set(metadata["workspace_members"])
    result = subprocess.run(
        cargo_policy.cargo_command(
            (
                "build",
                "--workspace",
                "--release",
                "--locked",
                "--offline",
                "--message-format=json",
            ),
            source,
        ),
        cwd=working_directory,
        env=environment,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        fail(result.stderr.strip() or "release build failed")

    artifacts: dict[tuple, str] = {}
    observed_targets: set[tuple] = set()
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact":
            continue
        if message.get("package_id") not in workspace_ids:
            continue

        manifest = Path(message["manifest_path"]).resolve()
        try:
            manifest_name = manifest.relative_to(source.resolve()).as_posix()
        except ValueError:
            fail(f"workspace manifest escapes its source copy: {manifest}")
        cargo_target = message["target"]
        target_identity = (
            manifest_name,
            cargo_target["name"],
            tuple(cargo_target["kind"]),
            tuple(cargo_target["crate_types"]),
        )
        if target_identity in observed_targets:
            fail(f"Cargo emitted a workspace target more than once: {target_identity}")
        observed_targets.add(target_identity)

        declared: list[tuple[Path, str]] = [
            (Path(filename), "filename") for filename in message.get("filenames", [])
        ]
        executable = message.get("executable")
        if executable is not None and Path(executable) not in {path for path, _ in declared}:
            declared.append((Path(executable), "executable"))
        if not declared:
            fail(f"Cargo declared no artifacts for {target_identity}")
        suffixes = tuple(sorted(path.suffix for path, _ in declared))
        if suffixes != (".rlib", ".rmeta"):
            fail(
                f"workspace library artifact set drifted for {target_identity}: "
                f"{suffixes}"
            )
        for path, role in declared:
            if not path.is_file():
                fail(f"Cargo-declared artifact does not exist: {path}")
            key = artifact_key(source, target, message, path, role)
            if key in artifacts:
                fail(f"artifact identity collision: {key}")
            artifacts[key] = digest(path)

    if observed_targets != EXPECTED_TARGETS:
        missing = sorted(EXPECTED_TARGETS - observed_targets)
        extra = sorted(observed_targets - EXPECTED_TARGETS)
        fail(f"production target set drifted; missing={missing}, extra={extra}")
    if not artifacts:
        fail("release build produced no workspace artifacts")
    return artifacts


def main() -> None:
    """Compare release outputs from two independent source paths and inodes."""
    with tempfile.TemporaryDirectory(prefix="whitefoot-reproducible-") as temporary:
        base = Path(temporary)
        first_source = base / "source-first" / "compiler"
        second_source = base / "source-second" / "compiler"
        copy_source(first_source)
        copy_source(second_source)
        require_distinct_copies(first_source, second_source)

        first = build(first_source, base / "target-first", base / "cargo-home-first")
        second = build(second_source, base / "target-second", base / "cargo-home-second")
        if first != second:
            differing = sorted(set(first) | set(second))
            details = "\n".join(
                f"  {key}: {first.get(key, '<missing>')} != {second.get(key, '<missing>')}"
                for key in differing
                if first.get(key) != second.get(key)
            )
            fail(f"cross-checkout release artifacts differ:\n{details}")
        print(
            f"reproducibility: {len(first)} Cargo-declared release artifacts match "
            "across two physical source copies"
        )


if __name__ == "__main__":
    main()
