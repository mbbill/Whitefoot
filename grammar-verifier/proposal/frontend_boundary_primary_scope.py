"""Primary closed-unit visibility, declaration order, and uniqueness evidence."""

from __future__ import annotations

from typing import Any

from frontend_boundary_primary_core import (
    BoundaryModelError,
    node_path,
    source_rejection,
    span,
)
from frontend_boundary_primary_ingress import bind, limits


def visibility_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    events = case["events"]
    if not isinstance(events, list) or len(events) > limits(descriptor)["max_observations"]:
        raise BoundaryModelError("visibility events exceed their shape or limit")
    prepared: list[tuple[tuple[int, int, int], dict[str, Any], tuple[int, int, int]]] = []
    functions: set[str] = set()
    vocabulary = {"action", "binding", "name", "node_path", "scope", "source", "span"}
    for ordinal, event in enumerate(events):
        if not isinstance(event, dict) or set(event) != vocabulary:
            raise BoundaryModelError("visibility event fields changed")
        coordinate = span(records, event["source"], event["span"])
        name = event["name"]
        if not isinstance(name, str) or not name.isascii() or not name:
            raise BoundaryModelError("visibility name is invalid")
        if records[coordinate[0]].source[coordinate[1] : coordinate[2]] != name.encode("ascii"):
            raise BoundaryModelError("visibility event is not anchored to its named source bytes")
        node_path(event["node_path"])
        if event["action"] not in {"declare", "reference"}:
            raise BoundaryModelError("visibility action changed")
        if event["binding"] not in {"constant", "function", "label", "local", "region"}:
            raise BoundaryModelError("visibility binding class changed")
        if not isinstance(event["scope"], str) or not event["scope"]:
            raise BoundaryModelError("visibility scope is invalid")
        if event["action"] == "declare" and event["binding"] == "function":
            functions.add(name)
        prepared.append(((coordinate[0], coordinate[1], ordinal), event, coordinate))
    visible: set[tuple[str, str, str]] = set()
    for _, event, coordinate in sorted(prepared, key=lambda item: item[0]):
        binding = event["binding"]
        name = event["name"]
        key = (binding, event["scope"], name)
        if event["action"] == "declare":
            if binding != "function":
                visible.add(key)
            continue
        if binding == "function" and name in functions:
            continue
        if key in visible:
            continue
        return source_rejection("TYPE-6", "SourceNode", coordinate, node_path(event["node_path"]))
    return {"family": "frontend-complete"}


def whole_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    declarations = case["declarations"]
    if not isinstance(declarations, list):
        raise BoundaryModelError("whole-unit declarations are not a list")
    prepared: list[tuple[tuple[int, int, int], dict[str, Any], tuple[int, int, int]]] = []
    for ordinal, declaration in enumerate(declarations):
        if not isinstance(declaration, dict) or set(declaration) != {
            "kind",
            "name",
            "node_path",
            "source",
            "span",
        }:
            raise BoundaryModelError("whole-unit declaration fields changed")
        coordinate = span(records, declaration["source"], declaration["span"])
        if not isinstance(declaration["name"], str) or not declaration["name"]:
            raise BoundaryModelError("whole-unit declaration name is invalid")
        node_path(declaration["node_path"])
        prepared.append(((coordinate[0], coordinate[1], ordinal), declaration, coordinate))
    seen: set[tuple[str, str]] = set()
    functions: list[str] = []
    for _, declaration, coordinate in sorted(prepared, key=lambda item: item[0]):
        key = (declaration["kind"], declaration["name"])
        if key in seen:
            return source_rejection(
                "TYPE-6", "SourceNode", coordinate, node_path(declaration["node_path"])
            )
        seen.add(key)
        if declaration["kind"] == "function":
            functions.append(declaration["name"])
    if functions.count("main") != 1:
        extent = [[record.ordinal, 0, len(record.source)] for record in records]
        return source_rejection("FN-7", "BundleRoot", node_path_value=[], extent=extent)
    return {"family": "frontend-complete"}
