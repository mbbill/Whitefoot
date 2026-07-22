"""Strict nested shape validation and fact extraction for route receipts."""

from __future__ import annotations

from dataclasses import dataclass
from hashlib import sha256
import struct

from counts import AGREEMENT_DERIVED_NAMES, FIELD_NAMES, TRACE_FIELDS
from identities import (
    CANDIDATE_SHA256,
    MEANING_DIGESTS,
    PROPOSAL_SHA256,
    meaning_file_hashes,
    verify_identities,
)
from model import RouteError
from parser_adapter import validate_logical_path
from receipt_diagnostic import validate_diagnostic
from receipt_schema import (
    ADMITTED_DERIVED,
    BASE_DERIVED,
    BUNDLE_DOMAIN,
    NOT_DERIVED_FIELDS,
    RECEIPT_SCHEMA,
    TOP_LEVEL_KEYS,
    TRACE_FIELD_ORDER,
)
from receipt_values import digest, exact_keys, text, text_list, u64


@dataclass(frozen=True)
class ReceiptFacts:
    field_values: dict[str, int | None]
    resolution_not_derived: bool
    derived: dict[str, int]
    agreement: dict[str, int] | None
    projection: dict[str, int]
    path_lengths: tuple[int, ...]
    source_lengths: tuple[int, ...]
    spelling: dict[str, int]
    source_origins: int
    prelude_origins: int
    diagnostic_paths: int
    path_components: int
    diagnostic_depth: int
    diagnostic_issue_elements: int


def _validate_counts(
    receipt: dict[str, object]
) -> tuple[dict[str, int | None], bool]:
    counts = receipt["counts"]
    if not isinstance(counts, list) or len(counts) != len(FIELD_NAMES):
        raise RouteError("receipt does not contain exactly 33 count rows")
    field_values: dict[str, int | None] = {}
    not_derived = set()
    for tag, (name, raw) in enumerate(zip(FIELD_NAMES, counts), 1):
        if (
            not isinstance(raw, dict)
            or raw.get("name") != name
            or raw.get("tag") != tag
        ):
            raise RouteError("receipt count row name or tag is wrong")
        state = raw.get("state")
        if state == "exact":
            row = exact_keys(
                raw, {"name", "state", "tag", "value"}, f"count {name}"
            )
            field_values[name] = u64(row["value"], name)
        elif state == "trace-required" and name in TRACE_FIELDS:
            row = exact_keys(
                raw, {"name", "reason", "state", "tag"}, f"count {name}"
            )
            text(row["reason"], f"count {name} trace reason")
            field_values[name] = None
        elif state == "not-derived" and name in NOT_DERIVED_FIELDS:
            row = exact_keys(
                raw, {"name", "reason", "state", "tag"}, f"count {name}"
            )
            text(row["reason"], f"count {name} not-derived reason")
            not_derived.add(name)
            field_values[name] = None
        else:
            raise RouteError(f"receipt count {name} has an invalid state")
    if any(field_values[name] is not None for name in TRACE_FIELDS):
        raise RouteError("receipt trace fields are not all unavailable")
    if not_derived and not_derived != NOT_DERIVED_FIELDS:
        raise RouteError("receipt has a partial not-derived resolution block")
    return field_values, bool(not_derived)


def _validate_identities(value: object) -> None:
    identities = exact_keys(
        value,
        {
            "candidate_sha256",
            "meaning_files",
            "meaning_sha256",
            "parser_audit_sha256",
            "profile_schema_sha256",
            "proposal_sha256",
            "route_code_sha256",
        },
        "identities",
    )
    meaning_files = exact_keys(
        identities["meaning_files"],
        {"SCHEMA-SEMANTICS.md", "STORAGE-MODEL.md", "WORK-SCHEDULE.md"},
        "meaning-file identities",
    )
    meaning_digests = exact_keys(
        identities["meaning_sha256"],
        {"semantics", "storage", "work"},
        "meaning identities",
    )
    for name, value in {**meaning_files, **meaning_digests}.items():
        digest(value, name)
    expected = verify_identities()
    if (
        identities["candidate_sha256"] != CANDIDATE_SHA256
        or identities["proposal_sha256"] != PROPOSAL_SHA256
        or identities["profile_schema_sha256"] != expected["schema.py"]
        or identities["parser_audit_sha256"] != expected["audited_parser_set"]
        or meaning_files != meaning_file_hashes()
        or meaning_digests != MEANING_DIGESTS
        or identities["route_code_sha256"] != expected["source_route_code"]
    ):
        raise RouteError("receipt authority identities do not match this route")


def _validate_sources(
    value: object,
) -> tuple[tuple[int, ...], tuple[int, ...]]:
    source_bundle = exact_keys(value, {"sha256", "sources"}, "source bundle")
    digest(source_bundle["sha256"], "source bundle")
    source_rows = source_bundle["sources"]
    if not isinstance(source_rows, list) or not source_rows:
        raise RouteError("receipt source bundle is empty")
    neutral = bytearray(BUNDLE_DOMAIN)
    neutral.extend(struct.pack(">Q", len(source_rows)))
    path_lengths: list[int] = []
    source_lengths: list[int] = []
    seen_paths: set[bytes] = set()
    for raw in source_rows:
        source = exact_keys(
            raw, {"byte_length", "logical_path", "sha256"}, "source identity"
        )
        if not isinstance(source["logical_path"], str):
            raise RouteError("receipt source logical path is not text")
        encoded_path = validate_logical_path(source["logical_path"])
        if encoded_path in seen_paths:
            raise RouteError("receipt source bundle repeats a logical path")
        seen_paths.add(encoded_path)
        length = u64(source["byte_length"], "source byte length")
        source_digest = digest(source["sha256"], "source identity")
        path_lengths.append(len(encoded_path))
        source_lengths.append(length)
        neutral.extend(struct.pack(">Q", len(encoded_path)))
        neutral.extend(encoded_path)
        neutral.extend(struct.pack(">Q", length))
        neutral.extend(bytes.fromhex(source_digest))
    if sha256(neutral).hexdigest() != source_bundle["sha256"]:
        raise RouteError("receipt source-bundle identity is inconsistent")
    return tuple(path_lengths), tuple(source_lengths)


def _validate_trace_gaps(value: object) -> None:
    if not isinstance(value, list) or len(value) != len(TRACE_FIELD_ORDER):
        raise RouteError("receipt trace-gap ledger is incomplete")
    for field, raw in zip(TRACE_FIELD_ORDER, value):
        gap = exact_keys(
            raw,
            {"allowed_inputs", "field", "required_replay", "tag"},
            "trace gap",
        )
        if gap["field"] != field or gap["tag"] != FIELD_NAMES.index(field) + 1:
            raise RouteError("receipt trace-gap field or tag is wrong")
        text_list(
            gap["allowed_inputs"],
            "trace-gap allowed inputs",
            allow_empty=False,
            require_sorted=False,
        )
        text(gap["required_replay"], "trace-gap required replay")


def _validate_workload(value: object) -> None:
    if value is None:
        return
    workload = exact_keys(
        value, {"family", "manifest_sha256", "units"}, "workload"
    )
    if workload["family"] not in {"compiler", "codec"}:
        raise RouteError("receipt workload family is not closed")
    digest(workload["manifest_sha256"], "workload manifest")
    if not 1 <= u64(workload["units"], "workload units") <= 32_768:
        raise RouteError("receipt workload units exceed the generator range")


def validate_structure(receipt: dict[str, object]) -> ReceiptFacts:
    """Validate every nested record and return typed relation inputs."""

    exact_keys(receipt, TOP_LEVEL_KEYS, "top level")
    if (
        receipt["schema"] != RECEIPT_SCHEMA
        or receipt["status"] != "trace-incomplete"
    ):
        raise RouteError("receipt schema or status is wrong")
    field_values, resolution_not_derived = _validate_counts(receipt)
    _validate_identities(receipt["identities"])

    derived = receipt["derived_counts"]
    expected_derived = (
        BASE_DERIVED
        if resolution_not_derived
        else BASE_DERIVED | ADMITTED_DERIVED
    )
    if not isinstance(derived, dict) or set(derived) != expected_derived:
        raise RouteError("receipt derived-count vocabulary is open or incomplete")
    for name, value in derived.items():
        u64(value, f"derived {name}")

    agreement = receipt["agreement_derived_counts"]
    if resolution_not_derived:
        if agreement is not None:
            raise RouteError(
                "receipt derives resolution agreement after FN-8 rejection"
            )
    else:
        if not isinstance(agreement, dict) or set(agreement) != set(
            AGREEMENT_DERIVED_NAMES
        ):
            raise RouteError(
                "receipt agreement-derived vocabulary is open or incomplete"
            )
        for name, value in agreement.items():
            u64(value, f"agreement derived {name}")

    projection = exact_keys(
        receipt["projection_summary"],
        {
            "declaration_facts_including_prelude_lookup",
            "production_nodes",
            "role_occurrences",
            "scopes",
            "terminals",
        },
        "projection summary",
    )
    for name, value in projection.items():
        u64(value, f"projection {name}")

    path_lengths, source_lengths = _validate_sources(receipt["source_bundle"])
    diagnostic = validate_diagnostic(
        receipt["selected_diagnostic"], source_lengths
    )
    origin_count = diagnostic.source_origins + diagnostic.prelude_origins
    issue_elements = (
        0
        if receipt["selected_diagnostic"] is None
        else 1 + origin_count + diagnostic.paths + diagnostic.path_components
    )

    spelling = receipt["spelling_components"]
    expected_spelling = (
        set()
        if resolution_not_derived
        else {
            "dotless_reservations",
            "mode_words",
            "operation_families",
            "prelude",
            "source_roles",
        }
    )
    if not isinstance(spelling, dict) or set(spelling) != expected_spelling:
        raise RouteError("receipt spelling components are open or incomplete")
    for name, value in spelling.items():
        u64(value, f"spelling {name}")

    _validate_trace_gaps(receipt["trace_gaps"])
    _validate_workload(receipt["workload"])
    return ReceiptFacts(
        field_values,
        resolution_not_derived,
        derived,
        agreement,
        projection,
        path_lengths,
        source_lengths,
        spelling,
        diagnostic.source_origins,
        diagnostic.prelude_origins,
        diagnostic.paths,
        diagnostic.path_components,
        diagnostic.depth,
        issue_elements,
    )
