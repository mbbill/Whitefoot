#!/usr/bin/env python3
"""Compile the pinned positive and negative Rust trait-implementation canaries."""

from __future__ import annotations

import hashlib
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CANARY_ROOT = ROOT / "canaries"
EXPECTED_VERSION = "rustc 1.97.0 (2d8144b78 2026-07-07)"
EXPECTED_COMMIT = "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3"
POSITIVE = {
    "xlang_range_step_stable_entrances.rs":
        "99a7243215ed384b2ba7fa0890bddbf3c05463e2c2b293552bda2f637da9208d",
}
NEGATIVE = {
    "xlang_range_step_ascii_char_rejected.rs": {
        "source_sha256": "a98c1a8c266b173c219127931f44f7c170076127252310886e04440d8465437b",
        "diagnostic_sha256": "b1f432d1a91a1990bdba1d121664649a898cb5119322bc45af54ce7215c01f31",
        "predicate": "error[E0658]: use of unstable library feature `ascii_char`",
        "count": 3,
    },
    "xlang_range_step_downstream_impl_rejected.rs": {
        "source_sha256": "aa9f46fa6f2aea5a850cf569f7f87867cca4d197c9f221ea083e4830b06171ce",
        "diagnostic_sha256": "a55b984ed695332c1b2ffdcfafbe20d0fa632403a2f9e7cddfc6fdd017a96733",
        "predicate": "error[E0658]: use of unstable library feature `step_trait`",
        "count": 4,
    },
}


def fail(message: str) -> None:
    raise SystemExit(f"Trait-implementation canary verification failed: {message}")


def run(command: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            command,
            cwd=cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except FileNotFoundError as error:
        fail(f"command is unavailable: {command[0]} ({error})")


def verify_toolchain() -> None:
    result = run(["rustc", "+1.97.0", "--version", "--verbose"], ROOT)
    if result.returncode != 0:
        fail("rustc +1.97.0 is unavailable:\n" + (result.stdout + result.stderr).rstrip())
    fields = {}
    lines = result.stdout.splitlines()
    if lines:
        fields["version"] = lines[0]
    for line in lines[1:]:
        key, separator, value = line.partition(":")
        if separator:
            fields[key.strip()] = value.strip()
    if fields.get("version") != EXPECTED_VERSION:
        fail(f"unexpected Rust version: {fields.get('version')!r}")
    if fields.get("commit-hash") != EXPECTED_COMMIT:
        fail(f"unexpected Rust commit: {fields.get('commit-hash')!r}")
    if fields.get("release") != "1.97.0":
        fail(f"unexpected Rust release: {fields.get('release')!r}")


def verify_hashes() -> None:
    expected = {**POSITIVE, **{name: row["source_sha256"] for name, row in NEGATIVE.items()}}
    for name, expected_hash in expected.items():
        actual = hashlib.sha256((CANARY_ROOT / name).read_bytes()).hexdigest()
        if actual != expected_hash:
            fail(f"source hash mismatch for {name}: {actual}")


def compile_canaries() -> None:
    with tempfile.TemporaryDirectory(prefix="xlang-trait-impl-canaries-") as directory:
        output_root = Path(directory)
        for name in POSITIVE:
            output = output_root / Path(name).stem
            result = run(
                [
                    "rustc", "+1.97.0", "--edition=2024",
                    "-C", "opt-level=0", "-C", "debuginfo=0",
                    name, "-o", str(output),
                ],
                CANARY_ROOT,
            )
            if result.returncode != 0:
                fail(f"positive compile failed for {name}:\n{result.stderr.rstrip()}")
            execution = run([str(output)], CANARY_ROOT)
            if execution.returncode != 0 or execution.stdout or execution.stderr:
                fail(f"positive canary did not execute silently for {name}")

        for name, expected in NEGATIVE.items():
            output = output_root / Path(name).stem
            result = run(
                [
                    "rustc", "+1.97.0", "--edition=2024", "--color=never",
                    "--error-format=short", name, "-o", str(output),
                ],
                CANARY_ROOT,
            )
            if result.returncode == 0:
                fail(f"negative canary unexpectedly compiled: {name}")
            if result.stdout:
                fail(f"negative canary emitted unexpected stdout: {name}")
            actual_digest = hashlib.sha256(result.stderr.encode("utf-8")).hexdigest()
            if actual_digest != expected["diagnostic_sha256"]:
                fail(f"diagnostic digest changed for {name}: {actual_digest}")
            if result.stderr.count(expected["predicate"]) != expected["count"]:
                fail(f"diagnostic predicate count changed for {name}")
            error_lines = [line for line in result.stderr.splitlines() if "error[" in line]
            if len(error_lines) != expected["count"]:
                fail(f"unexpected additional compiler errors for {name}: {error_lines}")


def main() -> None:
    verify_toolchain()
    verify_hashes()
    compile_canaries()
    print(
        "Trait-implementation canaries: PASS "
        "(21 stable Step endpoints compile; AsciiChar and downstream Step reject)"
    )


if __name__ == "__main__":
    main()
