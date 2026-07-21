"""Independent closed-unit visibility, declaration order, and uniqueness evidence."""

from __future__ import annotations

from typing import Any

from frontend_boundary_independent_core import (
    IndependentBoundaryError,
    coordinate,
    node_path,
    rejection,
)
from frontend_boundary_independent_ingress import profile, records


def visibility(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    events = probe["events"]
    observation_limit, _, _, _ = profile(cases, None)
    if not isinstance(events, list) or len(events) > observation_limit:
        raise IndependentBoundaryError("visibility observation list is invalid")
    normalized: list[tuple[int, int, int, dict[str, Any], list[int]]] = []
    all_functions: set[str] = set()
    vocabulary = {"action", "binding", "name", "node_path", "scope", "source", "span"}
    for sequence, event in enumerate(events):
        if not isinstance(event, dict) or set(event) != vocabulary:
            raise IndependentBoundaryError("visibility observation vocabulary changed")
        where = coordinate(bound, event["source"], event["span"])
        name = event["name"]
        if not isinstance(name, str) or not name.isascii() or not name:
            raise IndependentBoundaryError("visibility observation name is invalid")
        if bound[where[0]]["source"][where[1] : where[2]] != name.encode("ascii"):
            raise IndependentBoundaryError("visibility observation is not source-anchored")
        node_path(event["node_path"])
        if event["action"] not in ("declare", "reference") or event["binding"] not in (
            "constant",
            "function",
            "label",
            "local",
            "region",
        ):
            raise IndependentBoundaryError("visibility observation action or class changed")
        if not isinstance(event["scope"], str) or not event["scope"]:
            raise IndependentBoundaryError("visibility observation scope is invalid")
        if event["action"] == "declare" and event["binding"] == "function":
            all_functions.add(name)
        normalized.append((where[0], where[1], sequence, event, where))
    prior: set[tuple[str, str, str]] = set()
    for _, _, _, event, where in sorted(normalized):
        key = (event["binding"], event["scope"], event["name"])
        if event["action"] == "declare":
            if event["binding"] != "function":
                prior.add(key)
        elif not (
            event["binding"] == "function" and event["name"] in all_functions
        ) and key not in prior:
            return rejection("TYPE-6", "SourceNode", where, node_path(event["node_path"]))
    return {"family": "frontend-complete"}


def whole(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    declarations = probe["declarations"]
    if not isinstance(declarations, list):
        raise IndependentBoundaryError("whole-unit declarations are not a list")
    rows: list[tuple[int, int, int, dict[str, Any], list[int]]] = []
    for sequence, declaration in enumerate(declarations):
        if not isinstance(declaration, dict) or set(declaration) != {
            "kind",
            "name",
            "node_path",
            "source",
            "span",
        }:
            raise IndependentBoundaryError("whole-unit declaration vocabulary changed")
        where = coordinate(bound, declaration["source"], declaration["span"])
        if not isinstance(declaration["name"], str) or not declaration["name"]:
            raise IndependentBoundaryError("whole-unit declaration name is invalid")
        node_path(declaration["node_path"])
        rows.append((where[0], where[1], sequence, declaration, where))
    names: set[tuple[str, str]] = set()
    main_count = 0
    for _, _, _, declaration, where in sorted(rows):
        key = (declaration["kind"], declaration["name"])
        if key in names:
            return rejection("TYPE-6", "SourceNode", where, node_path(declaration["node_path"]))
        names.add(key)
        if key == ("function", "main"):
            main_count += 1
    if main_count != 1:
        extent = [[index, 0, len(record["source"])] for index, record in enumerate(bound)]
        return rejection("FN-7", "BundleRoot", node=[], extent=extent)
    return {"family": "frontend-complete"}
