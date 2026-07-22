#!/usr/bin/env python3
"""Build, validate, and release-check exact-v0.9 discrepancy sidecars.

The public sidecar is descriptive and non-authorizing. Every validation and
release check reloads exact authorities, validates the catalog that supplies
the affected facet IDs, and recomputes the closed predicate registry.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Sequence

try:
    import discrepancy_inputs as authority
    import discrepancy_predicates as predicates
except ModuleNotFoundError:  # Support import as ``tools.*``.
    from tools import discrepancy_inputs as authority  # type: ignore
    from tools import discrepancy_predicates as predicates  # type: ignore


DiscrepancyError = authority.DiscrepancyError
ROOT = authority.ROOT
SPEC_PATH = authority.SPEC_PATH
SOURCE_INDEX_PATH = authority.SOURCE_INDEX_PATH
GUARD_BASELINE_PATH = authority.GUARD_BASELINE_PATH
MANIFEST_PATH = authority.MANIFEST_PATH
CASE_ROOT = authority.CASE_ROOT
SPEC_SHA256 = authority.SPEC_SHA256
SOURCE_INDEX_SHA256 = authority.SOURCE_INDEX_SHA256
GUARD_BASELINE_SHA256 = authority.GUARD_BASELINE_SHA256
LEGACY_ORPHAN_PATH = authority.LEGACY_ORPHAN_PATH
LEGACY_ORPHAN_SHA256 = authority.LEGACY_ORPHAN_SHA256
MAX_JSON_DEPTH = authority.MAX_JSON_DEPTH
MAX_SIDECAR_BYTES = authority.MAX_SIDECAR_BYTES
MAX_CATALOG_BYTES = authority.MAX_CATALOG_BYTES
FORMAT = "whitefoot-open-discrepancies-v1"
SIDECAR_RELATIVE_PATH = Path("tests") / "spec-catalogs" / "v0.9" / "open-discrepancies.json"
SIDECAR_PATH = ROOT / SIDECAR_RELATIVE_PATH

sha256 = authority.sha256
canonical_bytes = authority.canonical_bytes
strict_json_loads = authority.strict_json_loads


def parse_canonical_sidecar(raw: bytes) -> dict[str, Any]:
    """Parse a bounded sidecar and require its unique canonical bytes."""
    value = authority.strict_json_loads(
        raw,
        max_bytes=MAX_SIDECAR_BYTES,
        label="sidecar",
    )
    if not isinstance(value, dict):
        raise DiscrepancyError("sidecar must be a JSON object")
    if canonical_bytes(value) != raw:
        raise DiscrepancyError("sidecar bytes are not canonical")
    return value


def _require_fields(value: Any, expected: Iterable[str], where: str) -> None:
    if not isinstance(value, dict):
        raise DiscrepancyError(f"{where} must be an object")
    expected_set = set(expected)
    actual_set = set(value)
    if actual_set != expected_set:
        raise DiscrepancyError(
            f"{where} fields differ; unknown={sorted(actual_set - expected_set)}, "
            f"missing={sorted(expected_set - actual_set)}"
        )


@dataclass(frozen=True)
class ValidatedCatalog:
    """Exact catalog identity plus IDs derived from those same bytes."""

    sha256: str
    facet_ids: tuple[str, ...]


def _validate_static_catalog(
    catalog_bytes: bytes,
    inputs: authority.AuthorityInputs,
) -> ValidatedCatalog:
    """Rebuild the normalized catalog against exact v0.9 authorities."""
    authority.check_json_envelope(catalog_bytes, MAX_CATALOG_BYTES, "static catalog")
    try:
        try:
            import semantics
        except ModuleNotFoundError:
            from tools import semantics  # type: ignore

        catalog = semantics.parse_strict_json(
            catalog_bytes,
            "static catalog",
            max_bytes=MAX_CATALOG_BYTES,
        )
        if not isinstance(catalog, dict):
            raise DiscrepancyError("static catalog must be an object")
        rules = inputs.source_index.get("rules")
        if not isinstance(rules, list):
            raise DiscrepancyError("source index rules must be an array")
        rule_ids = []
        for record in rules:
            if not isinstance(record, dict) or not isinstance(
                record.get("rule_id"), str
            ):
                raise DiscrepancyError("source index contains an invalid rule record")
            rule_ids.append(record["rule_id"])
        fragment = {
            "schema": 1,
            "kind": "whitefoot-semantic-decomposition-fragment",
            "rules": sorted(rule_ids, key=str.encode),
            "clauses": catalog.get("clauses"),
            "facets": catalog.get("facets"),
        }
        rebuilt = semantics.build_static_catalog(
            [semantics.canonical_bytes(fragment)],
            inputs.specification,
            inputs.source_index_bytes,
        )
        rebuilt_bytes = semantics.canonical_bytes(rebuilt)
    except DiscrepancyError:
        raise
    except (
        MemoryError,
        OverflowError,
        RecursionError,
        semantics.SemanticCatalogError,
    ) as error:
        raise DiscrepancyError(f"invalid static catalog: {error}") from error
    if rebuilt_bytes != catalog_bytes:
        raise DiscrepancyError(
            "static catalog does not equal the exact normalized v0.9 catalog"
        )
    return ValidatedCatalog(
        authority.sha256(catalog_bytes),
        tuple(facet["id"] for facet in rebuilt["facets"]),
    )


def recompute_observations(
    root: Path = ROOT,
) -> dict[str, predicates.Observation]:
    """Recompute every registered predicate from exact authoritative bytes."""
    return predicates.recompute(authority.load_authorities(root))


def build_sidecar(catalog_bytes: bytes, root: Path = ROOT) -> dict[str, Any]:
    """Build the exact expected sidecar without writing it to disk."""
    inputs = authority.load_authorities(root)
    catalog = _validate_static_catalog(catalog_bytes, inputs)
    observations = predicates.recompute(inputs)
    registrations = predicates.validate_registry(observations)
    known_facets = set(catalog.facet_ids)
    records = []
    for identifier in sorted(registrations):
        registration = registrations[identifier]
        missing = sorted(set(registration.affected_facet_ids) - known_facets)
        if missing:
            raise DiscrepancyError(
                f"registered discrepancy references unknown facets: {missing}"
            )
        observation = observations[identifier]
        if observation.is_open:
            records.append(
                {
                    "affected_facet_ids": list(registration.affected_facet_ids),
                    "class": registration.discrepancy_class,
                    "evidence": observation.evidence,
                    "id": identifier,
                    "predicate_id": registration.predicate_identifier,
                    "resolution_authorities": list(
                        registration.resolution_authorities
                    ),
                }
            )
    document = {
        "bindings": {
            "catalog_sha256": catalog.sha256,
            "guard_baseline_sha256": authority.sha256(
                inputs.guard_baseline_bytes
            ),
            "source_index_sha256": authority.sha256(inputs.source_index_bytes),
            "specification_sha256": authority.sha256(inputs.specification),
        },
        "format": FORMAT,
        "records": records,
    }
    if len(canonical_bytes(document)) > MAX_SIDECAR_BYTES:
        raise DiscrepancyError("generated sidecar exceeds its byte limit")
    return document


@dataclass(frozen=True)
class AuditResult:
    """Descriptive result returned only by exact sidecar validation."""

    open_discrepancy_ids: tuple[str, ...]


def validate_sidecar(
    document: Any,
    catalog_bytes: bytes,
    root: Path = ROOT,
) -> AuditResult:
    """Validate schema, bindings, registrations, and recomputed evidence."""
    encoded = canonical_bytes(document)
    if len(encoded) > MAX_SIDECAR_BYTES:
        raise DiscrepancyError("sidecar object exceeds its byte limit")
    _require_fields(document, ("bindings", "format", "records"), "sidecar")
    _require_fields(
        document["bindings"],
        (
            "catalog_sha256",
            "guard_baseline_sha256",
            "source_index_sha256",
            "specification_sha256",
        ),
        "sidecar bindings",
    )
    for name, digest in document["bindings"].items():
        if not isinstance(digest, str) or authority.HEX_SHA256.fullmatch(digest) is None:
            raise DiscrepancyError(f"binding {name} is not a lowercase SHA-256")
    if document["format"] != FORMAT:
        raise DiscrepancyError(f"unknown sidecar format: {document['format']!r}")
    records = document["records"]
    if not isinstance(records, list):
        raise DiscrepancyError("sidecar records must be an array")
    registered_ids = {item.identifier for item in predicates.REGISTRATIONS}
    record_ids = []
    for index, record in enumerate(records):
        _require_fields(
            record,
            (
                "affected_facet_ids",
                "class",
                "evidence",
                "id",
                "predicate_id",
                "resolution_authorities",
            ),
            f"sidecar record {index}",
        )
        if not isinstance(record["id"], str):
            raise DiscrepancyError(f"sidecar record {index} id must be a string")
        if record["id"] not in registered_ids:
            raise DiscrepancyError(f"unregistered discrepancy: {record['id']}")
        record_ids.append(record["id"])
    if record_ids != sorted(record_ids) or len(record_ids) != len(set(record_ids)):
        raise DiscrepancyError("sidecar record ids must be unique and sorted")

    expected = build_sidecar(catalog_bytes, root)
    if encoded != canonical_bytes(expected):
        raise DiscrepancyError(
            "sidecar does not equal the recomputed registered discrepancy document"
        )
    return AuditResult(tuple(record_ids))


def parse_and_validate_sidecar(
    raw: bytes,
    catalog_bytes: bytes,
    root: Path = ROOT,
) -> AuditResult:
    """Parse canonical bytes and validate their recomputed document."""
    return validate_sidecar(parse_canonical_sidecar(raw), catalog_bytes, root)


def require_no_open_discrepancies(
    sidecar_bytes: bytes,
    catalog_bytes: bytes,
    root: Path = ROOT,
) -> None:
    """Recompute exact inputs before enforcing the release implication."""
    result = parse_and_validate_sidecar(sidecar_bytes, catalog_bytes, root)
    if result.open_discrepancy_ids:
        raise DiscrepancyError(
            "release is blocked by open discrepancies: "
            + ", ".join(result.open_discrepancy_ids)
        )


def _live_catalog_bytes(root: Path) -> bytes:
    """Build the canonical catalog from the fixed repository inputs."""
    try:
        try:
            import semantics
        except ModuleNotFoundError:
            from tools import semantics  # type: ignore

        return semantics.canonical_bytes(
            semantics.build_from_files(root)
        )
    except DiscrepancyError:
        raise
    except (
        MemoryError,
        OverflowError,
        RecursionError,
        semantics.SemanticCatalogError,
    ) as error:
        raise DiscrepancyError(f"cannot build live static catalog: {error}") from error


def generated_sidecar_bytes(root: Path = ROOT) -> bytes:
    """Return the canonical sidecar derived from all live authorities."""
    return canonical_bytes(build_sidecar(_live_catalog_bytes(root), root))


def check_repository_sidecar(root: Path = ROOT) -> AuditResult:
    """Validate the checked-in sidecar against freshly loaded authorities."""
    raw = authority.read_regular(
        root,
        root / SIDECAR_RELATIVE_PATH,
        "open-discrepancy sidecar",
        MAX_SIDECAR_BYTES,
    )
    return parse_and_validate_sidecar(raw, _live_catalog_bytes(root), root)


def write_repository_sidecar(root: Path = ROOT) -> str:
    """Mechanically rewrite the descriptive sidecar from exact authorities."""
    parent = root / SIDECAR_RELATIVE_PATH.parent
    authority.require_directory(root, parent, "open-discrepancy sidecar directory")
    raw = generated_sidecar_bytes(root)
    (root / SIDECAR_RELATIVE_PATH).write_bytes(raw)
    return sha256(raw)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=("check", "write"),
        nargs="?",
        default="check",
    )
    arguments = parser.parse_args(argv)
    if arguments.command == "write":
        print(write_repository_sidecar())
        return 0
    result = check_repository_sidecar()
    print(f"{len(result.open_discrepancy_ids)} open discrepancies")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, DiscrepancyError) as error:
        print(f"facet discrepancies: {error}", file=__import__("sys").stderr)
        raise SystemExit(1)
