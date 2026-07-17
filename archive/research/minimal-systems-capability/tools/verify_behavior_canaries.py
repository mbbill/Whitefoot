#!/usr/bin/env python3
"""Verify the pinned Rust behavior canaries used by G0-Core."""

from __future__ import annotations

import hashlib
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CANARY_ROOT = ROOT / "canaries"
EXPECTED_VERSION = "rustc 1.97.0 (2d8144b78 2026-07-07)"
EXPECTED_COMMIT = "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3"
CANARIES = {
    "xlang_buildhasher_root_swap.rs": "76588ebc4bf1cc9c191e4b08f3cee00dffd96fa39e315ab3bd5a057bc7aa9a09",
    "xlang_buildhasher_transfer.rs": "0a366e560f3eaf10b85e8bee963a0204d218e3df4766e758e718087c12e9d962",
    "xlang_clone_source_effects.rs": "456a9ffe70b4df2b90c9d4eb0edf353aeb41f3b135b06e8b31da931293730642",
    "xlang_clone_helper_source_effects.rs": "6120847f27ff814f60f0f92a96ff016970ca64cb32aa3718cee6f96607be1284",
    "xlang_behavior_receiver_effects.rs": "ac42b0d6cd70a8ee3b04528184335768f725ee7f925c9d7253897cb8523fdef8",
    "xlang_repeat_n_source_effects.rs": "ca5cdad81136a15d5b5291a670963d8c4c9a97436bdfd85fb39ed646fe9f3ff5",
}
OTHER_CANARY_SOURCES = {
    "xlang_range_step_stable_entrances.rs",
    "xlang_range_step_ascii_char_rejected.rs",
    "xlang_range_step_downstream_impl_rejected.rs",
}


def fail(message: str) -> None:
    raise SystemExit(f"Behavior-canary verification failed: {message}")


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
    except FileNotFoundError as error:
        fail(f"command is unavailable: {command[0]} ({error})")


def verify_toolchain() -> None:
    result = run(["rustc", "+1.97.0", "--version", "--verbose"])
    if result.returncode != 0:
        fail("rustc +1.97.0 is unavailable:\n" + result.stdout.rstrip())
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


def verify_source_hashes() -> None:
    actual_sources = sorted(path.name for path in CANARY_ROOT.glob("*.rs"))
    if actual_sources != sorted(set(CANARIES) | OTHER_CANARY_SOURCES):
        fail("canary source set differs from the exact pinned set")
    for name, expected_hash in CANARIES.items():
        path = CANARY_ROOT / name
        actual_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            fail(f"source hash mismatch for {name}: {actual_hash}")


def compile_and_run() -> None:
    with tempfile.TemporaryDirectory(prefix="xlang-g0-canaries-") as directory:
        output_root = Path(directory)
        for name in CANARIES:
            source = CANARY_ROOT / name
            output = output_root / source.stem
            compile_result = run(
                [
                    "rustc",
                    "+1.97.0",
                    "--edition=2024",
                    "-C",
                    "opt-level=0",
                    "-C",
                    "debuginfo=0",
                    str(source),
                    "-o",
                    str(output),
                ]
            )
            if compile_result.returncode != 0:
                fail(f"compile failed for {name}:\n{compile_result.stdout.rstrip()}")
            run_result = run([str(output)])
            if run_result.returncode != 0:
                fail(f"execution failed for {name}:\n{run_result.stdout.rstrip()}")


def main() -> None:
    verify_toolchain()
    verify_source_hashes()
    compile_and_run()
    print(
        "Behavior canaries: PASS "
        f"({len(CANARIES)} exact sources, Rust 1.97.0 {EXPECTED_COMMIT[:10]})"
    )


if __name__ == "__main__":
    main()
