"""Primary source-tree topology, root, and canonical-gap evidence."""

from __future__ import annotations

from typing import Any

from frontend_boundary_primary_core import (
    BoundaryModelError,
    node_path,
    source_bytes,
    source_rejection,
    span,
    strict_hex,
)
from frontend_boundary_primary_ingress import bind, limits


def topology_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    fragments = case["fragments"]
    if not isinstance(fragments, list) or len(fragments) > limits(descriptor)["max_observations"]:
        raise BoundaryModelError("topology fragments exceed their shape or limit")
    for fragment in fragments:
        if not isinstance(fragment, dict) or set(fragment) != {
            "kind",
            "member_sources",
            "owner_source",
            "span",
        }:
            raise BoundaryModelError("topology fragment fields changed")
        kind = fragment["kind"]
        if kind not in {"production", "source-span", "token", "trivia"}:
            raise BoundaryModelError("topology fragment kind changed")
        owner = fragment["owner_source"]
        try:
            span(records, owner, fragment["span"])
        except BoundaryModelError:
            return {
                "code": f"{kind}-outside-source",
                "family": "compiler-invariant-failure",
            }
        members = fragment["member_sources"]
        if not isinstance(members, list) or not members:
            raise BoundaryModelError("topology fragment has no source members")
        if any(member != owner for member in members):
            return {
                "code": f"{kind}-crosses-record-boundary",
                "family": "compiler-invariant-failure",
            }
    return {"family": "frontend-complete"}


def root_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    children: list[dict[str, object]] = []
    extents: list[list[int]] = []
    forests: list[dict[str, object]] = []
    for record in records:
        extents.append([record.ordinal, 0, len(record.source)])
        forest_paths: list[list[int]] = []
        previous_end = 0
        for local_ordinal, item in enumerate(record.items):
            if not isinstance(item, dict) or set(item) != {"id", "span"}:
                raise BoundaryModelError("root item fields changed")
            coordinate = span(records, record.ordinal, item["span"])
            if coordinate[1] < previous_end:
                raise BoundaryModelError("source-local items are not in source order")
            previous_end = coordinate[2]
            identifier = item["id"]
            if not isinstance(identifier, str) or not identifier:
                raise BoundaryModelError("root item identifier is invalid")
            children.append(
                {
                    "coordinate": list(coordinate),
                    "id": identifier,
                    "node_path": [len(children)],
                    "source_local_item_ordinal": local_ordinal,
                }
            )
            forest_paths.append([len(children) - 1])
        forests.append({"item_node_paths": forest_paths, "source_ordinal": record.ordinal})
    return {
        "children": children,
        "extent": extents,
        "family": "root-projection",
        "node_path": [],
        "program_root_count": 1,
        "source_forests": forests,
    }


def _first_unequal(left: bytes, right: bytes) -> int:
    for index, (lhs, rhs) in enumerate(zip(left, right)):
        if lhs != rhs:
            return index
    return min(len(left), len(right))


def form2_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    source_ordinal = case["source"]
    actual = source_bytes(records, source_ordinal)
    rendered = strict_hex(case["rendered_hex"], "FORM-2 rendering")
    if actual == rendered:
        return {"family": "frontend-complete"}
    mismatch = case["mismatch"]
    if not isinstance(mismatch, dict) or set(mismatch) != {
        "actual_interval",
        "boundary",
        "left_terminal_path",
        "node_extent",
        "right_terminal_path",
    }:
        raise BoundaryModelError("FORM-2 mismatch fields changed")
    coordinate = span(records, source_ordinal, mismatch["actual_interval"])
    first = _first_unequal(actual, rendered)
    if not (coordinate[1] <= first <= coordinate[2]):
        raise BoundaryModelError("FORM-2 interval does not contain the first unequal boundary")
    boundary = mismatch["boundary"]
    if boundary == "production-separator":
        left = node_path(mismatch["left_terminal_path"])
        right = node_path(mismatch["right_terminal_path"])
        extent = span(records, source_ordinal, mismatch["node_extent"])
        if not (extent[1] <= coordinate[1] <= coordinate[2] <= extent[2]):
            raise BoundaryModelError("FORM-2 coordinate is outside its owning node extent")
        common = 0
        while common < min(len(left), len(right)) and left[common] == right[common]:
            common += 1
        if common == 0 or common == len(left) or common == len(right):
            raise BoundaryModelError("FORM-2 terminal paths lack one strict common production ancestor")
        return source_rejection("FORM-2", "SourceNode", coordinate, left[:common])
    if boundary not in {"inter-item", "source-final", "source-leading"}:
        raise BoundaryModelError("FORM-2 boundary class changed")
    if any(
        mismatch[name] is not None
        for name in ("left_terminal_path", "node_extent", "right_terminal_path")
    ):
        raise BoundaryModelError("source boundary mismatch carries tree ownership")
    return source_rejection("FORM-2", "SourceBytes", coordinate)
