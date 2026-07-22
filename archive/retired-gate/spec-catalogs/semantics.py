#!/usr/bin/env python3
"""Validate authored v0.9 semantic decomposition and build its static catalog.

This module owns only compiler-independent static metadata.  It deliberately
has no vocabulary for implementation status, handlers, tests, or witnesses.
"""

from __future__ import annotations

import argparse
import hashlib
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Mapping, Sequence

import semantics_io


ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = ROOT / "spec" / "kernel-spec-v0.9.md"
SOURCE_INDEX_PATH = ROOT / "tests" / "spec-catalogs" / "v0.9" / "source.json"
DECOMPOSITION_PATH = ROOT / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
SPEC_RELATIVE_PATH = "spec/kernel-spec-v0.9.md"
SOURCE_INDEX_RELATIVE_PATH = "facets/v0.9/source.json"
SPEC_VERSION = "0.9"
SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
SOURCE_INDEX_SHA256 = "cc9aa86de0d59b9288d1f8fd7a6bde6f94fff26da139d73f91bcbcf71219d663"
SPEC_BYTE_LENGTH = 98_044
SOURCE_INDEX_BYTE_LENGTH = 81_996

# Parsing and normalization are deliberately bounded.  The exact authorities
# are much smaller than these ceilings, leaving room for the complete authored
# decomposition without making hostile inputs an unbounded resource request.
MAX_JSON_DEPTH = 64
MAX_INTEGER_DIGITS = 12
MAX_JSON_STRING_BYTES = 16_384
MAX_JSON_LIST_ITEMS = 8_192
MAX_JSON_OBJECT_FIELDS = 128
MAX_JSON_NODES = 100_000
MAX_FRAGMENT_COUNT = 128
MAX_FRAGMENT_DIRECTORY_ENTRIES = 512
MAX_FRAGMENT_BYTES = 2_000_000
MAX_FRAGMENT_TOTAL_BYTES = 16_000_000
MAX_CLAUSE_COUNT = 4_096
MAX_FACET_COUNT = 4_096
MAX_NORMALIZED_OUTPUT_BYTES = 16_000_000

# These exclusions are reviewed exact-byte decisions bound to SPEC_SHA256.
# Marker words in prose have no authority to expand this table.
REVIEWED_DISPOSITIONS = (
    (
        "explicitly-deferred",
        "FORM-5",
        15_749,
        15_788,
        "0cbec3066cc7e216554e03b29f95f8664198f04cdbd6df0e0c7a97e791a49ed4",
    ),
    (
        "explicitly-deferred",
        "LEX-1",
        17_941,
        18_098,
        "23d98d39550622dd24d013800c1262209082accd33b73fa02ac3d840d7c4bacb",
    ),
    (
        "explicitly-nonnormative",
        "OWN-9",
        41_341,
        41_809,
        "1853bc29443f66b3e54c800026931ba0e380278a6a42980768580457b2d912a2",
    ),
)

STAGES = (
    "source-envelope",
    "canonical-tree",
    "declaration-resolution",
    "semantic-check",
    "whole-unit",
    "checked-artifact",
    "facts-off-lowering",
    "runtime",
    "diagnostic-report",
    "toolchain-policy",
)
LANES = (
    "frontend",
    "resolution",
    "checker",
    "artifact",
    "verifier",
    "lowerer",
    "runtime",
    "diagnostic",
    "report",
    "governance",
)
EVIDENCE_CLASSES = (
    "conformance-accept",
    "conformance-reject",
    "conformance-run",
    "conformance-trap",
    "diagnostic",
    "artifact-valid",
    "artifact-hostile",
    "reference-model",
    "property",
    "fuzz",
    "differential",
    "determinism",
    "resource",
    "abi",
    "code-shape",
    "target",
    "static-audit",
)
DISPOSITIONS = ("facet", "explicitly-deferred", "explicitly-nonnormative")

FRAGMENT_KEYS = {"schema", "kind", "rules", "clauses", "facets"}
CLAUSE_KEYS = {
    "owner_rule",
    "byte_start",
    "byte_end",
    "sha256",
    "disposition",
    "facet_ids",
}
FACET_KEYS = {
    "id",
    "owner_rule",
    "source_atoms",
    "owner_stage",
    "required_lanes",
    "required_evidence",
}
SOURCE_KEYS = {"byte_start", "byte_end", "line_start", "line_end", "sha256"}
SOURCE_INDEX_KEYS = {
    "byte_exact_fences",
    "counts",
    "generated_by",
    "kind",
    "operation_name_sets",
    "operation_rows",
    "report_rows",
    "rules",
    "schema",
    "scope",
    "specification",
    "syntax_productions",
}
CHILD_COLLECTIONS = (
    "syntax_productions",
    "operation_rows",
    "report_rows",
    "byte_exact_fences",
)

RULE_ID = re.compile(r"[A-Z]+-[0-9]+[a-z]?\Z")
FACET_ID = re.compile(
    r"facet:([A-Z]+-[0-9]+[a-z]?)/([a-z][a-z0-9]*(?:-[a-z0-9]+)*)\Z"
)
SHA256 = re.compile(r"[0-9a-f]{64}\Z")


class SemanticCatalogError(ValueError):
    """An input violates the closed static-catalog contract."""


def fail(message: str) -> None:
    """Raise a consistently typed contract failure."""
    raise SemanticCatalogError(message)


def sha256(data: bytes) -> str:
    """Return the lowercase SHA-256 identity of exact bytes."""
    return hashlib.sha256(data).hexdigest()


def canonical_bytes(
    value: Any, *, label: str = "JSON value", max_bytes: int | None = None
) -> bytes:
    """Encode one canonical ASCII JSON document with exactly one final LF."""
    ceiling = MAX_NORMALIZED_OUTPUT_BYTES if max_bytes is None else max_bytes
    try:
        return semantics_io.canonical_json_bytes(
            value,
            label,
            ceiling,
            max_depth=MAX_JSON_DEPTH,
            max_integer_digits=MAX_INTEGER_DIGITS,
            max_string_bytes=MAX_JSON_STRING_BYTES,
            max_list_items=MAX_JSON_LIST_ITEMS,
            max_object_fields=MAX_JSON_OBJECT_FIELDS,
            max_nodes=MAX_JSON_NODES,
        )
    except semantics_io.CatalogIOError as error:
        fail(str(error))


def parse_strict_json(
    raw: bytes, label: str = "JSON input", *, max_bytes: int = MAX_FRAGMENT_BYTES
) -> Any:
    """Parse one canonical ASCII JSON document and reject alternate encodings."""
    try:
        return semantics_io.parse_canonical_json(
            raw,
            label,
            max_bytes=max_bytes,
            max_depth=MAX_JSON_DEPTH,
            max_integer_digits=MAX_INTEGER_DIGITS,
            max_string_bytes=MAX_JSON_STRING_BYTES,
            max_list_items=MAX_JSON_LIST_ITEMS,
            max_object_fields=MAX_JSON_OBJECT_FIELDS,
            max_nodes=MAX_JSON_NODES,
        )
    except semantics_io.CatalogIOError as error:
        fail(str(error))


def parse_fragment_bytes(raw: bytes, label: str = "decomposition fragment") -> dict[str, Any]:
    """Parse one canonical authored fragment; semantic validation is global."""
    value = parse_strict_json(raw, label)
    if not isinstance(value, dict):
        fail(f"{label} must be a JSON object")
    return value


def _exact_keys(value: Mapping[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        fail(
            f"{label} keys differ: missing={sorted(expected - actual)!r}, "
            f"unknown={sorted(actual - expected)!r}"
        )


def _ascii_string(value: Any, label: str) -> str:
    if not isinstance(value, str):
        fail(f"{label} must be a string")
    try:
        value.encode("ascii")
    except UnicodeEncodeError:
        fail(f"{label} must be ASCII")
    return value


def _integer(value: Any, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        fail(f"{label} must be an integer")
    return value


def _object(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        fail(f"{label} must be an object")
    return value


def _array(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{label} must be an array")
    return value


def _strict_strings(value: Any, label: str) -> list[str]:
    return [
        _ascii_string(item, f"{label}[{index}]")
        for index, item in enumerate(_array(value, label))
    ]


def _require_ordered_unique(
    values: Sequence[str], expected_order: Sequence[str], label: str, *, nonempty: bool = False
) -> None:
    if nonempty and not values:
        fail(f"{label} must not be empty")
    duplicates = sorted(name for name, count in Counter(values).items() if count != 1)
    if duplicates:
        fail(f"{label} contains duplicates: {duplicates!r}")
    ranks = {value: index for index, value in enumerate(expected_order)}
    unknown = [value for value in values if value not in ranks]
    if unknown:
        fail(f"{label} contains unknown values: {unknown!r}")
    if list(values) != sorted(values, key=ranks.__getitem__):
        fail(f"{label} is not in required vocabulary order")


def _validate_source_span(
    source: Any, specification: bytes, label: str
) -> tuple[int, int]:
    value = _object(source, label)
    _exact_keys(value, SOURCE_KEYS, label)
    start = _integer(value["byte_start"], f"{label}.byte_start")
    end = _integer(value["byte_end"], f"{label}.byte_end")
    _integer(value["line_start"], f"{label}.line_start")
    _integer(value["line_end"], f"{label}.line_end")
    digest = _ascii_string(value["sha256"], f"{label}.sha256")
    if not (0 <= start < end <= len(specification)):
        fail(f"{label} is outside the exact specification")
    if not SHA256.fullmatch(digest) or sha256(specification[start:end]) != digest:
        fail(f"{label} has a stale source hash")
    return start, end


def _source_index_model(
    source_index: Any, specification: bytes
) -> tuple[
    list[str],
    dict[str, tuple[int, int]],
    list[str],
    dict[str, str],
]:
    index = _object(source_index, "source index")
    _exact_keys(index, SOURCE_INDEX_KEYS, "source index")
    if _integer(index["schema"], "source index schema") != 1:
        fail("unsupported source-index schema")
    if index["kind"] != "whitefoot-normative-source-index":
        fail("wrong source-index kind")
    specification_record = _object(index["specification"], "source index specification")
    _exact_keys(
        specification_record,
        {"byte_length", "path", "sha256", "version"},
        "source index specification",
    )
    if specification_record != {
        "byte_length": len(specification),
        "path": SPEC_RELATIVE_PATH,
        "sha256": SPEC_SHA256,
        "version": SPEC_VERSION,
    }:
        fail("source index does not bind the exact v0.9 specification")
    if sha256(specification) != SPEC_SHA256:
        fail("specification hash is not exact v0.9")

    rule_order: list[str] = []
    rule_spans: dict[str, tuple[int, int]] = {}
    all_atom_ids: set[str] = set()
    atom_owner: dict[str, str] = {}
    positioned_atoms: list[tuple[int, int, str]] = []
    for ordinal, raw_rule in enumerate(_array(index["rules"], "source index rules")):
        rule = _object(raw_rule, f"source index rule {ordinal}")
        _exact_keys(rule, {"id", "rule_id", "section", "source"}, f"source index rule {ordinal}")
        rule_id = _ascii_string(rule["rule_id"], f"source index rule {ordinal}.rule_id")
        atom_id = _ascii_string(rule["id"], f"source index rule {ordinal}.id")
        if not RULE_ID.fullmatch(rule_id) or atom_id != f"rule:{rule_id}":
            fail(f"invalid source-index rule identity: {atom_id!r}")
        if rule_id in rule_spans or atom_id in all_atom_ids:
            fail(f"duplicate source-index rule: {rule_id}")
        rule_span = _validate_source_span(
            rule["source"], specification, f"source index rule {rule_id}.source"
        )
        rule_spans[rule_id] = rule_span
        rule_order.append(rule_id)
        all_atom_ids.add(atom_id)
        atom_owner[atom_id] = rule_id
        positioned_atoms.append((rule_span[0], rule_span[1], atom_id))
    if len(rule_order) != 92:
        fail(f"source index must contain 92 rules, found {len(rule_order)}")

    positioned_children: list[tuple[int, int, str]] = []
    for collection_name in CHILD_COLLECTIONS:
        collection = _array(index[collection_name], f"source index {collection_name}")
        for ordinal, raw_atom in enumerate(collection):
            atom = _object(raw_atom, f"{collection_name}[{ordinal}]")
            atom_id = _ascii_string(atom.get("id"), f"{collection_name}[{ordinal}].id")
            owner = _ascii_string(
                atom.get("owner_rule"), f"{collection_name}[{ordinal}].owner_rule"
            )
            if atom_id in all_atom_ids:
                fail(f"duplicate source atom ID: {atom_id}")
            if owner not in rule_spans:
                fail(f"source atom {atom_id} has unknown owner {owner}")
            start, end = _validate_source_span(
                atom.get("source"), specification, f"source atom {atom_id}.source"
            )
            rule_start, rule_end = rule_spans[owner]
            if not (rule_start <= start < end <= rule_end):
                fail(f"source atom {atom_id} escapes owning rule {owner}")
            all_atom_ids.add(atom_id)
            atom_owner[atom_id] = owner
            positioned_children.append((start, end, atom_id))
    positioned_children.sort(key=lambda item: (item[0], item[1], item[2].encode("ascii")))
    positioned_atoms.extend(positioned_children)
    positioned_atoms.sort(key=lambda item: (item[0], item[1], item[2].encode("ascii")))
    atom_order = [item[2] for item in positioned_atoms]
    return rule_order, rule_spans, atom_order, atom_owner


def _validate_fragment_shell(fragment: Any, index: int) -> Mapping[str, Any]:
    value = _object(fragment, f"fragment {index}")
    _exact_keys(value, FRAGMENT_KEYS, f"fragment {index}")
    if _integer(value["schema"], f"fragment {index}.schema") != 1:
        fail(f"fragment {index} has unsupported schema")
    if value["kind"] != "whitefoot-semantic-decomposition-fragment":
        fail(f"fragment {index} has wrong kind")
    return value


def _require_exact_authority(raw: bytes, label: str, length: int, digest: str) -> None:
    if type(raw) is not bytes:
        fail(f"{label} must be exact bytes")
    if len(raw) != length:
        fail(f"{label} byte length is not exact v0.9: {len(raw)} != {length}")
    if sha256(raw) != digest:
        fail(f"{label} hash is not exact v0.9")


def _reviewed_disposition_set(
    specification: bytes,
    rule_spans: Mapping[str, tuple[int, int]],
) -> frozenset[tuple[str, str, int, int, str]]:
    reviewed: set[tuple[str, str, int, int, str]] = set()
    for entry in REVIEWED_DISPOSITIONS:
        disposition, owner, start, end, digest = entry
        if disposition not in DISPOSITIONS[1:]:
            fail(f"internal reviewed disposition has invalid kind: {disposition}")
        if owner not in rule_spans:
            fail(f"internal reviewed disposition has unknown owner: {owner}")
        rule_start, rule_end = rule_spans[owner]
        if not (rule_start <= start < end <= rule_end):
            fail(f"internal reviewed disposition escapes {owner}")
        if sha256(specification[start:end]) != digest:
            fail(f"internal reviewed disposition hash is stale for {owner}")
        if entry in reviewed:
            fail(f"duplicate internal reviewed disposition for {owner}")
        reviewed.add(entry)
    return frozenset(reviewed)


def _validate_decomposition(
    fragment_bytes: Sequence[bytes],
    specification: bytes,
    source_index_bytes: bytes,
    *,
    require_complete: bool,
) -> tuple[dict[str, Any] | None, dict[str, Any]]:
    """Validate exact-byte inputs and return a catalog or partial audit."""
    _require_exact_authority(
        specification, "specification", SPEC_BYTE_LENGTH, SPEC_SHA256
    )
    _require_exact_authority(
        source_index_bytes,
        "source index",
        SOURCE_INDEX_BYTE_LENGTH,
        SOURCE_INDEX_SHA256,
    )
    source_index = parse_strict_json(
        source_index_bytes,
        "source index",
        max_bytes=SOURCE_INDEX_BYTE_LENGTH,
    )
    rule_order, rule_spans, atom_order, atom_owner = _source_index_model(
        source_index, specification
    )
    reviewed_dispositions = _reviewed_disposition_set(specification, rule_spans)

    if isinstance(fragment_bytes, (bytes, bytearray, str)) or not isinstance(
        fragment_bytes, Sequence
    ):
        fail("fragments must be a sequence of exact canonical JSON byte documents")
    if not fragment_bytes:
        fail("at least one decomposition fragment is required")
    if len(fragment_bytes) > MAX_FRAGMENT_COUNT:
        fail(f"fragment count exceeds the {MAX_FRAGMENT_COUNT}-file limit")
    total_fragment_bytes = 0
    fragments: list[Mapping[str, Any]] = []
    for index, raw in enumerate(fragment_bytes):
        if type(raw) is not bytes:
            fail(f"fragment {index} must be exact bytes")
        if len(raw) > MAX_FRAGMENT_BYTES:
            fail(f"fragment {index} exceeds the {MAX_FRAGMENT_BYTES}-byte limit")
        total_fragment_bytes += len(raw)
        if total_fragment_bytes > MAX_FRAGMENT_TOTAL_BYTES:
            fail(
                "fragment bytes exceed the "
                f"{MAX_FRAGMENT_TOTAL_BYTES}-byte aggregate limit"
            )
        fragments.append(parse_fragment_bytes(raw, f"fragment {index}"))

    owned_rules: list[str] = []
    raw_clauses: list[Mapping[str, Any]] = []
    raw_facets: list[Mapping[str, Any]] = []
    for fragment_index, raw_fragment in enumerate(fragments):
        fragment = _validate_fragment_shell(raw_fragment, fragment_index)
        rules = _strict_strings(fragment["rules"], f"fragment {fragment_index}.rules")
        if not rules:
            fail(f"fragment {fragment_index}.rules must not be empty")
        if len(set(rules)) != len(rules) or rules != sorted(
            rules, key=lambda item: item.encode("ascii")
        ):
            fail(f"fragment {fragment_index}.rules must be unique and bytewise sorted")
        unknown_rules = [rule for rule in rules if rule not in rule_spans]
        if unknown_rules:
            fail(f"fragment {fragment_index} has unknown rules: {unknown_rules!r}")
        owned_rules.extend(rules)
        fragment_clauses = _array(fragment["clauses"], f"fragment {fragment_index}.clauses")
        fragment_facets = _array(fragment["facets"], f"fragment {fragment_index}.facets")
        if len(raw_clauses) + len(fragment_clauses) > MAX_CLAUSE_COUNT:
            fail(f"clause count exceeds the {MAX_CLAUSE_COUNT}-clause limit")
        if len(raw_facets) + len(fragment_facets) > MAX_FACET_COUNT:
            fail(f"facet count exceeds the {MAX_FACET_COUNT}-facet limit")
        for entry in fragment_clauses:
            clause = _object(entry, f"fragment {fragment_index} clause")
            _exact_keys(clause, CLAUSE_KEYS, f"fragment {fragment_index} clause")
            if clause.get("owner_rule") not in rules:
                fail(f"fragment {fragment_index} clause is outside its declared rules")
            raw_clauses.append(clause)
        for entry in fragment_facets:
            facet = _object(entry, f"fragment {fragment_index} facet")
            _exact_keys(facet, FACET_KEYS, f"fragment {fragment_index} facet")
            if facet.get("owner_rule") not in rules:
                fail(f"fragment {fragment_index} facet is outside its declared rules")
            raw_facets.append(facet)
        clause_starts = [
            _integer(entry.get("byte_start"), f"fragment {fragment_index} clause.byte_start")
            for entry in fragment_clauses
        ]
        if clause_starts != sorted(clause_starts):
            fail(f"fragment {fragment_index}.clauses are not sorted by byte_start")
        facet_ids = [
            _ascii_string(entry.get("id"), f"fragment {fragment_index} facet.id")
            for entry in fragment_facets
        ]
        if facet_ids != sorted(facet_ids, key=lambda value: value.encode("ascii")):
            fail(f"fragment {fragment_index}.facets are not bytewise sorted by ID")

    duplicate_rules = sorted(rule for rule, count in Counter(owned_rules).items() if count != 1)
    missing_rules = sorted(set(rule_order) - set(owned_rules))
    extra_rules = sorted(set(owned_rules) - set(rule_order))
    if duplicate_rules or extra_rules or (require_complete and missing_rules):
        fail(
            "fragment rules do not form the required v0.9 partition: "
            f"duplicates={duplicate_rules!r}, missing={missing_rules!r}, extra={extra_rules!r}"
        )

    normalized_facets: list[dict[str, Any]] = []
    facet_owner: dict[str, str] = {}
    for raw_facet in raw_facets:
        facet_id = _ascii_string(raw_facet["id"], "facet.id")
        owner = _ascii_string(raw_facet["owner_rule"], f"{facet_id}.owner_rule")
        match = FACET_ID.fullmatch(facet_id)
        if match is None or match.group(1) != owner:
            fail(f"invalid or owner-mismatched facet ID: {facet_id!r}")
        if facet_id in facet_owner:
            fail(f"duplicate facet ID: {facet_id}")
        stage = _ascii_string(raw_facet["owner_stage"], f"{facet_id}.owner_stage")
        if stage not in STAGES:
            fail(f"{facet_id} has unknown owner stage: {stage}")
        source_atoms = _strict_strings(raw_facet["source_atoms"], f"{facet_id}.source_atoms")
        _require_ordered_unique(
            source_atoms, atom_order, f"{facet_id}.source_atoms", nonempty=True
        )
        wrong_atoms = [
            atom for atom in source_atoms if atom_owner.get(atom) != owner
        ]
        if wrong_atoms:
            fail(f"{facet_id} has unknown or cross-owner source atoms: {wrong_atoms!r}")
        lanes = _strict_strings(raw_facet["required_lanes"], f"{facet_id}.required_lanes")
        evidence = _strict_strings(
            raw_facet["required_evidence"], f"{facet_id}.required_evidence"
        )
        _require_ordered_unique(lanes, LANES, f"{facet_id}.required_lanes", nonempty=True)
        _require_ordered_unique(
            evidence, EVIDENCE_CLASSES, f"{facet_id}.required_evidence", nonempty=True
        )
        facet_owner[facet_id] = owner
        normalized_facets.append(
            {
                "id": facet_id,
                "owner_rule": owner,
                "source_atoms": source_atoms,
                "owner_stage": stage,
                "required_lanes": lanes,
                "required_evidence": evidence,
            }
        )
    normalized_facets.sort(key=lambda facet: facet["id"].encode("ascii"))

    normalized_clauses: list[dict[str, Any]] = []
    clauses_by_rule: dict[str, list[dict[str, Any]]] = defaultdict(list)
    mapped_facets: set[str] = set()
    for raw_clause in raw_clauses:
        owner = _ascii_string(raw_clause["owner_rule"], "clause.owner_rule")
        start = _integer(raw_clause["byte_start"], f"{owner} clause.byte_start")
        end = _integer(raw_clause["byte_end"], f"{owner} clause.byte_end")
        digest = _ascii_string(raw_clause["sha256"], f"{owner} clause.sha256")
        disposition = _ascii_string(
            raw_clause["disposition"], f"{owner} clause.disposition"
        )
        if disposition not in DISPOSITIONS:
            fail(f"{owner} clause has unknown disposition: {disposition}")
        if not (0 <= start < end <= len(specification)):
            fail(f"{owner} clause has invalid byte span {start}-{end}")
        if not SHA256.fullmatch(digest) or sha256(specification[start:end]) != digest:
            fail(f"{owner} clause has a stale hash")
        facet_ids = _strict_strings(raw_clause["facet_ids"], f"{owner} clause.facet_ids")
        if len(set(facet_ids)) != len(facet_ids) or facet_ids != sorted(
            facet_ids, key=lambda item: item.encode("ascii")
        ):
            fail(f"{owner} clause.facet_ids must be unique and bytewise sorted")
        if disposition == "facet":
            if not facet_ids:
                fail(f"{owner} facet clause must name at least one facet")
            wrong = [facet for facet in facet_ids if facet_owner.get(facet) != owner]
            if wrong:
                fail(f"{owner} clause names unknown or cross-owner facets: {wrong!r}")
            mapped_facets.update(facet_ids)
        else:
            if facet_ids:
                fail(f"{owner} excluded clause must have no facet IDs")
            disposition_identity = (disposition, owner, start, end, digest)
            if disposition_identity not in reviewed_dispositions:
                fail(
                    f"{owner} excluded clause is not an exact reviewed "
                    "SPEC_SHA256-bound disposition"
                )
        normalized = {
            "owner_rule": owner,
            "byte_start": start,
            "byte_end": end,
            "sha256": digest,
            "disposition": disposition,
            "facet_ids": facet_ids,
        }
        normalized_clauses.append(normalized)
        clauses_by_rule[owner].append(normalized)

    normalized_clauses.sort(key=lambda clause: clause["byte_start"])
    present_rules = set(owned_rules)
    for rule in rule_order:
        if rule not in present_rules:
            continue
        clauses = sorted(clauses_by_rule.get(rule, []), key=lambda clause: clause["byte_start"])
        rule_start, rule_end = rule_spans[rule]
        cursor = rule_start
        for clause in clauses:
            if clause["byte_start"] != cursor:
                fail(f"{rule} clauses have a gap or overlap at byte {cursor}")
            if clause["byte_end"] > rule_end:
                fail(f"{rule} clause escapes its exact rule span")
            cursor = clause["byte_end"]
        if cursor != rule_end:
            fail(f"{rule} clauses do not tile its exact rule span")
    unmapped = sorted(set(facet_owner) - mapped_facets)
    if unmapped:
        fail(f"facets are not mapped by any clause: {unmapped!r}")

    reverse: dict[str, list[str]] = {atom: [] for atom in atom_order}
    for facet in normalized_facets:
        for atom in facet["source_atoms"]:
            reverse[atom].append(facet["id"])
    # Every atom needs a same-owner facet trace.  The sole exception is a rule
    # atom whose entire exact rule is a reviewed exclusion: its clause is the
    # trace, avoiding a fabricated normative facet for non-normative OWN-9.
    wholly_excluded_rules = {
        rule
        for rule in present_rules
        if len(clauses_by_rule[rule]) == 1
        and clauses_by_rule[rule][0]["disposition"] != "facet"
        and (
            clauses_by_rule[rule][0]["byte_start"],
            clauses_by_rule[rule][0]["byte_end"],
        )
        == rule_spans[rule]
    }
    covered_atoms = {atom for atom in atom_order if reverse[atom]}
    covered_atoms.update(f"rule:{rule}" for rule in wholly_excluded_rules)
    required_atoms = [
        atom for atom in atom_order if atom_owner[atom] in present_rules
    ]
    uncovered_atoms = [atom for atom in required_atoms if atom not in covered_atoms]
    if uncovered_atoms:
        fail(f"source-index atoms are uncovered: {uncovered_atoms!r}")
    source_atom_coverage = [
        {"source_atom": atom, "facet_ids": sorted(reverse[atom], key=str.encode)}
        for atom in atom_order
    ]

    decomposition = {"clauses": normalized_clauses, "facets": normalized_facets}
    decomposition_bytes = canonical_bytes(
        decomposition, label="normalized decomposition"
    )
    audit = {
        "schema": 1,
        "kind": "whitefoot-semantic-decomposition-partial-audit",
        "present_rules": [rule for rule in rule_order if rule in present_rules],
        "missing_rules": [rule for rule in rule_order if rule not in present_rules],
        "rule_count": len(present_rules),
        "clause_count": len(normalized_clauses),
        "facet_count": len(normalized_facets),
        "source_atom_count": len(required_atoms),
        "decomposition_sha256": sha256(decomposition_bytes),
    }
    if not require_complete:
        canonical_bytes(audit, label="partial audit")
        return None, audit
    catalog = {
        "schema": 1,
        "kind": "whitefoot-static-semantic-catalog",
        "specification": {
            "path": SPEC_RELATIVE_PATH,
            "version": SPEC_VERSION,
            "sha256": SPEC_SHA256,
        },
        "source_index": {
            "path": SOURCE_INDEX_RELATIVE_PATH,
            "sha256": SOURCE_INDEX_SHA256,
        },
        "decomposition_sha256": sha256(decomposition_bytes),
        "clauses": normalized_clauses,
        "facets": normalized_facets,
        "source_atom_coverage": source_atom_coverage,
    }
    canonical_bytes(catalog, label="static semantic catalog")
    return catalog, audit


def build_static_catalog(
    fragment_bytes: Sequence[bytes],
    specification: bytes,
    source_index_bytes: bytes,
) -> dict[str, Any]:
    """Validate a complete exact-byte decomposition and return its catalog."""
    catalog, _ = _validate_decomposition(
        fragment_bytes,
        specification,
        source_index_bytes,
        require_complete=True,
    )
    if catalog is None:  # pragma: no cover - guarded by require_complete
        fail("internal complete-catalog validation produced no catalog")
    return catalog


def audit_partial_static_decomposition(
    fragment_bytes: Sequence[bytes],
    specification: bytes,
    source_index_bytes: bytes,
) -> dict[str, Any]:
    """Validate the exact present subset and list every absent v0.9 rule."""
    _, audit = _validate_decomposition(
        fragment_bytes,
        specification,
        source_index_bytes,
        require_complete=False,
    )
    return audit


def _repository_inputs(repository_root: Path) -> tuple[tuple[bytes, ...], bytes, bytes]:
    try:
        specification = semantics_io.read_fixed_file(
            repository_root,
            ("spec", "kernel-spec-v0.9.md"),
            SPEC_BYTE_LENGTH,
            "v0.9 specification",
        )
        source_index = semantics_io.read_fixed_file(
            repository_root,
            ("tests", "spec-catalogs", "v0.9", "source.json"),
            SOURCE_INDEX_BYTE_LENGTH,
            "v0.9 source index",
        )
        fragments = semantics_io.read_fragment_directory(
            repository_root,
            ("tests", "spec-catalogs", "v0.9", "decomposition"),
            max_entries=MAX_FRAGMENT_DIRECTORY_ENTRIES,
            max_count=MAX_FRAGMENT_COUNT,
            max_file_bytes=MAX_FRAGMENT_BYTES,
            max_total_bytes=MAX_FRAGMENT_TOTAL_BYTES,
        )
    except semantics_io.CatalogIOError as error:
        fail(str(error))
    except OSError as error:
        normalized = semantics_io.CatalogIOError(
            f"catalog repository filesystem operation failed: {error}"
        )
        fail(str(normalized))
    return tuple(raw for _, raw in fragments), specification, source_index


def build_from_files(repository_root: Path = ROOT) -> dict[str, Any]:
    """Build from the fixed repository layout through descriptor-safe reads."""
    fragments, specification, source_index = _repository_inputs(repository_root)
    return build_static_catalog(fragments, specification, source_index)


def check_partial_from_files(repository_root: Path = ROOT) -> dict[str, Any]:
    """Audit the exact live subset without constructing a static catalog."""
    fragments, specification, source_index = _repository_inputs(repository_root)
    return audit_partial_static_decomposition(fragments, specification, source_index)


def main(argv: Sequence[str] | None = None) -> int:
    """Validate the discovered decomposition; never create catalog.json."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command", choices=("check", "check-partial"), nargs="?", default="check"
    )
    arguments = parser.parse_args(argv)
    if arguments.command == "check-partial":
        audit = check_partial_from_files()
        missing = ", ".join(audit["missing_rules"])
        print(
            f"{audit['rule_count']}/92 rules; "
            f"{audit['source_atom_count']}/204 source atoms; missing: {missing}"
        )
        return 0
    catalog = build_from_files()
    print(catalog["decomposition_sha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, SemanticCatalogError) as error:
        print(f"semantic catalog: {error}", file=__import__("sys").stderr)
        raise SystemExit(1)
