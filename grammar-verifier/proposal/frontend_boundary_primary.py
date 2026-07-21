"""Primary coordinator for the proposed frontend-boundary evidence models."""

from __future__ import annotations

from pathlib import Path
import sys
from typing import Any


MODULE_ROOT = Path(__file__).resolve().parent
if str(MODULE_ROOT) not in sys.path:
    sys.path.insert(0, str(MODULE_ROOT))

from frontend_boundary_diag_primary import (
    PrimaryDiagnosticError,
    evaluate_atom_shape_case,
    evaluate_dotted_position_case,
    evaluate_grammar_case,
)
from frontend_boundary_primary_core import (
    BoundaryModelError,
    source_rejection,
    span,
    strict_hex,
)
from frontend_boundary_primary_ingress import (
    bind,
    candidate_contract,
    envelope_case,
    identity_case,
    load_descriptor,
    sha256,
)
from frontend_boundary_primary_scope import visibility_case, whole_case
from frontend_boundary_primary_tree import form2_case, root_case, topology_case
from frontend_boundary_raw_primary import (
    PrimaryRawError,
    evaluate_authored_cases as evaluate_raw_cases,
    scan_sources,
)


_STAGE_ORDER = {
    "raw-lexical-formation": 0,
    "terminal-membership": 1,
    "grammar-derivation": 2,
    "canonical-rendering": 3,
}
_RAW_RULE = {
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


def _lexical_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    """Interpret the older authored defect rows retained as supplementary probes."""
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    defects = case["defects"]
    if not isinstance(defects, list) or not defects:
        raise BoundaryModelError("lexical case has no defects")
    ranked: list[tuple[tuple[int, int, int, int], dict[str, Any], tuple[int, int, int]]] = []
    for ordinal, defect in enumerate(defects):
        if not isinstance(defect, dict) or set(defect) != {
            "actual_hex",
            "kind",
            "source",
            "span",
            "stage",
        }:
            raise BoundaryModelError("frontend defect fields changed")
        stage = defect["stage"]
        if stage not in _STAGE_ORDER:
            raise BoundaryModelError("frontend defect stage changed")
        coordinate = span(records, defect["source"], defect["span"])
        actual = strict_hex(defect["actual_hex"], "defect actual bytes")
        if records[coordinate[0]].source[coordinate[1] : coordinate[2]] != actual:
            raise BoundaryModelError("frontend defect is not anchored to its source bytes")
        ranked.append(
            ((_STAGE_ORDER[stage], coordinate[0], coordinate[1], ordinal), defect, coordinate)
        )
    _, defect, coordinate = min(ranked, key=lambda item: item[0])
    if defect["stage"] != "raw-lexical-formation":
        raise BoundaryModelError("non-raw frontend probes belong in their focused case kind")
    rule = _RAW_RULE.get(defect["kind"])
    if rule is None:
        raise BoundaryModelError("raw lexical defect kind changed")
    return source_rejection(rule, "SourceBytes", coordinate)


def _grammar_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    try:
        return evaluate_grammar_case(tuple(record.source for record in records), case)
    except PrimaryDiagnosticError as error:
        raise BoundaryModelError(str(error)) from error


def _dotted_position_case(
    descriptor: dict[str, Any], case: dict[str, Any]
) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    try:
        return evaluate_dotted_position_case(
            tuple(record.source for record in records), case
        )
    except PrimaryDiagnosticError as error:
        raise BoundaryModelError(str(error)) from error


def _raw_scan_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    if set(case) != {"bundle", "expect", "id", "kind"}:
        raise BoundaryModelError("raw-scan case fields changed")
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    try:
        return scan_sources(tuple(record.source for record in records))
    except PrimaryRawError as error:
        raise BoundaryModelError(str(error)) from error


def _atom_shape_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, records = bind(descriptor, case["bundle"])
    if failure:
        return failure
    try:
        return evaluate_atom_shape_case(tuple(record.source for record in records), case)
    except PrimaryDiagnosticError as error:
        raise BoundaryModelError(str(error)) from error


def _evaluate_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    if not isinstance(case, dict) or not {"expect", "id", "kind"} <= set(case):
        raise BoundaryModelError("case has malformed common fields")
    handlers = {
        "atom-shape": _atom_shape_case,
        "dotted-position": _dotted_position_case,
        "envelope": envelope_case,
        "form2": form2_case,
        "grammar": _grammar_case,
        "identity": identity_case,
        "lexical": _lexical_case,
        "raw-scan": _raw_scan_case,
        "root": root_case,
        "topology": topology_case,
        "visibility": visibility_case,
        "whole-unit": whole_case,
    }
    handler = handlers.get(case["kind"])
    if handler is None:
        raise BoundaryModelError(f"unsupported primary case kind: {case['kind']!r}")
    result = handler(descriptor, case)
    if result != case["expect"]:
        raise BoundaryModelError(
            f"primary case {case['id']} disagrees: expected {case['expect']!r}, got {result!r}"
        )
    return result


def evaluate_descriptor(descriptor: dict[str, Any]) -> dict[str, object]:
    cases = descriptor["cases"]
    identifiers: set[str] = set()
    results: list[dict[str, object]] = []
    for case in cases:
        identifier = case.get("id") if isinstance(case, dict) else None
        if not isinstance(identifier, str) or not identifier or identifier in identifiers:
            raise BoundaryModelError("case identifiers are invalid or duplicated")
        identifiers.add(identifier)
        results.append({"id": identifier, "outcome": _evaluate_case(descriptor, case)})
    try:
        raw_results = evaluate_raw_cases(descriptor["raw_cases"])
    except PrimaryRawError as error:
        raise BoundaryModelError(str(error)) from error
    raw_names = {row["id"] for row in raw_results}
    if identifiers & raw_names:
        raise BoundaryModelError("raw and structured case identifiers overlap")
    identifiers.update(raw_names)
    results.extend(raw_results)
    requirements = descriptor["requirements"]
    if not isinstance(requirements, dict) or set(requirements) != {
        f"B{number:02d}" for number in range(1, 11)
    }:
        raise BoundaryModelError("the ten-case requirement ledger changed")
    for requirement, names in requirements.items():
        if (
            not isinstance(names, list)
            or not names
            or any(not isinstance(name, str) or name not in identifiers for name in names)
            or names != sorted(set(names))
        ):
            raise BoundaryModelError(f"requirement {requirement} has invalid case coverage")
    return {
        "cases": results,
        "raw_case_count": len(raw_results),
        "requirements": requirements,
        "schema": "whitefoot.frontend-boundary-primary.v1",
    }


def build_primary() -> dict[str, object]:
    descriptor, descriptor_raw = load_descriptor()
    contract, _ = candidate_contract()
    return {
        "candidate": contract,
        "descriptor": {
            "byte_length": len(descriptor_raw),
            "path": "grammar-verifier/proposal/frontend-boundary-cases.json",
            "sha256": sha256(descriptor_raw),
        },
        "evaluation": evaluate_descriptor(descriptor),
        "model": "primary typed-record boundary interpreter",
    }
