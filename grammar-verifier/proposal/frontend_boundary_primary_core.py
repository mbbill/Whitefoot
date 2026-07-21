"""Typed primitives shared by the primary frontend-boundary interpreters."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


class BoundaryModelError(RuntimeError):
    """The proposal evidence input is malformed or contradicts its expectation."""


@dataclass(frozen=True)
class Record:
    ordinal: int
    path: str
    source: bytes
    items: tuple[dict[str, Any], ...]


def strict_hex(value: object, label: str) -> bytes:
    if not isinstance(value, str) or len(value) % 2 != 0:
        raise BoundaryModelError(f"{label} is not even-length hexadecimal")
    if value != value.lower() or any(ch not in "0123456789abcdef" for ch in value):
        raise BoundaryModelError(f"{label} is not lowercase hexadecimal")
    try:
        return bytes.fromhex(value)
    except ValueError as error:
        raise BoundaryModelError(f"{label} is malformed hexadecimal") from error


def source_bytes(records: tuple[Record, ...], ordinal: object) -> bytes:
    if not isinstance(ordinal, int) or isinstance(ordinal, bool):
        raise BoundaryModelError("source ordinal is not an integer")
    if ordinal < 0 or ordinal >= len(records):
        raise BoundaryModelError("source ordinal is outside the bundle")
    return records[ordinal].source


def span(
    records: tuple[Record, ...], ordinal: object, value: object
) -> tuple[int, int, int]:
    source = source_bytes(records, ordinal)
    if (
        not isinstance(value, list)
        or len(value) != 2
        or any(not isinstance(part, int) or isinstance(part, bool) for part in value)
    ):
        raise BoundaryModelError("span is not a two-integer list")
    start, end = value
    if start < 0 or end < start or end > len(source):
        raise BoundaryModelError("span is outside its source")
    assert isinstance(ordinal, int)
    return ordinal, start, end


def node_path(value: object) -> list[int]:
    if not isinstance(value, list) or any(
        not isinstance(part, int) or isinstance(part, bool) or part < 0 for part in value
    ):
        raise BoundaryModelError("node path is not a nonnegative-integer list")
    return value


def source_rejection(
    rule: str,
    location_kind: str,
    coordinate: tuple[int, int, int] | None = None,
    node_path_value: object = None,
    extent: object = None,
    expected: list[str] | None = None,
) -> dict[str, object]:
    location: dict[str, object] = {"kind": location_kind}
    if coordinate is not None:
        location["coordinate"] = list(coordinate)
    if node_path_value is not None:
        location["node_path"] = node_path_value
    if extent is not None:
        location["extent"] = extent
    result: dict[str, object] = {
        "family": "source-language-rejection",
        "location": location,
        "rule": rule,
    }
    if expected is not None:
        result["expected_terminals"] = expected
    return result
