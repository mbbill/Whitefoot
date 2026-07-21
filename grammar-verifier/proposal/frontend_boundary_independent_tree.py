"""Independent source-tree topology, root, and canonical-gap evidence."""

from __future__ import annotations

from typing import Any

from frontend_boundary_independent_core import (
    IndependentBoundaryError,
    coordinate,
    node_path,
    rejection,
    unhex,
)
from frontend_boundary_independent_ingress import profile, records


def topology(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    fragments = probe["fragments"]
    observation_limit, _, _, _ = profile(cases, None)
    if not isinstance(fragments, list) or len(fragments) > observation_limit:
        raise IndependentBoundaryError("topology observation list is invalid")
    for fragment in fragments:
        if not isinstance(fragment, dict) or set(fragment) != {
            "kind",
            "member_sources",
            "owner_source",
            "span",
        }:
            raise IndependentBoundaryError("topology observation vocabulary changed")
        kind = fragment["kind"]
        if kind not in ("production", "source-span", "token", "trivia"):
            raise IndependentBoundaryError("topology observation kind changed")
        try:
            coordinate(bound, fragment["owner_source"], fragment["span"])
        except IndependentBoundaryError:
            return {"code": f"{kind}-outside-source", "family": "compiler-invariant-failure"}
        members = fragment["member_sources"]
        if not isinstance(members, list) or len(members) == 0:
            raise IndependentBoundaryError("topology observation has no members")
        if any(member != fragment["owner_source"] for member in members):
            return {"code": f"{kind}-crosses-record-boundary", "family": "compiler-invariant-failure"}
    return {"family": "frontend-complete"}


def root(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    children: list[dict[str, object]] = []
    extents: list[list[int]] = []
    forests: list[dict[str, object]] = []
    for source_index, record in enumerate(bound):
        extents.append([source_index, 0, len(record["source"])])
        forest_paths: list[list[int]] = []
        last = 0
        for local_index, item in enumerate(record["items"]):
            if not isinstance(item, dict) or set(item) != {"id", "span"}:
                raise IndependentBoundaryError("root item vocabulary changed")
            where = coordinate(bound, source_index, item["span"])
            if where[1] < last:
                raise IndependentBoundaryError("root items are not source-local ordered")
            last = where[2]
            if not isinstance(item["id"], str) or not item["id"]:
                raise IndependentBoundaryError("root item lacks an identifier")
            children.append(
                {
                    "coordinate": where,
                    "id": item["id"],
                    "node_path": [len(children)],
                    "source_local_item_ordinal": local_index,
                }
            )
            forest_paths.append([len(children) - 1])
        forests.append({"item_node_paths": forest_paths, "source_ordinal": source_index})
    return {
        "children": children,
        "extent": extents,
        "family": "root-projection",
        "node_path": [],
        "program_root_count": 1,
        "source_forests": forests,
    }


def _unequal(left: bytes, right: bytes) -> int:
    index = 0
    limit = min(len(left), len(right))
    while index < limit and left[index] == right[index]:
        index += 1
    return index


def form2(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    source_number = probe["source"]
    if type(source_number) is not int or source_number < 0 or source_number >= len(bound):
        raise IndependentBoundaryError("FORM-2 probe source ordinal is invalid")
    actual = bound[source_number]["source"]
    rendered = unhex(probe["rendered_hex"])
    if actual == rendered:
        return {"family": "frontend-complete"}
    mismatch = probe["mismatch"]
    if not isinstance(mismatch, dict) or set(mismatch) != {
        "actual_interval",
        "boundary",
        "left_terminal_path",
        "node_extent",
        "right_terminal_path",
    }:
        raise IndependentBoundaryError("FORM-2 mismatch vocabulary changed")
    where = coordinate(bound, source_number, mismatch["actual_interval"])
    first = _unequal(actual, rendered)
    if first < where[1] or first > where[2]:
        raise IndependentBoundaryError("FORM-2 reported trivia does not contain its first mismatch")
    if mismatch["boundary"] == "production-separator":
        left = node_path(mismatch["left_terminal_path"])
        right = node_path(mismatch["right_terminal_path"])
        extent = coordinate(bound, source_number, mismatch["node_extent"])
        if not (extent[1] <= where[1] <= where[2] <= extent[2]):
            raise IndependentBoundaryError("FORM-2 coordinate escapes its source node extent")
        common: list[int] = []
        for left_part, right_part in zip(left, right):
            if left_part != right_part:
                break
            common.append(left_part)
        if not common or len(common) in (len(left), len(right)):
            raise IndependentBoundaryError("FORM-2 terminal leaves lack one strict production LCA")
        return rejection("FORM-2", "SourceNode", where, common)
    if mismatch["boundary"] not in ("inter-item", "source-final", "source-leading"):
        raise IndependentBoundaryError("FORM-2 source boundary class changed")
    if any(
        mismatch[name] is not None
        for name in ("left_terminal_path", "node_extent", "right_terminal_path")
    ):
        raise IndependentBoundaryError("FORM-2 source boundary carries terminal-tree ownership")
    return rejection("FORM-2", "SourceBytes", where)
