#!/usr/bin/env python3
"""Descriptor-safe reads for the fixed semantic-catalog repository inputs."""

from __future__ import annotations

import json
import os
import stat
from pathlib import Path
from typing import Any, Sequence


class CatalogIOError(ValueError):
    """A catalog input violates the bounded trusted-input policy."""


def _validate_json_value(
    value: Any,
    label: str,
    *,
    max_depth: int,
    max_integer_digits: int,
    max_string_bytes: int,
    max_list_items: int,
    max_object_fields: int,
    max_nodes: int,
) -> None:
    """Accept only bounded values that preserve identity through JSON."""
    stack = [(value, 0)]
    seen_containers: set[int] = set()
    nodes = 0
    while stack:
        item, parent_depth = stack.pop()
        nodes += 1
        if nodes > max_nodes:
            raise CatalogIOError(f"{label} exceeds the {max_nodes}-node JSON limit")
        if item is None or type(item) is bool:
            continue
        if type(item) is int:
            digits = str(item)
            if digits.startswith("-"):
                digits = digits[1:]
            if len(digits) > max_integer_digits:
                raise CatalogIOError(
                    f"{label} contains an integer over {max_integer_digits} digits"
                )
            continue
        if type(item) is float:
            raise CatalogIOError(f"{label} contains a forbidden JSON float")
        if type(item) is str:
            try:
                encoded = item.encode("ascii")
            except UnicodeEncodeError as error:
                raise CatalogIOError(
                    f"{label} contains a non-ASCII string value"
                ) from error
            if len(encoded) > max_string_bytes:
                raise CatalogIOError(
                    f"{label} contains a string over {max_string_bytes} bytes"
                )
            continue
        if type(item) not in (list, dict):
            raise CatalogIOError(
                f"{label} contains a non-JSON value of type {type(item).__name__}"
            )
        identity = id(item)
        if identity in seen_containers:
            raise CatalogIOError(f"{label} is not an unshared acyclic JSON tree")
        seen_containers.add(identity)
        depth = parent_depth + 1
        if depth > max_depth:
            raise CatalogIOError(f"{label} exceeds the JSON depth limit {max_depth}")
        if type(item) is list:
            if len(item) > max_list_items:
                raise CatalogIOError(
                    f"{label} contains a list over {max_list_items} items"
                )
            stack.extend((child, depth) for child in item)
            continue
        if len(item) > max_object_fields:
            raise CatalogIOError(
                f"{label} contains an object over {max_object_fields} fields"
            )
        for key, child in item.items():
            if type(key) is not str:
                raise CatalogIOError(f"{label} contains a non-string object key")
            stack.append((key, depth))
            stack.append((child, depth))


def canonical_json_bytes(
    value: Any,
    label: str,
    max_bytes: int,
    *,
    max_depth: int = 64,
    max_integer_digits: int = 12,
    max_string_bytes: int = 16_384,
    max_list_items: int = 8_192,
    max_object_fields: int = 128,
    max_nodes: int = 100_000,
) -> bytes:
    """Encode bounded canonical ASCII JSON with exactly one final LF."""
    try:
        _validate_json_value(
            value,
            label,
            max_depth=max_depth,
            max_integer_digits=max_integer_digits,
            max_string_bytes=max_string_bytes,
            max_list_items=max_list_items,
            max_object_fields=max_object_fields,
            max_nodes=max_nodes,
        )
        encoded = json.dumps(
            value,
            allow_nan=False,
            ensure_ascii=True,
            indent=2,
            sort_keys=True,
        )
    except (MemoryError, OverflowError, RecursionError, TypeError, ValueError) as error:
        raise CatalogIOError(f"value is not canonical JSON: {error}") from error
    raw = (encoded + "\n").encode("ascii")
    if len(raw) > max_bytes:
        raise CatalogIOError(
            f"{label} exceeds the {max_bytes}-byte normalized-output limit"
        )
    return raw


def parse_canonical_json(
    raw: bytes,
    label: str,
    *,
    max_bytes: int,
    max_depth: int,
    max_integer_digits: int,
    max_string_bytes: int,
    max_list_items: int,
    max_object_fields: int,
    max_nodes: int,
) -> Any:
    """Parse canonical ASCII JSON under explicit structural budgets."""
    if type(raw) is not bytes:
        raise CatalogIOError(f"{label} must be exact bytes")
    if not raw:
        raise CatalogIOError(f"{label} is empty")
    if len(raw) > max_bytes:
        raise CatalogIOError(f"{label} exceeds the {max_bytes}-byte input limit")
    if any(byte > 0x7F for byte in raw):
        raise CatalogIOError(f"{label} contains non-ASCII bytes")

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
        elif byte == 0x22:
            in_string = True
        elif byte in (0x5B, 0x7B):
            depth += 1
            if depth > max_depth:
                raise CatalogIOError(f"{label} exceeds the JSON depth limit {max_depth}")
        elif byte in (0x5D, 0x7D):
            depth -= 1

    def reject_constant(value: str) -> None:
        raise CatalogIOError(f"non-finite JSON number is forbidden: {value}")

    def reject_float(value: str) -> None:
        raise CatalogIOError(f"JSON floats are forbidden: {value}")

    def bounded_integer(value: str) -> int:
        digits = value[1:] if value.startswith("-") else value
        if len(digits) > max_integer_digits:
            raise CatalogIOError(
                f"JSON integer exceeds the {max_integer_digits}-digit limit"
            )
        try:
            return int(value)
        except (OverflowError, ValueError) as error:
            raise CatalogIOError(f"invalid JSON integer: {error}") from error

    def closed_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in pairs:
            if key in result:
                raise CatalogIOError(f"duplicate JSON key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(
            raw.decode("ascii"),
            object_pairs_hook=closed_object,
            parse_int=bounded_integer,
            parse_float=reject_float,
            parse_constant=reject_constant,
        )
    except CatalogIOError:
        raise
    except (
        MemoryError,
        OverflowError,
        RecursionError,
        UnicodeDecodeError,
        ValueError,
    ) as error:
        raise CatalogIOError(f"invalid {label}: {error}") from error

    try:
        _validate_json_value(
            value,
            label,
            max_depth=max_depth,
            max_integer_digits=max_integer_digits,
            max_string_bytes=max_string_bytes,
            max_list_items=max_list_items,
            max_object_fields=max_object_fields,
            max_nodes=max_nodes,
        )
    except CatalogIOError:
        raise
    except (MemoryError, OverflowError, RecursionError, TypeError, ValueError) as error:
        raise CatalogIOError(f"invalid {label}: {error}") from error
    if canonical_json_bytes(
        value,
        label,
        max_bytes,
        max_depth=max_depth,
        max_integer_digits=max_integer_digits,
        max_string_bytes=max_string_bytes,
        max_list_items=max_list_items,
        max_object_fields=max_object_fields,
        max_nodes=max_nodes,
    ) != raw:
        raise CatalogIOError(f"{label} is not canonical or contains trailing bytes")
    return value


def _flags(*, directory: bool) -> int:
    if (
        not hasattr(os, "O_NOFOLLOW")
        or (directory and not hasattr(os, "O_DIRECTORY"))
        or (not directory and not hasattr(os, "O_NONBLOCK"))
    ):
        raise CatalogIOError("platform cannot provide symlink-safe descriptor reads")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | os.O_NOFOLLOW
    if directory:
        flags |= os.O_DIRECTORY
    else:
        # A FIFO or device path must not block before fstat can reject it.
        flags |= os.O_NONBLOCK
    return flags


def _open_root(root: Path) -> int:
    try:
        descriptor = os.open(os.fspath(root), _flags(directory=True))
    except OSError as error:
        raise CatalogIOError(
            f"repository root is not a readable non-symlink directory: {root}"
        ) from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise CatalogIOError(f"repository root is not a directory: {root}")
    return descriptor


def _open_directory_at(parent: int, name: str, label: str) -> int:
    _validate_component(name, label)
    try:
        descriptor = os.open(name, _flags(directory=True), dir_fd=parent)
    except OSError as error:
        raise CatalogIOError(f"{label} is not a readable non-symlink directory") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise CatalogIOError(f"{label} is not a directory")
    return descriptor


def _validate_component(name: str, label: str) -> None:
    if not name or name in {".", ".."} or "/" in name or "\x00" in name:
        raise CatalogIOError(f"invalid path component for {label}: {name!r}")


def _open_parent(root: Path, components: Sequence[str], label: str) -> tuple[int, str]:
    if not components:
        raise CatalogIOError(f"{label} has no path components")
    descriptors = [_open_root(root)]
    try:
        for ordinal, component in enumerate(components[:-1]):
            descriptors.append(
                _open_directory_at(
                    descriptors[-1], component, f"{label} parent component {ordinal}"
                )
            )
        result = os.dup(descriptors[-1])
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)
    _validate_component(components[-1], label)
    return result, components[-1]


def _read_regular_at(parent: int, name: str, max_bytes: int, label: str) -> bytes:
    _validate_component(name, label)
    try:
        descriptor = os.open(name, _flags(directory=False), dir_fd=parent)
    except OSError as error:
        raise CatalogIOError(f"{label} is not a readable regular non-symlink file") from error
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise CatalogIOError(f"{label} is not a regular file")
        if before.st_size > max_bytes:
            raise CatalogIOError(f"{label} exceeds the {max_bytes}-byte limit")
        chunks: list[bytes] = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(65_536, max_bytes + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            if total > max_bytes:
                raise CatalogIOError(f"{label} exceeds the {max_bytes}-byte limit")
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
            raise CatalogIOError(f"{label} changed while it was being read")
        return b"".join(chunks)
    finally:
        os.close(descriptor)


def read_fixed_file(
    root: Path, components: Sequence[str], max_bytes: int, label: str
) -> bytes:
    """Read one fixed relative file without following any path-component symlink."""
    try:
        parent, name = _open_parent(root, components, label)
        try:
            return _read_regular_at(parent, name, max_bytes, label)
        finally:
            os.close(parent)
    except CatalogIOError:
        raise
    except OSError as error:
        raise CatalogIOError(f"{label} filesystem operation failed: {error}") from error


def read_fragment_directory(
    root: Path,
    components: Sequence[str],
    *,
    label: str = "decomposition",
    max_entries: int | None = None,
    max_count: int,
    max_file_bytes: int,
    max_total_bytes: int,
) -> tuple[tuple[str, bytes], ...]:
    """Read direct ``*.json`` files from one labelled non-symlink directory."""
    try:
        if max_count <= 0:
            raise CatalogIOError("fragment count limit must be positive")
        entry_limit = max_count * 4 if max_entries is None else max_entries
        if entry_limit <= 0:
            raise CatalogIOError("directory entry limit must be positive")
        directory_label = f"{label} directory"
        parent, name = _open_parent(root, components, directory_label)
        try:
            directory = _open_directory_at(parent, name, directory_label)
        finally:
            os.close(parent)
        try:
            names: list[str] = []
            scanned = 0
            try:
                with os.scandir(directory) as entries:
                    for record in entries:
                        scanned += 1
                        if scanned > entry_limit:
                            raise CatalogIOError(
                                f"direct {label} entries exceed the "
                                f"{entry_limit}-entry limit"
                            )
                        entry = record.name
                        if not isinstance(entry, str):
                            raise CatalogIOError(f"{label} fragment filename is not text")
                        try:
                            entry.encode("ascii")
                        except UnicodeEncodeError as error:
                            if entry.endswith(".json"):
                                raise CatalogIOError(
                                    f"{label} fragment filename is not ASCII: {entry!r}"
                                ) from error
                            continue
                        if entry.endswith(".json"):
                            names.append(entry)
                            if len(names) > max_count:
                                raise CatalogIOError(
                                    f"{label} fragment count exceeds the "
                                    f"{max_count}-file limit"
                                )
            except CatalogIOError:
                raise
            except OSError as error:
                raise CatalogIOError(
                    f"cannot enumerate {label} directory: {error}"
                ) from error
            names.sort(key=str.encode)
            if not names:
                raise CatalogIOError(f"no {label} fragments found")
            result: list[tuple[str, bytes]] = []
            total = 0
            for entry in names:
                raw = _read_regular_at(
                    directory, entry, max_file_bytes, f"{label} fragment {entry}"
                )
                total += len(raw)
                if total > max_total_bytes:
                    raise CatalogIOError(
                        f"{label} fragment bytes exceed the "
                        f"{max_total_bytes}-byte aggregate limit"
                    )
                result.append((entry, raw))
            return tuple(result)
        finally:
            os.close(directory)
    except CatalogIOError:
        raise
    except OSError as error:
        raise CatalogIOError(
            f"{label} directory filesystem operation failed: {error}"
        ) from error
