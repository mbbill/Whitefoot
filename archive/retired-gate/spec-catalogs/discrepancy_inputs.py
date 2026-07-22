#!/usr/bin/env python3
"""Load exact v0.9 discrepancy authorities through bounded real paths."""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence


ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = ROOT / "spec" / "kernel-spec-v0.9.md"
SOURCE_INDEX_PATH = ROOT / "tests" / "spec-catalogs" / "v0.9" / "source.json"
GUARD_BASELINE_PATH = ROOT / "governance" / "guard-baseline.json"
MANIFEST_PATH = ROOT / "tests" / "conformance" / "manifest.jsonl"
CASE_ROOT = ROOT / "tests" / "conformance" / "cases"

SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
SOURCE_INDEX_SHA256 = "cc9aa86de0d59b9288d1f8fd7a6bde6f94fff26da139d73f91bcbcf71219d663"
GUARD_BASELINE_SHA256 = "bb7ce5ea5b3b2a169b259bcffc7add3234e89b50aa689d5f9df5a93a91325622"
LEGACY_ORPHAN_PATH = "tests/conformance/cases/pending-const2-item.wf"
LEGACY_ORPHAN_SHA256 = "5cec5d4e386df7383137fecdca7d4446274785f758fc563aa54ffb72e92cfe64"

MAX_JSON_DEPTH = 128
MAX_JSON_INTEGER_DIGITS = 20
MAX_JSON_STRING_BYTES = 16_384
MAX_JSON_LIST_ITEMS = 8_192
MAX_JSON_OBJECT_FIELDS = 1_024
MAX_JSON_NODES = 100_000
MAX_SIDECAR_BYTES = 1_000_000
MAX_CATALOG_BYTES = 16_000_000
MAX_SPECIFICATION_BYTES = 1_000_000
MAX_SOURCE_INDEX_BYTES = 8_000_000
MAX_GUARD_BASELINE_BYTES = 8_000_000
MAX_MANIFEST_BYTES = 8_000_000
MAX_PROTECTED_CASE_FILES = 10_000
MAX_CASE_BYTES = 16_000_000
MAX_CASE_TOTAL_BYTES = 512_000_000
MAX_PATH_BYTES = 1_024

HEX_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
CASE_ID = re.compile(r"[a-z0-9-]+\Z")


class DiscrepancyError(ValueError):
    """A discrepancy document or one of its bound inputs is invalid."""


def sha256(data: bytes) -> str:
    """Return the lowercase SHA-256 identity of exact bytes."""
    return hashlib.sha256(data).hexdigest()


def canonical_bytes(value: Any) -> bytes:
    """Serialize deterministic ASCII JSON with one terminal LF."""
    try:
        encoded = json.dumps(
            value,
            allow_nan=False,
            ensure_ascii=True,
            indent=2,
            sort_keys=True,
        )
        return (encoded + "\n").encode("ascii")
    except (MemoryError, OverflowError, RecursionError, TypeError, ValueError) as error:
        raise DiscrepancyError(f"value is not bounded canonical JSON: {error}") from error


def _reject_constant(value: str) -> None:
    raise DiscrepancyError(f"non-finite JSON number is forbidden: {value}")


def _reject_float(value: str) -> None:
    raise DiscrepancyError(f"JSON floats are forbidden: {value}")


def _unique_object(
    pairs: Sequence[tuple[str, Any]],
    max_object_fields: int,
) -> dict[str, Any]:
    if len(pairs) > max_object_fields:
        raise DiscrepancyError(
            f"JSON object exceeds the {max_object_fields}-field limit"
        )
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DiscrepancyError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def check_json_envelope(raw: bytes, max_bytes: int, label: str) -> None:
    """Reject mutable, empty, oversized, or excessively nested JSON bytes."""
    if not isinstance(raw, bytes):
        raise DiscrepancyError(f"{label} must be exact immutable bytes")
    if not raw:
        raise DiscrepancyError(f"{label} is empty")
    if len(raw) > max_bytes:
        raise DiscrepancyError(
            f"{label} is {len(raw)} bytes, exceeding the {max_bytes}-byte limit"
        )
    depth = 0
    in_string = False
    escaped = False
    for byte in raw:
        if in_string:
            if escaped:
                escaped = False
            elif byte == 0x5C:
                escaped = True
            elif byte == 0x22:
                in_string = False
            continue
        if byte == 0x22:
            in_string = True
        elif byte in (0x5B, 0x7B):
            depth += 1
            if depth > MAX_JSON_DEPTH:
                raise DiscrepancyError(
                    f"{label} exceeds the JSON nesting limit {MAX_JSON_DEPTH}"
                )
        elif byte in (0x5D, 0x7D):
            depth -= 1
            if depth < 0:
                break


def strict_json_loads(
    raw: bytes,
    *,
    max_bytes: int = MAX_SIDECAR_BYTES,
    label: str = "JSON input",
    max_integer_digits: int = MAX_JSON_INTEGER_DIGITS,
    max_string_bytes: int = MAX_JSON_STRING_BYTES,
    max_list_items: int = MAX_JSON_LIST_ITEMS,
    max_object_fields: int = MAX_JSON_OBJECT_FIELDS,
    max_nodes: int = MAX_JSON_NODES,
) -> Any:
    """Decode bounded UTF-8 JSON with closed object and numeric encodings."""
    limits = {
        "integer-digit": max_integer_digits,
        "string-byte": max_string_bytes,
        "list-item": max_list_items,
        "object-field": max_object_fields,
        "node": max_nodes,
    }
    for name, limit in limits.items():
        if isinstance(limit, bool) or not isinstance(limit, int) or limit < 1:
            raise DiscrepancyError(f"{label} {name} limit must be a positive integer")
    check_json_envelope(raw, max_bytes, label)
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise DiscrepancyError(f"{label} is not UTF-8: {error}") from error

    def bounded_integer(value: str) -> int:
        digits = value[1:] if value.startswith("-") else value
        if len(digits) > max_integer_digits:
            raise DiscrepancyError(
                f"JSON integer exceeds the {max_integer_digits}-digit limit"
            )
        try:
            return int(value)
        except (OverflowError, ValueError) as error:
            raise DiscrepancyError(f"invalid JSON integer: {error}") from error

    def closed_object(pairs: Sequence[tuple[str, Any]]) -> dict[str, Any]:
        return _unique_object(pairs, max_object_fields)

    try:
        value = json.loads(
            text,
            object_pairs_hook=closed_object,
            parse_int=bounded_integer,
            parse_float=_reject_float,
            parse_constant=_reject_constant,
        )
    except DiscrepancyError:
        raise
    except (MemoryError, OverflowError, RecursionError, ValueError) as error:
        raise DiscrepancyError(f"invalid {label}: {error}") from error

    stack = [value]
    nodes = 0
    while stack:
        item = stack.pop()
        nodes += 1
        if nodes > max_nodes:
            raise DiscrepancyError(
                f"{label} exceeds the {max_nodes}-node JSON limit"
            )
        if isinstance(item, str):
            try:
                encoded = item.encode("utf-8")
            except UnicodeEncodeError as error:
                raise DiscrepancyError(
                    f"{label} contains a string that is not valid UTF-8"
                ) from error
            if len(encoded) > max_string_bytes:
                raise DiscrepancyError(
                    f"{label} contains a string over {max_string_bytes} bytes"
                )
        elif isinstance(item, list):
            if len(item) > max_list_items:
                raise DiscrepancyError(
                    f"{label} contains a list over {max_list_items} items"
                )
            stack.extend(item)
        elif isinstance(item, dict):
            # Keys count as JSON nodes and are checked by the same string budget.
            stack.extend(item.keys())
            stack.extend(item.values())
    return value


def _absolute_inside(root: Path, path: Path, label: str) -> tuple[Path, Path]:
    root_absolute = Path(os.path.abspath(root))
    path_absolute = Path(os.path.abspath(path))
    try:
        relative = path_absolute.relative_to(root_absolute)
    except ValueError as error:
        raise DiscrepancyError(f"{label} escapes repository root: {path}") from error
    if len(relative.as_posix().encode("utf-8")) > MAX_PATH_BYTES:
        raise DiscrepancyError(f"{label} path exceeds {MAX_PATH_BYTES} bytes")
    return root_absolute, relative


def _descriptor_flags(*, directory: bool) -> int:
    if (
        not hasattr(os, "O_NOFOLLOW")
        or (directory and not hasattr(os, "O_DIRECTORY"))
        or (not directory and not hasattr(os, "O_NONBLOCK"))
    ):
        raise DiscrepancyError(
            "platform cannot provide descriptor-safe no-follow reads"
        )
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | os.O_NOFOLLOW
    if directory:
        flags |= os.O_DIRECTORY
    else:
        # A FIFO or device must not block before fstat can reject it.
        flags |= os.O_NONBLOCK
    return flags


def _inspect_new_descriptor(descriptor: int, label: str) -> os.stat_result:
    """Inspect a newly opened descriptor, closing it if inspection fails."""
    try:
        return os.fstat(descriptor)
    except OSError as error:
        try:
            os.close(descriptor)
        except OSError as close_error:
            raise DiscrepancyError(
                f"cannot inspect or close the opened descriptor for {label}"
            ) from close_error
        raise DiscrepancyError(
            f"cannot inspect the opened descriptor for {label}: {error}"
        ) from error


def _open_root(root: Path, label: str) -> int:
    try:
        descriptor = os.open(os.fspath(root), _descriptor_flags(directory=True))
    except OSError as error:
        raise DiscrepancyError(
            f"repository root for {label} is not a readable non-symlink directory"
        ) from error
    if not stat.S_ISDIR(_inspect_new_descriptor(descriptor, label).st_mode):
        os.close(descriptor)
        raise DiscrepancyError(f"repository root for {label} is not a directory")
    return descriptor


def _validate_component(component: str, label: str) -> None:
    if not component or component in {".", ".."} or "/" in component or "\x00" in component:
        raise DiscrepancyError(f"invalid path component for {label}: {component!r}")


def _open_directory_at(parent: int, component: str, label: str) -> int:
    _validate_component(component, label)
    try:
        descriptor = os.open(
            component,
            _descriptor_flags(directory=True),
            dir_fd=parent,
        )
    except OSError as error:
        raise DiscrepancyError(
            f"path component for {label} is not a readable non-symlink directory"
        ) from error
    if not stat.S_ISDIR(_inspect_new_descriptor(descriptor, label).st_mode):
        os.close(descriptor)
        raise DiscrepancyError(f"path component for {label} is not a directory")
    return descriptor


def _open_beneath(
    root: Path,
    path: Path,
    label: str,
    *,
    directory: bool | None,
) -> tuple[int, Path]:
    """Open one path through pinned no-follow directory descriptors."""
    root_absolute, relative = _absolute_inside(root, path, label)
    components = relative.parts
    descriptor = _open_root(root_absolute, label)
    try:
        for component in components[:-1]:
            child = _open_directory_at(descriptor, component, label)
            os.close(descriptor)
            descriptor = child
        if not components:
            if directory is False:
                raise DiscrepancyError(f"{label} names the repository directory")
            opened = descriptor
            descriptor = -1
            return opened, root_absolute
        final = components[-1]
        _validate_component(final, label)
        try:
            opened = os.open(
                final,
                _descriptor_flags(directory=directory is True),
                dir_fd=descriptor,
            )
        except OSError as error:
            raise DiscrepancyError(
                f"{label} is not a readable non-symlink path"
            ) from error
        metadata = _inspect_new_descriptor(opened, label)
        try:
            os.close(descriptor)
        except OSError:
            try:
                os.close(opened)
            finally:
                descriptor = -1
            raise
        descriptor = -1
        if directory is True and not stat.S_ISDIR(metadata.st_mode):
            os.close(opened)
            raise DiscrepancyError(f"{label} is not a directory")
        return opened, root_absolute / relative
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def reject_path_indirection(root: Path, path: Path, label: str) -> Path:
    """Validate a path through no-follow descriptors and return its location."""
    try:
        descriptor, checked = _open_beneath(root, path, label, directory=None)
        os.close(descriptor)
        return checked
    except DiscrepancyError:
        raise
    except OSError as error:
        raise DiscrepancyError(
            f"filesystem operation failed for {label}: {error}"
        ) from error


def require_directory(root: Path, path: Path, label: str) -> Path:
    """Validate and return one directory through no-follow descriptors."""
    try:
        descriptor, checked = _open_beneath(root, path, label, directory=True)
        os.close(descriptor)
        return checked
    except DiscrepancyError:
        raise
    except OSError as error:
        raise DiscrepancyError(
            f"filesystem operation failed for {label}: {error}"
        ) from error


def read_regular(root: Path, path: Path, label: str, max_bytes: int) -> bytes:
    """Read one bounded regular file through pinned no-follow descriptors."""
    if isinstance(max_bytes, bool) or not isinstance(max_bytes, int) or max_bytes < 0:
        raise DiscrepancyError(f"invalid byte limit for {label}")
    descriptor = -1
    try:
        descriptor, checked = _open_beneath(root, path, label, directory=False)
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise DiscrepancyError(f"{label} is not a regular file: {checked}")
        if before.st_size > max_bytes:
            raise DiscrepancyError(
                f"{label} is {before.st_size} bytes, exceeding {max_bytes}"
            )
        chunks: list[bytes] = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(65_536, max_bytes + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            if total > max_bytes:
                raise DiscrepancyError(f"{label} grew beyond {max_bytes} bytes")
        after = os.fstat(descriptor)
        identity_before = (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
        )
        identity_after = (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
        )
        if identity_before != identity_after or total != after.st_size:
            raise DiscrepancyError(f"{label} changed while it was read")
        return b"".join(chunks)
    except DiscrepancyError:
        raise
    except OSError as error:
        raise DiscrepancyError(
            f"filesystem operation failed while reading {label}: {error}"
        ) from error
    finally:
        if descriptor >= 0:
            try:
                os.close(descriptor)
            except OSError as error:
                raise DiscrepancyError(
                    f"filesystem operation failed while closing {label}: {error}"
                ) from error


def read_exact(
    root: Path,
    path: Path,
    expected_sha256: str,
    label: str,
    max_bytes: int,
) -> bytes:
    """Read a bounded regular file and require its exact digest."""
    raw = read_regular(root, path, label, max_bytes)
    actual = sha256(raw)
    if actual != expected_sha256:
        raise DiscrepancyError(
            f"{label} hash is {actual}, expected {expected_sha256}"
        )
    return raw


def _protected_case_ids(surface: Any) -> tuple[str, ...]:
    if not isinstance(surface, dict):
        raise DiscrepancyError("guard baseline conformance surface must be an object")
    identifiers = []
    for key, digest in surface.items():
        if not isinstance(key, str):
            raise DiscrepancyError("guard baseline conformance key must be a string")
        if not isinstance(digest, str) or HEX_SHA256.fullmatch(digest) is None:
            raise DiscrepancyError(
                f"guard baseline conformance digest is invalid for {key!r}"
            )
        if CASE_ID.fullmatch(key):
            identifiers.append(key)
    identifiers.sort(key=str.encode)
    if not identifiers or len(identifiers) > MAX_PROTECTED_CASE_FILES:
        raise DiscrepancyError(
            "guard baseline protected-case count is empty or exceeds "
            f"{MAX_PROTECTED_CASE_FILES}"
        )
    return tuple(identifiers)


def _load_protected_cases(
    root: Path,
    identifiers: Sequence[str],
) -> dict[str, bytes]:
    require_directory(root, root / "tests" / "conformance" / "cases", "conformance case root")
    sources: dict[str, bytes] = {}
    total_bytes = 0
    for identifier in identifiers:
        relative = f"tests/conformance/cases/{identifier}.wf"
        raw = read_regular(root, root / relative, "protected conformance case", MAX_CASE_BYTES)
        total_bytes += len(raw)
        if total_bytes > MAX_CASE_TOTAL_BYTES:
            raise DiscrepancyError(
                "protected conformance cases exceed the aggregate byte limit "
                f"{MAX_CASE_TOTAL_BYTES}"
            )
        sources[relative] = raw

    orphan = read_exact(
        root,
        root / LEGACY_ORPHAN_PATH,
        LEGACY_ORPHAN_SHA256,
        "legacy baseline orphan case",
        MAX_CASE_BYTES,
    )
    total_bytes += len(orphan)
    if total_bytes > MAX_CASE_TOTAL_BYTES:
        raise DiscrepancyError(
            "protected conformance cases exceed the aggregate byte limit "
            f"{MAX_CASE_TOTAL_BYTES}"
        )
    sources[LEGACY_ORPHAN_PATH] = orphan
    return sources


@dataclass(frozen=True)
class AuthorityInputs:
    """One bounded snapshot of every discrepancy authority."""

    specification: bytes
    source_index_bytes: bytes
    source_index: dict[str, Any]
    guard_baseline_bytes: bytes
    protected_conformance: dict[str, Any]
    protected_case_ids: tuple[str, ...]
    manifest: bytes
    case_sources: dict[str, bytes]


def load_authorities(root: Path = ROOT) -> AuthorityInputs:
    """Load exact authorities and only baseline-protected case sources."""
    root = require_directory(root, root, "repository root")
    specification = read_exact(
        root,
        root / "spec" / "kernel-spec-v0.9.md",
        SPEC_SHA256,
        "v0.9 specification",
        MAX_SPECIFICATION_BYTES,
    )
    source_index_bytes = read_exact(
        root,
        root / "tests" / "spec-catalogs" / "v0.9" / "source.json",
        SOURCE_INDEX_SHA256,
        "v0.9 source index",
        MAX_SOURCE_INDEX_BYTES,
    )
    guard_baseline_bytes = read_exact(
        root,
        root / "governance" / "guard-baseline.json",
        GUARD_BASELINE_SHA256,
        "guard baseline",
        MAX_GUARD_BASELINE_BYTES,
    )
    manifest = read_regular(
        root,
        root / "tests" / "conformance" / "manifest.jsonl",
        "conformance manifest",
        MAX_MANIFEST_BYTES,
    )
    source_index = strict_json_loads(
        source_index_bytes,
        max_bytes=MAX_SOURCE_INDEX_BYTES,
        label="v0.9 source index",
    )
    baseline = strict_json_loads(
        guard_baseline_bytes,
        max_bytes=MAX_GUARD_BASELINE_BYTES,
        label="guard baseline",
    )
    if not isinstance(source_index, dict):
        raise DiscrepancyError("v0.9 source index must be an object")
    if not isinstance(baseline, dict) or not isinstance(
        baseline.get("conformance"), dict
    ):
        raise DiscrepancyError("guard baseline has no conformance surface")
    protected_conformance = baseline["conformance"]
    protected_case_ids = _protected_case_ids(protected_conformance)
    case_sources = _load_protected_cases(root, protected_case_ids)
    return AuthorityInputs(
        specification,
        source_index_bytes,
        source_index,
        guard_baseline_bytes,
        protected_conformance,
        protected_case_ids,
        manifest,
        case_sources,
    )
