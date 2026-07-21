from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import stat
from typing import Any


MODULE_ROOT = Path(__file__).resolve().parent
VERIFIER_ROOT = MODULE_ROOT.parent
REPOSITORY_ROOT = VERIFIER_ROOT.parent

BASELINE = "governance/guard-baseline.json"
MANIFEST = "conformance/manifest.jsonl"
CASE_ROOT = "conformance/cases"
CANDIDATE = "grammar-verifier/proposal/kernel-spec-successor-candidate.md"
LEGACY_CASE = "conformance/cases/pending-const2-item.wf"
LEGACY_SHA256 = "ae99d9b9b99e02e9c6c5f2af54f0924b7b1a0f5ee0422d29958b01b597adf759"

MAX_FILE_BYTES = 1 << 20
MAX_METADATA_BYTES = 4 << 20
MAX_TOTAL_SOURCE_BYTES = 32 << 20
MAX_SOURCES = 1_000


class IndependentInputError(RuntimeError):
    pass


@dataclass(frozen=True)
class IndependentSource:
    identifier: str | None
    path: str
    raw: bytes
    manifest: dict[str, Any] | None

    @property
    def sha256(self) -> str:
        return digest(self.raw)


@dataclass(frozen=True)
class IndependentInputs:
    baseline_sha256: str
    candidate: bytes
    candidate_sha256: str
    manifest_sha256: str
    sources: tuple[IndependentSource, ...]


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_json(value: Any) -> bytes:
    text = json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
    return (text + "\n").encode("ascii")


def _read_regular(relative: str, limit: int) -> bytes:
    path = REPOSITORY_ROOT / relative
    try:
        first = path.lstat()
    except OSError as error:
        raise IndependentInputError(f"cannot inspect {relative}: {error}") from error
    if stat.S_ISLNK(first.st_mode) or not stat.S_ISREG(first.st_mode):
        raise IndependentInputError(f"input is not a regular non-symlink file: {relative}")
    if first.st_size < 0 or first.st_size > limit:
        raise IndependentInputError(f"input exceeds its byte limit: {relative}")

    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = -1
    try:
        descriptor = os.open(path, flags)
        opened = os.fstat(descriptor)
        chunks: list[bytes] = []
        remaining = opened.st_size
        while remaining:
            chunk = os.read(descriptor, min(remaining, 65_536))
            if not chunk:
                raise IndependentInputError(f"input shortened while reading: {relative}")
            chunks.append(chunk)
            remaining -= len(chunk)
        if os.read(descriptor, 1):
            raise IndependentInputError(f"input grew while reading: {relative}")
        final = os.fstat(descriptor)
    except OSError as error:
        raise IndependentInputError(f"cannot read {relative}: {error}") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)

    def identity(value: os.stat_result) -> tuple[int, int, int, int, int, int]:
        return (
            value.st_dev,
            value.st_ino,
            value.st_mode,
            value.st_size,
            value.st_mtime_ns,
            value.st_ctime_ns,
        )

    if identity(first) != identity(opened) or identity(opened) != identity(final):
        raise IndependentInputError(f"input identity changed while reading: {relative}")
    return b"".join(chunks)


def _load_manifest(raw: bytes) -> dict[str, dict[str, Any]]:
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise IndependentInputError("manifest is not UTF-8") from error
    result: dict[str, dict[str, Any]] = {}
    for number, line in enumerate(text.splitlines(), 1):
        if not line or line.startswith("#"):
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise IndependentInputError(f"manifest line {number} is malformed") from error
        if not isinstance(value, dict):
            raise IndependentInputError(f"manifest line {number} is not an object")
        identifier = value.get("id")
        if identifier is None:
            if not isinstance(value.get("rule"), str):
                raise IndependentInputError(f"manifest line {number} has no case or rule")
            continue
        if not isinstance(identifier, str) or identifier in result:
            raise IndependentInputError(f"manifest case id is invalid: {identifier!r}")
        if value.get("status") not in {"runnable", "pending", "xfail"}:
            raise IndependentInputError(f"manifest status is invalid: {identifier}")
        if not isinstance(value.get("expect"), dict) or not isinstance(value.get("rules"), list):
            raise IndependentInputError(f"manifest expectation is invalid: {identifier}")
        result[identifier] = value
    return result


def _guard_digest(entry: dict[str, Any], raw: bytes) -> str:
    projection = {name: entry.get(name) for name in ("id", "rules", "expect", "status")}
    encoded = json.dumps(
        projection,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return digest(encoded + b"\0" + raw)


def load_independent_inputs() -> IndependentInputs:
    baseline_raw = _read_regular(BASELINE, MAX_METADATA_BYTES)
    manifest_raw = _read_regular(MANIFEST, MAX_METADATA_BYTES)
    candidate = _read_regular(CANDIDATE, MAX_FILE_BYTES)
    try:
        baseline = json.loads(baseline_raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise IndependentInputError("guard baseline is malformed") from error
    if not isinstance(baseline, dict) or not isinstance(baseline.get("conformance"), dict):
        raise IndependentInputError("guard baseline has no conformance map")
    guarded: dict[str, Any] = baseline["conformance"]
    manifest = _load_manifest(manifest_raw)

    identifiers = sorted(
        (
            identifier
            for identifier in guarded
            if (REPOSITORY_ROOT / CASE_ROOT / f"{identifier}.wf").is_file()
        ),
        key=str.encode,
    )
    if set(identifiers) != set(manifest):
        raise IndependentInputError("guarded source ids and manifest ids disagree")
    if len(identifiers) + 1 > MAX_SOURCES:
        raise IndependentInputError("protected source count exceeds the independent limit")

    sources: list[IndependentSource] = []
    total = 0
    for identifier in identifiers:
        relative = f"{CASE_ROOT}/{identifier}.wf"
        raw = _read_regular(relative, MAX_FILE_BYTES)
        total += len(raw)
        if guarded[identifier] != _guard_digest(manifest[identifier], raw):
            raise IndependentInputError(f"guard digest mismatch: {relative}")
        sources.append(IndependentSource(identifier, relative, raw, manifest[identifier]))

    legacy = _read_regular(LEGACY_CASE, MAX_FILE_BYTES)
    total += len(legacy)
    if digest(legacy) != LEGACY_SHA256:
        raise IndependentInputError("legacy protected source digest mismatch")
    if total > MAX_TOTAL_SOURCE_BYTES:
        raise IndependentInputError("protected source aggregate exceeds the independent limit")
    sources.append(IndependentSource(None, LEGACY_CASE, legacy, None))
    sources.sort(key=lambda source: source.path.encode("utf-8"))

    discovered = sorted(
        path.relative_to(REPOSITORY_ROOT).as_posix()
        for path in (REPOSITORY_ROOT / CASE_ROOT).rglob("*.wf")
    )
    if discovered != [source.path for source in sources]:
        raise IndependentInputError("protected source directory inventory is not closed")

    return IndependentInputs(
        baseline_sha256=digest(baseline_raw),
        candidate=candidate,
        candidate_sha256=digest(candidate),
        manifest_sha256=digest(manifest_raw),
        sources=tuple(sources),
    )
