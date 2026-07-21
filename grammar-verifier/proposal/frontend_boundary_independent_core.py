"""Tuple primitives shared by the independent frontend-boundary interpreters."""

from __future__ import annotations

from typing import Any


class IndependentBoundaryError(RuntimeError):
    pass


def unhex(value: object) -> bytes:
    if not isinstance(value, str) or len(value) % 2 or value.lower() != value:
        raise IndependentBoundaryError("source byte field is not canonical hexadecimal")
    try:
        raw = bytes.fromhex(value)
    except ValueError as error:
        raise IndependentBoundaryError("source byte field is malformed") from error
    if raw.hex() != value:
        raise IndependentBoundaryError("source byte field has a noncanonical spelling")
    return raw


def coordinate(
    records: list[dict[str, Any]], source: object, interval: object
) -> list[int]:
    if type(source) is not int or source < 0 or source >= len(records):
        raise IndependentBoundaryError("coordinate has an invalid source ordinal")
    if (
        not isinstance(interval, list)
        or len(interval) != 2
        or any(type(value) is not int for value in interval)
    ):
        raise IndependentBoundaryError("coordinate interval is not two integers")
    start, end = interval
    raw = records[source]["source"]
    if start < 0 or start > end or end > len(raw):
        raise IndependentBoundaryError("coordinate interval escapes its source")
    return [source, start, end]


def node_path(value: object) -> list[int]:
    if not isinstance(value, list) or any(
        type(index) is not int or index < 0 for index in value
    ):
        raise IndependentBoundaryError("node path is malformed")
    return value


def rejection(
    rule: str,
    place: str,
    coordinate_value: list[int] | None = None,
    node: object = None,
    extent: object = None,
    expected: object = None,
) -> dict[str, object]:
    location: dict[str, object] = {"kind": place}
    if coordinate_value is not None:
        location["coordinate"] = coordinate_value
    if node is not None:
        location["node_path"] = node
    if extent is not None:
        location["extent"] = extent
    answer: dict[str, object] = {
        "family": "source-language-rejection",
        "location": location,
        "rule": rule,
    }
    if expected is not None:
        answer["expected_terminals"] = expected
    return answer
