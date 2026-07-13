#!/usr/bin/env python3
"""Build and orchestrate the frozen utf8parse benchmark campaign.

`score` is the preregistered 30-process measurement and requires an explicit
acknowledgement flag. `smoke` links a Rust-backed shim in place of both xlang
symbols, uses a small corpus, and emits validation artifacts explicitly marked
as non-scoring.
"""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import signal
import subprocess
import sys
import tarfile
from typing import Any, Iterable


sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
HARNESS = HERE / "harness"
ANALYZER = HERE / "analyze.py"
TARGET_RUN_DIR = (HERE / "runs" / "primary-terra-medium-preregistered").resolve()
PYTHON = Path(
    "/Applications/Xcode.app/Contents/Developer/Library/Frameworks/"
    "Python3.framework/Versions/3.9/bin/python3.9"
)
CLANG = Path(
    "/Applications/Xcode.app/Contents/Developer/Toolchains/"
    "XcodeDefault.xctoolchain/usr/bin/clang"
)
MACOS_SDK = Path(
    "/Applications/Xcode.app/Contents/Developer/Platforms/"
    "MacOSX.platform/Developer/SDKs/MacOSX.sdk"
)
MACOS_SDK_SETTINGS = MACOS_SDK / "SDKSettings.json"
PMSET = Path("/usr/bin/pmset")
SYSCTL = Path("/usr/sbin/sysctl")
SYSTEM_PROFILER = Path("/usr/sbin/system_profiler")
CARGO = Path("/Users/bytedance/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo")
RUSTC = Path("/Users/bytedance/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustc")
CODEX = Path("/opt/homebrew/bin/codex")
CODEX_JAVASCRIPT = Path(
    "/opt/homebrew/lib/node_modules/@openai/codex/bin/codex.js"
)
CODEX_NATIVE = Path(
    "/opt/homebrew/lib/node_modules/@openai/codex/node_modules/"
    "@openai/codex-darwin-arm64/vendor/aarch64-apple-darwin/bin/codex"
)
ADAPTER = HERE.parent / "codex_model_adapter.py"
GENERATOR = HERE.parent / "generate.py"
EVALUATOR = HERE / "verify.py"
PROMPT = HERE / "base-prompt.txt"
CARGO_HOME = Path("/Users/bytedance/.cargo")
NATIVE_TARGET = "aarch64-apple-darwin"
EXPECTED_BUILD_TOOL_SHA256 = {
    str(CARGO): "696d29bd6f5a136ef6c7088558e178efcabbd571de90e8169d0f6596c568eea8",
    str(RUSTC): "f556cd7caa76d85ff0e777ba8e96552b33b4e0ea0dda8fbe6cde1d23b6c4df27",
    str(CLANG): "7def90dd8829726686213a747fc5bff1583df933dae5edc55d755479e0bfe00a",
    str(PYTHON): "271143990bc83af0fb2404a255038f5faafb96df1584ed7f085e5018c0f33ffb",
    str(MACOS_SDK_SETTINGS): "f8d005f09381389167f9e0aeaa169bc9e7dff162ef22ca2fd8e98df7ff1acafe",
}
EXPECTED_GENERATION_TOOL_SHA256 = {
    str(CODEX_JAVASCRIPT): "134063e133f0b4244fa3b251acf973d4fe4b4aeeacbdc135211bf480f59f1477",
    str(CODEX_NATIVE): "978740e6bcbd9af2f850823b723fb74f16d8d1e44de05f7dd6737ae631f72017",
    **EXPECTED_BUILD_TOOL_SHA256,
}
EXPECTED_IRRELEVANT_CLANG_CONFIG_SHA256 = {
    str(CLANG.parent / "aarch64-swift-linux-musl-clang++.cfg"):
        "46dee6b61cb6efb3748a08aefe5088961f7435767491e2b719d6e939d9a0dcf0",
    str(CLANG.parent / "aarch64-swift-linux-musl-clang.cfg"):
        "46dee6b61cb6efb3748a08aefe5088961f7435767491e2b719d6e939d9a0dcf0",
    str(CLANG.parent / "x86_64-swift-linux-musl-clang++.cfg"):
        "b8ae741f12cebb7e1a4f3782ee75f83c0e4396a9c4dbdbcd3192d6ce48824afa",
    str(CLANG.parent / "x86_64-swift-linux-musl-clang.cfg"):
        "b8ae741f12cebb7e1a4f3782ee75f83c0e4396a9c4dbdbcd3192d6ce48824afa",
}
EXPECTED_IRRELEVANT_CLANG_CONFIG_SYMLINKS = {
    str(CLANG.parent / "aarch64-swift-linux-musl-clang++.cfg"):
        "aarch64-swift-linux-musl-clang.cfg",
    str(CLANG.parent / "x86_64-swift-linux-musl-clang++.cfg"):
        "x86_64-swift-linux-musl-clang.cfg",
}
SCORE_BYTES = 134_217_728
DEFAULT_SMOKE_BYTES = 1_048_576
CORPUS_CASES = 84_041
ORDER_SEED = 0x50444F5244455233
MASK64 = (1 << 64) - 1
VARIANTS = ("facts-on", "facts-off", "rust")
# FNR, FRN, NFR, NRF, RFN, and RNF in the protocol's frozen visit order.
ORDER_STRATA = (
    "facts-on,facts-off,rust",
    "facts-on,rust,facts-off",
    "facts-off,facts-on,rust",
    "facts-off,rust,facts-on",
    "rust,facts-on,facts-off",
    "rust,facts-off,facts-on",
)
FORBIDDEN_BUILD_FLAGS = (
    "target-cpu=native",
    "-ctarget-cpu",
    "-c target-cpu",
    "-ctarget-feature",
    "-c target-feature",
    "-march=",
    "-mcpu=",
    "profile-generate",
    "profile-use",
    "-c lto",
    "-clto",
    "-flto",
)


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f"could not read {label} JSON at {path}: {error}") from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{label} at {path} is not a JSON object")
    return value


def require_sha256(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(byte not in "0123456789abcdef" for byte in value)
    ):
        raise RuntimeError(f"{label} is not a lowercase SHA-256")
    return value


def argv_sha256(argv: list[str]) -> str:
    encoded = json.dumps(argv, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def canonical_generation_input_paths() -> tuple[Path, ...]:
    """Return the one exact preregistered generation-input set."""
    default_floor = HERE.parent
    return (
        ADAPTER,
        GENERATOR,
        default_floor / "PROTOCOL.md",
        default_floor / "README.md",
        default_floor / "tests" / "__init__.py",
        default_floor / "tests" / "mock_codex_cli.py",
        default_floor / "tests" / "mock_evaluator.py",
        default_floor / "tests" / "mock_model.py",
        default_floor / "tests" / "test_generate.py",
        default_floor / "tests" / "test_codex_model.py",
        EVALUATOR,
        PROMPT,
        HERE / "PROTOCOL.md",
        HERE / "BENCHMARK.md",
        HERE / ".gitignore",
        HERE / "analyze.py",
        HERE / "assemble_prompt.py",
        HERE / "benchmark.py",
        HERE / "run_generation.py",
        HERE / "task.md",
        HERE / "teaching-pack.md",
        HERE / "boundary.c",
        HARNESS / "Cargo.toml",
        HARNESS / "Cargo.lock",
        HARNESS / "src" / "bin" / "bench.rs",
        HARNESS / "src" / "bin" / "verify.rs",
        HERE / "rust-baseline" / "Cargo.toml",
        HERE / "rust-baseline" / "Cargo.lock",
        HERE / "rust-baseline" / "README.md",
        HERE / "rust-baseline" / "src" / "lib.rs",
        ROOT / "prototype" / "democ" / "democ.py",
        ROOT / "prototype" / "checker" / "checker.py",
    )


def cargo_config_search_paths(cwd: Path) -> list[Path]:
    """Enumerate Cargo's cwd-to-root search plus the locked Cargo home."""
    resolved = cwd.resolve()
    candidates: list[Path] = []
    for directory in (resolved, *resolved.parents):
        candidates.extend(
            (directory / ".cargo" / "config.toml", directory / ".cargo" / "config")
        )
    candidates.extend((CARGO_HOME / "config.toml", CARGO_HOME / "config"))
    unique: list[Path] = []
    seen: set[str] = set()
    for candidate in candidates:
        label = str(candidate)
        if label not in seen:
            seen.add(label)
            unique.append(candidate)
    return unique


def audit_cargo_config_absence(cwd: Path) -> dict[str, Any]:
    searched = cargo_config_search_paths(cwd)
    present = [path for path in searched if os.path.lexists(path)]
    if present:
        raise RuntimeError(
            "Cargo configuration injection point is present: "
            + ", ".join(str(path) for path in present)
        )
    return {
        "schema_version": 1,
        "status": "all-search-paths-absent",
        "search_cwd": str(cwd.resolve()),
        "cargo_home": str(CARGO_HOME),
        "searched_paths": [str(path) for path in searched],
    }


def cargo_cli_neutralization() -> list[str]:
    return [
        "--config",
        f'build.rustc="{RUSTC}"',
        "--config",
        'build.rustc-wrapper=""',
        "--config",
        'build.rustc-workspace-wrapper=""',
        "--config",
        "build.rustflags=[]",
        "--config",
        "build.rustdocflags=[]",
        "--config",
        "build.incremental=false",
        "--config",
        f'build.target="{NATIVE_TARGET}"',
        "--config",
        f'target.{NATIVE_TARGET}.linker="{CLANG}"',
        "--config",
        f"target.{NATIVE_TARGET}.rustflags=[]",
    ]


def validate_target_generation_config(
    config: dict[str, Any], run_dir: Path, *, require_target_identity: bool = True
) -> None:
    if require_target_identity and run_dir != TARGET_RUN_DIR:
        raise RuntimeError(f"generation run is not the preregistered identity: {run_dir}")
    if (
        config.get("repair_budget") != 3
        or config.get("max_rounds") != 4
        or config.get("source_name") != "utf8parse.xl"
        or config.get("model_timeout_seconds") != 660
        or config.get("evaluator_timeout_seconds") != 900
    ):
        raise RuntimeError("generation config does not match the utf8parse budget/timeouts")
    model_invocation = config["model_invocation"]
    evaluator_invocation = config["evaluator_invocation"]
    expected_model_argv = [
        str(PYTHON), str(ADAPTER), "--codex", str(CODEX), "--model",
        "gpt-5.6-terra", "--reasoning", "medium", "--service-tier", "default",
        "--timeout", "600",
    ]
    expected_evaluator_argv = [str(PYTHON), str(EVALUATOR)]
    if (
        model_invocation.get("argv_sha256") != argv_sha256(expected_model_argv)
        or model_invocation.get("argv_items") != len(expected_model_argv)
        or evaluator_invocation.get("argv_without_candidate_sha256")
        != argv_sha256(expected_evaluator_argv)
        or evaluator_invocation.get("argv_items_without_candidate")
        != len(expected_evaluator_argv)
    ):
        raise RuntimeError("generation invocation argv hash does not match the target launcher")
    model = model_invocation.get("public_metadata")
    evaluator = evaluator_invocation.get("public_metadata")
    if not isinstance(model, dict) or not isinstance(evaluator, dict):
        raise RuntimeError("generation config lacks public invocation metadata")
    required_model = {
        "surface": "codex-cli", "codex_version": "codex-cli 0.144.0",
        "model": "gpt-5.6-terra", "reasoning_effort": "medium",
        "service_tier": "default", "ephemeral": True,
        "sandbox": "read-only", "user_config": "ignored",
        "repository_rules": "ignored",
        "event_boundary": "exact-four-event-single-agent-message-no-tools",
    }
    required_evaluator = {
        "kind": "utf8parse-compile-and-correctness",
        "corpus_cases": CORPUS_CASES,
        "proof_feedback": "disabled-before-freeze",
    }
    if any(model.get(key) != value for key, value in required_model.items()):
        raise RuntimeError("generation model metadata is not Terra/medium preregistration")
    if any(evaluator.get(key) != value for key, value in required_evaluator.items()):
        raise RuntimeError("generation evaluator metadata is not the frozen correctness gate")
    if evaluator.get("utf8parse_registry") != verify_utf8parse_registry():
        raise RuntimeError("generation evaluator registry binding is missing or stale")
    input_hashes = evaluator.get("input_hashes")
    if not isinstance(input_hashes, dict):
        raise RuntimeError("generation evaluator metadata lacks input_hashes")
    required_inputs = {
        str(path.relative_to(ROOT)) for path in canonical_generation_input_paths()
    }
    if set(input_hashes) != required_inputs:
        missing = sorted(required_inputs - set(input_hashes))
        extra = sorted(set(input_hashes) - required_inputs)
        raise RuntimeError(
            "generation input hash keys differ from canonical input_paths: "
            f"missing={missing}, extra={extra}"
        )
    for relative, expected in input_hashes.items():
        path = resolve_run_artifact(ROOT, relative, f"generation input {relative}")
        if sha256_file(path) != require_sha256(expected, f"generation input {relative}"):
            raise RuntimeError(f"generation input changed after freeze: {relative}")
    model_inputs = model.get("input_hashes")
    expected_model_inputs = {
        str(path.relative_to(ROOT)): input_hashes[str(path.relative_to(ROOT))]
        for path in (ADAPTER, GENERATOR, PROMPT)
    }
    if model_inputs != expected_model_inputs:
        raise RuntimeError("generation model input hash subset is inconsistent")
    canonical_tools = canonical_generation_tool_manifest()
    if (
        model.get("tool_manifest") != canonical_tools
        or evaluator.get("tool_manifest") != canonical_tools
    ):
        raise RuntimeError("generation tool manifest is not the complete locked manifest")
    canonical_cargo_audit = audit_cargo_config_absence(ROOT)
    if (
        model.get("cargo_config_absence") != canonical_cargo_audit
        or evaluator.get("cargo_config_absence") != canonical_cargo_audit
    ):
        raise RuntimeError("generation Cargo config absence audit is missing or stale")


def resolve_run_artifact(run_dir: Path, relative: Any, label: str) -> Path:
    if not isinstance(relative, str) or not relative:
        raise RuntimeError(f"{label}.path must be a nonempty relative path")
    raw = Path(relative)
    if raw.is_absolute() or ".." in raw.parts:
        raise RuntimeError(f"{label}.path escapes the generation run: {relative!r}")
    resolved = (run_dir / raw).resolve()
    try:
        resolved.relative_to(run_dir)
    except ValueError as error:
        raise RuntimeError(
            f"{label}.path resolves outside the generation run: {relative!r}"
        ) from error
    if not resolved.is_file():
        raise RuntimeError(f"{label} artifact is missing: {resolved}")
    return resolved


def verify_artifact(
    run_dir: Path, specification: Any, label: str
) -> tuple[Path, str]:
    if not isinstance(specification, dict):
        raise RuntimeError(f"{label} binding is not an object")
    path = resolve_run_artifact(run_dir, specification.get("path"), label)
    expected = require_sha256(specification.get("sha256"), f"{label}.sha256")
    actual = sha256_file(path)
    if actual != expected:
        raise RuntimeError(
            f"{label} SHA-256 mismatch: manifest {expected}, artifact {actual}"
        )
    return path, expected


def validate_freeze_manifest(
    manifest_argument: Path,
    *,
    require_target_identity: bool = True,
    archived_result_paths_are_historical: bool = False,
) -> tuple[dict[str, Any], Path, str]:
    manifest_path = manifest_argument.resolve()
    if manifest_path.name != "trace-manifest.json" or manifest_path.parent.name != "frozen":
        raise RuntimeError(
            "--trace-manifest must name the generator's frozen/trace-manifest.json"
        )
    run_dir = manifest_path.parent.parent.resolve()
    manifest = read_json_object(manifest_path, "generation trace manifest")
    if manifest.get("protocol_version") != 1 or manifest.get("status") != "frozen":
        raise RuntimeError("generation manifest is not a protocol-v1 frozen result")
    frozen_round = manifest.get("frozen_round")
    if type(frozen_round) is not int or frozen_round < 0:
        raise RuntimeError("generation manifest has an invalid frozen_round")

    config_path, config_sha = verify_artifact(
        run_dir, manifest.get("config"), "generation config"
    )
    source_path, source_sha = verify_artifact(
        run_dir, manifest.get("source"), "frozen source"
    )
    trace_specification = manifest.get("trace")
    trace_path, trace_sha = verify_artifact(
        run_dir, trace_specification, "generation trace"
    )
    if config_path != run_dir / "config.json" or trace_path != run_dir / "trace.jsonl":
        raise RuntimeError("generation manifest does not bind the canonical config/trace paths")

    config = read_json_object(config_path, "generation config")
    repair_budget = config.get("repair_budget")
    if (
        config.get("protocol_version") != 1
        or config.get("trajectory_count") != 1
        or type(repair_budget) is not int
        or repair_budget < 0
        or config.get("max_rounds") != repair_budget + 1
        or frozen_round > repair_budget
    ):
        raise RuntimeError("generation config violates the one-trajectory repair budget")
    source_name = config.get("source_name")
    if (
        not isinstance(source_name, str)
        or Path(source_name).name != source_name
        or source_path != run_dir / "frozen" / source_name
    ):
        raise RuntimeError("generation config/source path binding is inconsistent")
    for invocation in ("model_invocation", "evaluator_invocation"):
        value = config.get(invocation)
        if not isinstance(value, dict):
            raise RuntimeError(f"generation config lacks {invocation}")
        hash_fields = [key for key in value if key.endswith("sha256")]
        if len(hash_fields) != 1:
            raise RuntimeError(f"generation config {invocation} has no unique argv hash")
        require_sha256(value[hash_fields[0]], f"generation config {invocation} argv hash")
    validate_target_generation_config(
        config, run_dir, require_target_identity=require_target_identity
    )

    if not isinstance(trace_specification, dict):
        raise RuntimeError("generation trace binding is not an object")
    manifest_rounds = trace_specification.get("rounds")
    if not isinstance(manifest_rounds, list) or len(manifest_rounds) != frozen_round + 1:
        raise RuntimeError("generation manifest round list does not end at frozen_round")
    trace_rounds: list[Any] = []
    try:
        for line_number, line in enumerate(trace_path.read_text(encoding="utf-8").splitlines(), 1):
            if not line:
                raise RuntimeError(f"generation trace contains a blank line at {line_number}")
            trace_rounds.append(json.loads(line))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f"generation trace is not valid UTF-8 JSONL: {error}") from error
    if trace_rounds != manifest_rounds:
        raise RuntimeError("generation trace.jsonl does not equal manifest.trace.rounds")

    rounds_dir = run_dir / "rounds"
    expected_round_directories = {f"{index:03d}" for index in range(frozen_round + 1)}
    if rounds_dir.is_symlink() or not rounds_dir.is_dir():
        raise RuntimeError("generation rounds path is not a real directory")
    observed_round_directories: set[str] = set()
    for entry in os.scandir(rounds_dir):
        if entry.is_symlink() or not entry.is_dir(follow_symlinks=False):
            raise RuntimeError(f"generation rounds contains a non-directory: {entry.path}")
        observed_round_directories.add(entry.name)
    if observed_round_directories != expected_round_directories:
        raise RuntimeError(
            "generation has missing, extra, or post-freeze round directories: "
            f"{sorted(observed_round_directories)}"
        )

    artifacts_verified = 0
    for expected_round, record in enumerate(manifest_rounds):
        if not isinstance(record, dict):
            raise RuntimeError(f"generation round {expected_round} is not an object")
        if (
            record.get("protocol_version") != 1
            or record.get("round") != expected_round
            or type(record.get("correct")) is not bool
            or record["correct"] is not (expected_round == frozen_round)
        ):
            raise RuntimeError(f"generation round {expected_round} has invalid freeze state")
        record_path = run_dir / "rounds" / f"{expected_round:03d}" / "record.json"
        if record_path.is_symlink() or not record_path.is_file():
            raise RuntimeError(f"generation round {expected_round} record is not a regular file")
        if read_json_object(record_path, f"generation round {expected_round} record") != record:
            raise RuntimeError(f"generation round {expected_round} record.json disagrees")
        artifacts = record.get("artifacts")
        round_prefix = f"rounds/{expected_round:03d}"
        expected_artifact_paths = {
            "prompt": f"{round_prefix}/prompt.txt",
            "raw": f"{round_prefix}/model.raw.txt",
            "source": f"{round_prefix}/{source_name}",
            "model_stderr": f"{round_prefix}/model.stderr.txt",
            "model_process": f"{round_prefix}/model-process.json",
            "evaluator_raw": f"{round_prefix}/evaluator.raw.txt",
            "evaluator_stderr": f"{round_prefix}/evaluator.stderr.txt",
            "evaluator_process": f"{round_prefix}/evaluator-process.json",
            "evaluator": f"{round_prefix}/evaluator.json",
        }
        if not isinstance(artifacts, dict) or set(artifacts) != set(expected_artifact_paths):
            raise RuntimeError(f"generation round {expected_round} artifact set is not canonical")
        artifact_paths: dict[str, Path] = {}
        for artifact_name, specification in artifacts.items():
            if (
                not isinstance(specification, dict)
                or specification.get("path") != expected_artifact_paths[artifact_name]
            ):
                raise RuntimeError(
                    f"generation round {expected_round} artifact {artifact_name} path is not canonical"
                )
            artifact_path, _artifact_sha = verify_artifact(
                run_dir,
                specification,
                f"generation round {expected_round} artifact {artifact_name}",
            )
            artifact_paths[artifact_name] = artifact_path
            artifacts_verified += 1
        if artifact_paths["raw"].read_bytes() != artifact_paths["source"].read_bytes():
            raise RuntimeError(f"generation round {expected_round} model.raw differs from source")
        evaluator_value = read_json_object(
            artifact_paths["evaluator"], f"generation round {expected_round} evaluator"
        )
        if evaluator_value != record.get("evaluator"):
            raise RuntimeError(f"generation round {expected_round} evaluator.json disagrees")
        compile_channel = evaluator_value.get("compile")
        correctness_channel = evaluator_value.get("correctness")
        if not isinstance(compile_channel, dict) or not isinstance(correctness_channel, dict):
            raise RuntimeError(f"generation round {expected_round} evaluator lacks channels")
        recomputed_correct = (
            compile_channel.get("passed") is True
            and correctness_channel.get("passed") is True
        )
        if record["correct"] is not recomputed_correct:
            raise RuntimeError(f"generation round {expected_round} correct flag was not recomputed")
        model_process = read_json_object(
            artifact_paths["model_process"], f"generation round {expected_round} model process"
        )
        evaluator_process = read_json_object(
            artifact_paths["evaluator_process"],
            f"generation round {expected_round} evaluator process",
        )
        if model_process != {"returncode": 0, "process_failure": None}:
            raise RuntimeError(f"generation round {expected_round} model process was not clean")
        if evaluator_process != {
            "returncode": 0,
            "process_failure": None,
            "candidate_was_last_argument": True,
        }:
            raise RuntimeError(f"generation round {expected_round} evaluator process was not clean")
    last_source = manifest_rounds[-1]["artifacts"].get("source")
    _, last_source_sha = verify_artifact(
        run_dir, last_source, "frozen round source"
    )
    if last_source_sha != source_sha:
        raise RuntimeError("frozen source differs from the correctness-green round source")

    source_hash_path = manifest_path.parent / "source.sha256"
    expected_source_hash_file = f"{source_sha}  {source_name}\n".encode("ascii")
    if source_hash_path.read_bytes() != expected_source_hash_file:
        raise RuntimeError("frozen/source.sha256 does not bind the manifest source")
    result_path = run_dir / "result.json"
    result = read_json_object(result_path, "generation result")
    result_source = Path(result.get("source", ""))
    result_manifest = Path(result.get("manifest", ""))
    result_core_valid = (
        result.get("status") == "frozen"
        and result.get("round") == frozen_round
        and result.get("sha256") == source_sha
    )
    if archived_result_paths_are_historical:
        result_paths_valid = (
            result_source.is_absolute()
            and result_manifest.is_absolute()
            and result_source.name == source_name
            and result_source.parent.name == "frozen"
            and result_manifest.name == "trace-manifest.json"
            and result_manifest.parent.name == "frozen"
        )
        result_path_role = "historical-original-run-reference"
    else:
        result_paths_valid = (
            result_source.resolve() == source_path
            and result_manifest.resolve() == manifest_path
        )
        result_path_role = "live-canonical-run-binding"
    if not result_core_valid or not result_paths_valid:
        raise RuntimeError("generation result.json does not bind the frozen manifest/source")

    binding = {
        "protocol_version": 1,
        "status": "verified-frozen",
        "run_dir": str(run_dir),
        "manifest": {
            "path": str(manifest_path),
            "sha256": sha256_file(manifest_path),
            "value": manifest,
        },
        "config": {
            "path": str(config_path),
            "sha256": config_sha,
            "value": config,
        },
        "trace": {
            "path": str(trace_path),
            "sha256": trace_sha,
            "rounds": len(trace_rounds),
            "artifacts_verified": artifacts_verified,
        },
        "source": {"path": str(source_path), "sha256": source_sha},
        "source_sha256_file": {
            "path": str(source_hash_path),
            "sha256": sha256_file(source_hash_path),
        },
        "result": {
            "path": str(result_path),
            "sha256": sha256_file(result_path),
            "value": result,
            "absolute_path_fields": {
                "role": result_path_role,
                "source": str(result_source),
                "manifest": str(result_manifest),
            },
        },
        "frozen_round": frozen_round,
    }
    return binding, source_path, source_sha


def strict_regular_tree_manifest(root: Path) -> dict[str, Any]:
    if root.is_symlink() or not root.is_dir():
        raise RuntimeError(f"tree root is not a real directory: {root}")
    files: dict[str, str] = {}
    directories: list[str] = []

    def visit(directory: Path, relative: Path) -> None:
        try:
            entries = sorted(os.scandir(directory), key=lambda entry: entry.name)
        except OSError as error:
            raise RuntimeError(f"could not enumerate tree {directory}: {error}") from error
        for entry in entries:
            child_relative = relative / entry.name
            label = child_relative.as_posix()
            if entry.is_symlink():
                raise RuntimeError(f"tree contains a symbolic link: {root / child_relative}")
            if entry.is_dir(follow_symlinks=False):
                directories.append(label)
                visit(Path(entry.path), child_relative)
            elif entry.is_file(follow_symlinks=False):
                files[label] = sha256_file(Path(entry.path))
            else:
                raise RuntimeError(f"tree contains a special node: {root / child_relative}")

    visit(root, Path())
    return {"directories": sorted(directories), "files_sha256": dict(sorted(files.items()))}


def archive_validation_identity(binding: dict[str, Any]) -> dict[str, Any]:
    return {
        "manifest_sha256": binding["manifest"]["sha256"],
        "config_sha256": binding["config"]["sha256"],
        "trace_sha256": binding["trace"]["sha256"],
        "source_sha256": binding["source"]["sha256"],
        "source_sha256_file_sha256": binding["source_sha256_file"]["sha256"],
        "result_sha256": binding["result"]["sha256"],
        "frozen_round": binding["frozen_round"],
    }


def verify_generation_archive(record: dict[str, Any]) -> None:
    archive = Path(record["root"])
    current_tree = strict_regular_tree_manifest(archive)
    if current_tree != record.get("tree"):
        raise RuntimeError("generation-freeze copy changed or is incomplete")
    index_path = Path(record["index"]["path"])
    if sha256_file(index_path) != record["index"]["sha256"]:
        raise RuntimeError("generation-freeze index changed")
    index = read_json_object(index_path, "generation-freeze index")
    if index.get("tree") != current_tree:
        raise RuntimeError("generation-freeze index does not match copied tree")
    archived_binding, _source, _source_sha = validate_freeze_manifest(
        archive / "frozen" / "trace-manifest.json",
        require_target_identity=False,
        archived_result_paths_are_historical=True,
    )
    if archive_validation_identity(archived_binding) != record.get("validation_identity"):
        raise RuntimeError("generation-freeze copy no longer validates to the frozen identity")


def archive_generation_binding(output: Path, binding: dict[str, Any]) -> dict[str, Any]:
    source_root = Path(binding["run_dir"])
    archive = output / "generation-freeze"
    archive.mkdir()
    before = strict_regular_tree_manifest(source_root)
    for relative in before["directories"]:
        (archive / relative).mkdir(parents=True, exist_ok=False)
    for relative, expected_sha in before["files_sha256"].items():
        source = source_root / relative
        destination = archive / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())
        if sha256_file(destination) != expected_sha:
            raise RuntimeError(f"generation-freeze copy hash mismatch: {relative}")
    after = strict_regular_tree_manifest(source_root)
    copied = strict_regular_tree_manifest(archive)
    if before != after or copied != before:
        raise RuntimeError("generation run changed while making the complete freeze copy")

    archived_binding, _source, _source_sha = validate_freeze_manifest(
        archive / "frozen" / "trace-manifest.json",
        require_target_identity=False,
        archived_result_paths_are_historical=True,
    )
    validation_identity = archive_validation_identity(archived_binding)
    if validation_identity != archive_validation_identity(binding):
        raise RuntimeError("generation-freeze copy validates to a different identity")
    historical_paths = binding["result"]["absolute_path_fields"]
    index_value = {
        "schema_version": 1,
        "kind": "complete-generation-run-freeze",
        "original_run_dir": str(source_root),
        "archive_root": str(archive),
        "tree": copied,
        "validation_identity": validation_identity,
        "result_absolute_paths": {
            **historical_paths,
            "role": "historical-original-run-reference",
        },
    }
    index_path = output / "generation-freeze-index.json"
    atomic_json(index_path, index_value)
    record = {
        "schema_version": 1,
        "root": str(archive),
        "original_run_dir": str(source_root),
        "tree": copied,
        "validation_identity": validation_identity,
        "result_absolute_paths": index_value["result_absolute_paths"],
        "index": {"path": str(index_path), "sha256": sha256_file(index_path)},
    }
    verify_generation_archive(record)
    return record


def atomic_json(path: Path, value: Any) -> None:
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary, path)


def run_capture(
    argv: list[str],
    *,
    cwd: Path = ROOT,
    env: dict[str, str] | None = None,
    timeout: int | None = 120,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        argv,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=timeout,
    )


def checked_capture(
    argv: list[str],
    *,
    cwd: Path = ROOT,
    env: dict[str, str] | None = None,
    timeout: int = 120,
) -> str:
    completed = run_capture(argv, cwd=cwd, env=env, timeout=timeout)
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}): {argv!r}\n"
            f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return completed.stdout.strip()


def run_logged(
    argv: list[str],
    log_path: Path,
    *,
    cwd: Path = ROOT,
    env: dict[str, str] | None = None,
    timeout: int = 600,
) -> subprocess.CompletedProcess[str]:
    completed = run_capture(argv, cwd=cwd, env=env, timeout=timeout)
    log_path.write_text(
        "argv: " + json.dumps(argv) + "\n\nstdout:\n" + completed.stdout
        + "\n\nstderr:\n" + completed.stderr,
        encoding="utf-8",
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}); see {log_path}"
        )
    return completed


class XorShift64Star:
    def __init__(self, seed: int) -> None:
        self.state = seed

    def next(self) -> int:
        self.state ^= self.state >> 12
        self.state ^= (self.state << 25) & MASK64
        self.state ^= self.state >> 27
        self.state &= MASK64
        return (self.state * 2_685_821_657_736_338_717) & MASK64


def schedule(repetitions: int) -> list[str]:
    orders = list(ORDER_STRATA) * repetitions
    rng = XorShift64Star(ORDER_SEED)
    # Frozen descending Fisher-Yates shuffle.
    for index in range(len(orders) - 1, 0, -1):
        swap = rng.next() % (index + 1)
        orders[index], orders[swap] = orders[swap], orders[index]
    validate_schedule(orders, repetitions)
    return orders


def validate_schedule(orders: list[str], repetitions: int) -> None:
    expected_blocks = 6 * repetitions
    if len(orders) != expected_blocks:
        raise AssertionError("incorrect schedule length")
    for stratum in ORDER_STRATA:
        if orders.count(stratum) != repetitions:
            raise AssertionError(f"incorrect count for order stratum {stratum}")
    for variant in VARIANTS:
        for ordinal in range(3):
            count = sum(order.split(",")[ordinal] == variant for order in orders)
            if count != 2 * repetitions:
                raise AssertionError(
                    f"{variant} occurs {count} times at ordinal {ordinal}"
                )
    for position in (0, 1):
        for left in VARIANTS:
            for right in VARIANTS:
                if left == right:
                    continue
                count = sum(
                    order.split(",")[position : position + 2] == [left, right]
                    for order in orders
                )
                if count != repetitions:
                    raise AssertionError(
                        f"ordered pair {left}/{right} occurs {count} times "
                        f"at adjacency position {position}"
                    )


def command_observation(argv: list[str]) -> dict[str, Any]:
    try:
        completed = run_capture(argv, timeout=30)
        return {
            "argv": argv,
            "returncode": completed.returncode,
            "stdout": completed.stdout.strip(),
            "stderr": completed.stderr.strip(),
        }
    except (OSError, subprocess.SubprocessError) as error:
        return {"argv": argv, "unavailable": True, "error": str(error)}


def power_snapshot() -> dict[str, Any]:
    battery = command_observation([str(PMSET), "-g", "batt"])
    first_line = battery.get("stdout", "").splitlines()[:1]
    match = re.search(r"Now drawing from '([^']+)'", first_line[0] if first_line else "")
    power_source = match.group(1) if match else None

    thermal = command_observation([str(PMSET), "-g", "therm"])
    thermal_output = thermal.get("stdout", "")
    thermal_available = (
        thermal.get("returncode") == 0
        and bool(thermal_output.strip())
        and "error:" not in thermal_output.lower()
    )
    if thermal_available:
        thermal_state: dict[str, Any] = {
            "available": True,
            "stdout": thermal_output,
        }
    else:
        thermal_state = {
            "available": False,
            "returncode": thermal.get("returncode"),
            "stdout": thermal.get("stdout", ""),
            "stderr": thermal.get("stderr", ""),
        }
    return {
        "captured_at": utc_now(),
        "power_source": power_source,
        "power_source_available": power_source is not None,
        "power_source_probe": {
            "argv": battery.get("argv"),
            "returncode": battery.get("returncode"),
            "available": power_source is not None,
            "stderr": battery.get("stderr", ""),
        },
        "thermal": thermal_state,
    }


def power_transition(before: dict[str, Any], after: dict[str, Any]) -> str | None:
    if before["power_source_available"] != after["power_source_available"]:
        return "power-source availability changed"
    if before["power_source_available"] and before["power_source"] != after["power_source"]:
        return f"power source changed from {before['power_source']} to {after['power_source']}"
    before_thermal = before["thermal"]
    after_thermal = after["thermal"]
    if before_thermal["available"] != after_thermal["available"]:
        return "thermal-state availability changed"
    if before_thermal["available"] and before_thermal["stdout"] != after_thermal["stdout"]:
        return "pmset thermal state changed"
    return None


def version(argv: list[str]) -> dict[str, Any]:
    return command_observation(argv)


def audit_clang_default_configs() -> dict[str, Any]:
    search_directories = (
        CLANG.parent,
        CLANG.parent.parent / "etc" / "clang",
        Path("/etc/clang"),
        Path("/Users/bytedance/.config/clang"),
        Path("/Users/bytedance/Library/Preferences/clang"),
    )
    observed: dict[str, str] = {}
    observed_symlinks: dict[str, str] = {}
    directory_status: dict[str, str] = {}
    for directory in search_directories:
        if not os.path.lexists(directory):
            directory_status[str(directory)] = "absent"
            continue
        if directory.is_symlink() or not directory.is_dir():
            raise RuntimeError(f"Clang config search directory is not a real directory: {directory}")
        directory_status[str(directory)] = "present"
        for config in sorted(directory.glob("*.cfg")):
            if config.is_symlink():
                observed_symlinks[str(config)] = os.readlink(config)
            elif not config.is_file():
                raise RuntimeError(f"Clang config candidate is not a regular file: {config}")
            observed[str(config)] = sha256_file(config)
    if observed != EXPECTED_IRRELEVANT_CLANG_CONFIG_SHA256:
        raise RuntimeError(
            "Clang default-config candidates changed; linker invocation is not sealed: "
            f"{observed}"
        )
    if observed_symlinks != EXPECTED_IRRELEVANT_CLANG_CONFIG_SYMLINKS:
        raise RuntimeError(f"Clang irrelevant config symlink set changed: {observed_symlinks}")
    return {
        "schema_version": 1,
        "status": "only-hash-locked-swift-linux-configs-no-macos-match",
        "search_directories": directory_status,
        "cfg_files_sha256": observed,
        "cfg_symlink_targets": observed_symlinks,
        "direct_clang_policy": "--no-default-config",
    }


def locked_build_tool_manifest() -> dict[str, Any]:
    hashes: dict[str, str] = {}
    for raw_path, expected in EXPECTED_BUILD_TOOL_SHA256.items():
        path = Path(raw_path)
        actual = sha256_file(path)
        if actual != expected:
            raise RuntimeError(f"locked build tool hash mismatch for {path}: {actual}")
        hashes[raw_path] = actual
    versions = {
        "cargo": checked_capture([str(CARGO), "--version", "--verbose"]),
        "rustc": checked_capture([str(RUSTC), "--version", "--verbose"]),
        "clang": checked_capture([str(CLANG), "--version"]),
        "python": checked_capture([str(PYTHON), "--version"]),
    }
    if not versions["cargo"].startswith("cargo 1.91.1 (ea2d97820 2025-10-10)\n"):
        raise RuntimeError("locked Cargo version changed")
    if not versions["rustc"].startswith("rustc 1.91.1 (ed61e7d7e 2025-11-07)\n"):
        raise RuntimeError("locked rustc version changed")
    if not versions["clang"].startswith("Apple clang version 21.0.0 (clang-2100.1.1.101)\n"):
        raise RuntimeError("locked Clang version changed")
    if versions["python"] != "Python 3.9.6":
        raise RuntimeError("locked Python version changed")
    sdk_settings = read_json_object(MACOS_SDK_SETTINGS, "macOS SDK settings")
    sdk_identity = {
        "canonical_name": sdk_settings.get("CanonicalName"),
        "display_name": sdk_settings.get("DisplayName"),
        "version": sdk_settings.get("Version"),
    }
    if sdk_identity != {
        "canonical_name": "macosx26.5",
        "display_name": "macOS 26.5",
        "version": "26.5",
    }:
        raise RuntimeError("locked macOS SDK identity changed")
    return {
        "paths": {
            "cargo": str(CARGO),
            "rustc": str(RUSTC),
            "clang": str(CLANG),
            "python": str(PYTHON),
            "macos_sdk": str(MACOS_SDK),
            "macos_sdk_settings": str(MACOS_SDK_SETTINGS),
        },
        "sha256": hashes,
        "versions": versions,
        "macos_sdk": sdk_identity,
        "clang_default_configs": audit_clang_default_configs(),
    }


def canonical_generation_tool_manifest() -> dict[str, Any]:
    build = locked_build_tool_manifest()
    hashes: dict[str, str] = {}
    for raw_path, expected in EXPECTED_GENERATION_TOOL_SHA256.items():
        actual = sha256_file(Path(raw_path))
        if actual != expected:
            raise RuntimeError(f"locked generation tool hash mismatch for {raw_path}: {actual}")
        hashes[raw_path] = actual
    codex_version = checked_capture([str(CODEX), "--version"])
    codex_native_version = checked_capture([str(CODEX_NATIVE), "--version"])
    if codex_version != "codex-cli 0.144.0" or codex_native_version != codex_version:
        raise RuntimeError("locked Codex CLI version changed")
    return {
        "paths": {
            "codex_launcher": str(CODEX),
            "codex_javascript": str(CODEX_JAVASCRIPT),
            "codex_native": str(CODEX_NATIVE),
            **build["paths"],
        },
        "sha256": hashes,
        "versions": {
            "codex": codex_version,
            "codex_native": codex_native_version,
            **build["versions"],
        },
        "macos_sdk": build["macos_sdk"],
        "clang_default_configs": build["clang_default_configs"],
    }


def verify_utf8parse_registry() -> dict[str, Any]:
    expected = "06abde3611657adf66d383f00b093d7faecc7fa57071cce2578660c9f1010821"
    crates = sorted(CARGO_HOME.glob("registry/cache/*/utf8parse-0.2.2.crate"))
    if not crates:
        raise RuntimeError("no checksummed utf8parse-0.2.2 .crate is cached")
    for path in crates:
        if path.is_symlink() or path.parent.is_symlink() or not path.is_file():
            raise RuntimeError(f"utf8parse cache entry is not a regular file: {path}")
    bad_crates = [path for path in crates if sha256_file(path) != expected]
    if bad_crates:
        raise RuntimeError(f"utf8parse .crate hash mismatch: {bad_crates}")
    matching_crates = crates
    source_trees = sorted(CARGO_HOME.glob("registry/src/*/utf8parse-0.2.2"))
    verified_trees: list[dict[str, Any]] = []
    with tarfile.open(matching_crates[0], mode="r:*") as archive:
        all_members = archive.getmembers()
        if any(not (member.isfile() or member.isdir()) for member in all_members):
            raise RuntimeError("utf8parse .crate contains a non-file member")
        archive_files: dict[str, bytes] = {}
        archive_directories: set[str] = set()
        seen_members: set[str] = set()
        for member in all_members:
            raw_name = member.name.rstrip("/") if member.isdir() else member.name
            pure = PurePosixPath(raw_name)
            parts = pure.parts
            if (
                pure.is_absolute()
                or not parts
                or parts[0] != "utf8parse-0.2.2"
                or any(part in ("", ".", "..") for part in parts)
                or "/".join(parts) != raw_name
            ):
                raise RuntimeError(f"unexpected path in utf8parse crate: {member.name}")
            relative = "/".join(parts[1:])
            member_key = relative or "."
            if member_key in seen_members:
                raise RuntimeError(f"duplicate path in utf8parse crate: {member.name}")
            seen_members.add(member_key)
            if member.isdir():
                if relative:
                    archive_directories.add(relative)
                continue
            if not relative:
                raise RuntimeError("utf8parse .crate contains a root file")
            extracted = archive.extractfile(member)
            if extracted is None:
                raise RuntimeError(f"could not read crate member {member.name}")
            archive_files[relative] = extracted.read()
        implicit_directories: set[str] = set()
        for relative in archive_files:
            for parent in PurePosixPath(relative).parents:
                if str(parent) != ".":
                    implicit_directories.add(str(parent))
        extra_archive_directories = archive_directories - implicit_directories
        if extra_archive_directories:
            raise RuntimeError(
                "utf8parse .crate contains empty/extra directories: "
                f"{sorted(extra_archive_directories)}"
            )
    for tree in source_trees:
        if tree.is_symlink() or tree.parent.is_symlink() or not tree.is_dir():
            raise RuntimeError(f"registry source candidate is not a real directory: {tree}")
        tree_manifest = strict_regular_tree_manifest(tree)
        expected_tree_files = set(archive_files) | {".cargo-ok"}
        if set(tree_manifest["files_sha256"]) != expected_tree_files:
            raise RuntimeError(f"registry source tree file set differs from .crate: {tree}")
        if set(tree_manifest["directories"]) != implicit_directories:
            raise RuntimeError(
                f"registry source tree has missing or extra/empty directories: {tree}"
            )
        if any((tree / name).read_bytes() != data for name, data in archive_files.items()):
            raise RuntimeError(f"registry source tree bytes differ from .crate: {tree}")
        manifest = {
            name: hashlib.sha256(data).hexdigest()
            for name, data in sorted(archive_files.items())
        }
        cargo_ok = tree / ".cargo-ok"
        if cargo_ok.is_symlink() or not cargo_ok.is_file():
            raise RuntimeError(f"registry source tree lacks regular root .cargo-ok: {tree}")
        verified_trees.append(
            {
                "path": str(tree.resolve()),
                "tree_files_sha256": manifest,
                "tree_directories": sorted(implicit_directories),
                "cargo_ok_sha256": sha256_file(cargo_ok),
            }
        )
    if not source_trees or not verified_trees:
        raise RuntimeError("no extracted utf8parse source tree matches the .crate")
    return {
        "package": "utf8parse",
        "version": "0.2.2",
        "expected_crates_io_sha256": expected,
        "crate_files": [
            {"path": str(path.resolve()), "sha256": sha256_file(path)}
            for path in matching_crates
        ],
        "verified_source_trees": verified_trees,
    }


def hardware_snapshot() -> dict[str, Any]:
    observation = command_observation(
        [str(SYSTEM_PROFILER), "SPHardwareDataType", "-json"]
    )
    if observation.get("returncode") != 0:
        return {
            "available": False,
            "returncode": observation.get("returncode"),
            "stderr": observation.get("stderr", ""),
        }
    try:
        payload = json.loads(observation.get("stdout", ""))
        item = payload["SPHardwareDataType"][0]
        # Deliberately whitelist non-identifying fields; system_profiler also
        # returns serial numbers and UUIDs that do not belong in run artifacts.
        return {
            "available": True,
            "chip_type": item.get("chip_type"),
            "machine_model": item.get("machine_model"),
            "machine_name": item.get("machine_name"),
            "number_processors": item.get("number_processors"),
            "physical_memory": item.get("physical_memory"),
        }
    except (KeyError, IndexError, TypeError, json.JSONDecodeError) as error:
        return {"available": False, "parse_error": str(error)}


def hash_manifest(paths: Iterable[Path]) -> dict[str, str]:
    manifest: dict[str, str] = {}
    for path in sorted(set(paths)):
        if path.is_file():
            try:
                label = str(path.relative_to(ROOT))
            except ValueError:
                label = str(path.resolve())
            manifest[label] = sha256_file(path)
    return manifest


def tool_source_paths() -> list[Path]:
    paths = [
        HERE / "PROTOCOL.md",
        HERE / "task.md",
        HERE / "teaching-pack.md",
        HERE / "benchmark.py",
        HERE / "analyze.py",
        HARNESS / "Cargo.toml",
        HARNESS / "Cargo.lock",
        HARNESS / "src/bin/bench.rs",
        HERE / "rust-baseline/Cargo.toml",
        HERE / "rust-baseline/Cargo.lock",
        HERE / "rust-baseline/src/lib.rs",
    ]
    for directory in (ROOT / "prototype/democ", ROOT / "prototype/checker"):
        paths.extend(
            path
            for path in directory.rglob("*")
            if path.is_file() and "__pycache__" not in path.parts
        )
    return paths


def git_metadata() -> dict[str, Any]:
    return {
        "revision": checked_capture(["git", "rev-parse", "HEAD"]),
        "status_porcelain_v1": checked_capture(
            ["git", "status", "--porcelain=v1", "--untracked-files=all"]
        ).splitlines(),
    }


def rerun_metadata(
    args: argparse.Namespace,
    mode: str,
    freeze_binding: dict[str, Any] | None,
    source_sha: str | None,
) -> dict[str, Any] | None:
    prior = getattr(args, "rerun_of_invalid_campaign", None)
    reason = getattr(args, "rerun_reason", None)
    if (prior is None) != (reason is None):
        raise RuntimeError(
            "--rerun-of-invalid-campaign and --rerun-reason must be supplied together"
        )
    if prior is None:
        return None
    if not reason.strip():
        raise RuntimeError("--rerun-reason must be nonempty")
    prior_metadata_path = prior.resolve() / "metadata.json"
    prior_metadata = read_json_object(prior_metadata_path, "prior invalid campaign")
    if (
        mode != "score"
        or freeze_binding is None
        or source_sha is None
        or prior_metadata.get("status") != "invalid"
        or prior_metadata.get("mode") != "score"
        or prior_metadata.get("not_a_score") is not False
    ):
        raise RuntimeError("a scoring rerun may reference only an invalid score campaign")
    current_protocol_sha = sha256_file(HERE / "PROTOCOL.md")
    if prior_metadata.get("protocol_sha256") != current_protocol_sha:
        raise RuntimeError("prior invalid campaign used a different protocol")
    if prior_metadata.get("generation_freeze") != freeze_binding:
        raise RuntimeError("prior invalid campaign used a different generation freeze")
    prior_source = prior_metadata.get("frozen_source")
    if (
        not isinstance(prior_source, dict)
        or prior_source.get("sha256") != source_sha.lower()
        or prior_source.get("path") != freeze_binding["source"]["path"]
    ):
        raise RuntimeError("prior invalid campaign used a different frozen source")
    return {
        "prior_campaign": str(prior.resolve()),
        "prior_metadata_sha256": sha256_file(prior_metadata_path),
        "protocol_sha256": current_protocol_sha,
        "generation_manifest_sha256": freeze_binding["manifest"]["sha256"],
        "frozen_source_sha256": source_sha.lower(),
        "reason": reason,
    }


def sanitized_build_environment() -> tuple[dict[str, str], list[str]]:
    environment = dict(os.environ)
    removed: list[str] = []
    exact = {
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTC_BOOTSTRAP",
        "RUSTC",
        "RUSTDOC",
        "CARGO_HOME",
        "CARGO_INCREMENTAL",
        "CARGO_TARGET_DIR",
        "CC",
        "CFLAGS",
        "CPPFLAGS",
        "LDFLAGS",
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
        "OBJC_INCLUDE_PATH",
        "LIBRARY_PATH",
        "CCC_OVERRIDE_OPTIONS",
        "RC_DEBUG_OPTIONS",
        "SDKROOT",
        "RUSTUP_TOOLCHAIN",
        "MACOSX_DEPLOYMENT_TARGET",
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
    }
    for key in list(environment):
        upper = key.upper()
        if (
            upper in exact
            or upper.startswith("CARGO_BUILD_")
            or upper.startswith("CARGO_PROFILE_")
            or upper.startswith("CARGO_TARGET_")
            or upper.startswith("CARGO_ALIAS_")
            or upper.startswith("CFLAGS_")
            or upper.startswith("CPPFLAGS_")
            or upper.startswith("LDFLAGS_")
            or upper.startswith("DYLD_")
            or upper.startswith("CLANG_CONFIG_FILE_")
        ):
            removed.append(key)
            del environment[key]
    environment["RUSTC"] = str(RUSTC)
    environment["CARGO_HOME"] = str(CARGO_HOME)
    environment["CC"] = str(CLANG)
    environment["CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER"] = str(CLANG)
    environment["CARGO_INCREMENTAL"] = "0"
    environment["SDKROOT"] = str(MACOS_SDK)
    return environment, sorted(removed)


LLVM_FUNCTION = re.compile(
    r"^define\b[^\n@]*@([A-Za-z_.$][A-Za-z0-9_.$-]*)\(", re.MULTILINE
)


def namespace_module(ir: str, namespace: str, entry: str) -> str:
    definitions = set(LLVM_FUNCTION.findall(ir))
    if "parse" not in definitions:
        raise RuntimeError("compiled module does not define parse")
    replacements = {
        name: entry if name == "parse" else f"xlang_{namespace}_{name}"
        for name in definitions
    }
    names = "|".join(
        re.escape(name) for name in sorted(definitions, key=len, reverse=True)
    )
    return re.sub(
        rf"@({names})(?=\()",
        lambda match: f"@{replacements[match.group(1)]}",
        ir,
    )


def proof_report_summary(path: Path, sites: list[dict[str, Any]]) -> dict[str, Any]:
    statuses: dict[str, int] = {}
    proofs: dict[str, int] = {}
    for site in sites:
        status = str(site.get("status", "<missing>"))
        proof = str(site.get("proof", "<missing>"))
        statuses[status] = statuses.get(status, 0) + 1
        proofs[proof] = proofs.get(proof, 0) + 1
    return {
        "path": str(path),
        "sha256": sha256_file(path),
        "site_count": len(sites),
        "status_counts": dict(sorted(statuses.items())),
        "proof_counts": dict(sorted(proofs.items())),
        "ir_byte_identical_with_and_without_report": True,
    }


def compile_xlang(
    candidate: Path, build: Path
) -> tuple[Path, Path, str, dict[str, Any]]:
    expected_source = candidate.read_bytes()
    source = expected_source.decode("utf-8")
    sys.path.insert(0, str(ROOT / "prototype/democ"))
    import democ  # type: ignore  # noqa: PLC0415

    diagnostics = io.StringIO()
    facts_report: list[dict[str, Any]] = []
    nofacts_report: list[dict[str, Any]] = []
    try:
        with contextlib.redirect_stdout(diagnostics), contextlib.redirect_stderr(diagnostics):
            facts_ir = democ.compile_program(source, alias=True)
            facts_report_ir = democ.compile_program(
                source, alias=True, proof_report=facts_report
            )
            nofacts_ir = democ.compile_program(source, alias=False)
            nofacts_report_ir = democ.compile_program(
                source, alias=False, proof_report=nofacts_report
            )
    except BaseException as error:
        raise RuntimeError(
            "frozen xlang source no longer compiles: "
            + (diagnostics.getvalue() or str(error) or type(error).__name__)
        ) from error
    if candidate.read_bytes() != expected_source:
        raise RuntimeError("frozen source changed while compiling")
    if facts_report_ir != facts_ir or nofacts_report_ir != nofacts_ir:
        raise RuntimeError("requesting a proof report changed generated LLVM IR")

    facts_ll = build / "facts.ll"
    nofacts_ll = build / "nofacts.ll"
    facts_report_path = build / "facts-proof-report.json"
    nofacts_report_path = build / "nofacts-proof-report.json"
    facts_ll.write_text(
        namespace_module(facts_ir, "facts", "xlang_parse_facts"), encoding="utf-8"
    )
    nofacts_ll.write_text(
        namespace_module(nofacts_ir, "nofacts", "xlang_parse_nofacts"),
        encoding="utf-8",
    )
    facts_report_path.write_text(
        json.dumps(facts_report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    nofacts_report_path.write_text(
        json.dumps(nofacts_report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    proof_reports = {
        "facts-on": proof_report_summary(facts_report_path, facts_report),
        "facts-off": proof_report_summary(nofacts_report_path, nofacts_report),
    }
    return facts_ll, nofacts_ll, diagnostics.getvalue(), proof_reports


def build_benchmark(
    mode: str,
    output: Path,
    candidate: Path | None,
    expected_source_sha: str | None,
    metadata: dict[str, Any],
) -> Path:
    build = output / "build"
    build.mkdir()
    cargo_target = build / "cargo-target"
    environment, removed = sanitized_build_environment()
    cargo_config_absence = audit_cargo_config_absence(ROOT)
    metadata["build_environment"] = {
        "removed_override_variable_names": removed,
        "enforced": {
            "RUSTC": str(RUSTC),
            "CARGO_HOME": str(CARGO_HOME),
            "CC": str(CLANG),
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": str(CLANG),
            "CARGO_INCREMENTAL": "0",
            "SDKROOT": str(MACOS_SDK),
        },
        "cargo_config_absence": cargo_config_absence,
        "clang_default_configs": audit_clang_default_configs(),
        "cargo_cli_neutralization": cargo_cli_neutralization(),
        "generic_cpu_policy": (
            f"no native CPU flags; explicit generic {NATIVE_TARGET} target"
        ),
    }
    metadata["locked_build_tools"] = locked_build_tool_manifest()
    metadata["utf8parse_registry"] = verify_utf8parse_registry()
    commands: list[list[str]] = []

    if mode == "score":
        if candidate is None or expected_source_sha is None:
            raise RuntimeError("score build requires a frozen source and SHA-256")
        actual_source_sha = sha256_file(candidate)
        if actual_source_sha != expected_source_sha.lower():
            raise RuntimeError(
                f"frozen source SHA-256 mismatch: expected {expected_source_sha}, "
                f"got {actual_source_sha}"
            )
        facts_ll, nofacts_ll, compiler_output, proof_reports = compile_xlang(
            candidate, build
        )
        metadata["proof_reports"] = proof_reports
        (build / "democ-output.txt").write_text(compiler_output, encoding="utf-8")
        facts_obj = build / "facts.o"
        nofacts_obj = build / "nofacts.o"
        for source, obj, label in (
            (facts_ll, facts_obj, "facts"),
            (nofacts_ll, nofacts_obj, "nofacts"),
        ):
            command = [
                str(CLANG), "--no-default-config", "-isysroot", str(MACOS_SDK), "-O3", "-c",
                str(source), "-o", str(obj),
            ]
            commands.append(command)
            run_logged(command, build / f"clang-{label}.log", env=environment)
        cargo_command = [
            str(CARGO),
            *cargo_cli_neutralization(),
            "rustc",
            "--manifest-path",
            str(HARNESS / "Cargo.toml"),
            "--target-dir",
            str(cargo_target),
            "--bin",
            "bench",
            "--release",
            "--locked",
            "--offline",
            "--target",
            NATIVE_TARGET,
            "-vv",
            "--",
            "-C",
            f"link-arg={facts_obj}",
            "-C",
            f"link-arg={nofacts_obj}",
        ]
    else:
        cargo_command = [
            str(CARGO),
            *cargo_cli_neutralization(),
            "build",
            "--manifest-path",
            str(HARNESS / "Cargo.toml"),
            "--target-dir",
            str(cargo_target),
            "--bin",
            "bench",
            "--release",
            "--locked",
            "--offline",
            "--target",
            NATIVE_TARGET,
            "--features",
            "smoke-shim",
            "-vv",
        ]
    commands.append(cargo_command)
    cargo_result = run_logged(
        cargo_command, build / "cargo-build.log", env=environment, timeout=900
    )
    if audit_cargo_config_absence(ROOT) != cargo_config_absence:
        raise RuntimeError("Cargo config search-chain state changed during native build")
    if audit_clang_default_configs() != metadata["build_environment"]["clang_default_configs"]:
        raise RuntimeError("Clang default-config state changed during native build")
    verbose_build = (cargo_result.stdout + "\n" + cargo_result.stderr).lower()
    found = [flag for flag in FORBIDDEN_BUILD_FLAGS if flag in verbose_build]
    if found:
        raise RuntimeError(f"build log contains forbidden build flag(s): {found}")
    rust_opt_levels = re.findall(r"-c\s+opt-level=([^\s'\"`]+)", verbose_build)
    if not rust_opt_levels or any(level != "3" for level in rust_opt_levels):
        raise RuntimeError(
            "verbose Cargo build did not demonstrate the ordinary release "
            f"opt-level=3 profile: {rust_opt_levels}"
        )
    forbidden_profile_settings = re.findall(
        r"-c\s+(?:codegen-units|debug-assertions|overflow-checks|incremental|panic|debuginfo)"
        r"(?:=[^\s'\"`]+)?",
        verbose_build,
    )
    if forbidden_profile_settings:
        raise RuntimeError(
            "verbose Cargo build contains a non-default release-profile setting: "
            f"{forbidden_profile_settings}"
        )
    strip_settings = re.findall(r"-c\s+strip=([^\s'\"`]+)", verbose_build)
    if any(setting != "debuginfo" for setting in strip_settings):
        raise RuntimeError(
            f"verbose Cargo build contains a release strip override: {strip_settings}"
        )
    expected_link_args = 2 if mode == "score" else 0
    link_arg_count = verbose_build.count("-c link-arg=")
    if link_arg_count != expected_link_args:
        raise RuntimeError(
            f"verbose Cargo build has {link_arg_count} link args; expected "
            f"{expected_link_args}"
        )
    verified_registry_paths = [
        tree["path"]
        for tree in metadata["utf8parse_registry"]["verified_source_trees"]
    ]
    referenced_registry_paths = [
        path for path in verified_registry_paths if path.lower() in verbose_build
    ]
    if len(referenced_registry_paths) != 1:
        raise RuntimeError(
            "verbose Cargo build did not reference exactly one verified "
            f"utf8parse source tree: {referenced_registry_paths}"
        )
    metadata["utf8parse_registry"]["cargo_referenced_source_tree"] = (
        referenced_registry_paths[0]
    )
    executable = cargo_target / NATIVE_TARGET / "release" / "bench"
    if not executable.is_file():
        raise RuntimeError(f"benchmark executable missing at {executable}")
    if mode == "score" and sha256_file(candidate) != expected_source_sha.lower():
        raise RuntimeError("frozen source changed during native build")
    metadata["build_commands"] = commands
    metadata["build_artifact_sha256"] = hash_manifest(
        path
        for path in (
            build / "facts.ll",
            build / "nofacts.ll",
            build / "facts.o",
            build / "nofacts.o",
            build / "facts-proof-report.json",
            build / "nofacts-proof-report.json",
            executable,
        )
        if path.exists()
    )
    return executable


def parse_single_json(stdout: str, label: str) -> dict[str, Any]:
    lines = [line for line in stdout.splitlines() if line.strip()]
    if len(lines) != 1:
        raise RuntimeError(f"{label} emitted {len(lines)} nonempty stdout lines")
    try:
        value = json.loads(lines[0])
    except json.JSONDecodeError as error:
        raise RuntimeError(f"{label} emitted invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{label} JSON is not an object")
    return value


def prepare_corpus(
    executable: Path, mode: str, corpus_bytes: int, output: Path
) -> dict[str, Any]:
    corpus = output / "corpus.bin"
    command = [
        str(executable),
        "prepare-corpus",
        "--mode",
        mode,
        "--bytes",
        str(corpus_bytes),
        "--output",
        str(corpus),
    ]
    completed = run_logged(command, output / "corpus-generation.log", timeout=600)
    record = parse_single_json(completed.stdout, "corpus generator")
    actual_sha = sha256_file(corpus)
    if (
        record.get("kind") != "corpus"
        or record.get("mode") != mode
        or record.get("not_a_score") is not (mode == "smoke")
        or record.get("bytes") != corpus_bytes
        or record.get("sha256") != actual_sha
    ):
        raise RuntimeError("corpus generator record does not match completed corpus")
    return {"path": str(corpus), "sha256": actual_sha, "bytes": corpus_bytes, "command": command}


def validate_block_record(
    record: dict[str, Any], mode: str, block_index: int, order: str, corpus: dict[str, Any]
) -> None:
    if (
        record.get("kind") != "benchmark-block"
        or record.get("mode") != mode
        or record.get("not_a_score") is not (mode == "smoke")
    ):
        raise RuntimeError("benchmark child returned the wrong record kind or mode")
    if record.get("block_index") != block_index or record.get("order") != order:
        raise RuntimeError("benchmark child returned the wrong block identity or order")
    if (
        record.get("corpus_bytes") != corpus["bytes"]
        or record.get("corpus_sha256") != corpus["sha256"]
    ):
        raise RuntimeError("benchmark child returned the wrong corpus identity")
    samples = record.get("samples")
    if not isinstance(samples, list) or len(samples) != 3:
        raise RuntimeError("benchmark child did not return exactly three samples")
    expected_order = order.split(",")
    for ordinal, sample in enumerate(samples):
        if (
            not isinstance(sample, dict)
            or sample.get("variant") != expected_order[ordinal]
            or sample.get("ordinal") != ordinal
            or sample.get("input_bytes") != corpus["bytes"]
            or type(sample.get("elapsed_ns")) is not int
            or sample["elapsed_ns"] <= 0
            or type(sample.get("output_events")) is not int
            or not 0 <= sample["output_events"] <= corpus["bytes"]
            or not isinstance(sample.get("output_sha256"), str)
            or re.fullmatch(r"[0-9a-f]{64}", sample["output_sha256"]) is None
        ):
            raise RuntimeError("benchmark child returned a malformed sample row")
    lengths = {sample.get("output_events") for sample in samples}
    digests = {sample.get("output_sha256") for sample in samples}
    if len(lengths) != 1 or len(digests) != 1:
        raise RuntimeError("benchmark child output verification disagreed")


INTERRUPTED = False


def note_signal(signum: int, _frame: Any) -> None:
    global INTERRUPTED
    INTERRUPTED = True
    raise KeyboardInterrupt(f"received signal {signum}")


def base_metadata(
    mode: str,
    source: Path | None,
    source_sha: str | None,
    freeze_binding: dict[str, Any] | None,
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "mode": mode,
        "status": "preparing",
        "not_a_score": mode == "smoke",
        "created_at": utc_now(),
        "protocol_sha256": sha256_file(HERE / "PROTOCOL.md"),
        "frozen_source": (
            {"path": str(source.resolve()), "sha256": source_sha.lower()}
            if source is not None and source_sha is not None
            else None
        ),
        "generation_freeze": freeze_binding,
        "repository": git_metadata(),
        "source_tool_sha256": hash_manifest(tool_source_paths()),
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "uname": list(platform.uname()),
            "hardware": hardware_snapshot(),
            "cpu_brand_sysctl": command_observation(
                [str(SYSCTL), "-n", "machdep.cpu.brand_string"]
            ),
        },
        "tool_versions": {
            "python": version([sys.executable, "--version"]),
            "rustc": version([str(RUSTC), "--version", "--verbose"]),
            "cargo": version([str(CARGO), "--version", "--verbose"]),
            "clang": version([str(CLANG), "--version"]),
        },
        "clock": "std::time::Instant for every timed variant",
        "power_before_preparation": power_snapshot(),
    }


def run_campaign(args: argparse.Namespace) -> int:
    mode = args.command
    if mode == "score" and not args.acknowledge_preregistered_score:
        raise RuntimeError(
            "score requires --acknowledge-preregistered-score; use smoke for validation"
        )
    if (
        mode == "score"
        and sha256_file(Path(sys.executable))
        != EXPECTED_BUILD_TOOL_SHA256[str(PYTHON)]
    ):
        raise RuntimeError(
            f"score must run under the locked native Python: {PYTHON}"
        )
    freeze_binding: dict[str, Any] | None = None
    source: Path | None = None
    source_sha: str | None = None
    if mode == "score":
        freeze_binding, source, source_sha = validate_freeze_manifest(
            args.trace_manifest
        )
    rerun = rerun_metadata(args, mode, freeze_binding, source_sha)
    output = args.out_dir.resolve()
    if output.exists():
        raise RuntimeError(f"refusing to overwrite output directory {output}")
    output.mkdir(parents=True)
    corpus_bytes = SCORE_BYTES if mode == "score" else args.smoke_bytes
    repetitions = 5 if mode == "score" else 1
    metadata = base_metadata(mode, source, source_sha, freeze_binding)
    metadata["rerun"] = rerun
    if freeze_binding is not None:
        metadata["generation_archive"] = archive_generation_binding(
            output, freeze_binding
        )
    metadata_path = output / "metadata.json"
    atomic_json(metadata_path, metadata)

    try:
        executable = build_benchmark(mode, output, source, source_sha, metadata)
        measurement_environment, measurement_removed = sanitized_build_environment()
        metadata["measurement_environment"] = {
            "removed_override_variable_names": measurement_removed,
            "dynamic_loader_injection_scrubbed": True,
        }
        executable_sha = sha256_file(executable)
        corpus = prepare_corpus(executable, mode, corpus_bytes, output)
        orders = schedule(repetitions)
        schedule_record = {
            "schema_version": 1,
            "mode": mode,
            "not_a_score": mode == "smoke",
            "identity_order": list(VARIANTS),
            "strata_order": list(ORDER_STRATA),
            "repetitions_per_stratum": repetitions,
            "seed_hex": f"0x{ORDER_SEED:016x}",
            "shuffle": "descending Fisher-Yates; i=n-1..1, j=next()%(i+1)",
            "orders": [
                {"block_index": index, "order": order}
                for index, order in enumerate(orders)
            ],
        }
        atomic_json(output / "schedule.json", schedule_record)
        metadata.update(
            {
                "status": "running",
                "prepared_at": utc_now(),
                "corpus": corpus,
                "schedule_sha256": sha256_file(output / "schedule.json"),
                "power_before_measurement": power_snapshot(),
                "blocks_expected": len(orders),
                "fresh_process_per_block": True,
                "benchmark_executable": {
                    "path": str(executable),
                    "sha256": executable_sha,
                },
            }
        )
        atomic_json(metadata_path, metadata)

        raw_path = output / "raw.jsonl"
        previous_power = metadata["power_before_measurement"]
        with raw_path.open("x", encoding="utf-8") as raw:
            for block_index, order in enumerate(orders):
                if INTERRUPTED:
                    raise KeyboardInterrupt("campaign interrupted")
                if sha256_file(executable) != executable_sha:
                    raise RuntimeError(
                        f"benchmark executable changed before block {block_index}"
                    )
                command = [
                    str(executable),
                    "run-block",
                    "--mode",
                    mode,
                    "--corpus",
                    corpus["path"],
                    "--expected-sha256",
                    corpus["sha256"],
                    "--block-index",
                    str(block_index),
                    "--order",
                    order,
                ]
                before = power_snapshot()
                gap_transition = power_transition(previous_power, before)
                if gap_transition is not None:
                    metadata["transition_observation"] = {
                        "phase": f"before block {block_index}",
                        "reason": gap_transition,
                        "previous": previous_power,
                        "current": before,
                    }
                    raise RuntimeError(
                        f"campaign invalidated before block {block_index} by "
                        f"{gap_transition}"
                    )
                started_at = utc_now()
                completed = run_capture(
                    command, env=measurement_environment, timeout=args.block_timeout
                )
                finished_at = utc_now()
                after = power_snapshot()
                block_log = output / f"block-{block_index:02d}.log"
                block_log.write_text(
                    "argv: " + json.dumps(command) + "\n\nstdout:\n" + completed.stdout
                    + "\n\nstderr:\n" + completed.stderr,
                    encoding="utf-8",
                )
                if completed.returncode != 0:
                    raise RuntimeError(
                        f"block {block_index} process failed ({completed.returncode}); "
                        f"see {block_log}"
                    )
                record = parse_single_json(completed.stdout, f"block {block_index}")
                validate_block_record(record, mode, block_index, order, corpus)
                transition = power_transition(before, after)
                record.update(
                    {
                        "order_stratum": order,
                        "process_command": command,
                        "started_at": started_at,
                        "finished_at": finished_at,
                        "power_before": before,
                        "power_after": after,
                        "power_or_thermal_transition": transition,
                        "child_stderr": completed.stderr,
                    }
                )
                raw.write(json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n")
                raw.flush()
                os.fsync(raw.fileno())
                if transition is not None:
                    metadata["transition_observation"] = {
                        "phase": f"block {block_index}",
                        "reason": transition,
                        "before": before,
                        "after": after,
                    }
                    raise RuntimeError(
                        f"block {block_index} invalidated by {transition}"
                    )
                previous_power = after

        final_power = power_snapshot()
        final_transition = power_transition(previous_power, final_power)
        if final_transition is not None:
            metadata["transition_observation"] = {
                "phase": "after final block",
                "reason": final_transition,
                "previous": previous_power,
                "current": final_power,
            }
            raise RuntimeError(
                f"campaign invalidated after final block by {final_transition}"
            )
        if sha256_file(executable) != executable_sha:
            raise RuntimeError("benchmark executable changed during the campaign")
        if mode == "score":
            if source is None or source_sha is None:
                raise RuntimeError("score campaign lost its frozen-source identity")
            if sha256_file(source) != source_sha.lower():
                raise RuntimeError("frozen source changed during the campaign")
            final_binding, final_source, final_source_sha = validate_freeze_manifest(
                args.trace_manifest
            )
            if (
                final_binding != freeze_binding
                or final_source != source
                or final_source_sha != source_sha
            ):
                raise RuntimeError("generation freeze binding changed during campaign")
            if "generation_archive" not in metadata:
                raise RuntimeError("score campaign lost its generation-freeze archive")
            verify_generation_archive(metadata["generation_archive"])
        if audit_cargo_config_absence(ROOT) != metadata["build_environment"]["cargo_config_absence"]:
            raise RuntimeError("Cargo config search-chain state changed during campaign")
        if audit_clang_default_configs() != metadata["build_environment"]["clang_default_configs"]:
            raise RuntimeError("Clang default-config search state changed during campaign")
        current_tool_manifest = hash_manifest(tool_source_paths())
        if current_tool_manifest != metadata["source_tool_sha256"]:
            raise RuntimeError("protocol, compiler, or harness source changed during campaign")
        if locked_build_tool_manifest() != metadata["locked_build_tools"]:
            raise RuntimeError("locked build tool identity changed during campaign")
        final_registry = verify_utf8parse_registry()
        initial_registry = dict(metadata["utf8parse_registry"])
        initial_registry.pop("cargo_referenced_source_tree", None)
        if final_registry != initial_registry:
            raise RuntimeError("utf8parse registry artifact changed during campaign")
        metadata.update(
            {
                "status": "complete",
                "completed_at": utc_now(),
                "blocks_completed": len(orders),
                "raw_sha256": sha256_file(raw_path),
                "power_after_measurement": final_power,
            }
        )
        atomic_json(metadata_path, metadata)
        if mode == "score":
            analysis_command = [str(PYTHON), str(ANALYZER), str(output)]
            analysis = run_logged(
                analysis_command, output / "analysis.log", timeout=300
            )
            parse_single_json(analysis.stdout, "analysis")
            metadata["analysis"] = {
                "path": str(output / "analysis.json"),
                "sha256": sha256_file(output / "analysis.json"),
                "command": analysis_command,
            }
            atomic_json(metadata_path, metadata)
        else:
            atomic_json(
                output / "SMOKE_ONLY.json",
                {
                    "schema_version": 1,
                    "mode": "smoke",
                    "not_a_score": True,
                    "validation_passed": True,
                    "message": (
                        "Harness wiring validation only. The xlang slots used the "
                        "Rust-backed smoke shim; timings are not benchmark results."
                    ),
                    "blocks_completed": len(orders),
                    "corpus_bytes": corpus_bytes,
                },
            )
        print(
            json.dumps(
                {
                    "status": "complete",
                    "mode": mode,
                    "not_a_score": mode == "smoke",
                    "output": str(output),
                },
                sort_keys=True,
            )
        )
        return 0
    except BaseException as error:
        metadata.update(
            {
                "status": "invalid",
                "invalidated_at": utc_now(),
                "invalidation_reason": str(error) or type(error).__name__,
                "interrupted": isinstance(error, KeyboardInterrupt) or INTERRUPTED,
            }
        )
        atomic_json(metadata_path, metadata)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    score = subparsers.add_parser("score", help="run the preregistered scoring campaign")
    score.add_argument("--trace-manifest", type=Path, required=True)
    score.add_argument("--out-dir", type=Path, required=True)
    score.add_argument("--acknowledge-preregistered-score", action="store_true")
    score.add_argument("--rerun-of-invalid-campaign", type=Path)
    score.add_argument("--rerun-reason")
    score.set_defaults(block_timeout=None)

    smoke = subparsers.add_parser("smoke", help="run non-scoring harness validation")
    smoke.add_argument("--out-dir", type=Path, required=True)
    smoke.add_argument("--smoke-bytes", type=int, default=DEFAULT_SMOKE_BYTES)
    smoke.add_argument("--block-timeout", type=int, default=300)
    return parser.parse_args()


def main() -> int:
    previous = {}
    for signum in (signal.SIGINT, signal.SIGTERM):
        previous[signum] = signal.signal(signum, note_signal)
    try:
        return run_campaign(parse_args())
    except KeyboardInterrupt as error:
        print(f"benchmark campaign interrupted: {error}", file=sys.stderr)
        return 130
    except (OSError, UnicodeError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"benchmark campaign error: {error}", file=sys.stderr)
        return 2
    finally:
        for signum, handler in previous.items():
            signal.signal(signum, handler)


if __name__ == "__main__":
    raise SystemExit(main())
