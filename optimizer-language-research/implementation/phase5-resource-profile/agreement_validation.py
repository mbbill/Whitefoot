"""Closed schemas and validation for cross-route resource agreement."""

from __future__ import annotations

from hashlib import sha256
from pathlib import Path


ROOT = Path(__file__).resolve().parent
WORKLOADS = ROOT / "workloads.py"
FIELD_NAMES = (
    "max_sources",
    "max_logical_path_bytes",
    "max_source_bytes",
    "max_total_source_bytes",
    "max_binding_bytes",
    "max_token_bytes",
    "max_tokens",
    "max_lexemes",
    "max_lexical_scan_work",
    "max_classified_tokens",
    "max_production_nodes",
    "max_mixed_elements",
    "max_tree_depth",
    "max_parser_stack_entries",
    "max_list_members",
    "max_expected_terminals",
    "max_syntax_work",
    "max_tree_bytes",
    "max_declarations",
    "max_scopes",
    "max_scope_depth",
    "max_declaration_events",
    "max_lexical_uses",
    "max_deferred_uses",
    "max_spelling_bytes",
    "max_lookup_entries",
    "max_ancestry_steps",
    "max_node_path_depth",
    "max_diagnostic_origins",
    "max_diagnostic_paths",
    "max_diagnostic_path_components",
    "max_coverage_records",
    "max_resolution_work",
)
TRACE_FIELDS = (
    "max_lexical_scan_work",
    "max_parser_stack_entries",
    "max_list_members",
    "max_expected_terminals",
    "max_syntax_work",
    "max_resolution_work",
)
ANALYTIC_DERIVED_NAMES = (
    "terminals",
    "private_derivation_elements",
    "gaps",
    "source_extents",
    "source_role_occurrences",
    "prelude_declarations",
    "prelude_lookup_entries",
    "operation_lookup_entries",
    "source_lookup_entries",
    "ordering_scratch_elements",
    "source_spelling_bytes",
    "prelude_spelling_bytes",
    "operation_spelling_bytes",
    "dotless_reservation_spelling_bytes",
    "mode_word_spelling_bytes",
    "derivation_tree_bytes",
    "node_tree_bytes",
    "mixed_tree_bytes",
    "terminal_tree_bytes",
    "source_extent_tree_bytes",
    "source_diagnostic_origins",
    "prelude_diagnostic_origins",
    "diagnostic_issue_elements",
)
ANALYTIC_CODE_FILES = (
    "dependency_audit.py",
    "manifest.py",
    "measure.py",
    "receipt.py",
    "relation.py",
    "run.py",
    "selection.py",
)
SOURCE_CODE_FILES = (
    "counts.py",
    "fn8.py",
    "identities.py",
    "inventory_selection.py",
    "lexical_selection.py",
    "manifest.py",
    "measurement.py",
    "model.py",
    "parser_adapter.py",
    "receipt.py",
    "receipt_diagnostic.py",
    "receipt_schema.py",
    "receipt_structure.py",
    "receipt_validation.py",
    "receipt_values.py",
    "resolution.py",
    "roles.py",
    "run.py",
    "topology.py",
)
EXPECTED_AUTHORITY_IDENTITIES = {
    "proposal": "7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d",
    "candidate specification": "71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9",
    "semantics": "981878811e38716acfd5dc0bbacccf278c68b2db29aa987af98937e65649d754",
    "storage": "6d624da13ddd48d6dd46f3a2feaac38b83b51e4154e0e70e08a73524e9e7505a",
    "work schedule": "2d085436e8d9288a982ef83a13554c2310cead38892e8223d7f2661b60b3c7e7",
}
SOURCE_TOP_LEVEL_KEYS = {
    "agreement_derived_counts",
    "counts",
    "derived_counts",
    "identities",
    "projection_summary",
    "schema",
    "selected_diagnostic",
    "source_bundle",
    "spelling_components",
    "status",
    "trace_gaps",
    "workload",
}
ANALYTIC_TOP_LEVEL_KEYS = {
    "analytic_code_files",
    "analytic_code_sha256",
    "bundle_sha256",
    "derived",
    "expected_diagnostic",
    "family",
    "fields",
    "generator_revision",
    "manifest_sha256",
    "profile_semantics_sha256",
    "proposal_sha256",
    "receipt_sha256",
    "schema",
    "source_sha256",
    "sources",
    "specification_sha256",
    "status",
    "storage_sha256",
    "trace_gaps",
    "units",
    "work_sha256",
}
SOURCE_IDENTITY_KEYS = {
    "candidate_sha256",
    "meaning_files",
    "meaning_sha256",
    "parser_audit_sha256",
    "profile_schema_sha256",
    "proposal_sha256",
    "route_code_sha256",
}
MEANING_IDENTITY_KEYS = {"semantics", "storage", "work"}
MANIFEST_KEYS = {
    "family",
    "generator_revision",
    "parameters",
    "schema",
    "sources",
    "units",
}
SOURCE_DESCRIPTOR_KEYS = {"byte_length", "logical_path", "sha256"}
U64_MAX = (1 << 64) - 1
NEUTRAL_BUNDLE_DOMAIN = b"WHITEFOOT-RESOURCE-NEUTRAL-BUNDLE-V1\0"


class AgreementError(ValueError):
    """The independently produced receipts disagree or are malformed."""

def _source_fields(receipt: dict[str, object]) -> dict[str, int | None]:
    rows = receipt.get("counts")
    if not isinstance(rows, list) or len(rows) != len(FIELD_NAMES):
        raise AgreementError("source route did not report exactly 33 fields")
    result: dict[str, int | None] = {}
    for expected_tag, (expected_name, row) in enumerate(zip(FIELD_NAMES, rows), 1):
        if not isinstance(row, dict):
            raise AgreementError("source field row is not an object")
        name = row.get("name")
        state = row.get("state")
        if (
            name != expected_name
            or row.get("tag") != expected_tag
            or name in result
        ):
            raise AgreementError("source field order, tag, or name is malformed")
        if state == "exact" and set(row) == {"name", "state", "tag", "value"}:
            value = row.get("value")
            if type(value) is not int or value < 0 or value > U64_MAX:
                raise AgreementError(f"source field {name} is outside u64")
            result[expected_name] = value
        elif state == "trace-required" and name in TRACE_FIELDS:
            if set(row) != {"name", "reason", "state", "tag"} or not isinstance(
                row.get("reason"), str
            ) or not row["reason"]:
                raise AgreementError(f"source trace field {name} is malformed")
            result[expected_name] = None
        else:
            raise AgreementError(f"source field {name} is not an exact smoke claim")
    return result


def _analytic_fields(summary: dict[str, object]) -> dict[str, int | None]:
    rows = summary.get("fields")
    if not isinstance(rows, list) or len(rows) != len(FIELD_NAMES):
        raise AgreementError("analytic route did not report exactly 33 fields")
    result: dict[str, int | None] = {}
    for expected_name, row in zip(FIELD_NAMES, rows):
        if not isinstance(row, dict):
            raise AgreementError("analytic field row is not an object")
        name = row.get("name")
        state = row.get("state")
        value = row.get("value")
        if name != expected_name or name in result or set(row) != {"name", "state", "value"}:
            raise AgreementError("analytic field order or name is malformed")
        if state == "available" and type(value) is int and 0 <= value <= U64_MAX:
            result[expected_name] = value
        elif state == "unavailable" and value is None and name in TRACE_FIELDS:
            result[expected_name] = None
        else:
            raise AgreementError(f"analytic field {name} has a malformed state")
    return result


def _source_derived(receipt: dict[str, object]) -> dict[str, int]:
    """Read only the closed values independently reported by the source route."""

    raw = receipt.get("agreement_derived_counts")
    if not isinstance(raw, dict) or set(raw) != set(ANALYTIC_DERIVED_NAMES):
        raise AgreementError("source agreement-derived vocabulary is not closed")
    result: dict[str, int] = {}
    for name in ANALYTIC_DERIVED_NAMES:
        value = raw[name]
        if type(value) is not int:
            raise AgreementError(f"source agreement-derived {name} is not an integer")
        result[name] = value
    return result


def _analytic_derived(summary: dict[str, object]) -> dict[str, int]:
    rows = summary.get("derived")
    if not isinstance(rows, list):
        raise AgreementError("analytic derived counts are absent")
    result: dict[str, int] = {}
    for row in rows:
        if not isinstance(row, dict):
            raise AgreementError("analytic derived-count row is malformed")
        name = row.get("name")
        value = row.get("value")
        if not isinstance(name, str) or type(value) is not int or name in result:
            raise AgreementError("analytic derived-count name or value is malformed")
        result[name] = value
    if tuple(result) != ANALYTIC_DERIVED_NAMES:
        raise AgreementError("analytic derived-count vocabulary is not closed")
    return result


def _digest(value: object, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise AgreementError(f"{label} is not lowercase SHA-256")
    return value


def _source_code_identity() -> str:
    hasher = sha256(b"WHITEFOOT-SOURCE-ROUTE-CODE-V1\0")
    for name in SOURCE_CODE_FILES:
        encoded_name = name.encode("ascii")
        raw = (ROOT / "source-route" / name).read_bytes()
        hasher.update(len(encoded_name).to_bytes(2, "big"))
        hasher.update(encoded_name)
        hasher.update(sha256(raw).digest())
    return hasher.hexdigest()


def _analytic_code_identity() -> str:
    hasher = sha256(b"WHITEFOOT-ANALYTIC-CODE-V1\0")
    hasher.update(len(ANALYTIC_CODE_FILES).to_bytes(2, "big"))
    for name in ANALYTIC_CODE_FILES:
        encoded_name = name.encode("ascii")
        raw = (ROOT / "analytic-route" / name).read_bytes()
        hasher.update(len(encoded_name).to_bytes(2, "big"))
        hasher.update(encoded_name)
        hasher.update(len(raw).to_bytes(8, "big"))
        hasher.update(sha256(raw).digest())
    return hasher.hexdigest()


def _validate_manifest(
    manifest: dict[str, object],
    family: str,
    units: int,
    source: bytes,
) -> dict[str, object]:
    if set(manifest) != MANIFEST_KEYS or manifest.get("schema") != "whitefoot-resource-workload-v1":
        raise AgreementError("neutral manifest top level is not closed")
    if manifest.get("family") != family or manifest.get("units") != units:
        raise AgreementError("neutral manifest case identity is wrong")
    revision = _digest(manifest.get("generator_revision"), "generator revision")
    if revision != sha256(WORKLOADS.read_bytes()).hexdigest():
        raise AgreementError("neutral manifest does not bind the invoked producer")
    parameters = manifest.get("parameters")
    expected_parameters = (
        ("name_decimal_width", 6),
        ("source_records", 1),
        ("unit_count", units),
    )
    if not isinstance(parameters, list) or len(parameters) != len(expected_parameters):
        raise AgreementError("neutral manifest parameter vector is malformed")
    for row, (name, value) in zip(parameters, expected_parameters):
        if not isinstance(row, dict) or row != {"name": name, "value": value}:
            raise AgreementError("neutral manifest parameter row is malformed")
    descriptors = manifest.get("sources")
    if not isinstance(descriptors, list) or len(descriptors) != 1:
        raise AgreementError("neutral manifest does not contain one source identity")
    descriptor = descriptors[0]
    if not isinstance(descriptor, dict) or set(descriptor) != SOURCE_DESCRIPTOR_KEYS:
        raise AgreementError("neutral manifest source identity is malformed")
    logical_path = descriptor.get("logical_path")
    if not isinstance(logical_path, str) or not logical_path or not logical_path.isascii():
        raise AgreementError("neutral manifest logical path is malformed")
    if (
        descriptor.get("byte_length") != len(source)
        or descriptor.get("sha256") != sha256(source).hexdigest()
    ):
        raise AgreementError("neutral manifest does not bind the generated source")
    return descriptor


def _neutral_bundle_identity(descriptors: list[dict[str, object]]) -> str:
    """Bind the shared ordered path, length, and source-digest boundary."""

    output = bytearray(NEUTRAL_BUNDLE_DOMAIN)
    output.extend(len(descriptors).to_bytes(8, "big"))
    for descriptor in descriptors:
        if set(descriptor) != SOURCE_DESCRIPTOR_KEYS:
            raise AgreementError("neutral bundle source identity is malformed")
        logical_path = descriptor.get("logical_path")
        byte_length = descriptor.get("byte_length")
        if (
            not isinstance(logical_path, str)
            or not logical_path.isascii()
            or type(byte_length) is not int
            or not 0 <= byte_length <= U64_MAX
        ):
            raise AgreementError("neutral bundle path or byte length is malformed")
        path = logical_path.encode("ascii")
        output.extend(len(path).to_bytes(8, "big"))
        output.extend(path)
        output.extend(byte_length.to_bytes(8, "big"))
        output.extend(bytes.fromhex(_digest(descriptor.get("sha256"), "source")))
    return sha256(output).hexdigest()


def _validate_route_identities(
    source_receipt: dict[str, object], analytic_summary: dict[str, object]
) -> None:
    identities = source_receipt.get("identities")
    if not isinstance(identities, dict) or set(identities) != SOURCE_IDENTITY_KEYS:
        raise AgreementError("source authority identity set is not closed")
    meaning = identities.get("meaning_sha256")
    meaning_files = identities.get("meaning_files")
    if (
        not isinstance(meaning, dict)
        or set(meaning) != MEANING_IDENTITY_KEYS
        or not isinstance(meaning_files, dict)
        or set(meaning_files) != {
            "SCHEMA-SEMANTICS.md", "STORAGE-MODEL.md", "WORK-SCHEDULE.md"
        }
    ):
        raise AgreementError("source meaning identity set is not closed")
    for name, value in identities.items():
        if name in {"meaning_sha256", "meaning_files"}:
            continue
        _digest(value, f"source {name}")
    for name, value in meaning.items():
        _digest(value, f"source meaning {name}")
    for name, value in meaning_files.items():
        _digest(value, f"source meaning file {name}")
    shared = (
        (identities["proposal_sha256"], analytic_summary.get("proposal_sha256"), "proposal"),
        (
            identities["candidate_sha256"],
            analytic_summary.get("specification_sha256"),
            "candidate specification",
        ),
        (meaning["semantics"], analytic_summary.get("profile_semantics_sha256"), "semantics"),
        (meaning["storage"], analytic_summary.get("storage_sha256"), "storage"),
        (meaning["work"], analytic_summary.get("work_sha256"), "work schedule"),
    )
    for source_value, analytic_value, label in shared:
        _digest(analytic_value, f"analytic {label}")
        if (
            source_value != analytic_value
            or source_value != EXPECTED_AUTHORITY_IDENTITIES[label]
        ):
            raise AgreementError(f"route authority identities disagree for {label}")
    if tuple(analytic_summary.get("analytic_code_files", ())) != ANALYTIC_CODE_FILES:
        raise AgreementError("analytic code file identity set is not closed")
    if identities["route_code_sha256"] != _source_code_identity():
        raise AgreementError("source receipt does not bind the invoked route code")
    analytic_code = _digest(analytic_summary.get("analytic_code_sha256"), "analytic code")
    if analytic_code != _analytic_code_identity():
        raise AgreementError("analytic summary does not bind the invoked route code")


def _validate_trace_gaps(
    source_receipt: dict[str, object], analytic_summary: dict[str, object]
) -> None:
    source_gaps = source_receipt.get("trace_gaps")
    analytic_gaps = analytic_summary.get("trace_gaps")
    if not isinstance(source_gaps, list) or not isinstance(analytic_gaps, list):
        raise AgreementError("trace-gap ledgers are absent")
    if len(source_gaps) != len(TRACE_FIELDS) or len(analytic_gaps) != len(TRACE_FIELDS):
        raise AgreementError("trace-gap ledgers are incomplete")
    for field, source_row, analytic_row in zip(TRACE_FIELDS, source_gaps, analytic_gaps):
        if (
            not isinstance(source_row, dict)
            or set(source_row) != {"allowed_inputs", "field", "required_replay", "tag"}
            or source_row.get("field") != field
            or source_row.get("tag") != FIELD_NAMES.index(field) + 1
            or not isinstance(source_row.get("allowed_inputs"), list)
            or not source_row["allowed_inputs"]
            or not isinstance(source_row.get("required_replay"), str)
            or not source_row["required_replay"]
        ):
            raise AgreementError("source trace-gap ledger is malformed")
        if (
            not isinstance(analytic_row, dict)
            or set(analytic_row) != {"field", "reason"}
            or analytic_row.get("field") != field
            or not isinstance(analytic_row.get("reason"), str)
            or not analytic_row["reason"]
        ):
            raise AgreementError("analytic trace-gap ledger is malformed")


def _compare_profile_claims(
    source_receipt: dict[str, object], analytic_summary: dict[str, object]
) -> dict[str, int | None]:
    source_fields = _source_fields(source_receipt)
    analytic_fields = _analytic_fields(analytic_summary)
    if source_fields != analytic_fields:
        disagreements = {
            name: (source_fields.get(name), analytic_fields.get(name))
            for name in FIELD_NAMES
            if source_fields.get(name) != analytic_fields.get(name)
        }
        raise AgreementError(f"profile fields disagree: {disagreements}")
    source_derived = _source_derived(source_receipt)
    analytic_derived = _analytic_derived(analytic_summary)
    if source_derived != analytic_derived:
        disagreements = {
            name: (source_derived.get(name), analytic_derived.get(name))
            for name in ANALYTIC_DERIVED_NAMES
            if source_derived.get(name) != analytic_derived.get(name)
        }
        raise AgreementError(f"derived counts disagree: {disagreements}")
    return source_fields


def _validate_case_bindings(
    manifest: dict[str, object],
    source_receipt: dict[str, object],
    analytic_summary: dict[str, object],
    *,
    family: str,
    units: int,
    manifest_bytes: bytes,
    source_bytes: bytes,
    analytic_receipt_bytes: bytes,
) -> tuple[list[str], str]:
    if set(source_receipt) != SOURCE_TOP_LEVEL_KEYS:
        raise AgreementError("source receipt top level is not closed")
    if set(analytic_summary) != ANALYTIC_TOP_LEVEL_KEYS:
        raise AgreementError("analytic summary top level is not closed")
    descriptor = _validate_manifest(manifest, family, units, source_bytes)
    manifest_digest = sha256(manifest_bytes).hexdigest()
    source_digest = descriptor["sha256"]
    workload = source_receipt.get("workload")
    source_bundle = source_receipt.get("source_bundle")
    if not isinstance(workload, dict) or set(workload) != {"family", "manifest_sha256", "units"}:
        raise AgreementError("source workload identity is malformed")
    if not isinstance(source_bundle, dict) or set(source_bundle) != {"sha256", "sources"}:
        raise AgreementError("source bundle identity is malformed")
    _digest(source_bundle.get("sha256"), "source bundle")
    expected_sources = [descriptor]
    if (
        workload
        != {"family": family, "manifest_sha256": manifest_digest, "units": units}
        or source_bundle.get("sources") != expected_sources
        or analytic_summary.get("family") != family
        or analytic_summary.get("units") != units
        or analytic_summary.get("generator_revision") != manifest.get("generator_revision")
        or analytic_summary.get("manifest_sha256") != manifest_digest
        or analytic_summary.get("sources") != expected_sources
        or analytic_summary.get("source_sha256") != [source_digest]
    ):
        raise AgreementError("route receipts do not bind the same manifest and sources")
    if (
        source_receipt.get("schema") != "whitefoot-resource-source-route-receipt-v1"
        or analytic_summary.get("schema") != "whitefoot-resource-analytic-summary-v1"
        or source_receipt.get("status") != "trace-incomplete"
        or analytic_summary.get("status") != "trace-incomplete"
    ):
        raise AgreementError("route receipt schema or status is wrong")
    if analytic_summary.get("receipt_sha256") != sha256(analytic_receipt_bytes).hexdigest():
        raise AgreementError("analytic summary does not bind its binary receipt")
    _digest(analytic_summary.get("bundle_sha256"), "analytic bundle")
    _validate_route_identities(source_receipt, analytic_summary)
    return [source_digest], _neutral_bundle_identity(expected_sources)
