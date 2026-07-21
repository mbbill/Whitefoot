"""Independent coordinator for the proposed frontend-boundary evidence."""

from __future__ import annotations

from pathlib import Path
import sys
from typing import Any


HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from frontend_boundary_diag_independent import (
    IndependentDiagnosticError,
    evaluate_atom_shape_case,
    evaluate_dotted_position_case,
    evaluate_grammar_case,
)
from frontend_boundary_independent_core import (
    IndependentBoundaryError,
    coordinate,
    rejection,
    unhex,
)
from frontend_boundary_independent_ingress import (
    digest,
    envelope,
    identity,
    read_cases,
    records,
    specification_contract,
)
from frontend_boundary_independent_scope import visibility, whole
from frontend_boundary_independent_tree import form2, root, topology
from frontend_boundary_raw_independent import (
    IndependentRawError,
    evaluate_authored_cases as evaluate_raw_cases,
    scan_sources,
)


def _lexical(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    """Interpret the older authored defect rows retained as supplementary probes."""
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    stage_order = [
        "raw-lexical-formation",
        "terminal-membership",
        "grammar-derivation",
        "canonical-rendering",
    ]
    defects = probe["defects"]
    if not isinstance(defects, list) or not defects:
        raise IndependentBoundaryError("lexical probe has no observations")
    candidates: list[tuple[int, int, int, int, dict[str, Any], list[int]]] = []
    for sequence, defect in enumerate(defects):
        if not isinstance(defect, dict) or set(defect) != {
            "actual_hex",
            "kind",
            "source",
            "span",
            "stage",
        }:
            raise IndependentBoundaryError("lexical observation vocabulary changed")
        if defect["stage"] not in stage_order:
            raise IndependentBoundaryError("lexical observation stage changed")
        where = coordinate(bound, defect["source"], defect["span"])
        actual = unhex(defect["actual_hex"])
        if bound[where[0]]["source"][where[1] : where[2]] != actual:
            raise IndependentBoundaryError("lexical observation is not source-anchored")
        candidates.append(
            (stage_order.index(defect["stage"]), where[0], where[1], sequence, defect, where)
        )
    _, _, _, _, selected, where = min(candidates)
    if selected["stage"] != "raw-lexical-formation":
        raise IndependentBoundaryError(
            "lexical probe selected an observation owned by another model stage"
        )
    owners = {
        "invalid-utf8": "FORM-2",
        "cr": "FORM-2",
        "tab": "FORM-2",
        "other-whitespace": "FORM-2",
        "line-comment-prefix": "FORM-4",
        "block-comment-prefix": "FORM-4",
        "malformed-string": "FORM-5",
        "malformed-region": "FORM-3",
        "malformed-label": "FORM-3",
        "unrecognized-token-start": "FORM-1",
    }
    owner = owners.get(selected["kind"])
    if owner is None:
        raise IndependentBoundaryError("lexical observation class changed")
    return rejection(owner, "SourceBytes", where)


def _grammar(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    try:
        return evaluate_grammar_case([record["source"] for record in bound], probe)
    except IndependentDiagnosticError as error:
        raise IndependentBoundaryError(str(error)) from error


def _dotted_position(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    try:
        return evaluate_dotted_position_case(
            [record["source"] for record in bound], probe
        )
    except IndependentDiagnosticError as error:
        raise IndependentBoundaryError(str(error)) from error


def _raw_scan(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    if set(probe) != {"bundle", "expect", "id", "kind"}:
        raise IndependentBoundaryError("raw-scan probe vocabulary changed")
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    try:
        return scan_sources([record["source"] for record in bound])
    except IndependentRawError as error:
        raise IndependentBoundaryError(str(error)) from error


def _atom_shape(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, bound = records(cases, probe["bundle"])
    if failure is not None:
        return failure
    try:
        return evaluate_atom_shape_case([record["source"] for record in bound], probe)
    except IndependentDiagnosticError as error:
        raise IndependentBoundaryError(str(error)) from error


def _run_case(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    if not isinstance(probe, dict) or not {"expect", "id", "kind"} <= set(probe):
        raise IndependentBoundaryError("probe lacks common fields")
    functions = {
        "atom-shape": _atom_shape,
        "dotted-position": _dotted_position,
        "envelope": envelope,
        "form2": form2,
        "grammar": _grammar,
        "identity": identity,
        "lexical": _lexical,
        "raw-scan": _raw_scan,
        "root": root,
        "topology": topology,
        "visibility": visibility,
        "whole-unit": whole,
    }
    implementation = functions.get(probe["kind"])
    if implementation is None:
        raise IndependentBoundaryError(f"unsupported independent probe kind: {probe['kind']!r}")
    actual = implementation(cases, probe)
    if actual != probe["expect"]:
        raise IndependentBoundaryError(
            f"independent probe {probe['id']} disagrees with its expectation"
        )
    return actual


def interpret(cases: dict[str, Any]) -> dict[str, object]:
    probes = cases.get("cases")
    if not isinstance(probes, list):
        raise IndependentBoundaryError("case collection is not a list")
    results: list[dict[str, object]] = []
    names: set[str] = set()
    for probe in probes:
        name = probe.get("id") if isinstance(probe, dict) else None
        if not isinstance(name, str) or not name or name in names:
            raise IndependentBoundaryError("probe identifier is absent or duplicated")
        names.add(name)
        results.append({"id": name, "outcome": _run_case(cases, probe)})
    try:
        raw_results = evaluate_raw_cases(cases.get("raw_cases"))
    except IndependentRawError as error:
        raise IndependentBoundaryError(str(error)) from error
    raw_names = {row["id"] for row in raw_results}
    if names & raw_names:
        raise IndependentBoundaryError("raw and structured probe identifiers overlap")
    names.update(raw_names)
    results.extend(raw_results)
    ledger = cases.get("requirements")
    required = {f"B{number:02d}" for number in range(1, 11)}
    if not isinstance(ledger, dict) or set(ledger) != required:
        raise IndependentBoundaryError("ten-entry requirement ledger changed")
    for requirement, members in ledger.items():
        if (
            not isinstance(members, list)
            or not members
            or members != sorted(set(members))
            or any(member not in names for member in members)
        ):
            raise IndependentBoundaryError(f"requirement {requirement} is not completely bound")
    return {
        "cases": results,
        "raw_case_count": len(raw_results),
        "requirements": ledger,
        "schema": "whitefoot.frontend-boundary-independent.v1",
    }


def build_independent() -> dict[str, object]:
    cases, case_raw = read_cases()
    contract, _ = specification_contract()
    return {
        "candidate": contract,
        "descriptor": {
            "byte_length": len(case_raw),
            "path": "grammar-verifier/proposal/frontend-boundary-cases.json",
            "sha256": digest(case_raw),
        },
        "evaluation": interpret(cases),
        "model": "independent tuple-and-event boundary interpreter",
    }
