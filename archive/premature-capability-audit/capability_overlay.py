#!/usr/bin/env python3
"""Validate the non-authorizing Rust capability overlay.

The overlay is repository audit metadata. It may name generic handler
responsibilities and exact evidence-receipt identities, but it cannot state a
verdict. This foundation has no evidence replay provider, so references remain
unresolved and grant no capability. Production compiler code must not consume
the overlay or any derived report.
"""

from __future__ import annotations

import argparse
import hashlib
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Sequence

try:
    import facet_discrepancies
    import facet_discrepancy_inputs as discrepancy_inputs
    import semantic_catalog
    import semantic_catalog_io
except ModuleNotFoundError:  # Support import as ``tools.*``.
    from tools import facet_discrepancies  # type: ignore
    from tools import facet_discrepancy_inputs as discrepancy_inputs  # type: ignore
    from tools import semantic_catalog  # type: ignore
    from tools import semantic_catalog_io  # type: ignore


ROOT = Path(__file__).resolve().parent.parent
FRAGMENT_DIRECTORY = ("capabilities", "whitefoot-rust", "v0.9")
FORMAT = "whitefoot-capability-fragment-v1"
IMPLEMENTATION_ID = "whitefoot-rust"

MAX_FRAGMENT_COUNT = 128
MAX_DIRECTORY_ENTRIES = 512
MAX_FRAGMENT_BYTES = 2_000_000
MAX_FRAGMENT_TOTAL_BYTES = 16_000_000
MAX_HANDLER_COUNT = 4_096
MAX_EVIDENCE_REFERENCE_COUNT = 16_384
MAX_HANDLER_FACETS = 4_096

FRAGMENT_KEYS = {
    "catalog_sha256",
    "evidence",
    "format",
    "handlers",
    "implementation_id",
}
HANDLER_KEYS = {"facet_ids", "id", "lane"}
EVIDENCE_REFERENCE_KEYS = {"id", "receipt_sha256"}

HANDLER_ID = re.compile(r"handler:[a-z][a-z0-9-]*(?:/[a-z][a-z0-9-]*)+\Z")
RECEIPT_ID = re.compile(r"receipt:[a-z][a-z0-9-]*(?:/[a-z][a-z0-9-]*)+\Z")
HEX_SHA256 = re.compile(r"[0-9a-f]{64}\Z")


class CapabilityOverlayError(ValueError):
    """The overlay or its derived audit state is invalid."""


def fail(message: str) -> None:
    raise CapabilityOverlayError(message)


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_bytes(value: Any, label: str = "capability value") -> bytes:
    """Encode one bounded canonical overlay value."""
    try:
        return semantic_catalog_io.canonical_json_bytes(
            value,
            label,
            semantic_catalog.MAX_NORMALIZED_OUTPUT_BYTES,
            max_depth=semantic_catalog.MAX_JSON_DEPTH,
            max_integer_digits=semantic_catalog.MAX_INTEGER_DIGITS,
            max_string_bytes=semantic_catalog.MAX_JSON_STRING_BYTES,
            max_list_items=semantic_catalog.MAX_JSON_LIST_ITEMS,
            max_object_fields=semantic_catalog.MAX_JSON_OBJECT_FIELDS,
            max_nodes=semantic_catalog.MAX_JSON_NODES,
        )
    except semantic_catalog_io.CatalogIOError as error:
        fail(str(error))


def parse_fragment_bytes(raw: bytes, label: str) -> dict[str, Any]:
    """Parse one canonical bounded capability fragment."""
    try:
        value = semantic_catalog_io.parse_canonical_json(
            raw,
            label,
            max_bytes=MAX_FRAGMENT_BYTES,
            max_depth=semantic_catalog.MAX_JSON_DEPTH,
            max_integer_digits=semantic_catalog.MAX_INTEGER_DIGITS,
            max_string_bytes=semantic_catalog.MAX_JSON_STRING_BYTES,
            max_list_items=semantic_catalog.MAX_JSON_LIST_ITEMS,
            max_object_fields=semantic_catalog.MAX_JSON_OBJECT_FIELDS,
            max_nodes=semantic_catalog.MAX_JSON_NODES,
        )
    except semantic_catalog_io.CatalogIOError as error:
        fail(str(error))
    if not isinstance(value, dict):
        fail(f"{label} must contain one object")
    return value


def _object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{label} must be an object")
    return value


def _array(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{label} must be an array")
    return value


def _string(value: Any, label: str) -> str:
    if not isinstance(value, str):
        fail(f"{label} must be a string")
    try:
        value.encode("ascii")
    except UnicodeEncodeError as error:
        raise CapabilityOverlayError(f"{label} must be ASCII") from error
    return value


def _exact_keys(value: Mapping[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        fail(
            f"{label} fields differ; unknown={sorted(actual - expected)}, "
            f"missing={sorted(expected - actual)}"
        )


def _sorted_unique(values: Sequence[str], label: str) -> None:
    if list(values) != sorted(set(values), key=str.encode):
        fail(f"{label} must be a bytewise-sorted unique list")


@dataclass(frozen=True)
class Handler:
    """One generic implementation responsibility in exactly one lane."""

    identifier: str
    lane: str
    facet_ids: tuple[str, ...]


@dataclass(frozen=True)
class EvidenceReference:
    """An opaque receipt identity; no reference is evidence by itself."""

    identifier: str
    receipt_sha256: str


@dataclass(frozen=True)
class _ValidatedOverlay:
    """Structurally checked claims with no admitted evidence."""

    catalog_sha256: str
    implementation_id: str
    handlers: tuple[Handler, ...]
    evidence_references: tuple[EvidenceReference, ...]


@dataclass(frozen=True)
class FacetCapability:
    """Derived missing work and blockers; no field comes from a verdict."""

    facet_id: str
    missing_handler_lanes: tuple[str, ...]
    unexercised_lanes: tuple[str, ...]
    missing_evidence_classes: tuple[str, ...]
    blocking_discrepancy_ids: tuple[str, ...]

    @property
    def is_closed(self) -> bool:
        """Return whether every independent requirement is currently met."""
        return not (
            self.missing_handler_lanes
            or self.unexercised_lanes
            or self.missing_evidence_classes
            or self.blocking_discrepancy_ids
        )


@dataclass(frozen=True)
class CapabilityReport:
    """Read-only audit output; it is never accepted as release authority."""

    facets: tuple[FacetCapability, ...]
    open_discrepancy_ids: tuple[str, ...]
    unresolved_receipt_ids: tuple[str, ...]

    def explain(self, facet_id: str) -> FacetCapability:
        """Return one exact facet result or fail for an unknown identifier."""
        for facet in self.facets:
            if facet.facet_id == facet_id:
                return facet
        fail(f"unknown facet in capability report: {facet_id}")

    @property
    def closed_facet_ids(self) -> tuple[str, ...]:
        return tuple(facet.facet_id for facet in self.facets if facet.is_closed)

    @property
    def blocked_facet_ids(self) -> tuple[str, ...]:
        return tuple(
            facet.facet_id
            for facet in self.facets
            if facet.blocking_discrepancy_ids
        )


def _blocked_facets(
    sidecar: Mapping[str, Any],
) -> tuple[dict[str, tuple[str, ...]], tuple[str, ...]]:
    blockers: dict[str, list[str]] = {}
    identifiers = []
    for ordinal, raw_record in enumerate(_array(sidecar.get("records"), "sidecar records")):
        record = _object(raw_record, f"sidecar record {ordinal}")
        identifier = _string(record.get("id"), f"sidecar record {ordinal} id")
        identifiers.append(identifier)
        for raw_facet_id in _array(
            record.get("affected_facet_ids"),
            f"sidecar record {ordinal} affected_facet_ids",
        ):
            facet_id = _string(raw_facet_id, "affected facet ID")
            blockers.setdefault(facet_id, []).append(identifier)
    _sorted_unique(identifiers, "sidecar record IDs")
    return (
        {
            facet: tuple(sorted(values, key=str.encode))
            for facet, values in blockers.items()
        },
        tuple(identifiers),
    )


def _catalog_facet_map(catalog: Mapping[str, Any]) -> dict[str, dict[str, Any]]:
    facets: dict[str, dict[str, Any]] = {}
    for ordinal, raw_facet in enumerate(_array(catalog.get("facets"), "catalog facets")):
        facet = _object(raw_facet, f"catalog facet {ordinal}")
        identifier = _string(facet.get("id"), f"catalog facet {ordinal} id")
        if identifier in facets:
            fail(f"duplicate catalog facet ID: {identifier}")
        facets[identifier] = facet
    return facets


def _validate_fragments(
    fragment_bytes: Sequence[tuple[str, bytes]],
    catalog: Mapping[str, Any],
    sidecar: Mapping[str, Any],
) -> _ValidatedOverlay:
    """Validate claims against supplied already-validated test authorities."""
    if not fragment_bytes:
        fail("no capability fragments supplied")
    if len(fragment_bytes) > MAX_FRAGMENT_COUNT:
        fail(f"capability fragment count exceeds {MAX_FRAGMENT_COUNT}")

    catalog_sha256 = sha256(semantic_catalog.canonical_bytes(catalog))
    facets = _catalog_facet_map(catalog)
    blocked, _ = _blocked_facets(sidecar)
    handlers: list[Handler] = []
    references: list[EvidenceReference] = []
    handler_ids: set[str] = set()
    receipt_ids: set[str] = set()
    receipt_digests: set[str] = set()
    responsibility_owner: dict[tuple[str, str], str] = {}

    for fragment_name, raw in fragment_bytes:
        fragment = parse_fragment_bytes(raw, f"capability fragment {fragment_name}")
        _exact_keys(fragment, FRAGMENT_KEYS, f"capability fragment {fragment_name}")
        if fragment.get("format") != FORMAT:
            fail(f"capability fragment {fragment_name} has an unknown format")
        if fragment.get("implementation_id") != IMPLEMENTATION_ID:
            fail(f"capability fragment {fragment_name} has an unknown implementation")
        if fragment.get("catalog_sha256") != catalog_sha256:
            fail(f"capability fragment {fragment_name} has a stale catalog binding")

        raw_handlers = _array(
            fragment.get("handlers"), f"capability fragment {fragment_name} handlers"
        )
        fragment_handler_ids = []
        for ordinal, raw_handler in enumerate(raw_handlers):
            label = f"capability fragment {fragment_name} handler {ordinal}"
            handler = _object(raw_handler, label)
            _exact_keys(handler, HANDLER_KEYS, label)
            identifier = _string(handler.get("id"), f"{label} id")
            if HANDLER_ID.fullmatch(identifier) is None:
                fail(f"{label} has an invalid generic handler ID")
            if identifier in handler_ids:
                fail(f"duplicate capability handler ID: {identifier}")
            lane = _string(handler.get("lane"), f"{label} lane")
            if lane not in semantic_catalog.LANES:
                fail(f"{label} names an unknown lane: {lane}")
            facet_ids = tuple(
                _string(value, f"{label} facet ID")
                for value in _array(handler.get("facet_ids"), f"{label} facet_ids")
            )
            if not facet_ids or len(facet_ids) > MAX_HANDLER_FACETS:
                fail(f"{label} must name 1..{MAX_HANDLER_FACETS} facets")
            _sorted_unique(facet_ids, f"{label} facet_ids")
            for facet_id in facet_ids:
                facet = facets.get(facet_id)
                if facet is None:
                    fail(f"{label} names an unknown facet: {facet_id}")
                if facet_id in blocked:
                    fail(f"{label} claims discrepancy-blocked facet: {facet_id}")
                if lane not in facet.get("required_lanes", []):
                    fail(f"{label} claims an unrequired lane for {facet_id}: {lane}")
                key = (facet_id, lane)
                if key in responsibility_owner:
                    fail(f"facet lane has multiple handlers: {facet_id} {lane}")
                responsibility_owner[key] = identifier
            handlers.append(Handler(identifier, lane, facet_ids))
            handler_ids.add(identifier)
            fragment_handler_ids.append(identifier)
            if len(handlers) > MAX_HANDLER_COUNT:
                fail(f"capability handler count exceeds {MAX_HANDLER_COUNT}")
        _sorted_unique(fragment_handler_ids, f"{fragment_name} handler IDs")

        raw_evidence = _array(
            fragment.get("evidence"), f"capability fragment {fragment_name} evidence"
        )
        fragment_receipt_ids = []
        for ordinal, raw_reference in enumerate(raw_evidence):
            label = f"capability fragment {fragment_name} evidence {ordinal}"
            reference = _object(raw_reference, label)
            _exact_keys(reference, EVIDENCE_REFERENCE_KEYS, label)
            identifier = _string(reference.get("id"), f"{label} id")
            if RECEIPT_ID.fullmatch(identifier) is None:
                fail(f"{label} has an invalid receipt ID")
            digest = _string(reference.get("receipt_sha256"), f"{label} receipt_sha256")
            if HEX_SHA256.fullmatch(digest) is None:
                fail(f"{label} has an invalid receipt SHA-256")
            if identifier in receipt_ids:
                fail(f"duplicate evidence receipt ID: {identifier}")
            if digest in receipt_digests:
                fail(f"aliased evidence receipt digest: {digest}")
            receipt_ids.add(identifier)
            receipt_digests.add(digest)
            references.append(EvidenceReference(identifier, digest))
            fragment_receipt_ids.append(identifier)
            if len(references) > MAX_EVIDENCE_REFERENCE_COUNT:
                fail(
                    "capability evidence-reference count exceeds "
                    f"{MAX_EVIDENCE_REFERENCE_COUNT}"
                )
        _sorted_unique(fragment_receipt_ids, f"{fragment_name} evidence IDs")

    handlers.sort(key=lambda value: value.identifier.encode("ascii"))
    references.sort(key=lambda value: value.identifier.encode("ascii"))
    return _ValidatedOverlay(
        catalog_sha256,
        IMPLEMENTATION_ID,
        tuple(handlers),
        tuple(references),
    )


def _derive_report(
    catalog: Mapping[str, Any],
    sidecar: Mapping[str, Any],
    overlay: _ValidatedOverlay,
) -> CapabilityReport:
    """Derive fail-closed state while no evidence replay provider exists."""
    blocked, discrepancy_ids = _blocked_facets(sidecar)
    handled = {
        (facet_id, handler.lane)
        for handler in overlay.handlers
        for facet_id in handler.facet_ids
    }
    results = []
    for raw_facet in _array(catalog.get("facets"), "catalog facets"):
        facet = _object(raw_facet, "catalog facet")
        facet_id = _string(facet.get("id"), "catalog facet ID")
        required_lanes = tuple(facet.get("required_lanes", []))
        missing_handlers = tuple(
            lane for lane in required_lanes if (facet_id, lane) not in handled
        )
        unexercised = tuple(
            lane for lane in required_lanes if (facet_id, lane) in handled
        )
        results.append(
            FacetCapability(
                facet_id,
                missing_handlers,
                unexercised,
                tuple(facet.get("required_evidence", [])),
                blocked.get(facet_id, ()),
            )
        )
    return CapabilityReport(
        tuple(results),
        discrepancy_ids,
        tuple(reference.identifier for reference in overlay.evidence_references),
    )


def _repository_fragments(root: Path) -> tuple[tuple[str, bytes], ...]:
    try:
        return semantic_catalog_io.read_fragment_directory(
            root,
            FRAGMENT_DIRECTORY,
            label="capability overlay",
            max_entries=MAX_DIRECTORY_ENTRIES,
            max_count=MAX_FRAGMENT_COUNT,
            max_file_bytes=MAX_FRAGMENT_BYTES,
            max_total_bytes=MAX_FRAGMENT_TOTAL_BYTES,
        )
    except semantic_catalog_io.CatalogIOError as error:
        fail(str(error))


def audit_repository(root: Path = ROOT) -> CapabilityReport:
    """Rebuild exact authorities and audit the live Rust overlay."""
    try:
        catalog = semantic_catalog.build_from_files(root)
        catalog_bytes = semantic_catalog.canonical_bytes(catalog)
        sidecar_raw = discrepancy_inputs.read_regular(
            root,
            root / facet_discrepancies.SIDECAR_RELATIVE_PATH,
            "open-discrepancy sidecar",
            discrepancy_inputs.MAX_SIDECAR_BYTES,
        )
        facet_discrepancies.parse_and_validate_sidecar(sidecar_raw, catalog_bytes, root)
        sidecar = facet_discrepancies.parse_canonical_sidecar(sidecar_raw)
    except (
        CapabilityOverlayError,
        discrepancy_inputs.DiscrepancyError,
        semantic_catalog.SemanticCatalogError,
    ) as error:
        fail(str(error))
    overlay = _validate_fragments(_repository_fragments(root), catalog, sidecar)
    return _derive_report(catalog, sidecar, overlay)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check",), nargs="?", default="check")
    parser.parse_args(argv)
    report = audit_repository()
    if report.closed_facet_ids:
        fail("capability closed without a registered evidence replay provider")
    print(
        "capability overlay: exact catalog binding verified; no capability "
        "claims admitted; release remains blocked"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, CapabilityOverlayError) as error:
        print(f"capability overlay: {error}", file=__import__("sys").stderr)
        raise SystemExit(1)
