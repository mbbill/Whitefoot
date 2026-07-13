#!/usr/bin/env python3
"""Build and record the frozen native F-SOA cold-wrapper baseline.

This is experiment infrastructure, not a candidate implementation or comparison
harness.  It builds the existing facts-off frontend and launches one fresh
native process for each sample.
"""

from __future__ import annotations

import argparse
import copy
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import sys
import tempfile
from typing import Any, Iterable


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
COMPILER = ROOT / "compiler"
SOURCES_FILE = COMPILER / "sources.txt"
DRIVER = HERE / "native" / "fsoa_sample.c"
CLANG = Path("/usr/bin/clang")

FROZEN_SOURCE_BYTES = 1_029_044
FROZEN_SOURCE_SHA256 = "17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65"
FROZEN_IR_BYTES = 1_860_733
FROZEN_IR_SHA256 = "23baa6cce795a7c8c21b66af2c2c01dbbeade8e40b5fe7dda64966db9f8e615a"
MAIN_STACK_BYTES = 64 * 1024 * 1024
MAIN_STACK_LINKER_FLAG = "-Wl,-stack_size,0x4000000"
STACK_PROTOCOL_AMENDMENT = "PROTOCOL_AMENDMENTS.md#A1"
FROZEN_REPORT = {
    "stage": 0,
    "status": 0,
    "error_start": 0,
    "error_end": 0,
    "node": 18_446_744_073_709_551_615,
    "related": 18_446_744_073_709_551_615,
    "token_count": 211_374,
    "node_count": 105_550,
    "type_count": 11_429,
    "symbol_count": 3_629,
    "function_count": 477,
}
REPORT_FIELDS = tuple(FROZEN_REPORT)
CORRECTNESS_PREFIX = b"xlang-fsoa-frontend-report-v1\0"

VARIANT = "F-SOA"
PHASE = "cold-wrapper"
SMOKE_SAMPLES = 2
SCORE_SAMPLES = 30
FORBIDDEN_FACT_MARKERS = (
    " noalias",
    " readonly",
    " dereferenceable(",
    " willreturn",
    "!alias.scope",
    "!noalias",
    " memory(",
)
ENVIRONMENT_KEYS = (
    "CFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
    "LANG",
    "LC_ALL",
    "MACOSX_DEPLOYMENT_TARGET",
    "MallocNanoZone",
    "PYTHONHASHSEED",
    "SDKROOT",
    "TZ",
)


class HarnessError(RuntimeError):
    pass


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def fsync_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())


def write_json(path: Path, value: Any) -> None:
    temporary = path.with_name(path.name + ".tmp")
    fsync_write(temporary, canonical_json_bytes(value) + b"\n")
    os.replace(temporary, path)


def append_jsonl(path: Path, value: Any) -> None:
    with path.open("ab") as output:
        output.write(canonical_json_bytes(value) + b"\n")
        output.flush()
        os.fsync(output.fileno())


def run_checked(
    argv: list[str], *, cwd: Path = ROOT
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        raise HarnessError(
            f"command failed ({result.returncode}): {argv!r}\n{result.stderr[:4000]}"
        )
    return result


def git_text(*arguments: str) -> str:
    return run_checked(["git", *arguments]).stdout.strip()


def repository_state() -> dict[str, Any]:
    status = run_checked(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"]
    ).stdout.splitlines()
    return {
        "head": git_text("rev-parse", "HEAD"),
        "dirty": bool(status),
        "dirty_entries": status,
    }


def require_clean_repository(repository: dict[str, Any], context: str) -> None:
    if repository.get("dirty"):
        entries = repository.get("dirty_entries")
        detail = "; ".join(entries[:20]) if isinstance(entries, list) else "<unknown>"
        raise HarnessError(f"{context} rejects a dirty repository: {detail}")


def source_entries() -> tuple[Path, ...]:
    entries: list[Path] = []
    for raw in SOURCES_FILE.read_text(encoding="utf-8").splitlines():
        spelling = raw.strip()
        if not spelling:
            continue
        path = (COMPILER / spelling).resolve()
        if not path.is_file() or COMPILER.resolve() not in path.parents:
            raise HarnessError(f"invalid compiler source entry: {spelling!r}")
        entries.append(path)
    if not entries:
        raise HarnessError("compiler/sources.txt is empty")
    return tuple(entries)


def canonical_compiler_source() -> tuple[str, bytes, tuple[Path, ...]]:
    entries = source_entries()
    pieces = [path.read_text(encoding="utf-8").rstrip("\n") for path in entries]
    text = "\n\n".join(pieces) + "\n"
    try:
        encoded = text.encode("ascii")
    except UnicodeEncodeError as error:
        raise HarnessError("canonical compiler source is no longer ASCII") from error
    observed = sha256_bytes(encoded)
    if len(encoded) != FROZEN_SOURCE_BYTES or observed != FROZEN_SOURCE_SHA256:
        raise HarnessError(
            "canonical compiler source drifted from the frozen F-SOA baseline: "
            f"expected {FROZEN_SOURCE_BYTES}/{FROZEN_SOURCE_SHA256}, "
            f"observed {len(encoded)}/{observed}"
        )
    return text, encoded, entries


def relative_record(path: Path) -> dict[str, Any]:
    resolved = path.resolve()
    try:
        spelling = str(resolved.relative_to(ROOT.resolve()))
    except ValueError:
        spelling = str(resolved)
    return {
        "path": spelling,
        "bytes": resolved.stat().st_size,
        "sha256": sha256_file(resolved),
    }


def harness_input_paths() -> tuple[Path, ...]:
    selected = []
    for path in HERE.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(HERE)
        if any(part in ("runs", "__pycache__") for part in relative.parts):
            continue
        selected.append(path)
    return tuple(sorted(selected))


def tool_record(path: Path, version_argv: list[str]) -> dict[str, Any]:
    resolved = path.resolve()
    if not resolved.is_file():
        raise HarnessError(f"required tool is absent: {resolved}")
    result = run_checked(version_argv)
    version = (result.stdout + result.stderr).strip()
    return {
        "path": str(resolved),
        "sha256": sha256_file(resolved),
        "version": version,
    }


def native_stack_record() -> dict[str, Any]:
    if platform.system() != "Darwin":
        raise HarnessError(
            "the v1 native baseline implements A1 only for Darwin; a Linux "
            "runner must first implement and bind an equal process stack limit"
        )
    return {
        "bytes": MAIN_STACK_BYTES,
        "linker_flag": MAIN_STACK_LINKER_FLAG,
        "protocol_amendment": STACK_PROTOCOL_AMENDMENT,
        "equal_for_every_arm": True,
    }


def build_binding() -> dict[str, Any]:
    _, source, entries = canonical_compiler_source()
    compiler_inputs = (SOURCES_FILE, *entries)
    stage0_inputs = (
        ROOT / "prototype" / "democ" / "democ.py",
        ROOT / "prototype" / "checker" / "checker.py",
    )
    python_path = Path(sys.executable)
    binding = {
        "schema_version": 1,
        "repository_head": git_text("rev-parse", "HEAD"),
        "canonical_source": {
            "bytes": len(source),
            "sha256": sha256_bytes(source),
            "construction": "compiler_source-v1",
            "inputs": [relative_record(path) for path in compiler_inputs],
        },
        "harness_inputs": [relative_record(path) for path in harness_input_paths()],
        "stage0_inputs": [relative_record(path) for path in stage0_inputs],
        "tools": {
            "clang": tool_record(CLANG, [str(CLANG), "--version"]),
            "python": tool_record(python_path, [str(python_path), "--version"]),
        },
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "python_implementation": platform.python_implementation(),
            "allocator": "macOS system allocator through libSystem",
        },
        "environment": {key: os.environ.get(key) for key in ENVIRONMENT_KEYS},
        "build_profile": {
            "stage0_optimizer_facts": False,
            "frozen_ir_bytes": FROZEN_IR_BYTES,
            "frozen_ir_sha256": FROZEN_IR_SHA256,
            "clang": "/usr/bin/clang",
            "clang_optimization": "-O2",
            "main_stack": native_stack_record(),
            "native_cpu": False,
            "lto": False,
            "pgo": False,
        },
    }
    binding["binding_sha256"] = sha256_bytes(canonical_json_bytes(binding))
    return binding


def expected_correctness_sha256() -> str:
    encoded = bytearray(CORRECTNESS_PREFIX)
    for field in REPORT_FIELDS:
        encoded.extend(struct.pack("<Q", FROZEN_REPORT[field]))
    return sha256_bytes(bytes(encoded))


def load_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise HarnessError(f"could not read {label} at {path}: {error}") from error
    if not isinstance(value, dict):
        raise HarnessError(f"{label} at {path} is not a JSON object")
    return value


def validate_score_lock(lock: dict[str, Any], current_binding: dict[str, Any]) -> None:
    if lock.get("schema_version") != 1 or lock.get("kind") != "f-soa-baseline-lock":
        raise HarnessError("score lock has the wrong schema or kind")
    campaign = lock.get("campaign")
    expected_campaign = {
        "mode": "score",
        "variant": VARIANT,
        "phase": PHASE,
        "samples": SCORE_SAMPLES,
    }
    if campaign != expected_campaign:
        raise HarnessError("score lock campaign does not match the frozen campaign")
    if lock.get("expected_report") != FROZEN_REPORT:
        raise HarnessError("score lock expected report does not match the frozen report")
    if lock.get("expected_correctness_sha256") != expected_correctness_sha256():
        raise HarnessError("score lock correctness digest does not match")
    locked_binding = lock.get("binding")
    if locked_binding != current_binding:
        expected = (
            locked_binding.get("binding_sha256")
            if isinstance(locked_binding, dict)
            else "<missing>"
        )
        raise HarnessError(
            "score lock/input hash mismatch: "
            f"lock binding {expected}, current {current_binding['binding_sha256']}"
        )


def is_within(path: Path, directory: Path) -> bool:
    try:
        path.resolve().relative_to(directory.resolve())
        return True
    except ValueError:
        return False


def freeze_lock(output: Path) -> None:
    output = output.resolve()
    if output.exists():
        raise HarnessError(f"refusing to overwrite lock: {output}")
    if is_within(output, ROOT):
        raise HarnessError("write the preregistration lock outside the repository")
    repository = repository_state()
    require_clean_repository(repository, "score-lock freeze")
    binding = build_binding()
    lock = {
        "schema_version": 1,
        "kind": "f-soa-baseline-lock",
        "created_utc": utc_now(),
        "campaign": {
            "mode": "score",
            "variant": VARIANT,
            "phase": PHASE,
            "samples": SCORE_SAMPLES,
        },
        "binding": binding,
        "expected_report": FROZEN_REPORT,
        "expected_correctness_sha256": expected_correctness_sha256(),
    }
    write_json(output, lock)
    print(json.dumps({"lock": str(output), "sha256": sha256_file(output)}))


def import_stage0() -> Any:
    democ_path = str(ROOT / "prototype" / "democ")
    checker_path = str(ROOT / "prototype" / "checker")
    for path in (democ_path, checker_path):
        if path not in sys.path:
            sys.path.insert(0, path)
    import democ  # type: ignore

    return democ


def relative_artifact_record(path: Path, campaign_dir: Path) -> dict[str, Any]:
    return {
        "path": str(path.resolve().relative_to(campaign_dir.resolve())),
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def build_native(build_dir: Path, source_text: str) -> dict[str, Any]:
    build_dir.mkdir(parents=True, exist_ok=False)
    democ = import_stage0()
    ir = democ.compile_program(source_text, alias=False)
    leaked = [marker for marker in FORBIDDEN_FACT_MARKERS if marker in ir]
    if leaked:
        raise HarnessError(f"facts-off IR contains optimizer facts: {leaked}")

    ir_bytes = ir.encode("utf-8")
    ir_sha256 = sha256_bytes(ir_bytes)
    if len(ir_bytes) != FROZEN_IR_BYTES or ir_sha256 != FROZEN_IR_SHA256:
        raise HarnessError(
            "facts-off F-SOA IR failed the byte-identity gate: "
            f"expected {FROZEN_IR_BYTES}/{FROZEN_IR_SHA256}, "
            f"observed {len(ir_bytes)}/{ir_sha256}"
        )

    ir_path = build_dir / "fsoa.ll"
    fsync_write(ir_path, ir_bytes)
    executable = build_dir / "fsoa_sample"
    stack = native_stack_record()
    argv = [
        str(CLANG),
        "-O2",
        str(DRIVER),
        str(ir_path),
        stack["linker_flag"],
        "-o",
        str(executable),
    ]
    environment = os.environ.copy()
    for key in ("CFLAGS", "CPPFLAGS", "LDFLAGS", "LLVM_PROFILE_FILE"):
        environment.pop(key, None)
    result = subprocess.run(
        argv, cwd=ROOT, env=environment, capture_output=True, text=True
    )
    fsync_write(build_dir / "clang.stdout", result.stdout.encode("utf-8"))
    fsync_write(build_dir / "clang.stderr", result.stderr.encode("utf-8"))
    if result.returncode != 0:
        raise HarnessError(f"clang build failed:\n{result.stderr[:4000]}")
    record = {
        "schema_version": 1,
        "argv": argv,
        "cwd": str(ROOT),
        "returncode": result.returncode,
        "stage0_optimizer_facts": False,
        "frozen_ir_identity": {
            "bytes": FROZEN_IR_BYTES,
            "sha256": FROZEN_IR_SHA256,
        },
        "main_stack": stack,
        "forbidden_fact_markers": list(FORBIDDEN_FACT_MARKERS),
        "ir": relative_artifact_record(ir_path, build_dir.parent),
        "driver": relative_record(DRIVER),
        "executable": relative_artifact_record(executable, build_dir.parent),
        "stdout": "build/clang.stdout",
        "stderr": "build/clang.stderr",
    }
    write_json(build_dir / "build.json", record)
    return record


def sample_required_keys() -> set[str]:
    return {
        "schema_version",
        "kind",
        "variant",
        "phase",
        "mode",
        "not_a_score",
        "sample_index",
        "pid",
        "clock",
        "corpus_bytes",
        "corpus_sha256",
        "executable_sha256",
        "elapsed_ns",
        "report",
        "correctness_schema",
        "correctness_sha256",
    }


def validate_sample(
    sample: dict[str, Any],
    *,
    mode: str,
    index: int,
    executable_sha256: str,
) -> None:
    if set(sample) != sample_required_keys():
        raise HarnessError(
            f"sample {index} fields differ from schema: {sorted(set(sample))}"
        )
    expected_scalars = {
        "schema_version": 1,
        "kind": "f-soa-baseline-sample",
        "variant": VARIANT,
        "phase": PHASE,
        "mode": mode,
        "not_a_score": mode == "smoke",
        "sample_index": index,
        "corpus_bytes": FROZEN_SOURCE_BYTES,
        "corpus_sha256": FROZEN_SOURCE_SHA256,
        "executable_sha256": executable_sha256,
        "correctness_schema": "frontend-report-le-v1",
        "correctness_sha256": expected_correctness_sha256(),
    }
    for key, expected in expected_scalars.items():
        if sample.get(key) != expected:
            raise HarnessError(
                f"sample {index} {key} mismatch: expected {expected!r}, "
                f"observed {sample.get(key)!r}"
            )
    if sample.get("report") != FROZEN_REPORT:
        raise HarnessError(f"sample {index} frontend report mismatch")
    if not isinstance(sample.get("elapsed_ns"), int) or sample["elapsed_ns"] <= 0:
        raise HarnessError(f"sample {index} has invalid elapsed_ns")
    if not isinstance(sample.get("pid"), int) or sample["pid"] <= 0:
        raise HarnessError(f"sample {index} has invalid pid")
    if sample.get("clock") not in ("CLOCK_MONOTONIC_RAW", "CLOCK_MONOTONIC"):
        raise HarnessError(f"sample {index} has an unexpected clock")


def run_one_sample(
    executable: Path,
    corpus: Path,
    samples_dir: Path,
    *,
    mode: str,
    index: int,
    executable_sha256: str,
) -> dict[str, Any]:
    argv = [
        str(executable),
        "--corpus",
        str(corpus),
        "--expected-source-sha256",
        FROZEN_SOURCE_SHA256,
        "--expected-executable-sha256",
        executable_sha256,
        "--mode",
        mode,
        "--sample-index",
        str(index),
    ]
    result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
    stem = samples_dir / f"sample-{index:03d}"
    fsync_write(stem.with_suffix(".stdout"), result.stdout.encode("utf-8"))
    fsync_write(stem.with_suffix(".stderr"), result.stderr.encode("utf-8"))
    if result.returncode != 0:
        raise HarnessError(
            f"native sample {index} failed ({result.returncode}): "
            f"{result.stderr[:2000]}"
        )
    lines = result.stdout.splitlines()
    if len(lines) != 1:
        raise HarnessError(f"native sample {index} did not emit exactly one JSON line")
    try:
        sample = json.loads(lines[0])
    except json.JSONDecodeError as error:
        raise HarnessError(f"native sample {index} emitted malformed JSON") from error
    if not isinstance(sample, dict):
        raise HarnessError(f"native sample {index} JSON is not an object")
    validate_sample(
        sample,
        mode=mode,
        index=index,
        executable_sha256=executable_sha256,
    )
    return sample


def score_output_path_allowed(out_dir: Path) -> bool:
    if not is_within(out_dir, ROOT):
        return True
    return is_within(out_dir, HERE / "runs")


def run_campaign(
    *,
    mode: str,
    out_dir: Path,
    lock_path: Path | None = None,
    acknowledged: bool = False,
) -> dict[str, Any]:
    if mode not in ("smoke", "score"):
        raise HarnessError(f"unknown campaign mode: {mode}")
    out_dir = out_dir.resolve()
    if out_dir.exists():
        raise HarnessError(f"campaign directory already exists: {out_dir}")

    source_text, source_bytes, _ = canonical_compiler_source()
    repository = repository_state()
    binding = build_binding()
    lock_record: dict[str, Any] | None = None
    sample_count = SMOKE_SAMPLES
    if mode == "score":
        if not acknowledged:
            raise HarnessError("score mode requires --acknowledge-baseline-recording")
        if lock_path is None:
            raise HarnessError("score mode requires --lock")
        require_clean_repository(repository, "score mode")
        if not score_output_path_allowed(out_dir):
            raise HarnessError(
                "score output must be outside the repository or below the ignored "
                "experiments/data-layout-owning-sequence/runs directory"
            )
        lock_path = lock_path.resolve()
        if is_within(lock_path, ROOT):
            raise HarnessError("score lock must be outside the repository")
        lock = load_json_object(lock_path, "score lock")
        validate_score_lock(lock, binding)
        lock_record = {"path": str(lock_path), "sha256": sha256_file(lock_path)}
        sample_count = SCORE_SAMPLES

    out_dir.mkdir(parents=True, exist_ok=False)
    manifest_path = out_dir / "manifest.json"
    raw_path = out_dir / "raw.jsonl"
    samples_dir = out_dir / "samples"
    samples_dir.mkdir()
    manifest: dict[str, Any] = {
        "schema_version": 1,
        "kind": "f-soa-baseline-campaign",
        "status": "building",
        "created_utc": utc_now(),
        "mode": mode,
        "not_a_score": mode == "smoke",
        "variant": VARIANT,
        "phase": PHASE,
        "repository": repository,
        "binding": binding,
        "lock": lock_record,
        "source": {
            "path": "compiler-source.xl",
            "bytes": len(source_bytes),
            "sha256": sha256_bytes(source_bytes),
        },
        "sample_plan": {
            "samples": sample_count,
            "fresh_process_per_sample": True,
            "one_wrapper_call_per_process": True,
        },
        "samples_completed": 0,
        "expected_report": FROZEN_REPORT,
        "expected_correctness_sha256": expected_correctness_sha256(),
        "verdict": None,
        "limitations": [
            "baseline-only; no AoS arm and no performance comparison",
            "correctness digest covers the public FrontendReport, not internal tapes",
            "cold-wrapper timing combines allocation, eager fill, and all frontend phases",
            "the wrapper retains its thirty allocations; process exit reclaims them",
            "all native arms must reserve the same 64 MiB main stack under A1",
            "no RSS, allocator, hardware-counter, or retained-phase measurement yet",
        ],
    }
    write_json(manifest_path, manifest)

    try:
        fsync_write(out_dir / "compiler-source.xl", source_bytes)
        fsync_write(
            out_dir / "compiler-source.sha256",
            (FROZEN_SOURCE_SHA256 + "\n").encode("ascii"),
        )
        build = build_native(out_dir / "build", source_text)
        manifest["build"] = build
        manifest["status"] = "measuring"
        write_json(manifest_path, manifest)

        executable = out_dir / "build" / "fsoa_sample"
        executable_sha256 = build["executable"]["sha256"]
        corpus = out_dir / "compiler-source.xl"
        observed_pids: set[int] = set()
        for index in range(sample_count):
            sample = run_one_sample(
                executable,
                corpus,
                samples_dir,
                mode=mode,
                index=index,
                executable_sha256=executable_sha256,
            )
            if sample["pid"] in observed_pids:
                raise HarnessError("fresh-process invariant failed: PID was reused")
            observed_pids.add(sample["pid"])
            append_jsonl(raw_path, sample)
            manifest["samples_completed"] = index + 1
            write_json(manifest_path, manifest)

        if sha256_file(executable) != executable_sha256:
            raise HarnessError("native executable changed during the campaign")
        if sha256_file(corpus) != FROZEN_SOURCE_SHA256:
            raise HarnessError("canonical corpus changed during the campaign")

        if mode == "score":
            final_repository = repository_state()
            require_clean_repository(final_repository, "score finalization")
            final_binding = build_binding()
            assert lock_path is not None
            lock = load_json_object(lock_path.resolve(), "score lock")
            validate_score_lock(lock, final_binding)
            if final_binding != binding:
                raise HarnessError("bound inputs changed during scored recording")

        manifest["status"] = "complete"
        manifest["completed_utc"] = utc_now()
        manifest["raw"] = {
            "path": "raw.jsonl",
            "records": sample_count,
            "bytes": raw_path.stat().st_size,
            "sha256": sha256_file(raw_path),
        }
        manifest["sample_pids"] = sorted(observed_pids)
        write_json(manifest_path, manifest)
        return manifest
    except Exception as error:
        manifest["status"] = "invalid"
        manifest["invalid_utc"] = utc_now()
        manifest["error"] = f"{type(error).__name__}: {error}"
        write_json(manifest_path, manifest)
        raise


def read_raw(path: Path) -> list[dict[str, Any]]:
    records = []
    for line in path.read_text(encoding="utf-8").splitlines():
        value = json.loads(line)
        if not isinstance(value, dict):
            raise HarnessError("raw.jsonl contains a non-object")
        records.append(value)
    return records


def load_schemas() -> None:
    paths = tuple(sorted((HERE / "schemas").glob("*.json")))
    if len(paths) != 3:
        raise HarnessError("expected exactly three artifact schemas")
    for path in paths:
        load_json_object(path, "JSON schema")


def self_test() -> None:
    load_schemas()
    with tempfile.TemporaryDirectory(prefix="xlang-fsoa-self-test-") as raw:
        campaign = Path(raw) / "campaign"
        manifest = run_campaign(mode="smoke", out_dir=campaign)
        if manifest["status"] != "complete" or manifest["samples_completed"] != 2:
            raise HarnessError("smoke campaign did not complete two samples")
        records = read_raw(campaign / "raw.jsonl")
        if len(records) != 2 or records[0]["pid"] == records[1]["pid"]:
            raise HarnessError("self-test did not observe two fresh processes")
        if records[0]["correctness_sha256"] != records[1]["correctness_sha256"]:
            raise HarnessError("correctness digest changed between smoke samples")

        executable = campaign / "build" / "fsoa_sample"
        bad_hash = "0" * 64
        base_arguments = [
            str(executable),
            "--corpus",
            str(campaign / "compiler-source.xl"),
            "--expected-source-sha256",
        ]
        rejected_source = subprocess.run(
            [
                *base_arguments,
                bad_hash,
                "--expected-executable-sha256",
                records[0]["executable_sha256"],
                "--mode",
                "smoke",
                "--sample-index",
                "999",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if rejected_source.returncode == 0 or rejected_source.stdout:
            raise HarnessError("native source-hash mismatch was not rejected")

        rejected_executable = subprocess.run(
            [
                *base_arguments,
                FROZEN_SOURCE_SHA256,
                "--expected-executable-sha256",
                bad_hash,
                "--mode",
                "smoke",
                "--sample-index",
                "999",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if rejected_executable.returncode == 0 or rejected_executable.stdout:
            raise HarnessError("native executable-hash mismatch was not rejected")

        tampered_binding = copy.deepcopy(manifest["binding"])
        tampered_binding["canonical_source"]["sha256"] = bad_hash
        lock = {
            "schema_version": 1,
            "kind": "f-soa-baseline-lock",
            "campaign": {
                "mode": "score",
                "variant": VARIANT,
                "phase": PHASE,
                "samples": SCORE_SAMPLES,
            },
            "binding": tampered_binding,
            "expected_report": FROZEN_REPORT,
            "expected_correctness_sha256": expected_correctness_sha256(),
        }
        try:
            validate_score_lock(lock, manifest["binding"])
        except HarnessError:
            pass
        else:
            raise HarnessError("tampered score binding was accepted")

        altered = copy.deepcopy(records[0])
        altered["correctness_sha256"] = bad_hash
        try:
            validate_sample(
                altered,
                mode="smoke",
                index=0,
                executable_sha256=records[0]["executable_sha256"],
            )
        except HarnessError:
            pass
        else:
            raise HarnessError("tampered sample digest was accepted")

        try:
            require_clean_repository(
                {"dirty": True, "dirty_entries": [" M hostile-review-sentinel"]},
                "self-test score mode",
            )
        except HarnessError:
            pass
        else:
            raise HarnessError("dirty scored repository was accepted")

    print(
        "F-SOA baseline self-test: build, two fresh-process smoke samples, "
        "digest equality, source/executable-hash rejection, dirty-tree rejection, "
        "and lock mismatch rejection pass"
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    commands = result.add_subparsers(dest="command", required=True)
    commands.add_parser("self-test", help="run a temporary two-process smoke test")

    smoke = commands.add_parser("smoke", help="record two non-scoring samples")
    smoke.add_argument("--out-dir", type=Path, required=True)

    freeze = commands.add_parser(
        "freeze-lock", help="freeze a clean-tree score lock outside the repository"
    )
    freeze.add_argument("--output", type=Path, required=True)

    score = commands.add_parser(
        "score", help="record the locked baseline only; no comparison or verdict"
    )
    score.add_argument("--out-dir", type=Path, required=True)
    score.add_argument("--lock", type=Path, required=True)
    score.add_argument("--acknowledge-baseline-recording", action="store_true")
    return result


def main(argv: Iterable[str] | None = None) -> int:
    arguments = parser().parse_args(list(argv) if argv is not None else None)
    try:
        if arguments.command == "self-test":
            self_test()
        elif arguments.command == "smoke":
            manifest = run_campaign(mode="smoke", out_dir=arguments.out_dir)
            print(
                json.dumps(
                    {
                        "campaign": str(arguments.out_dir.resolve()),
                        "status": manifest["status"],
                        "not_a_score": True,
                        "samples": manifest["samples_completed"],
                    }
                )
            )
        elif arguments.command == "freeze-lock":
            freeze_lock(arguments.output)
        elif arguments.command == "score":
            manifest = run_campaign(
                mode="score",
                out_dir=arguments.out_dir,
                lock_path=arguments.lock,
                acknowledged=arguments.acknowledge_baseline_recording,
            )
            print(
                json.dumps(
                    {
                        "campaign": str(arguments.out_dir.resolve()),
                        "status": manifest["status"],
                        "not_a_score": False,
                        "samples": manifest["samples_completed"],
                        "verdict": None,
                    }
                )
            )
        else:
            raise HarnessError(f"unhandled command: {arguments.command}")
    except HarnessError as error:
        print(f"F-SOA baseline harness: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
