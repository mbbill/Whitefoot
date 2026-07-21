from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import stat
from typing import Any


PROPOSAL_ROOT = Path(__file__).resolve().parent
GRAMMAR_ROOT = PROPOSAL_ROOT.parent
REPOSITORY = GRAMMAR_ROOT.parent

SPECIFICATION_PATH = "spec/kernel-spec-v0.8.md"
GUARD_BASELINE_PATH = "governance/guard-baseline.json"
MANIFEST_PATH = "conformance/manifest.jsonl"
CASE_DIRECTORY = "conformance/cases"
LEGACY_CASE_PATH = "conformance/cases/pending-const2-item.wf"
LEGACY_CASE_SHA256 = "ae99d9b9b99e02e9c6c5f2af54f0924b7b1a0f5ee0422d29958b01b597adf759"

MAX_SOURCE_COUNT = 1_000
MAX_SOURCE_BYTES = 1 << 20
MAX_AGGREGATE_SOURCE_BYTES = 32 << 20
MAX_MANIFEST_BYTES = 4 << 20
MAX_BASELINE_BYTES = 4 << 20


class InputError(RuntimeError):
    pass


@dataclass(frozen=True)
class ProtectedSource:
    identifier: str | None
    path: str
    raw: bytes
    sha256: str
    manifest: dict[str, Any] | None


@dataclass(frozen=True)
class ProtectedInputs:
    baseline_raw: bytes
    baseline_sha256: str
    manifest_raw: bytes
    manifest_sha256: str
    specification_raw: bytes
    specification_sha256: str
    form2_rule: str
    form2_rule_sha256: str
    sources: tuple[ProtectedSource, ...]


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(
            value,
            ensure_ascii=True,
            separators=(",", ":"),
            sort_keys=True,
        )
        + "\n"
    ).encode("ascii")


def _guard_case_digest(entry: dict[str, Any], raw: bytes) -> str:
    guarded = {field: entry.get(field) for field in ("id", "rules", "expect", "status")}
    encoded = json.dumps(
        guarded,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return sha256(encoded + b"\0" + raw)


def _read_bounded(relative: str, limit: int) -> bytes:
    path = REPOSITORY / relative
    try:
        path_before = path.lstat()
    except OSError as error:
        raise InputError(f"cannot stat required input {relative}: {error}") from error
    if stat.S_ISLNK(path_before.st_mode) or not stat.S_ISREG(path_before.st_mode):
        raise InputError(f"required input is not a regular non-symlink file: {relative}")
    if path_before.st_size < 0 or path_before.st_size > limit:
        raise InputError(f"required input exceeds byte limit: {relative}")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = -1
    try:
        descriptor = os.open(path, flags)
        before = os.fstat(descriptor)
        remaining = before.st_size
        chunks: list[bytes] = []
        while remaining:
            chunk = os.read(descriptor, min(65_536, remaining))
            if not chunk:
                raise InputError(f"required input changed while reading: {relative}")
            chunks.append(chunk)
            remaining -= len(chunk)
        if os.read(descriptor, 1):
            raise InputError(f"required input grew while reading: {relative}")
        after = os.fstat(descriptor)
    except OSError as error:
        raise InputError(f"cannot read required input {relative}: {error}") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    identity = lambda value: (
        value.st_dev,
        value.st_ino,
        value.st_mode,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )
    if identity(path_before) != identity(before) or identity(before) != identity(after):
        raise InputError(f"required input changed while reading: {relative}")
    return b"".join(chunks)


def _json_object(raw: bytes, description: str) -> dict[str, Any]:
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InputError(f"malformed {description}: {error}") from error
    if not isinstance(value, dict):
        raise InputError(f"{description} must be a JSON object")
    return value


def _load_manifest(raw: bytes) -> dict[str, dict[str, Any]]:
    entries: dict[str, dict[str, Any]] = {}
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise InputError(f"manifest is not UTF-8: {error}") from error
    for line_number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        try:
            entry = json.loads(stripped)
        except json.JSONDecodeError as error:
            raise InputError(f"manifest line {line_number} is malformed: {error}") from error
        if not isinstance(entry, dict):
            raise InputError(f"manifest line {line_number} is not an object")
        if "id" not in entry:
            if not isinstance(entry.get("rule"), str):
                raise InputError(
                    f"manifest line {line_number} lacks a case id or rule annotation"
                )
            continue
        if not isinstance(entry.get("id"), str):
            raise InputError(f"manifest line {line_number} has a non-string id")
        identifier = entry["id"]
        if identifier in entries:
            raise InputError(f"duplicate manifest id: {identifier}")
        if entry.get("status") not in {"runnable", "pending", "xfail"}:
            raise InputError(f"manifest entry has unknown status: {identifier}")
        if not isinstance(entry.get("expect"), dict):
            raise InputError(f"manifest entry lacks an expected verdict: {identifier}")
        entries[identifier] = entry
    return entries


def _extract_form2_rule(specification: bytes) -> tuple[str, str]:
    prefix = b"[FORM-2] "
    matches = [line for line in specification.splitlines() if line.startswith(prefix)]
    if len(matches) != 1:
        raise InputError("exactly one FORM-2 rule line is required")
    try:
        text = matches[0].decode("utf-8")
    except UnicodeDecodeError as error:
        raise InputError(f"FORM-2 rule is not UTF-8: {error}") from error
    required_fragments = (
        "UTF-8; LF endings; file ends with exactly one LF",
        "indentation exactly two spaces per `{` nesting level",
        "exactly one space between adjacent tokens except:",
        "no space after `(` `<` `&`",
        "no space around `.` in places",
        "no line wrapping (a statement is one line)",
        "declarations separated by exactly one blank line",
        "no trailing whitespace",
    )
    missing = [fragment for fragment in required_fragments if fragment not in text]
    if missing:
        raise InputError(f"FORM-2 rule shape changed: missing {missing!r}")
    return text, sha256(matches[0])


def load_protected_inputs() -> ProtectedInputs:
    baseline_raw = _read_bounded(GUARD_BASELINE_PATH, MAX_BASELINE_BYTES)
    manifest_raw = _read_bounded(MANIFEST_PATH, MAX_MANIFEST_BYTES)
    specification_raw = _read_bounded(SPECIFICATION_PATH, MAX_SOURCE_BYTES)
    baseline = _json_object(baseline_raw, "guard baseline")
    protected = baseline.get("conformance")
    if not isinstance(protected, dict):
        raise InputError("guard baseline lacks conformance records")
    manifest = _load_manifest(manifest_raw)

    case_ids = sorted(
        (
            identifier
            for identifier in protected
            if (REPOSITORY / CASE_DIRECTORY / f"{identifier}.wf").is_file()
        ),
        key=str.encode,
    )
    if len(case_ids) + 1 > MAX_SOURCE_COUNT:
        raise InputError("protected source count exceeds limit")
    if set(case_ids) != set(manifest):
        missing = sorted(set(case_ids) - set(manifest), key=str.encode)
        extra = sorted(set(manifest) - set(case_ids), key=str.encode)
        raise InputError(
            f"protected baseline/manifest mismatch: missing={missing!r}, extra={extra!r}"
        )

    sources: list[ProtectedSource] = []
    aggregate = 0
    for identifier in case_ids:
        relative = f"{CASE_DIRECTORY}/{identifier}.wf"
        raw = _read_bounded(relative, MAX_SOURCE_BYTES)
        aggregate += len(raw)
        expected = protected[identifier]
        actual = sha256(raw)
        guarded_actual = _guard_case_digest(manifest[identifier], raw)
        if not isinstance(expected, str) or guarded_actual != expected:
            raise InputError(
                f"protected guard record mismatch for {relative}: {guarded_actual} != {expected}"
            )
        sources.append(
            ProtectedSource(identifier, relative, raw, actual, manifest[identifier])
        )

    legacy = _read_bounded(LEGACY_CASE_PATH, MAX_SOURCE_BYTES)
    aggregate += len(legacy)
    legacy_sha = sha256(legacy)
    if legacy_sha != LEGACY_CASE_SHA256:
        raise InputError(
            f"legacy protected source hash mismatch: {legacy_sha} != {LEGACY_CASE_SHA256}"
        )
    if aggregate > MAX_AGGREGATE_SOURCE_BYTES:
        raise InputError("protected source bytes exceed aggregate limit")
    sources.append(
        ProtectedSource(None, LEGACY_CASE_PATH, legacy, legacy_sha, None)
    )
    sources.sort(key=lambda source: source.path.encode("utf-8"))

    discovered = sorted(
        (
            path.relative_to(REPOSITORY).as_posix()
            for path in (REPOSITORY / CASE_DIRECTORY).glob("*.wf")
        ),
        key=str.encode,
    )
    expected_paths = [source.path for source in sources]
    if discovered != expected_paths:
        raise InputError("protected case-directory inventory changed")

    form2_rule, form2_rule_sha = _extract_form2_rule(specification_raw)
    return ProtectedInputs(
        baseline_raw=baseline_raw,
        baseline_sha256=sha256(baseline_raw),
        manifest_raw=manifest_raw,
        manifest_sha256=sha256(manifest_raw),
        specification_raw=specification_raw,
        specification_sha256=sha256(specification_raw),
        form2_rule=form2_rule,
        form2_rule_sha256=form2_rule_sha,
        sources=tuple(sources),
    )
