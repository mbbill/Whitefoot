#!/usr/bin/env python3
"""Pinned RG-BASE correctness and measurement runner.

This is deliberately experiment-local. It knows only the frozen ripgrep
baseline protocol and is not reusable benchmark infrastructure.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import plistlib
import random
import re
import statistics
import subprocess
import sys
import time
from typing import Any


HERE = Path(__file__).resolve().parent
DEFAULT_MANIFEST = HERE / "manifest.json"
RUN_ID_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,79}\Z")
FORBIDDEN_VALUES = ("TODO", "TBD", "PLACEHOLDER")
MANIFEST_PHASES = {
    "draft-before-freeze",
    "frozen-before-selection",
    "selected-before-baseline",
    "complete",
}


class ProtocolError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ProtocolError(message)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def reject_placeholders(value: Any, at: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            reject_placeholders(child, f"{at}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            reject_placeholders(child, f"{at}[{index}]")
    elif isinstance(value, str):
        upper = value.strip().upper()
        if upper in FORBIDDEN_VALUES or "<FILL" in upper:
            raise ProtocolError(f"unresolved marker {value!r} at {at}")


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        raw = path.read_bytes()
        manifest = json.loads(raw)
    except (OSError, json.JSONDecodeError) as error:
        raise ProtocolError(f"cannot load manifest {path}: {error}") from error
    require(manifest.get("schema_version") == 1, "unsupported manifest schema")
    reject_placeholders(manifest)
    validate_phase_shape(manifest)
    return manifest


def validate_phase_shape(manifest: dict[str, Any]) -> None:
    phase = manifest.get("phase")
    require(phase in MANIFEST_PHASES, f"unknown manifest phase: {phase!r}")
    selected = [case.get("selected_comparator") for case in manifest["cases"]]
    if phase in ("draft-before-freeze", "frozen-before-selection"):
        require("selection_evidence" not in manifest,
                f"selection evidence is forbidden in phase {phase}")
        require("baseline_evidence" not in manifest,
                f"baseline evidence is forbidden in phase {phase}")
        require(all(value is None for value in selected),
                f"selected comparators are forbidden in phase {phase}")
    elif phase == "selected-before-baseline":
        require(isinstance(manifest.get("selection_evidence"), dict),
                "selected phase requires selection evidence")
        require("baseline_evidence" not in manifest,
                "baseline evidence is forbidden before baseline")
        require(all(value in ("official", "native") for value in selected),
                "selected phase requires one comparator per case")
    else:
        require(isinstance(manifest.get("selection_evidence"), dict),
                "complete phase requires selection evidence")
        require(isinstance(manifest.get("baseline_evidence"), dict),
                "complete phase requires baseline evidence")
        require(all(value in ("official", "native") for value in selected),
                "complete phase requires one comparator per case")


def manifest_sha256(path: Path) -> str:
    return sha256_file(path)


def resolve_work_path(work_root: Path, relative: str) -> Path:
    require(not Path(relative).is_absolute(), f"work path must be relative: {relative}")
    root = work_root.resolve()
    path = (root / relative).resolve()
    try:
        path.relative_to(root)
    except ValueError as error:
        raise ProtocolError(f"work path escapes root: {relative}") from error
    return path


def capture(argv: list[str], cwd: Path | None = None) -> bytes:
    try:
        return subprocess.check_output(argv, cwd=cwd, stderr=subprocess.STDOUT)
    except (OSError, subprocess.CalledProcessError) as error:
        raise ProtocolError(f"command failed: {argv!r}: {error}") from error


def sysctl(name: str) -> str:
    return capture(["sysctl", "-n", name]).decode("utf-8", "strict").strip()


def host_facts() -> dict[str, Any]:
    return {
        "machine": platform.machine(),
        "model": sysctl("hw.model"),
        "physical_cores": int(sysctl("hw.physicalcpu")),
        "logical_cores": int(sysctl("hw.logicalcpu")),
        "performance_cores": int(sysctl("hw.perflevel0.physicalcpu")),
        "efficiency_cores": int(sysctl("hw.perflevel1.physicalcpu")),
        "memory_bytes": int(sysctl("hw.memsize")),
        "page_bytes": int(sysctl("hw.pagesize")),
        "cache_line_bytes": int(sysctl("hw.cachelinesize")),
        "macos": platform.mac_ver()[0],
        "macos_build": capture(["sw_vers", "-buildVersion"])
        .decode("utf-8", "strict")
        .strip(),
        "darwin": platform.release(),
    }


def python_facts(manifest: dict[str, Any]) -> dict[str, Any]:
    executable = Path(sys.executable).resolve()
    library = Path(manifest["runner"]["python_library_path"])
    return {
        "implementation": sys.implementation.name,
        "version": platform.python_version(),
        "executable_path": str(executable),
        "executable": file_identity(executable),
        "library_path": str(library),
        "library": file_identity(library),
    }


def file_identity(path: Path) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    return {
        "size": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def binary_inventory(path: Path) -> dict[str, Any]:
    identity = file_identity(path)
    version = capture([str(path), "--version"])
    return {
        **identity,
        "version_sha256": sha256_bytes(version),
    }


def git_text(root: Path, *args: str) -> str:
    return capture(["git", "-C", str(root), *args]).decode("utf-8", "strict").strip()


def repository_facts(manifest_path: Path) -> dict[str, str]:
    require(manifest_path.resolve() == DEFAULT_MANIFEST.resolve(),
            "timing only accepts the repository's default manifest")
    root = Path(git_text(HERE, "rev-parse", "--show-toplevel"))
    relative_manifest = manifest_path.resolve().relative_to(root.resolve())
    capture(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "--error-unmatch",
            str(relative_manifest),
        ]
    )
    status = capture(
        [
            "git",
            "-C",
            str(root),
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
        ]
    )
    require(status == b"", "timing requires a clean experiment checkout")
    return {
        "head": git_text(root, "rev-parse", "HEAD"),
        "root_name": root.name,
        "manifest_path": relative_manifest.as_posix(),
    }


def git_inventory(root: Path) -> dict[str, Any]:
    require(root.is_dir(), f"missing git corpus: {root}")
    status = capture(
        [
            "git",
            "-C",
            str(root),
            "status",
            "--porcelain=v1",
            "--ignored",
            "-z",
            "--untracked-files=all",
        ]
    )
    require(status == b"", f"git corpus is not clean: {root}")

    raw_entries = capture(["git", "-C", str(root), "ls-files", "--stage", "-z"])
    entries: list[tuple[bytes, bytes]] = []
    for raw_entry in raw_entries.split(b"\0"):
        if not raw_entry:
            continue
        metadata, separator, path_bytes = raw_entry.partition(b"\t")
        require(separator == b"\t", "malformed git index entry")
        mode, _object_id, stage = metadata.split(b" ")
        require(stage == b"0", f"unmerged git index entry: {path_bytes!r}")
        require(b"\n" not in path_bytes and b":" not in path_bytes,
                f"ambiguous output path in corpus: {path_bytes!r}")
        entries.append((path_bytes, mode))
    entries.sort(key=lambda item: item[0])

    digest = hashlib.sha256()
    digest.update(b"whitefoot-rg-base-tree-v1\0")
    total_bytes = 0
    ignore_files = 0
    root_bytes = os.fsencode(root)
    for path_bytes, mode in entries:
        full_path = os.path.join(root_bytes, path_bytes)
        if mode == b"120000":
            content = os.readlink(full_path)
            if isinstance(content, str):
                content = os.fsencode(content)
            content_digest = sha256_bytes(content)
            size = len(content)
        elif mode in (b"100644", b"100755"):
            content_hash = hashlib.sha256()
            size = 0
            with open(full_path, "rb") as stream:
                while chunk := stream.read(1024 * 1024):
                    size += len(chunk)
                    content_hash.update(chunk)
            content_digest = content_hash.hexdigest()
        else:
            raise ProtocolError(
                f"unsupported git mode {mode!r} for {path_bytes!r}"
            )
        total_bytes += size
        if path_bytes.rsplit(b"/", 1)[-1] in (
            b".gitignore",
            b".ignore",
            b".rgignore",
        ):
            ignore_files += 1
        digest.update(len(path_bytes).to_bytes(8, "big"))
        digest.update(path_bytes)
        digest.update(mode)
        digest.update(size.to_bytes(8, "big"))
        digest.update(bytes.fromhex(content_digest))

    info_exclude = file_identity(root / ".git" / "info" / "exclude")
    return {
        "commit": git_text(root, "rev-parse", "HEAD"),
        "tree": git_text(root, "rev-parse", "HEAD^{tree}"),
        "tracked_entries": len(entries),
        "tracked_bytes": total_bytes,
        "ignore_files": ignore_files,
        "checkout_sha256": digest.hexdigest(),
        "info_exclude": info_exclude,
    }


def split_delimited_records(data: bytes, delimiter: bytes) -> list[bytes]:
    require(len(delimiter) == 1, "record delimiter must be one byte")
    records: list[bytes] = []
    start = 0
    while True:
        end = data.find(delimiter, start)
        if end < 0:
            if start < len(data):
                records.append(data[start:])
            break
        records.append(data[start : end + 1])
        start = end + 1
    return records


def canonical_record_sha256(records: list[bytes]) -> str:
    record_hashes = []
    for record in records:
        payload = len(record).to_bytes(8, "big") + record
        record_hashes.append(hashlib.sha256(payload).digest())
    record_hashes.sort()
    digest = hashlib.sha256()
    digest.update(b"whitefoot-rg-base-record-multiset-v1\0")
    digest.update(len(records).to_bytes(8, "big"))
    for record_hash in record_hashes:
        digest.update(record_hash)
    return digest.hexdigest()


def canonical_file_blocks(
    records: list[bytes],
) -> tuple[str, int]:
    blocks: list[tuple[bytes, bytes]] = []
    seen_paths: set[bytes] = set()
    current_path: bytes | None = None
    current_records: list[bytes] = []
    for record in records:
        path, separator, _remainder = record.partition(b":")
        require(separator == b":" and path,
                f"malformed prefixed output record: {record[:120]!r}")
        if path != current_path:
            if current_path is not None:
                blocks.append((current_path, b"".join(current_records)))
            require(path not in seen_paths,
                    f"non-contiguous output block for path: {path!r}")
            seen_paths.add(path)
            current_path = path
            current_records = []
        current_records.append(record)
    if current_path is not None:
        blocks.append((current_path, b"".join(current_records)))

    blocks.sort(key=lambda item: item[0])
    digest = hashlib.sha256()
    digest.update(b"whitefoot-rg-base-file-blocks-v1\0")
    digest.update(len(blocks).to_bytes(8, "big"))
    digest.update(len(records).to_bytes(8, "big"))
    for path, block in blocks:
        digest.update(len(path).to_bytes(8, "big"))
        digest.update(path)
        digest.update(len(block).to_bytes(8, "big"))
        digest.update(hashlib.sha256(block).digest())
    return digest.hexdigest(), len(blocks)


def output_fingerprint(data: bytes, mode: str) -> dict[str, Any]:
    if mode == "exact":
        records = split_delimited_records(data, b"\n")
        digest = sha256_bytes(data)
        blocks = None
    elif mode == "file_blocks":
        records = split_delimited_records(data, b"\n")
        digest, blocks = canonical_file_blocks(records)
    elif mode == "nul_records":
        records = split_delimited_records(data, b"\0")
        digest = canonical_record_sha256(records)
        blocks = None
    else:
        raise ProtocolError(f"unknown output mode: {mode}")
    result = {
        "mode": mode,
        "bytes": len(data),
        "records": len(records),
        "sha256": digest,
    }
    if blocks is not None:
        result["blocks"] = blocks
    return result


def result_fingerprint(
    returncode: int, stdout: bytes, stderr: bytes, stdout_mode: str
) -> dict[str, Any]:
    return {
        "status": returncode,
        "stdout": output_fingerprint(stdout, stdout_mode),
        "stderr": {
            "bytes": len(stderr),
            "sha256": sha256_bytes(stderr),
        },
    }


def invoke_with_output(
    binary: Path,
    cwd: Path,
    argv: list[str],
    environment: dict[str, str],
    timeout_seconds: int,
    stdout_mode: str,
) -> tuple[int, dict[str, Any], bytes, bytes]:
    env = environment.copy()
    command = [str(binary), *argv]
    started = time.perf_counter_ns()
    try:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        stdout, stderr = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired as error:
        process.kill()
        process.communicate()
        raise ProtocolError(
            f"timeout after {timeout_seconds}s: {command!r}"
        ) from error
    except OSError as error:
        raise ProtocolError(f"cannot run {command!r}: {error}") from error
    elapsed_ns = time.perf_counter_ns() - started
    return (
        elapsed_ns,
        result_fingerprint(process.returncode, stdout, stderr, stdout_mode),
        stdout,
        stderr,
    )


def invoke(
    binary: Path,
    cwd: Path,
    argv: list[str],
    environment: dict[str, str],
    timeout_seconds: int,
    stdout_mode: str,
) -> tuple[int, dict[str, Any]]:
    elapsed, result, _stdout, _stderr = invoke_with_output(
        binary, cwd, argv, environment, timeout_seconds, stdout_mode
    )
    return elapsed, result


def expected_equal(actual: dict[str, Any], expected: dict[str, Any], label: str) -> None:
    if actual != expected:
        raise ProtocolError(
            f"{label} output mismatch\n"
            f"expected={json.dumps(expected, sort_keys=True)}\n"
            f"actual={json.dumps(actual, sort_keys=True)}"
        )


def binary_paths(manifest: dict[str, Any], work_root: Path) -> dict[str, Path]:
    return {
        name: resolve_work_path(work_root, record["path"])
        for name, record in manifest["binaries"].items()
        if record.get("role") in ("contender", "diagnostic")
    }


def corpus_paths(manifest: dict[str, Any], work_root: Path) -> dict[str, Path]:
    return {
        name: resolve_work_path(work_root, record["path"])
        for name, record in manifest["corpora"].items()
    }


def workload_environment(
    manifest: dict[str, Any], work_root: Path
) -> dict[str, str]:
    temporary = resolve_work_path(work_root, "work")
    require(temporary.is_dir(), f"missing scratch work directory: {temporary}")
    return {
        **manifest["environment"],
        "TMPDIR": str(temporary),
    }


def actual_inventory(manifest: dict[str, Any], work_root: Path) -> dict[str, Any]:
    binaries = binary_paths(manifest, work_root)
    corpora = corpus_paths(manifest, work_root)
    binary_facts = {
        name: binary_inventory(path) for name, path in binaries.items()
    }
    corpus_facts: dict[str, Any] = {}
    for name, record in manifest["corpora"].items():
        path = corpora[name]
        if record["kind"] == "git":
            corpus_facts[name] = git_inventory(path)
        elif record["kind"] == "file":
            facts = file_identity(path)
            if record.get("archive_path"):
                facts["archive"] = file_identity(
                    resolve_work_path(work_root, record["archive_path"])
                )
            corpus_facts[name] = facts
        else:
            raise ProtocolError(f"unknown corpus kind for {name}")
    return {
        "host": host_facts(),
        "runner": python_facts(manifest),
        "binaries": binary_facts,
        "corpora": corpus_facts,
    }


def validate_storage(
    work_root: Path,
    corpus_root: Path,
    storage: dict[str, Any],
) -> None:
    image = resolve_work_path(work_root, storage["image_path"])
    mount = resolve_work_path(work_root, storage["mount_path"])
    require(image.exists(), f"missing Linux sparse image: {image}")
    require(mount.is_mount(), f"Linux corpus mount is not attached: {mount}")
    require(corpus_root.resolve().is_relative_to(mount.resolve()),
            "Linux corpus is not inside its frozen mount")

    image_info = plistlib.loads(capture(["hdiutil", "info", "-plist"]))
    matching_images = [
        item
        for item in image_info.get("images", [])
        if Path(item.get("image-path", "")).resolve() == image.resolve()
    ]
    require(len(matching_images) == 1,
            "Linux sparse image is not attached exactly once")
    attached_mounts = {
        Path(entity["mount-point"]).resolve()
        for entity in matching_images[0].get("system-entities", [])
        if entity.get("mount-point")
    }
    require(mount.resolve() in attached_mounts,
            "Linux sparse image is attached at the wrong mount point")
    require(matching_images[0].get("image-type") == storage["image_type"],
            "Linux image type mismatch")

    volume = plistlib.loads(
        capture(["diskutil", "info", "-plist", str(mount)])
    )
    required = {
        "BusProtocol": storage["bus_protocol"],
        "FilesystemName": storage["filesystem_name"],
        "FilesystemType": storage["filesystem_type"],
        "MountPoint": str(mount),
        "VolumeName": storage["volume_name"],
    }
    for key, expected in required.items():
        require(volume.get(key) == expected,
                f"Linux volume {key} mismatch: {volume.get(key)!r}")
    require(volume.get("Writable") is True, "Linux corpus volume is not writable")


def validate_work_root_storage(
    work_root: Path, expected: dict[str, Any]
) -> None:
    df = capture(["df", "-P", str(work_root)]).decode("utf-8", "strict")
    lines = [line for line in df.splitlines() if line.strip()]
    require(len(lines) >= 2, "cannot identify work-root volume")
    mount = Path(lines[-1].split()[-1])
    volume = plistlib.loads(
        capture(["diskutil", "info", "-plist", str(mount)])
    )
    required = {
        "BusProtocol": expected["bus_protocol"],
        "FilesystemName": expected["filesystem_name"],
        "FilesystemType": expected["filesystem_type"],
        "Internal": expected["internal"],
        "SolidState": expected["solid_state"],
        "DeviceBlockSize": expected["device_block_bytes"],
        "VolumeAllocationBlockSize": expected["allocation_block_bytes"],
    }
    for key, value in required.items():
        require(volume.get(key) == value,
                f"work-root storage {key} mismatch: {volume.get(key)!r}")


def validate_case_sensitive_probe(root: Path, probe: list[str]) -> None:
    require(len(probe) == 2, "case-sensitive probe needs two paths")
    left = resolve_work_path(root, probe[0])
    right = resolve_work_path(root, probe[1])
    require(left.exists() and right.exists(), "case-sensitive probe paths missing")
    require(not os.path.samefile(left, right), "corpus filesystem is case-insensitive")


def validate_inventory(manifest: dict[str, Any], work_root: Path) -> None:
    validate_work_root_storage(
        work_root, manifest["target"]["work_root_storage"]
    )
    actual = actual_inventory(manifest, work_root)
    expected_host = manifest["target"]["identity"]
    require(actual["host"] == expected_host,
            f"host identity mismatch: expected {expected_host}, actual {actual['host']}")
    require(actual["runner"] == manifest["runner"]["identity"],
            f"Python runner identity mismatch: {actual['runner']}")

    for name, expected in manifest["binaries"].items():
        if expected.get("role") not in ("contender", "diagnostic"):
            continue
        actual_binary = actual["binaries"][name]
        identity = {
            "size": expected["size"],
            "sha256": expected["sha256"],
            "version_sha256": expected["version_sha256"],
        }
        require(actual_binary == identity,
                f"binary identity mismatch for {name}: {actual_binary}")

    corpus_roots = corpus_paths(manifest, work_root)
    for name, expected in manifest["corpora"].items():
        if expected["kind"] == "git":
            identity_keys = (
                "commit",
                "tree",
                "tracked_entries",
                "tracked_bytes",
                "ignore_files",
                "checkout_sha256",
                "info_exclude",
            )
        else:
            identity_keys = ("size", "sha256", "archive")
        expected_identity = {key: expected[key] for key in identity_keys}
        require(actual["corpora"][name] == expected_identity,
                f"corpus identity mismatch for {name}: {actual['corpora'][name]}")
        if expected.get("case_sensitive_probe"):
            validate_case_sensitive_probe(
                corpus_roots[name], expected["case_sensitive_probe"]
            )
        if expected.get("storage"):
            validate_storage(
                work_root, corpus_roots[name], expected["storage"]
            )


def command_for_case(manifest: dict[str, Any], case: dict[str, Any]) -> list[str]:
    return [*manifest["common_argv"], *case["argv"]]


def run_case(
    manifest: dict[str, Any],
    work_root: Path,
    binary_name: str,
    case: dict[str, Any],
) -> tuple[int, dict[str, Any]]:
    binaries = binary_paths(manifest, work_root)
    corpora = corpus_paths(manifest, work_root)
    corpus_record = manifest["corpora"][case["corpus"]]
    corpus_path = corpora[case["corpus"]]
    cwd = corpus_path if corpus_record["kind"] == "git" else corpus_path.parent
    return invoke(
        binaries[binary_name],
        cwd,
        command_for_case(manifest, case),
        workload_environment(manifest, work_root),
        manifest["measurement"]["timeout_seconds"],
        case["stdout_mode"],
    )


def selected_path_stats(corpus_root: Path, stdout: bytes) -> dict[str, int]:
    records = split_delimited_records(stdout, b"\0")
    total_bytes = 0
    for record in records:
        path_bytes = record[:-1] if record.endswith(b"\0") else record
        require(path_bytes, "empty selected path")
        require(b"\n" not in path_bytes and b":" not in path_bytes,
                f"ambiguous selected path: {path_bytes!r}")
        path = (corpus_root / os.fsdecode(path_bytes)).resolve()
        try:
            path.relative_to(corpus_root.resolve())
        except ValueError as error:
            raise ProtocolError(f"selected path escapes corpus: {path}") from error
        require(path.is_file(), f"selected path is not a file: {path}")
        total_bytes += path.stat().st_size
    return {"files": len(records), "bytes": total_bytes}


def compute_oracles(
    manifest: dict[str, Any], work_root: Path
) -> dict[str, Any]:
    binaries = binary_paths(manifest, work_root)
    corpora = corpus_paths(manifest, work_root)
    timeout = manifest["measurement"]["timeout_seconds"]
    environment = workload_environment(manifest, work_root)
    result: dict[str, Any] = {
        "traversals": {},
        "cases": {},
        "controls": {},
    }

    for traversal in manifest["traversals"]:
        pair = {}
        for arm in ("official", "native"):
            elapsed, fingerprint, stdout, _stderr = invoke_with_output(
                binaries[arm],
                corpora[traversal["corpus"]],
                traversal["argv"],
                environment,
                timeout,
                "nul_records",
            )
            del elapsed
            pair[arm] = fingerprint
            if arm == "official":
                official_stdout = stdout
        expected_equal(pair["native"], pair["official"],
                       f"traversal {traversal['id']} native vs official")
        require(pair["official"]["status"] == 0,
                f"traversal {traversal['id']} did not succeed")
        path_stats = selected_path_stats(
            corpora[traversal["corpus"]], official_stdout
        )
        result["traversals"][traversal["id"]] = {
            **pair["official"],
            "selected": path_stats,
        }

    for group_name in ("cases", "controls"):
        for case in manifest[group_name]:
            pair = {}
            for arm in ("official", "native"):
                _elapsed, pair[arm] = run_case(
                    manifest, work_root, arm, case
                )
            expected_equal(pair["native"], pair["official"],
                           f"{group_name[:-1]} {case['id']} native vs official")
            expected_status = case.get("expected_status", 0)
            require(pair["official"]["status"] == expected_status,
                    f"{case['id']} status is not {expected_status}")
            if group_name == "cases":
                require(pair["official"]["stdout"]["records"] > 0,
                        f"timed case {case['id']} has no output records")
            result[group_name][case["id"]] = pair["official"]
    return result


def verify_oracles(manifest: dict[str, Any], work_root: Path) -> None:
    require("oracles" in manifest, "manifest has no frozen oracles")
    actual = compute_oracles(manifest, work_root)
    expected_equal(actual, manifest["oracles"], "frozen oracle set")


def power_snapshot() -> dict[str, Any]:
    battery = capture(["pmset", "-g", "batt"]).decode("utf-8", "replace")
    custom = capture(["pmset", "-g", "custom"]).decode("utf-8", "replace")
    thermal = capture(["pmset", "-g", "therm"]).decode("utf-8", "replace")
    require("AC Power" in battery, "timing requires AC power")
    source_match = re.search(r"Now drawing from '([^']+)'", battery)
    percent_match = re.search(r"\b(\d{1,3})%;", battery)
    state_match = re.search(
        r"\b(charged|charging|discharging|finishing charge)\b", battery
    )
    require(source_match is not None and percent_match is not None,
            "cannot parse power source and charge")
    ac_section = custom.split("AC Power:", 1)
    require(len(ac_section) == 2, "cannot find AC power settings")
    require(re.search(r"(?m)^\s*lowpowermode\s+0\s*$", ac_section[1]) is not None,
            "timing requires Low Power Mode off")
    return {
        "battery": {
            "source": source_match.group(1),
            "percent": int(percent_match.group(1)),
            "state": state_match.group(1) if state_match else "unknown",
        },
        "custom_sha256": sha256_bytes(custom.encode()),
        "thermal": thermal.strip(),
        "load_average": list(os.getloadavg()),
    }


def rotated(items: list[dict[str, Any]], offset: int) -> list[dict[str, Any]]:
    if not items:
        return []
    offset %= len(items)
    return items[offset:] + items[:offset]


def arm_order(round_index: int, case_index: int) -> tuple[str, str]:
    if (round_index + case_index) % 2 == 0:
        return ("official", "native")
    return ("native", "official")


def median(values: list[int]) -> float:
    require(bool(values), "cannot take median of no samples")
    return float(statistics.median(values))


def paired_bootstrap_ratio(
    official: list[int],
    native: list[int],
    samples: int,
    seed: int,
) -> dict[str, Any]:
    require(len(official) == len(native) and official,
            "paired samples must be non-empty and equal length")
    rng = random.Random(seed)
    ratios: list[float] = []
    count = len(official)
    for _ in range(samples):
        indices = [rng.randrange(count) for _ in range(count)]
        off = median([official[index] for index in indices])
        nat = median([native[index] for index in indices])
        ratios.append(off / nat)
    ratios.sort()
    low_index = math.floor(0.025 * (samples - 1))
    high_index = math.ceil(0.975 * (samples - 1))
    estimate = median(official) / median(native)
    return {
        "official_median_ns": median(official),
        "native_median_ns": median(native),
        "official_over_native": estimate,
        "ci95": [ratios[low_index], ratios[high_index]],
        "relative_half_width": (
            ratios[high_index] - ratios[low_index]
        ) / (2.0 * estimate),
    }


def bootstrap_median(
    values: list[int], samples: int, seed: int
) -> dict[str, Any]:
    require(bool(values), "samples must be non-empty")
    rng = random.Random(seed)
    medians = []
    count = len(values)
    for _ in range(samples):
        medians.append(median([values[rng.randrange(count)] for _ in range(count)]))
    medians.sort()
    low_index = math.floor(0.025 * (samples - 1))
    high_index = math.ceil(0.975 * (samples - 1))
    estimate = median(values)
    return {
        "median_ns": estimate,
        "ci95": [medians[low_index], medians[high_index]],
        "relative_half_width": (
            medians[high_index] - medians[low_index]
        ) / (2.0 * estimate),
    }


def selection_summaries(
    manifest: dict[str, Any],
    measurements: dict[str, dict[str, list[int]]],
) -> tuple[dict[str, Any], list[str]]:
    summaries = {}
    inconclusive = []
    for case_index, case in enumerate(manifest["cases"]):
        case_id = case["id"]
        summary = paired_bootstrap_ratio(
            measurements[case_id]["official"],
            measurements[case_id]["native"],
            manifest["measurement"]["bootstrap_samples"],
            manifest["measurement"]["bootstrap_seed"] + case_index,
        )
        summary["selected"] = (
            "native" if summary["official_over_native"] > 1.0 else "official"
        )
        low, high = summary["ci95"]
        summary["difference_resolved"] = low > 1.0 or high < 1.0
        summary["precise"] = (
            summary["relative_half_width"]
            <= manifest["measurement"]["max_relative_ci_half_width"]
        )
        if not summary["precise"]:
            inconclusive.append(case_id)
        summaries[case_id] = summary
    return summaries, inconclusive


def baseline_summaries(
    manifest: dict[str, Any],
    measurements: dict[str, list[int]],
) -> tuple[dict[str, Any], list[str]]:
    summaries = {}
    inconclusive = []
    for case_index, case in enumerate(manifest["cases"]):
        summary = bootstrap_median(
            measurements[case["id"]],
            manifest["measurement"]["bootstrap_samples"],
            manifest["measurement"]["bootstrap_seed"] + 1000 + case_index,
        )
        summary["selected"] = case["selected_comparator"]
        summary["precise"] = (
            summary["relative_half_width"]
            <= manifest["measurement"]["max_relative_ci_half_width"]
        )
        if not summary["precise"]:
            inconclusive.append(case["id"])
        summaries[case["id"]] = summary
    return summaries, inconclusive


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    records = []
    try:
        with path.open("r", encoding="utf-8") as stream:
            for line_number, line in enumerate(stream, 1):
                require(line.endswith("\n"),
                        f"JSONL line {line_number} lacks a newline")
                records.append(json.loads(line))
    except (OSError, json.JSONDecodeError) as error:
        raise ProtocolError(f"cannot load evidence {path}: {error}") from error
    require(bool(records), f"empty evidence file: {path}")
    return records


def normalized_frozen_manifest(manifest: dict[str, Any]) -> dict[str, Any]:
    normalized = copy.deepcopy(manifest)
    normalized["phase"] = "frozen-before-selection"
    normalized.pop("selection_evidence", None)
    normalized.pop("baseline_evidence", None)
    for case in normalized["cases"]:
        case["selected_comparator"] = None
    return normalized


def normalized_selected_manifest(manifest: dict[str, Any]) -> dict[str, Any]:
    normalized = copy.deepcopy(manifest)
    normalized["phase"] = "selected-before-baseline"
    normalized.pop("baseline_evidence", None)
    return normalized


def require_tracked_file(root: Path, path: Path) -> str:
    relative = path.resolve().relative_to(root.resolve()).as_posix()
    capture(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "--error-unmatch",
            relative,
        ]
    )
    return relative


def verify_frozen_apparatus(
    root: Path,
    freeze_commit: str,
    apparatus_files: list[str],
) -> None:
    for relative in apparatus_files:
        path = (root / relative).resolve()
        require_tracked_file(root, path)
        current = path.read_bytes()
        frozen = capture(
            ["git", "-C", str(root), "show", f"{freeze_commit}:{relative}"]
        )
        require(current == frozen,
                f"measurement apparatus changed since freeze: {relative}")


def validate_power_evidence(power: Any) -> None:
    require(isinstance(power, dict), "evidence power snapshot is missing")
    battery = power.get("battery")
    require(isinstance(battery, dict), "evidence battery summary is missing")
    require(battery.get("source") == "AC Power",
            "evidence was not recorded on AC power")
    require(isinstance(battery.get("percent"), int)
            and 0 <= battery["percent"] <= 100,
            "evidence battery percentage is invalid")
    require(set(battery) == {"source", "percent", "state"},
            "evidence battery summary contains unexpected fields")
    require(isinstance(power.get("custom_sha256"), str),
            "evidence power-settings digest is missing")
    require(isinstance(power.get("thermal"), str),
            "evidence thermal record is missing")
    load_average = power.get("load_average")
    require(isinstance(load_average, list) and len(load_average) == 3,
            "evidence load-average record is invalid")


def verify_selection_evidence(
    manifest: dict[str, Any], manifest_path: Path
) -> dict[str, Any]:
    require(manifest["phase"] in ("selected-before-baseline", "complete"),
            "selection evidence is not valid in this manifest phase")
    evidence = manifest.get("selection_evidence")
    require(isinstance(evidence, dict), "manifest has no selection evidence")
    evidence_path = (HERE / evidence["path"]).resolve()
    try:
        evidence_path.relative_to(HERE)
    except ValueError as error:
        raise ProtocolError("selection evidence path escapes bundle") from error
    require(sha256_file(evidence_path) == evidence["sha256"],
            "selection evidence SHA-256 mismatch")

    root = Path(git_text(HERE, "rev-parse", "--show-toplevel"))
    require_tracked_file(root, evidence_path)
    relative_manifest = manifest_path.resolve().relative_to(root.resolve())
    freeze_commit = evidence["freeze_commit"]
    capture(["git", "-C", str(root), "merge-base", "--is-ancestor",
             freeze_commit, "HEAD"])
    frozen_bytes = capture(
        [
            "git",
            "-C",
            str(root),
            "show",
            f"{freeze_commit}:{relative_manifest.as_posix()}",
        ]
    )
    require(sha256_bytes(frozen_bytes) == evidence["input_manifest_sha256"],
            "frozen manifest digest does not match selection evidence")
    try:
        frozen_manifest = json.loads(frozen_bytes)
    except json.JSONDecodeError as error:
        raise ProtocolError("frozen manifest is not valid JSON") from error
    require(frozen_manifest["phase"] == "frozen-before-selection",
            "selection input commit is not a frozen manifest")
    require(normalized_frozen_manifest(manifest) == frozen_manifest,
            "selected manifest changed more than selection evidence permits")
    verify_frozen_apparatus(
        root, freeze_commit, frozen_manifest["apparatus_files"]
    )

    records = load_jsonl(evidence_path)
    header = records[0]
    summary_record = records[-1]
    require(header.get("kind") == "header"
            and header.get("phase") == "selection",
            "selection evidence has no valid header")
    require(header.get("run_id") == evidence["run_id"]
            == frozen_manifest["measurement"]["selection_run_id"],
            "selection evidence run id mismatch")
    require(header.get("manifest_sha256") == evidence["input_manifest_sha256"],
            "selection header manifest digest mismatch")
    require(header.get("repository", {}).get("head") == freeze_commit,
            "selection header freeze commit mismatch")
    validate_power_evidence(header.get("power"))
    require(summary_record.get("kind") == "summary"
            and summary_record.get("phase") == "selection",
            "selection evidence has no valid summary")
    validate_power_evidence(summary_record.get("power"))

    cases = frozen_manifest["cases"]
    positions = {case["id"]: index for index, case in enumerate(cases)}
    expected_schedule: list[tuple[str, int, str, str, int]] = []
    for kind, rounds in (
        ("warmup", frozen_manifest["measurement"]["warmups"]),
        ("sample", frozen_manifest["measurement"]["selection_pairs"]),
    ):
        for round_index in range(rounds):
            for case in rotated(cases, round_index):
                case_index = positions[case["id"]]
                for order_index, arm in enumerate(
                    arm_order(round_index, case_index)
                ):
                    expected_schedule.append(
                        (kind, round_index, case["id"], arm, order_index)
                    )
    actual_records = records[1:-1]
    require(len(actual_records) == len(expected_schedule),
            "selection evidence record count mismatch")
    measurements: dict[str, dict[str, list[int]]] = {
        case["id"]: {"official": [], "native": []} for case in cases
    }
    for record, expected in zip(actual_records, expected_schedule, strict=True):
        kind, round_index, case_id, arm, order_index = expected
        actual_schedule = (
            record.get("kind"),
            record.get("round"),
            record.get("case"),
            record.get("arm"),
            record.get("order"),
        )
        require(actual_schedule == expected,
                f"selection schedule mismatch: {actual_schedule} != {expected}")
        require(isinstance(record.get("elapsed_ns"), int)
                and record["elapsed_ns"] > 0,
                "selection elapsed value is invalid")
        expected_equal(
            record.get("result"),
            frozen_manifest["oracles"]["cases"][case_id],
            f"selection evidence {case_id} {arm}",
        )
        if kind == "sample":
            measurements[case_id][arm].append(record["elapsed_ns"])

    summaries, inconclusive = selection_summaries(
        frozen_manifest, measurements
    )
    require(summary_record.get("cases") == summaries,
            "selection summary does not recompute")
    require(summary_record.get("inconclusive") == inconclusive,
            "selection precision result does not recompute")
    require(not inconclusive,
            "inconclusive selection cannot authorize a baseline")
    selected = {
        case_id: summary["selected"] for case_id, summary in summaries.items()
    }
    require(evidence.get("selected") == selected,
            "manifest selection map does not match raw evidence")
    require(
        {
            case["id"]: case.get("selected_comparator")
            for case in manifest["cases"]
        }
        == selected,
        "selected comparator fields do not match raw evidence",
    )
    return summaries


def verify_baseline_evidence(
    manifest: dict[str, Any], manifest_path: Path
) -> dict[str, Any]:
    require(manifest["phase"] == "complete",
            "baseline evidence is only valid in complete phase")
    evidence = manifest.get("baseline_evidence")
    require(isinstance(evidence, dict), "manifest has no baseline evidence")
    evidence_path = (HERE / evidence["path"]).resolve()
    try:
        evidence_path.relative_to(HERE)
    except ValueError as error:
        raise ProtocolError("baseline evidence path escapes bundle") from error
    require(sha256_file(evidence_path) == evidence["sha256"],
            "baseline evidence SHA-256 mismatch")

    root = Path(git_text(HERE, "rev-parse", "--show-toplevel"))
    require_tracked_file(root, evidence_path)
    relative_manifest = manifest_path.resolve().relative_to(root.resolve())
    selection_commit = evidence["selection_commit"]
    capture(["git", "-C", str(root), "merge-base", "--is-ancestor",
             selection_commit, "HEAD"])
    selected_bytes = capture(
        [
            "git",
            "-C",
            str(root),
            "show",
            f"{selection_commit}:{relative_manifest.as_posix()}",
        ]
    )
    require(sha256_bytes(selected_bytes) == evidence["input_manifest_sha256"],
            "selected manifest digest does not match baseline evidence")
    try:
        selected_manifest = json.loads(selected_bytes)
    except json.JSONDecodeError as error:
        raise ProtocolError("selected manifest is not valid JSON") from error
    require(selected_manifest["phase"] == "selected-before-baseline",
            "baseline input commit is not a selected manifest")
    require(normalized_selected_manifest(manifest) == selected_manifest,
            "complete manifest changed more than baseline evidence permits")

    records = load_jsonl(evidence_path)
    header = records[0]
    summary_record = records[-1]
    require(header.get("kind") == "header"
            and header.get("phase") == "baseline",
            "baseline evidence has no valid header")
    require(header.get("run_id") == evidence["run_id"]
            == selected_manifest["measurement"]["baseline_run_id"],
            "baseline evidence run id mismatch")
    require(header.get("manifest_sha256") == evidence["input_manifest_sha256"],
            "baseline header manifest digest mismatch")
    require(header.get("repository", {}).get("head") == selection_commit,
            "baseline header selection commit mismatch")
    validate_power_evidence(header.get("power"))
    require(summary_record.get("kind") == "summary"
            and summary_record.get("phase") == "baseline",
            "baseline evidence has no valid summary")
    validate_power_evidence(summary_record.get("power"))

    cases = selected_manifest["cases"]
    expected_schedule: list[tuple[str, int, str, str]] = []
    for kind, rounds in (
        ("warmup", selected_manifest["measurement"]["warmups"]),
        ("sample", selected_manifest["measurement"]["baseline_repetitions"]),
    ):
        for round_index in range(rounds):
            for case in rotated(cases, round_index):
                expected_schedule.append(
                    (
                        kind,
                        round_index,
                        case["id"],
                        case["selected_comparator"],
                    )
                )
    actual_records = records[1:-1]
    require(len(actual_records) == len(expected_schedule),
            "baseline evidence record count mismatch")
    measurements: dict[str, list[int]] = {
        case["id"]: [] for case in cases
    }
    for record, expected in zip(actual_records, expected_schedule, strict=True):
        kind, round_index, case_id, arm = expected
        actual_schedule = (
            record.get("kind"),
            record.get("round"),
            record.get("case"),
            record.get("arm"),
        )
        require(actual_schedule == expected,
                f"baseline schedule mismatch: {actual_schedule} != {expected}")
        require(isinstance(record.get("elapsed_ns"), int)
                and record["elapsed_ns"] > 0,
                "baseline elapsed value is invalid")
        expected_equal(
            record.get("result"),
            selected_manifest["oracles"]["cases"][case_id],
            f"baseline evidence {case_id} {arm}",
        )
        if kind == "sample":
            measurements[case_id].append(record["elapsed_ns"])

    summaries, inconclusive = baseline_summaries(
        selected_manifest, measurements
    )
    require(summary_record.get("cases") == summaries,
            "baseline summary does not recompute")
    require(summary_record.get("inconclusive") == inconclusive,
            "baseline precision result does not recompute")
    require(not inconclusive,
            "inconclusive baseline cannot complete RG-BASE")
    return summaries


def open_run_file(work_root: Path, run_id: str, phase: str):
    require(RUN_ID_RE.fullmatch(run_id) is not None, "invalid run id")
    run_root = resolve_work_path(work_root, f"work/runs/{run_id}")
    try:
        run_root.mkdir(parents=True, exist_ok=False)
    except FileExistsError as error:
        raise ProtocolError(f"run id already exists: {run_id}") from error
    output = run_root / f"{phase}.jsonl"
    return output, output.open("x", encoding="utf-8")


def write_jsonl(stream, record: dict[str, Any]) -> None:
    stream.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
    stream.write("\n")
    stream.flush()


def checked_timed_case(
    manifest: dict[str, Any],
    work_root: Path,
    arm: str,
    case: dict[str, Any],
) -> tuple[int, dict[str, Any]]:
    elapsed, result = run_case(manifest, work_root, arm, case)
    expected_equal(
        result, manifest["oracles"]["cases"][case["id"]],
        f"timed {case['id']} {arm}"
    )
    return elapsed, result


def timing_prelude(
    manifest: dict[str, Any], work_root: Path, manifest_path: Path
) -> dict[str, Any]:
    repository = repository_facts(manifest_path)
    validate_inventory(manifest, work_root)
    verify_oracles(manifest, work_root)
    return {
        "manifest_sha256": manifest_sha256(manifest_path),
        "repository": repository,
        "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "power": power_snapshot(),
    }


def run_selection(
    manifest: dict[str, Any],
    work_root: Path,
    manifest_path: Path,
    run_id: str,
) -> Path:
    require(manifest["phase"] == "frozen-before-selection",
            "selection requires frozen-before-selection phase")
    require(run_id == manifest["measurement"]["selection_run_id"],
            "selection run id does not match the frozen first attempt")
    context = timing_prelude(manifest, work_root, manifest_path)
    output_path, stream = open_run_file(work_root, run_id, "selection")
    cases = manifest["cases"]
    warmups = manifest["measurement"]["warmups"]
    repetitions = manifest["measurement"]["selection_pairs"]
    measurements: dict[str, dict[str, list[int]]] = {
        case["id"]: {"official": [], "native": []} for case in cases
    }
    case_positions = {case["id"]: index for index, case in enumerate(cases)}
    try:
        write_jsonl(stream, {
            "kind": "header",
            "phase": "selection",
            "run_id": run_id,
            **context,
        })
        for round_index in range(warmups):
            for case in rotated(cases, round_index):
                case_index = case_positions[case["id"]]
                for order_index, arm in enumerate(arm_order(round_index, case_index)):
                    elapsed, result = checked_timed_case(
                        manifest, work_root, arm, case
                    )
                    write_jsonl(stream, {
                        "kind": "warmup",
                        "round": round_index,
                        "case": case["id"],
                        "arm": arm,
                        "order": order_index,
                        "elapsed_ns": elapsed,
                        "result": result,
                    })
        for round_index in range(repetitions):
            for case in rotated(cases, round_index):
                case_index = case_positions[case["id"]]
                for order_index, arm in enumerate(arm_order(round_index, case_index)):
                    elapsed, result = checked_timed_case(
                        manifest, work_root, arm, case
                    )
                    measurements[case["id"]][arm].append(elapsed)
                    write_jsonl(stream, {
                        "kind": "sample",
                        "round": round_index,
                        "case": case["id"],
                        "arm": arm,
                        "order": order_index,
                        "elapsed_ns": elapsed,
                        "result": result,
                    })

        summaries, inconclusive = selection_summaries(
            manifest, measurements
        )
        write_jsonl(stream, {
            "kind": "summary",
            "phase": "selection",
            "finished_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "power": power_snapshot(),
            "cases": summaries,
            "inconclusive": inconclusive,
        })
    finally:
        stream.close()
    if inconclusive:
        raise ProtocolError(
            f"selection precision gate failed for: {', '.join(inconclusive)}; "
            f"raw evidence: {output_path}"
        )
    return output_path


def run_baseline(
    manifest: dict[str, Any],
    work_root: Path,
    manifest_path: Path,
    run_id: str,
) -> Path:
    require(manifest["phase"] == "selected-before-baseline",
            "baseline requires selected-before-baseline phase")
    require(run_id == manifest["measurement"]["baseline_run_id"],
            "baseline run id does not match the frozen first attempt")
    for case in manifest["cases"]:
        require(case.get("selected_comparator") in ("official", "native"),
                f"case {case['id']} has no selected comparator")
    repository_facts(manifest_path)
    verify_selection_evidence(manifest, manifest_path)
    context = timing_prelude(manifest, work_root, manifest_path)
    output_path, stream = open_run_file(work_root, run_id, "baseline")
    cases = manifest["cases"]
    warmups = manifest["measurement"]["warmups"]
    repetitions = manifest["measurement"]["baseline_repetitions"]
    measurements: dict[str, list[int]] = {case["id"]: [] for case in cases}
    try:
        write_jsonl(stream, {
            "kind": "header",
            "phase": "baseline",
            "run_id": run_id,
            **context,
        })
        for round_index in range(warmups):
            for case in rotated(cases, round_index):
                arm = case["selected_comparator"]
                elapsed, result = checked_timed_case(
                    manifest, work_root, arm, case
                )
                write_jsonl(stream, {
                    "kind": "warmup",
                    "round": round_index,
                    "case": case["id"],
                    "arm": arm,
                    "elapsed_ns": elapsed,
                    "result": result,
                })
        for round_index in range(repetitions):
            for case in rotated(cases, round_index):
                arm = case["selected_comparator"]
                elapsed, result = checked_timed_case(
                    manifest, work_root, arm, case
                )
                measurements[case["id"]].append(elapsed)
                write_jsonl(stream, {
                    "kind": "sample",
                    "round": round_index,
                    "case": case["id"],
                    "arm": arm,
                    "elapsed_ns": elapsed,
                    "result": result,
                })
        summaries, inconclusive = baseline_summaries(
            manifest, measurements
        )
        write_jsonl(stream, {
            "kind": "summary",
            "phase": "baseline",
            "finished_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "power": power_snapshot(),
            "cases": summaries,
            "inconclusive": inconclusive,
        })
    finally:
        stream.close()
    if inconclusive:
        raise ProtocolError(
            f"baseline precision gate failed for: {', '.join(inconclusive)}; "
            f"raw evidence: {output_path}"
        )
    return output_path


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    subparsers = parser.add_subparsers(dest="command", required=True)
    for name in ("inventory", "freeze-oracles", "verify"):
        command = subparsers.add_parser(name)
        command.add_argument("--work-root", type=Path, required=True)
    for name in ("select", "baseline"):
        command = subparsers.add_parser(name)
        command.add_argument("--work-root", type=Path, required=True)
        command.add_argument("--run-id", required=True)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    arguments = parse_args(sys.argv[1:] if argv is None else argv)
    manifest_path = arguments.manifest.resolve()
    try:
        manifest = load_manifest(manifest_path)
        work_root = arguments.work_root.resolve()
        if arguments.command == "inventory":
            print(json.dumps(actual_inventory(manifest, work_root),
                             indent=2, sort_keys=True))
        elif arguments.command == "freeze-oracles":
            validate_inventory(manifest, work_root)
            print(json.dumps(compute_oracles(manifest, work_root),
                             indent=2, sort_keys=True))
        elif arguments.command == "verify":
            validate_inventory(manifest, work_root)
            verify_oracles(manifest, work_root)
            if manifest["phase"] in ("selected-before-baseline", "complete"):
                verify_selection_evidence(manifest, manifest_path)
            if manifest["phase"] == "complete":
                verify_baseline_evidence(manifest, manifest_path)
            print("RG-BASE manifest, inputs, and correctness oracles verified")
        elif arguments.command == "select":
            path = run_selection(
                manifest, work_root, manifest_path, arguments.run_id
            )
            print(path)
        elif arguments.command == "baseline":
            path = run_baseline(
                manifest, work_root, manifest_path, arguments.run_id
            )
            print(path)
        else:
            raise AssertionError(arguments.command)
    except ProtocolError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
