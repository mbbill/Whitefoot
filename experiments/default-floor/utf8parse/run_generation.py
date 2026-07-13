#!/usr/bin/env python3
"""Launch the single preregistered Terra utf8parse trajectory."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tarfile

from benchmark import (
    audit_cargo_config_absence,
    canonical_generation_input_paths,
    canonical_generation_tool_manifest,
    sanitized_build_environment,
    CORPUS_CASES,
    verify_utf8parse_registry,
)


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
DEFAULT_FLOOR = HERE.parent
CODEX = Path("/opt/homebrew/bin/codex")
PYTHON_NATIVE = Path(
    "/Applications/Xcode.app/Contents/Developer/Library/Frameworks/"
    "Python3.framework/Versions/3.9/bin/python3.9"
)
PYTHON = PYTHON_NATIVE
ADAPTER = DEFAULT_FLOOR / "codex_model_adapter.py"
GENERATOR = DEFAULT_FLOOR / "generate.py"
EVALUATOR = HERE / "verify.py"
PROMPT = HERE / "base-prompt.txt"
RUN_DIR = HERE / "runs" / "primary-terra-medium-preregistered"
MODEL = "gpt-5.6-terra"
PROMPT_SEPARATOR = b"\n===== BEGIN COMPLETE XLANG WRITER'S PACK =====\n\n"
PROMPT_SHA256 = "81f023e583987d4610f15faa529b6481805dc4094fda2168146cf9ea9e9c903a"
COMPONENT_SHA256 = {
    "task.md": "9f301b9a0776b855439fb23d403e990ebc5ce8b2add9730c4040de99071732d9",
    "teaching-pack.md": "88917635d551c9352fd788a0c339369e65ad54459ae16157b566fb0e05782672",
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def checked_output(argv: list[str]) -> str:
    completed = subprocess.run(
        argv,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"metadata command failed: {argv[0]}: {completed.stderr.strip()}")
    return completed.stdout.strip()


def input_paths() -> tuple[Path, ...]:
    return canonical_generation_input_paths()


def file_hashes(paths: tuple[Path, ...]) -> dict[str, str]:
    return {str(path.relative_to(ROOT)): sha256_file(path) for path in paths}


def require_committed_inputs(paths: tuple[Path, ...]) -> None:
    labels = [str(path.relative_to(ROOT)) for path in paths]
    for path in paths:
        if not path.is_file():
            raise RuntimeError(f"preregistered input is missing: {path}")
    tracked = set(checked_output(["git", "ls-files", "--", *labels]).splitlines())
    missing = sorted(set(labels) - tracked)
    if missing:
        raise RuntimeError(f"preregistered inputs are not tracked: {missing}")
    dirty = checked_output(
        ["git", "status", "--porcelain=v1", "--untracked-files=all", "--", *labels]
    )
    if dirty:
        raise RuntimeError(f"preregistered inputs differ from HEAD:\n{dirty}")


def tool_manifest() -> dict[str, object]:
    return canonical_generation_tool_manifest()


def main() -> int:
    try:
        if len(sys.argv) != 1:
            raise RuntimeError("the preregistration launcher accepts no arguments")
        if RUN_DIR.exists():
            raise RuntimeError(f"preregistered run directory already exists: {RUN_DIR}")
        assembled_prompt = (
            (HERE / "task.md").read_bytes()
            + PROMPT_SEPARATOR
            + (HERE / "teaching-pack.md").read_bytes()
        )
        if PROMPT.read_bytes() != assembled_prompt:
            raise RuntimeError("base-prompt.txt is not the exact component assembly")
        if sha256_file(PROMPT) != PROMPT_SHA256:
            raise RuntimeError("base-prompt.txt does not match the preregistered hash")
        for name, expected in COMPONENT_SHA256.items():
            if sha256_file(HERE / name) != expected:
                raise RuntimeError(f"{name} does not match the preregistered hash")
        paths = input_paths()
        require_committed_inputs(paths)
        tools = tool_manifest()
        cargo_config_absence = audit_cargo_config_absence(ROOT)
        registry = verify_utf8parse_registry()
        codex_version = tools["versions"]["codex"]
        revision = checked_output(["git", "rev-parse", "HEAD"])
        status = checked_output(["git", "status", "--porcelain=v1", "--untracked-files=all"])
        hashes = file_hashes(paths)
    except (OSError, RuntimeError, tarfile.TarError) as error:
        print(f"generation preflight failed: {error}", file=sys.stderr)
        return 2

    model_argv = [
        str(PYTHON),
        str(ADAPTER),
        "--codex",
        str(CODEX),
        "--model",
        MODEL,
        "--reasoning",
        "medium",
        "--service-tier",
        "default",
        "--timeout",
        "600",
    ]
    evaluator_argv = [str(PYTHON), str(EVALUATOR)]
    common = {
        "host_platform": platform.platform(),
        "python": platform.python_version(),
        "repository_revision": revision,
        "git_status_porcelain_v1": status.splitlines(),
        "cargo_config_absence": cargo_config_absence,
    }
    model_metadata = {
        **common,
        "surface": "codex-cli",
        "codex_version": codex_version,
        "tool_manifest": tools,
        "model": MODEL,
        "reasoning_effort": "medium",
        "service_tier": "default",
        "ephemeral": True,
        "sandbox": "read-only",
        "user_config": "ignored",
        "repository_rules": "ignored",
        "event_boundary": "exact-four-event-single-agent-message-no-tools",
        "input_hashes": {
            str(ADAPTER.relative_to(ROOT)): hashes[str(ADAPTER.relative_to(ROOT))],
            str(GENERATOR.relative_to(ROOT)): hashes[str(GENERATOR.relative_to(ROOT))],
            str(PROMPT.relative_to(ROOT)): hashes[str(PROMPT.relative_to(ROOT))],
        },
    }
    evaluator_metadata = {
        **common,
        "kind": "utf8parse-compile-and-correctness",
        "corpus_cases": CORPUS_CASES,
        "proof_feedback": "disabled-before-freeze",
        "tool_manifest": tools,
        "utf8parse_registry": registry,
        "input_hashes": hashes,
    }
    argv = [
        str(PYTHON),
        str(GENERATOR),
        "--run-dir",
        str(RUN_DIR),
        "--prompt-file",
        str(PROMPT),
        "--model-argv-json",
        json.dumps(model_argv, separators=(",", ":")),
        "--evaluator-argv-json",
        json.dumps(evaluator_argv, separators=(",", ":")),
        "--public-model-metadata-json",
        json.dumps(model_metadata, sort_keys=True, separators=(",", ":")),
        "--public-evaluator-metadata-json",
        json.dumps(evaluator_metadata, sort_keys=True, separators=(",", ":")),
        "--repair-budget",
        "3",
        "--source-name",
        "utf8parse.xl",
        "--model-timeout",
        "660",
        "--evaluator-timeout",
        "900",
    ]
    environment, _removed = sanitized_build_environment()
    if audit_cargo_config_absence(ROOT) != cargo_config_absence:
        raise RuntimeError("Cargo configuration search chain changed during preflight")
    os.execve(str(PYTHON), argv, environment)
    return 127


if __name__ == "__main__":
    raise SystemExit(main())
