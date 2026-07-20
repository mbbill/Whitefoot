#!/usr/bin/env python3
"""Recompute the closed exact-v0.8 discrepancy predicate registry."""

from __future__ import annotations

import json
import re
from bisect import bisect_right
from collections import Counter
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

try:
    import facet_discrepancy_inputs as inputs
except ModuleNotFoundError:  # Support import as ``tools.*``.
    from tools import facet_discrepancy_inputs as inputs  # type: ignore

try:
    import v08_terminal_ident_audit as terminal_ident
except ModuleNotFoundError:  # Support import as ``tools.*``.
    from tools import v08_terminal_ident_audit as terminal_ident  # type: ignore


DiscrepancyError = inputs.DiscrepancyError

MAX_MANIFEST_ENTRIES = 20_000
MAX_DECLARATIONS_PER_CASE = 4_096
MAX_DECLARATIONS_TOTAL = 100_000

IDENT = re.compile(rb"[a-z][a-z0-9_]*")
OP_NAME = re.compile(rb"`([a-z_]+(?:\.[a-z]+)?)`")
DOTLESS_LIST = re.compile(rb"or a dotless IDENT \(`([^`\r\n]+)`\); both")


@dataclass(frozen=True)
class ExactSource:
    """Exact bytes plus byte-to-line coordinates for evidence anchors."""

    path: str
    raw: bytes
    line_offsets: tuple[int, ...]

    @classmethod
    def from_bytes(cls, path: str, raw: bytes) -> "ExactSource":
        offsets = [0]
        for line in raw.splitlines(keepends=True):
            offsets.append(offsets[-1] + len(line))
        return cls(path, raw, tuple(offsets))

    def evidence(self, byte_start: int, byte_end: int) -> dict[str, Any]:
        if not 0 <= byte_start < byte_end <= len(self.raw):
            raise DiscrepancyError(
                f"invalid evidence span {self.path}:{byte_start}-{byte_end}"
            )
        return {
            "byte_end": byte_end,
            "byte_start": byte_start,
            "line_end": bisect_right(self.line_offsets, byte_end - 1),
            "line_start": bisect_right(self.line_offsets, byte_start),
            "path": self.path,
            "sha256": inputs.sha256(self.raw[byte_start:byte_end]),
        }

    def unique(self, fragment: bytes) -> tuple[int, int]:
        count = self.raw.count(fragment)
        if count != 1:
            raise DiscrepancyError(
                f"expected one exact fragment in {self.path}, found {count}: "
                f"{fragment!r}"
            )
        start = self.raw.index(fragment)
        return start, start + len(fragment)


@dataclass(frozen=True)
class Observation:
    """The recomputed truth value and exact evidence of one predicate."""

    is_open: bool
    evidence: dict[str, Any]


@dataclass(frozen=True)
class Registration:
    identifier: str
    discrepancy_class: str
    predicate_identifier: str
    affected_facet_ids: tuple[str, ...]
    resolution_authorities: tuple[str, ...]


REGISTRATIONS = (
    Registration(
        "discrepancy:v0.8/affine-deref-storage-lifecycle",
        "internal-specification-gap",
        "predicate:affine-deref-storage-lifecycle-completeness",
        (
            "facet:STOR-3/deallocation-compiler-derived",
            "facet:STOR-3/drop-and-arena-release-artifact-operations",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/diag1-pre-tree-node-path",
        "internal-specification-gap",
        "predicate:diag1-pre-tree-node-path-completeness",
        ("facet:DIAG-1/node-path-attribution",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/diag3-retained-proof-ref",
        "internal-specification-gap",
        "predicate:diag3-retained-check-proof-ref-completeness",
        ("facet:DIAG-3/check-report-schema",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/eff1-row-canonicality",
        "specification-protected-surface-conflict",
        "predicate:eff1-row-canonicality-completeness",
        (
            "facet:EFF-1/canonical-effect-order",
            "facet:EFF-1/effect-row-grammar",
        ),
        (
            "owner-approved-protected-surface-change",
            "successor-numbered-specification",
        ),
    ),
    Registration(
        "discrepancy:v0.8/eff2-local-region-effects",
        "internal-specification-inconsistency",
        "predicate:eff2-local-region-effect-row-consistency",
        (
            "facet:EFF-2/effect-row-bidirectional-exactness",
            "facet:EFF-2/syntactic-effect-exhibit-closure",
            "facet:EX-1/byte-exact-canonical-program",
            "facet:FN-7/main-effect-ceiling",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/fn3-contract-member-semantics",
        "internal-specification-gap",
        "predicate:fn3-contract-member-semantics-completeness",
        (
            "facet:FN-3/contract-member-checking-boundary",
            "facet:FN-5/behavior-parameterization",
            "facet:FN-5/env-struct-direct-calls",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/fn4-law-admission",
        "specification-protected-surface-conflict",
        "predicate:fn4-law-admission-completeness",
        ("facet:FN-4/optimizer-law-fact-admission",),
        (
            "owner-approved-protected-surface-change",
            "successor-numbered-specification",
        ),
    ),
    Registration(
        "discrepancy:v0.8/fn8-reserved-rule-attribution",
        "specification-protected-surface-conflict",
        "predicate:fn8-reserved-rule-attribution-consistency",
        (
            "facet:FN-8/keyword-reserved",
            "facet:FORM-3/ident-lexical-class",
        ),
        (
            "owner-approved-protected-surface-change",
            "successor-numbered-specification",
        ),
    ),
    Registration(
        "discrepancy:v0.8/op1-dotless-reservation",
        "internal-specification-ambiguity",
        "predicate:op1-dotless-reservation-set-equality",
        ("facet:OP-1/dotless-operation-reservation",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/form2-protected-conformance-spacing",
        "specification-protected-surface-conflict",
        "predicate:form2-conformance-function-spacing",
        ("facet:FORM-2/no-space-before-open-paren",),
        (
            "owner-approved-protected-surface-change",
            "successor-numbered-specification",
        ),
    ),
    Registration(
        "discrepancy:v0.8/form4-doc-cross-reference",
        "internal-specification-inconsistency",
        "predicate:form4-doc-production-owner-consistency",
        ("facet:FORM-4/documentation-field-only",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/form5-form7-float-canonical-spelling",
        "internal-specification-inconsistency",
        "predicate:form5-form7-float-canonical-spelling-consistency",
        (
            "facet:FORM-1/noncanonical-input-rejected",
            "facet:FORM-1/one-byte-format",
            "facet:FORM-1/one-spelling-per-construct",
            "facet:FORM-1/toolchain-does-not-autoformat",
            "facet:FORM-5/float-lowercase-exponent",
            "facet:FORM-5/float-no-leading-zeros",
            "facet:FORM-5/float-required-integer-and-fraction-digits",
            "facet:FORM-5/float-shortest-rne-roundtrip",
            "facet:META-1/one-spelling-enforcement",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/gram1-gram7-match-node-bijection",
        "internal-specification-inconsistency",
        "predicate:gram1-gram7-match-node-bijection-consistency",
        (
            "facet:GRAM-1/production-node-bijection",
            "facet:GRAM-7/shared-match-node-kind",
            "facet:META-1/production-node-bijection-enforcement",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/gram-terminal-ident-partition",
        "internal-specification-gap",
        "predicate:gram-terminal-ident-partition-completeness",
        (
            "facet:FORM-3/ident-lexical-class",
            "facet:FORM-6/unit-grammar-positions-disjoint",
            "facet:FORM-6/unit-lowercase-keyword",
            "facet:FORM-6/unit-production-local-resolution",
            "facet:FORM-6/unit-token-type-position",
            "facet:FORM-6/unit-token-value-position",
            "facet:GRAM-1/deterministic-single-parse",
            "facet:GRAM-1/two-token-overlap-resolution",
            "facet:GRAM-3/constant-value-tree-shape",
            "facet:GRAM-3/type-argument-shapes",
            "facet:GRAM-5/atom-form-closed-set",
            "facet:GRAM-5/call-and-callee-shapes",
            "facet:GRAM-5/expression-form-closed-set",
            "facet:GRAM-5/place-chain-shapes",
            "facet:GRAM-6/index-place-only",
            "facet:META-2/context-independent-spellings",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.8/fn7-main-return-spelling",
        "internal-specification-inconsistency",
        "predicate:fn7-main-return-spelling-consistency",
        (
            "facet:EX-1/byte-exact-canonical-program",
            "facet:FN-7/main-return-spelling",
            "facet:GRAM-2/function-and-contract-declaration-shapes",
            "facet:GRAM-3/return-mode-type-shape",
        ),
        ("successor-numbered-specification",),
    ),
)


def _pin(actual: Any, expected: Any, label: str) -> None:
    if inputs.canonical_bytes(actual) != inputs.canonical_bytes(expected):
        raise DiscrepancyError(f"{label} no longer matches the pinned audit")


def _exact_fragment_sources(
    source: ExactSource,
    fragments: Mapping[str, bytes],
    expected: Mapping[str, Mapping[str, Any]],
    label: str,
    *,
    enforce_pins: bool,
) -> dict[str, Any]:
    """Locate unique exact fragments and optionally pin their spans and hashes."""
    evidence: dict[str, Any] = {}
    audit: dict[str, Any] = {}
    for name, fragment in fragments.items():
        span = source.unique(fragment)
        record = source.evidence(*span)
        evidence[f"{name}_source"] = record
        audit[name] = {
            "byte_end": span[1],
            "byte_start": span[0],
            "sha256": record["sha256"],
        }
    if enforce_pins:
        _pin(audit, expected, label)
    return evidence


def _protected_case_evidence(
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    identifier: str,
) -> dict[str, Any]:
    """Validate the protected surface and return one exact case projection."""
    entries, cases = _manifest_entries(manifest)
    protected_ids = _protected_surface(
        entries,
        cases,
        case_sources,
        protected_surface,
    )
    if identifier not in protected_ids or identifier not in cases:
        raise DiscrepancyError(f"protected case is missing: {identifier}")
    path = f"conformance/cases/{identifier}.wf"
    if path not in case_sources:
        raise DiscrepancyError(f"protected case source is missing: {path}")
    entry = cases[identifier]
    return {
        "manifest": {
            field: entry.get(field)
            for field in ("id", "rules", "expect", "status")
        },
        "path": path,
        "sha256": inputs.sha256(case_sources[path]),
    }


def observe_op1(specification: bytes, *, enforce_pins: bool = True) -> Observation:
    """Compare OP-1 table dotless names with its explicit listed set."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    header = b"| op | domain | signature | effects |\n"
    table_start, _ = source.unique(header)
    table_end = specification.find(b"\n\n", table_start)
    if table_end < 0:
        raise DiscrepancyError("OP-1 table has no terminating blank line")
    table_end += 1
    lines = specification[table_start:table_end].splitlines(keepends=True)
    if len(lines) < 3 or lines[:2] != [
        header,
        b"|---|---|---|---|\n",
    ]:
        raise DiscrepancyError("OP-1 table header is not the audited shape")

    occurrences: list[str] = []
    for line in lines[2:]:
        cells = line.split(b"|")
        if len(cells) != 6 or cells[0] or cells[-1] != b"\n":
            raise DiscrepancyError("OP-1 table row is malformed")
        names = [match.decode("ascii") for match in OP_NAME.findall(cells[1])]
        if not names:
            raise DiscrepancyError("OP-1 table row has no operation name")
        occurrences.extend(names)

    table_dotless = list(dict.fromkeys(name for name in occurrences if "." not in name))
    listed_match = DOTLESS_LIST.search(specification, table_end)
    if listed_match is None:
        raise DiscrepancyError("OP-1 explicit dotless identifier list is missing")
    listed = listed_match.group(1).decode("ascii").split(" ")
    if len(listed) != len(set(listed)) or any(
        IDENT.fullmatch(name.encode("ascii")) is None for name in listed
    ):
        raise DiscrepancyError("OP-1 explicit dotless list is malformed or duplicated")
    table_set = set(table_dotless)
    listed_set = set(listed)
    table_only = [name for name in table_dotless if name not in listed_set]
    listed_only = [name for name in listed if name not in table_set]
    evidence = {
        "listed_distinct_dotless_count": len(listed),
        "listed_dotless_identifiers": listed,
        "listed_only_count": len(listed_only),
        "listed_only_identifiers": listed_only,
        "listed_source": source.evidence(listed_match.start(1), listed_match.end(1)),
        "operation_name_occurrence_count": len(occurrences),
        "operation_row_count": len(lines) - 2,
        "table_distinct_dotless_count": len(table_dotless),
        "table_dotless_identifiers": table_dotless,
        "table_only_count": len(table_only),
        "table_only_identifiers": table_only,
        "table_source": source.evidence(table_start, table_end),
    }
    if enforce_pins:
        _pin(
            {
                "listed_count": len(listed),
                "listed_sha256": evidence["listed_source"]["sha256"],
                "listed_only_count": len(listed_only),
                "occurrences": len(occurrences),
                "rows": len(lines) - 2,
                "table_count": len(table_dotless),
                "table_only_count": len(table_only),
                "table_sha256": evidence["table_source"]["sha256"],
                "unique_operations": len(set(occurrences)),
            },
            {
                "listed_count": 20,
                "listed_sha256": "bca1f3a8ad911092756f1f18a459de95cd91062991b837b792e9d9de78fd41fc",
                "listed_only_count": 0,
                "occurrences": 84,
                "rows": 44,
                "table_count": 51,
                "table_only_count": 31,
                "table_sha256": "415a65e25e5c070ccbb7a51ebfb0b3d4ff2a8c42f2f151d3a23720198c352297",
                "unique_operations": 83,
            },
            "OP-1 discrepancy evidence",
        )
    return Observation(table_set != listed_set, evidence)


def _manifest_entries(raw: bytes) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    entries: list[dict[str, Any]] = []
    cases: dict[str, dict[str, Any]] = {}
    for line_number, line in enumerate(raw.splitlines(), 1):
        if not line.strip() or line.lstrip().startswith(b"#"):
            continue
        if len(entries) >= MAX_MANIFEST_ENTRIES:
            raise DiscrepancyError(
                f"manifest exceeds {MAX_MANIFEST_ENTRIES} semantic entries"
            )
        value = inputs.strict_json_loads(
            line,
            max_bytes=inputs.MAX_MANIFEST_BYTES,
            label=f"manifest line {line_number}",
        )
        if not isinstance(value, dict):
            raise DiscrepancyError(f"manifest line {line_number} is not an object")
        entries.append(value)
        if "id" in value:
            identifier = value["id"]
            if not isinstance(identifier, str) or inputs.CASE_ID.fullmatch(identifier) is None:
                raise DiscrepancyError(f"invalid manifest case id at line {line_number}")
            if identifier in cases:
                raise DiscrepancyError(f"duplicate manifest case id: {identifier}")
            cases[identifier] = value
    return entries, cases


def _protected_surface(
    entries: Sequence[dict[str, Any]],
    manifest_cases: Mapping[str, dict[str, Any]],
    case_sources: Mapping[str, bytes],
    protected: Mapping[str, Any],
) -> tuple[str, ...]:
    """Recompute every baseline entry while ignoring additive entries."""
    live: dict[str, str] = {}
    for entry in entries:
        if "id" in entry and entry["id"] in protected:
            identifier = entry["id"]
            path = f"conformance/cases/{identifier}.wf"
            if path not in case_sources:
                raise DiscrepancyError(f"protected manifest case has no source: {path}")
            projection = {
                field: entry.get(field)
                for field in ("id", "rules", "expect", "status")
            }
            encoded = json.dumps(
                projection, ensure_ascii=True, sort_keys=True, separators=(",", ":")
            ).encode("ascii")
            key = identifier
            digest = inputs.sha256(encoded + b"\0" + case_sources[path])
        elif "rule" in entry and f"rule:{entry['rule']}" in protected:
            key = f"rule:{entry['rule']}"
            digest = inputs.sha256(
                json.dumps(
                    entry, ensure_ascii=True, sort_keys=True, separators=(",", ":")
                ).encode("ascii")
            )
        else:
            continue
        if key in live:
            raise DiscrepancyError(f"duplicate protected-surface key in manifest: {key}")
        live[key] = digest

    case_ids = []
    for key, expected_digest in protected.items():
        if not isinstance(key, str):
            raise DiscrepancyError("guard baseline conformance key must be a string")
        if not isinstance(expected_digest, str) or inputs.HEX_SHA256.fullmatch(
            expected_digest
        ) is None:
            raise DiscrepancyError(
                f"guard baseline conformance digest is invalid for {key!r}"
            )
        if key not in live:
            raise DiscrepancyError(f"protected conformance entry is missing: {key}")
        if live[key] != expected_digest:
            raise DiscrepancyError(f"protected conformance entry changed: {key}")
        if inputs.CASE_ID.fullmatch(key):
            if key not in manifest_cases:
                raise DiscrepancyError(f"protected manifest case is missing: {key}")
            case_ids.append(key)
    case_ids.sort(key=str.encode)
    return tuple(case_ids)


def _skip_horizontal(line: bytes, cursor: int) -> tuple[int, bytes]:
    start = cursor
    while cursor < len(line) and line[cursor] in (0x20, 0x09):
        cursor += 1
    return cursor, line[start:cursor]


def _scan_balanced(line: bytes, cursor: int, opening: int, closing: int) -> int | None:
    if cursor >= len(line) or line[cursor] != opening:
        return None
    depth = 0
    while cursor < len(line):
        byte = line[cursor]
        if byte == opening:
            depth += 1
        elif byte == closing:
            depth -= 1
            if depth == 0:
                return cursor + 1
        cursor += 1
    return None


def scan_direct_fn_head(line: bytes) -> tuple[bytes, str] | None:
    """Classify spacing before a direct fn_decl parameter opener in one pass."""
    if not line.startswith(b"fn "):
        return None
    cursor = 3
    name_start = cursor
    name_terminators = b" \t<[(\r\n"
    while cursor < len(line) and line[cursor] not in name_terminators:
        cursor += 1
    if cursor == name_start:
        return None
    name = line[name_start:cursor]
    cursor, final_gap = _skip_horizontal(line, cursor)
    if cursor < len(line) and line[cursor] == 0x3C:  # <...>
        balanced = _scan_balanced(line, cursor, 0x3C, 0x3E)
        if balanced is None:
            return None
        cursor, final_gap = _skip_horizontal(line, balanced)
    if cursor < len(line) and line[cursor] == 0x5B:  # [...]
        balanced = _scan_balanced(line, cursor, 0x5B, 0x5D)
        if balanced is None:
            return None
        cursor, final_gap = _skip_horizontal(line, balanced)
    if cursor >= len(line) or line[cursor] != 0x28:  # (
        return None
    spacing = (
        "attached"
        if final_gap == b""
        else "single-space"
        if final_gap == b" "
        else "other-whitespace"
    )
    return name, spacing


def scan_case_declarations(
    raw: bytes,
    path: str,
    *,
    declarations_before: int = 0,
) -> tuple[dict[str, Any], ...]:
    """Return capped exact records for direct declarations in one case."""
    declarations: list[dict[str, Any]] = []
    for line_number, line in enumerate(raw.splitlines(keepends=True), 1):
        scanned = scan_direct_fn_head(line)
        if scanned is None:
            continue
        if len(declarations) >= MAX_DECLARATIONS_PER_CASE:
            raise DiscrepancyError(
                f"{path} exceeds {MAX_DECLARATIONS_PER_CASE} declarations"
            )
        if declarations_before + len(declarations) >= MAX_DECLARATIONS_TOTAL:
            raise DiscrepancyError(
                f"protected corpus exceeds {MAX_DECLARATIONS_TOTAL} declarations"
            )
        name, spacing = scanned
        declarations.append(
            {
                "line": line_number,
                "line_sha256": inputs.sha256(line),
                "name_bytes_hex": name.hex(),
                "spacing": spacing,
            }
        )
    return tuple(declarations)


def _protected_form2_sources(
    protected_case_ids: Sequence[str],
    manifest_cases: Mapping[str, dict[str, Any]],
    case_sources: Mapping[str, bytes],
    *,
    include_legacy_orphan: bool,
) -> tuple[tuple[str, dict[str, Any] | None], ...]:
    """Name every protected source and distinguish manifest-backed authority."""
    sources: list[tuple[str, dict[str, Any] | None]] = [
        (f"conformance/cases/{identifier}.wf", manifest_cases[identifier])
        for identifier in protected_case_ids
    ]
    if include_legacy_orphan:
        orphan = case_sources.get(inputs.LEGACY_ORPHAN_PATH)
        if orphan is None:
            raise DiscrepancyError("legacy protected orphan source is missing")
        actual = inputs.sha256(orphan)
        if actual != inputs.LEGACY_ORPHAN_SHA256:
            raise DiscrepancyError(
                "legacy protected orphan source changed: "
                f"{actual} != {inputs.LEGACY_ORPHAN_SHA256}"
            )
        if any(path == inputs.LEGACY_ORPHAN_PATH for path, _ in sources):
            raise DiscrepancyError(
                "legacy protected orphan unexpectedly has a manifest entry"
            )
        sources.append((inputs.LEGACY_ORPHAN_PATH, None))
    sources.sort(key=lambda item: item[0].encode("utf-8"))
    return tuple(sources)


def observe_form2(
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    *,
    protected_surface: Mapping[str, Any] | None = None,
    enforce_pins: bool = True,
) -> Observation:
    """Audit protected fn_decl heads with capped linear line scans."""
    entries, manifest_cases = _manifest_entries(manifest)
    if protected_surface is None:
        if enforce_pins:
            raise DiscrepancyError(
                "FORM-2 pinned audit requires the protected baseline surface"
            )
        protected_case_ids = tuple(sorted(manifest_cases, key=str.encode))
    else:
        protected_case_ids = _protected_surface(
            entries, manifest_cases, case_sources, protected_surface
        )

    protected_sources = _protected_form2_sources(
        protected_case_ids,
        manifest_cases,
        case_sources,
        include_legacy_orphan=protected_surface is not None,
    )

    rows = []
    total_declarations = 0
    counts: Counter[str] = Counter()
    manifested_counts: Counter[str] = Counter()
    unmanifested_counts: Counter[str] = Counter()
    affected_rows = []
    affected_manifest_entries = []
    for path, manifest_entry in protected_sources:
        if path not in case_sources:
            raise DiscrepancyError(f"protected case source is missing: {path}")
        raw = case_sources[path]
        declarations = scan_case_declarations(
            raw,
            path,
            declarations_before=total_declarations,
        )
        for declaration in declarations:
            total_declarations += 1
            spacing = declaration["spacing"]
            counts[spacing] += 1
            if manifest_entry is None:
                unmanifested_counts[spacing] += 1
            else:
                manifested_counts[spacing] += 1
        projection = None
        if manifest_entry is not None:
            projection = {
                field: manifest_entry.get(field)
                for field in ("id", "rules", "expect", "status")
            }
        row = {
            "direct_function_declarations": list(declarations),
            "manifest": projection,
            "path": path,
            "sha256": inputs.sha256(raw),
        }
        rows.append(row)
        if any(item["spacing"] == "single-space" for item in declarations):
            affected_rows.append(row)
            if manifest_entry is not None:
                affected_manifest_entries.append(manifest_entry)

    runnable = [
        entry
        for entry in affected_manifest_entries
        if entry.get("status") == "runnable"
    ]
    pending = [
        entry
        for entry in affected_manifest_entries
        if entry.get("status") == "pending"
    ]
    expected_rejections = [
        entry
        for entry in runnable
        if entry.get("expect") == {"kind": "reject", "rule": "FORM-2"}
    ]
    conflicting = len(runnable) - len(expected_rejections)
    manifested_rows = [row for row in rows if row["manifest"] is not None]
    unmanifested_rows = [row for row in rows if row["manifest"] is None]
    affected_manifested_rows = [
        row for row in affected_rows if row["manifest"] is not None
    ]
    affected_unmanifested_rows = [
        row for row in affected_rows if row["manifest"] is None
    ]
    inventory = [
        {
            "manifested": row["manifest"] is not None,
            "path": row["path"],
            "sha256": row["sha256"],
        }
        for row in rows
    ]
    positive = next(
        (
            row
            for row in rows
            if row["path"] == "conformance/cases/form2-pos-canonical-bytes.wf"
        ),
        None,
    )
    if positive is None and enforce_pins:
        raise DiscrepancyError("FORM-2 positive fixture is missing")
    evidence = {
        "attached_declaration_count": counts["attached"],
        "direct_function_declaration_count": total_declarations,
        "manifested_direct_function_declaration_count": sum(
            manifested_counts.values()
        ),
        "manifested_expected_form2_rejection_count": len(expected_rejections),
        "manifested_pending_source_files_with_single_space_count": len(pending),
        "manifested_runnable_other_expectation_count": conflicting,
        "manifested_runnable_source_files_with_single_space_count": len(runnable),
        "manifested_single_space_declaration_count": manifested_counts[
            "single-space"
        ],
        "other_whitespace_declaration_count": counts["other-whitespace"],
        "positive_fixture": positive,
        "protected_census_sha256": inputs.sha256(inputs.canonical_bytes(rows)),
        "protected_manifested_source_count": len(manifested_rows),
        "protected_manifested_source_files_with_single_space_count": len(
            affected_manifested_rows
        ),
        "protected_source_count": len(rows),
        "protected_source_files_with_single_space_count": len(affected_rows),
        "protected_source_inventory_sha256": inputs.sha256(
            inputs.canonical_bytes(inventory)
        ),
        "protected_surface_sha256": inputs.sha256(
            inputs.canonical_bytes(protected_surface)
        ),
        "protected_unmanifested_source_count": len(unmanifested_rows),
        "protected_unmanifested_source_files_with_single_space_count": len(
            affected_unmanifested_rows
        ),
        "single_space_declaration_count": counts["single-space"],
        "unmanifested_direct_function_declaration_count": sum(
            unmanifested_counts.values()
        ),
        "unmanifested_protected_sources": unmanifested_rows,
        "unmanifested_single_space_declaration_count": unmanifested_counts[
            "single-space"
        ],
    }
    if enforce_pins:
        assert positive is not None
        _pin(
            {
                key: evidence[key]
                for key in (
                    "attached_declaration_count",
                    "direct_function_declaration_count",
                    "manifested_direct_function_declaration_count",
                    "manifested_expected_form2_rejection_count",
                    "manifested_pending_source_files_with_single_space_count",
                    "manifested_runnable_other_expectation_count",
                    "manifested_runnable_source_files_with_single_space_count",
                    "manifested_single_space_declaration_count",
                    "other_whitespace_declaration_count",
                    "protected_census_sha256",
                    "protected_manifested_source_count",
                    "protected_manifested_source_files_with_single_space_count",
                    "protected_source_count",
                    "protected_source_files_with_single_space_count",
                    "protected_source_inventory_sha256",
                    "protected_unmanifested_source_count",
                    "protected_unmanifested_source_files_with_single_space_count",
                    "single_space_declaration_count",
                    "unmanifested_direct_function_declaration_count",
                    "unmanifested_single_space_declaration_count",
                )
            },
            {
                "attached_declaration_count": 2,
                "direct_function_declaration_count": 400,
                "manifested_direct_function_declaration_count": 399,
                "manifested_expected_form2_rejection_count": 2,
                "manifested_pending_source_files_with_single_space_count": 14,
                "manifested_runnable_other_expectation_count": 274,
                "manifested_runnable_source_files_with_single_space_count": 276,
                "manifested_single_space_declaration_count": 397,
                "other_whitespace_declaration_count": 0,
                "protected_census_sha256": (
                    "61fe48b74371fd2ea476cc901db8d30ce"
                    "07921ffa4ce30ba9c32577a6394beb5"
                ),
                "protected_manifested_source_count": 292,
                "protected_manifested_source_files_with_single_space_count": 290,
                "protected_source_count": 293,
                "protected_source_files_with_single_space_count": 291,
                "protected_source_inventory_sha256": (
                    "944773a3012e40d529f33b1bfe4d9069"
                    "a11eb0c365ae938e27d58977830c9700"
                ),
                "protected_unmanifested_source_count": 1,
                "protected_unmanifested_source_files_with_single_space_count": 1,
                "single_space_declaration_count": 398,
                "unmanifested_direct_function_declaration_count": 1,
                "unmanifested_single_space_declaration_count": 1,
            },
            "FORM-2 protected-surface evidence",
        )
        _pin(
            evidence["unmanifested_protected_sources"],
            [
                {
                    "direct_function_declarations": [
                        {
                            "line": 3,
                            "line_sha256": (
                                "e58d474de015a09840860e8f233684239"
                                "005a29ace4a3ba58c17903fdd13326e"
                            ),
                            "name_bytes_hex": "6d61696e",
                            "spacing": "single-space",
                        }
                    ],
                    "manifest": None,
                    "path": inputs.LEGACY_ORPHAN_PATH,
                    "sha256": inputs.LEGACY_ORPHAN_SHA256,
                }
            ],
            "FORM-2 unmanifested protected-source evidence",
        )
        _pin(
            {
                "declarations": positive["direct_function_declarations"],
                "manifest": positive["manifest"],
                "sha256": positive["sha256"],
            },
            {
                "declarations": [
                    {
                        "line": 1,
                        "line_sha256": (
                            "01da3f7b8d2822839e71e050e99e46eb"
                            "38a794d3e4b8054e5d5f361a184ab29c"
                        ),
                        "name_bytes_hex": "6d61696e",
                        "spacing": "single-space",
                    }
                ],
                "manifest": {
                    "expect": {"exit": 0, "kind": "run"},
                    "id": "form2-pos-canonical-bytes",
                    "rules": ["FORM-2"],
                    "status": "runnable",
                },
                "sha256": "202d27f9d94e35c1a5d36eb04046e25774273f73783efb0609fb4e7e9e5c9218",
            },
            "FORM-2 positive-fixture evidence",
        )
    return Observation(conflicting > 0, evidence)


def observe_form4(
    specification: bytes,
    source_index: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Compare FORM-4's doc citation with the indexed doc production owner."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    statement = b"Documentation is the `doc` field of declarations [GRAM-3]."
    statement_span = source.unique(statement)
    citation = re.search(rb"\[(GRAM-[0-9]+)\]", statement)
    if citation is None:
        raise DiscrepancyError("FORM-4 documentation citation is missing")
    cited_owner = citation.group(1).decode("ascii")

    productions = source_index.get("syntax_productions")
    if not isinstance(productions, list):
        raise DiscrepancyError("source index syntax_productions must be an array")
    matches = [
        item
        for item in productions
        if isinstance(item, dict) and item.get("id") == "production:GRAM-2:doc"
    ]
    if len(matches) != 1:
        raise DiscrepancyError(
            "source index must contain exactly one production:GRAM-2:doc"
        )
    production = matches[0]
    production_source = production.get("source")
    if production.get("lhs") != "doc" or not isinstance(production_source, dict):
        raise DiscrepancyError("indexed doc production has invalid content")
    owner = production.get("owner_rule")
    start = production_source.get("byte_start")
    end = production_source.get("byte_end")
    if not isinstance(owner, str) or isinstance(start, bool) or not isinstance(start, int):
        raise DiscrepancyError("indexed doc production has invalid owner or start")
    if isinstance(end, bool) or not isinstance(end, int):
        raise DiscrepancyError("indexed doc production has invalid end")
    indexed_evidence = source.evidence(start, end)
    source_fields = ("byte_end", "byte_start", "line_end", "line_start", "sha256")
    if {key: indexed_evidence[key] for key in source_fields} != {
        key: production_source.get(key) for key in source_fields
    }:
        raise DiscrepancyError("indexed doc production source span is stale")
    evidence = {
        "doc_production_id": production["id"],
        "doc_production_lhs": production["lhs"],
        "doc_production_owner": owner,
        "doc_production_source": indexed_evidence,
        "form4_cited_owner": cited_owner,
        "form4_citation_source": source.evidence(*statement_span),
    }
    if enforce_pins:
        _pin(
            {
                "doc_owner": owner,
                "doc_sha256": indexed_evidence["sha256"],
                "form4_owner": cited_owner,
                "form4_sha256": evidence["form4_citation_source"]["sha256"],
            },
            {
                "doc_owner": "GRAM-2",
                "doc_sha256": "62075dc6f83e384ce0bea4df8876944089e916f855f9e78b315d02c34e3fccb1",
                "form4_owner": "GRAM-3",
                "form4_sha256": "73ef840b5d5d7f45dcacb899e42f7d6a0be0af400a1cce8371da8721a1cd56d4",
            },
            "FORM-4 documentation cross-reference evidence",
        )
    return Observation(cited_owner != owner, evidence)


def observe_form5_form7(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record the normative-vs-deferred float-canonicality conflict."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    form5_clause = (
        b"the canonical spelling is the unique shortest decimal digit string that "
        b"round-trips under round-to-nearest-even, with at least one integer and "
        b"one fraction digit, lowercase `e`, and no leading zeros; "
    )
    form7_clause = (
        b"The canonical decimal spelling of a float value is gated on the FORM-1 "
        b"reject-vs-canonicalize decision and DEFERRED.\n"
    )
    form5_span = source.unique(form5_clause)
    form7_span = source.unique(form7_clause)
    evidence = {
        "form5_disposition": "normative-unique-shortest-rne-roundtrip",
        "form5_source": source.evidence(*form5_span),
        "form7_disposition": "explicitly-deferred",
        "form7_source": source.evidence(*form7_span),
    }
    if enforce_pins:
        _pin(
            {
                "form5_span": [*form5_span],
                "form5_sha256": evidence["form5_source"]["sha256"],
                "form7_span": [*form7_span],
                "form7_sha256": evidence["form7_source"]["sha256"],
            },
            {
                "form5_span": [9125, 9325],
                "form5_sha256": "3b25dca20138b9ef5f797dae0565329818835c7190f04e1d1b83108800157c6a",
                "form7_span": [11240, 11357],
                "form7_sha256": "268dc6c19eacf8f247fad3a3402746692fee5ce709aeace0b3a1254f497f118e",
            },
            "FORM-5/FORM-7 float-canonicality evidence",
        )
    return Observation(True, evidence)


def observe_fn7(specification: bytes, *, enforce_pins: bool = True) -> Observation:
    """Compare FN-7's main return spelling with grammar and EX-1."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    fn_decl = (
        b'fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"\n'
        b'                "->" rtype effects requires_block? "{" doc? stmt* "}"\n'
    )
    rtype = b"rtype  := mode type\n"
    fn7_signature = b"fn main() -> unit"
    example_line = b"fn main() -> own unit traps {\n"
    fn_decl_span = source.unique(fn_decl)
    rtype_span = source.unique(rtype)
    fn7_span = source.unique(fn7_signature)
    example_span = source.unique(example_line)
    evidence = {
        "example_main_return_spelling": "own unit",
        "example_main_source": source.evidence(*example_span),
        "fn7_main_return_spelling": "unit",
        "fn7_main_source": source.evidence(*fn7_span),
        "fn_decl_return_nonterminal": "rtype",
        "fn_decl_source": source.evidence(*fn_decl_span),
        "rtype_shape": "mode type",
        "rtype_source": source.evidence(*rtype_span),
    }
    if enforce_pins:
        _pin(
            {
                key: evidence[key]["sha256"]
                for key in (
                    "example_main_source",
                    "fn7_main_source",
                    "fn_decl_source",
                    "rtype_source",
                )
            },
            {
                "example_main_source": (
                    "3c21e58f403384c5f0b6f119e1c6e64"
                    "e916b48bea8a58355724ec5a9f4642165"
                ),
                "fn7_main_source": (
                    "2687d72b3742432ba69de3203f96d293"
                    "08347ac8825558f829fa766f6a3b8fc8"
                ),
                "fn_decl_source": (
                    "7937beaad997465d1e80dcc0eae3573d"
                    "3de754bd5b7762a561760bf98adc9508"
                ),
                "rtype_source": "c9e5b6dead005a9feb5a7adb849ce60d0bc0dcd6051f3b64892a0f7c383eeac4",
            },
            "FN-7 discrepancy evidence",
        )
    return Observation(True, evidence)


def observe_gram1_gram7(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Compare the global production bijection with GRAM-7's shared node."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    gram1_clause = (
        b"Every production maps 1:1 to one core-tree node kind; there is no "
        b"desugaring."
    )
    gram7_clause = (
        b"appears in two disjoint productions sharing one core-tree node kind "
        b"[META-1]"
    )
    gram1_span = source.unique(gram1_clause)
    gram7_span = source.unique(gram7_clause)
    evidence = {
        "gram1_constraint": "one-production-to-one-node-kind-bijection",
        "gram1_source": source.evidence(*gram1_span),
        "gram7_constraint": "two-productions-share-one-match-node-kind",
        "gram7_source": source.evidence(*gram7_span),
    }
    if enforce_pins:
        _pin(
            {
                "gram1_span": [*gram1_span],
                "gram1_sha256": evidence["gram1_source"]["sha256"],
                "gram7_span": [*gram7_span],
                "gram7_sha256": evidence["gram7_source"]["sha256"],
            },
            {
                "gram1_span": [12268, 12345],
                "gram1_sha256": "0f10dd0af3c839004ed9fae81ee6118043a10ed9cf48476a0c406f6a3386cfe2",
                "gram7_span": [16129, 16205],
                "gram7_sha256": "d131bea381d9e91bac481198b82a0c931bbff919066db77073cd50a67d9fe414",
            },
            "GRAM-1/GRAM-7 node-bijection evidence",
        )
    return Observation(True, evidence)


def observe_gram_terminal_ident_partition(
    specification: bytes,
    source_index: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record the missing fixed-terminal versus IDENT partition contract."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    source_fields = ("byte_end", "byte_start", "line_end", "line_start", "sha256")

    def indexed_fragment(
        record: Mapping[str, Any],
        label: str,
    ) -> tuple[bytes, dict[str, Any]]:
        indexed_source = record.get("source")
        if not isinstance(indexed_source, dict) or set(indexed_source) != set(
            source_fields
        ):
            raise DiscrepancyError(f"{label} source fields drifted")
        start = indexed_source.get("byte_start")
        end = indexed_source.get("byte_end")
        if (
            isinstance(start, bool)
            or not isinstance(start, int)
            or isinstance(end, bool)
            or not isinstance(end, int)
        ):
            raise DiscrepancyError(f"{label} source span is invalid")
        exact = source.evidence(start, end)
        if {field: exact[field] for field in source_fields} != indexed_source:
            raise DiscrepancyError(f"{label} source is stale")
        return specification[start:end], exact

    productions = source_index.get("syntax_productions")
    if not isinstance(productions, list) or len(productions) != 59:
        raise DiscrepancyError("terminal audit requires exactly 59 syntax productions")
    audited_productions: list[terminal_ident.Production] = []
    production_sources: dict[str, dict[str, Any]] = {}
    seen_productions: set[str] = set()
    for ordinal, record in enumerate(productions):
        if not isinstance(record, dict) or not isinstance(record.get("id"), str):
            raise DiscrepancyError(f"syntax production {ordinal} has invalid identity")
        identifier = record["id"]
        if identifier in seen_productions:
            raise DiscrepancyError(f"duplicate syntax production: {identifier}")
        seen_productions.add(identifier)
        fragment, exact = indexed_fragment(record, f"syntax production {identifier}")
        audited_productions.append(
            terminal_ident.Production(identifier, fragment, exact["sha256"])
        )
        production_sources[identifier] = exact

    try:
        audit = terminal_ident.audit(audited_productions)
    except terminal_ident.GrammarAuditError as error:
        raise DiscrepancyError(str(error)) from error

    rules = source_index.get("rules")
    if not isinstance(rules, list):
        raise DiscrepancyError("terminal audit requires indexed rules")
    indexed_rules: dict[str, Mapping[str, Any]] = {}
    for ordinal, record in enumerate(rules):
        if not isinstance(record, dict) or not isinstance(record.get("id"), str):
            raise DiscrepancyError(f"indexed rule {ordinal} has invalid identity")
        identifier = record["id"]
        if identifier in indexed_rules:
            raise DiscrepancyError(f"duplicate indexed rule: {identifier}")
        indexed_rules[identifier] = record
    required_rules = ("rule:FORM-3", "rule:FORM-5")
    missing_rules = sorted(set(required_rules) - set(indexed_rules))
    if missing_rules:
        raise DiscrepancyError(f"terminal audit is missing rules: {missing_rules}")
    rule_sources = {
        identifier: indexed_fragment(indexed_rules[identifier], identifier)[1]
        for identifier in required_rules
    }

    exact_sources = _exact_fragment_sources(
        source,
        {
            "fn8_requires_reservation": (
                b"`requires` is RESERVED and cannot bind any IDENT declaration."
            ),
            "form3_ident": b"IDENT `[a-z][a-z0-9_]*`",
            "form3_opname": (
                b"OPNAME `[a-z][a-z0-9_]*\\.(wrap|trap|checked|sat|strict)`"
            ),
            "form3_regionid": b"REGIONID `'[a-z][a-z0-9_]*`",
            "form3_typeid": b"TYPEID `[A-Z][A-Za-z0-9]*`",
            "form5_unit_literal": b"`unit`; ",
            "form6_keyword": (
                b"The lowercase spelling follows the primitive-type convention "
                b"(TYPE-1: primitives are lowercase keywords, not TYPEIDs)"
            ),
            "form6_unit_resolution": (
                b"The token `unit` names the unit type in type position and the "
                b"unit value in expression position; the grammar positions are "
                b"disjoint productions, so resolution is production-local, not "
                b"contextual."
            ),
            "gram1_determinism": (
                b"The grammar is deterministic and unambiguous (one parse per "
                b"input; resolved with two-token lookahead where FIRST sets overlap)."
            ),
            "gram6_index_home": b"`index` is a place (its sole home)",
            "meta2_context_independence": (
                b"No context-dependent spellings or rule variants: no rule's "
                b"meaning depends on surrounding context; defaulting rules do not exist."
            ),
            "op1_selective_reservation": (
                b"The dotless operation IDENTs above and the mode-words `wrap` "
                b"`trap` `checked` `sat` `strict` are RESERVED: no `fn_decl`, "
                b"field, param, binder, or region binds them"
            ),
        },
        {
            "fn8_requires_reservation": {
                "byte_end": 53853,
                "byte_start": 53792,
                "sha256": "881850a5d323fcfe7705fc3392ba2fa93191b747efbe21ab9206fef361d08d39",
            },
            "form3_ident": {
                "byte_end": 8321,
                "byte_start": 8298,
                "sha256": "f2655036507f78d0d0dacc6aa8556cf2fee139401d9bbeab590343de856ac909",
            },
            "form3_opname": {
                "byte_end": 8510,
                "byte_start": 8454,
                "sha256": "c51c99225e3ada9d5743c4d68bc1032584d80ec6b9e031d0b2d0e1ef50f0a0a5",
            },
            "form3_regionid": {
                "byte_end": 8378,
                "byte_start": 8351,
                "sha256": "a3172b34a9e0bad32c0b6ff9f897501b67de272dac52d0b3d205625e0cd829e5",
            },
            "form3_typeid": {
                "byte_end": 8349,
                "byte_start": 8323,
                "sha256": "252294dda9f8eb6ee8ab0f9609444ee4b7c0b7051e058f439cbaa7dccce14c35",
            },
            "form5_unit_literal": {
                "byte_end": 9397,
                "byte_start": 9389,
                "sha256": "32745a55c4b621a035287bd17fbbe56552aa159b5d5195947f9805de07ef4b95",
            },
            "form6_keyword": {
                "byte_end": 10518,
                "byte_start": 10401,
                "sha256": "0e3a17cb22f0e356c002afbc4c7cc40c211e4da00e5c52edbeb1cd4942c75c85",
            },
            "form6_unit_resolution": {
                "byte_end": 10400,
                "byte_start": 10205,
                "sha256": "a8b1e0b461438d1d9eca3551378100f537c96b609f6231dd7cdf55660e6cc54a",
            },
            "gram1_determinism": {
                "byte_end": 12267,
                "byte_start": 12140,
                "sha256": "0a5099694735abd2a68f7b04f7d2ac2b29b48f97f6364f13123eab81aac0692d",
            },
            "gram6_index_home": {
                "byte_end": 16034,
                "byte_start": 16000,
                "sha256": "ace5cb8ae0cf8ee1c6f5aa07129add47cbb6c0e22b284659b3805ebd8d91ecf4",
            },
            "meta2_context_independence": {
                "byte_end": 62815,
                "byte_start": 62686,
                "sha256": "236063e683e4e0f1b13f2279230e64e3041f8d07f6b6153e36da930fe865c3bc",
            },
            "op1_selective_reservation": {
                "byte_end": 40719,
                "byte_start": 40556,
                "sha256": "00adc90c57057478c47cdc4048be08a36a167bbcdf380363d7692b90c44b02d6",
            },
        },
        "terminal versus IDENT evidence",
        enforce_pins=enforce_pins,
    )
    if enforce_pins:
        _pin(
            audit.pin_payload(),
            terminal_ident.EXPECTED_PIN_PAYLOAD,
            "terminal versus IDENT census",
        )
        _pin(
            {
                identifier: {
                    "byte_end": rule_sources[identifier]["byte_end"],
                    "byte_start": rule_sources[identifier]["byte_start"],
                    "sha256": rule_sources[identifier]["sha256"],
                }
                for identifier in required_rules
            },
            {
                "rule:FORM-3": {
                    "byte_end": 8689,
                    "byte_start": 8272,
                    "sha256": "a8d051a71ed3c2b30a7bdaf311906b763a4f582ac98c696e167b68d3ceb07740",
                },
                "rule:FORM-5": {
                    "byte_end": 10195,
                    "byte_start": 8821,
                    "sha256": "ccb123759eac5548f7c9432545dc95fdadbec32213e3a655b8047c90e72b6126",
                },
            },
            "terminal versus IDENT rule sources",
        )

    return Observation(
        True,
        {
            **exact_sources,
            **audit.evidence(),
            "form3_rule_source": rule_sources["rule:FORM-3"],
            "form5_rule_source": rule_sources["rule:FORM-5"],
            "missing_contract": "fixed-terminal-versus-ident-priority-or-exclusion",
            "production_sources": {
                identifier: production_sources[identifier]
                for identifier in terminal_ident.REQUIRED_PRODUCTION_SHAPES
            },
        },
    )


def observe_affine_deref_lifecycle(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record affine referent move permission beside unspecified cleanup."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "own1_partial_move": (
                b"After a move, the whole binding rooting `p` is dead "
                b"(partial moves kill the whole binding)"
            ),
            "stor2_deref_access": b"Content access is through `deref`.",
            "stor3_derived_drop": (
                b"every drop and arena release appears as an explicit operation "
                b"in the elaborated artifact"
            ),
            "type7_affine_move": (
                b"a use of that place copies it when T is copy and requires `move` "
                b"when T is affine [OWN-1]"
            ),
        },
        {
            "own1_partial_move": {
                "byte_end": 26085,
                "byte_start": 25995,
                "sha256": (
                    "385e180cf9a88c4fc0a2673b3a2a710f"
                    "a8f27b7006dcef1c74d7c3acae2404fc"
                ),
            },
            "stor2_deref_access": {
                "byte_end": 34917,
                "byte_start": 34883,
                "sha256": (
                    "4e6f6d92f02bc31211cf29b2c676cd18"
                    "91dffcb246752d9ed97cf2c70b57ca25"
                ),
            },
            "stor3_derived_drop": {
                "byte_end": 35072,
                "byte_start": 34984,
                "sha256": (
                    "450ffec2a8008c821a4181e34333a4c55"
                    "bd938408183f45d4554779ea803dfad"
                ),
            },
            "type7_affine_move": {
                "byte_end": 23201,
                "byte_start": 23112,
                "sha256": (
                    "9e03775a3ee20b6d5779bb6fbe51ab8c"
                    "a921cf58907d431e07a04714e86c0283"
                ),
            },
        },
        "affine deref lifecycle evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = (
        "backing-allocation-and-remaining-payload-disposition-after-affine-referent-move"
    )
    return Observation(True, evidence)


def observe_eff1_row_canonicality(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record underdefined row canonicality and the protected rejection."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "effect_kind_order": (
                b"in exactly this canonical order (reads, writes, allocates, traps)"
            ),
            "row_grammar": b"Row grammar: `effects := \"pure\" | effect (\",\" effect)*`",
        },
        {
            "effect_kind_order": {
                "byte_end": 55810,
                "byte_start": 55745,
                "sha256": (
                    "8294f4112b0cca9c4c12fd43e3a1b702"
                    "9187dd480248961b653f58dbddbb0763"
                ),
            },
            "row_grammar": {
                "byte_end": 55610,
                "byte_start": 55555,
                "sha256": (
                    "bca021af57224f0154c77083fe0baf3e"
                    "d3b0a0b88aea8e2b9189b934cd59ea15"
                ),
            },
        },
        "EFF-1 row-canonicality evidence",
        enforce_pins=enforce_pins,
    )
    evidence["protected_case"] = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "x-eff-dup-reads-effect",
    )
    evidence["missing_contract"] = (
        "effect-kind-and-region-or-allocation-entry-uniqueness-and-order"
    )
    return Observation(True, evidence)


def observe_eff2_local_region_effects(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record exact row checking where body-local regions cannot be named."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "eff2_bidirectional_check": (
                b"Rows are checked both ways against the syntactic definition"
            ),
            "eff2_local_exhibits": (
                b"they exhibit reads/writes/allocates per the operation table "
                b"and borrow modes they use"
            ),
            "example_local_region": (
                b"region 'r {\n    let p: &'r i32 = &'r a;"
            ),
            "fn7_main_effect_ceiling": (
                b"effect row at most `allocates(heap), traps`"
            ),
        },
        {
            "eff2_bidirectional_check": {
                "byte_end": 56375,
                "byte_start": 56316,
                "sha256": (
                    "eb726a4ce9068a4da2877415481fa8539"
                    "96b5f2679610f97460a373b44fd8096"
                ),
            },
            "eff2_local_exhibits": {
                "byte_end": 56314,
                "byte_start": 56229,
                "sha256": (
                    "85180ce9cdf2eddb8a9cbc45651832517"
                    "dd2258bede1efd8aa974e411ef2a07a"
                ),
            },
            "example_local_region": {
                "byte_end": 62093,
                "byte_start": 62054,
                "sha256": (
                    "ea63a7f4873f5f2b803e5f10e71f6730"
                    "026ecbe63389a5450331c3d82b0ce886"
                ),
            },
            "fn7_main_effect_ceiling": {
                "byte_end": 52805,
                "byte_start": 52762,
                "sha256": (
                    "a206d85444b96a73e8c06f8bd745ea46"
                    "fae4bfde3a281b26900bce4cbe4c982d"
                ),
            },
        },
        "EFF-2 local-region evidence",
        enforce_pins=enforce_pins,
    )
    evidence["protected_case"] = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "stor4-pos-arena-confined",
    )
    evidence["missing_contract"] = "body-local-region-effect-discharge-or-row-projection"
    return Observation(True, evidence)


def observe_fn4_law_admission(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record law-fact channels without a defined universal proof contract."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "fn4_fact_admission": (
                b"Laws become optimizer-usable facts only via the stated-and-checked channel"
            ),
            "op5_deferred_vocabulary": (
                b"the fuller stated-and-checked vocabulary (loop invariants, ranges) "
                b"is DEFERRED"
            ),
        },
        {
            "fn4_fact_admission": {
                "byte_end": 51489,
                "byte_start": 51415,
                "sha256": (
                    "fedf1625497c0d6087d80e4b15200dbb"
                    "e96870652b60890095012bfb75d028bb"
                ),
            },
            "op5_deferred_vocabulary": {
                "byte_end": 43209,
                "byte_start": 43131,
                "sha256": (
                    "d3066130c269f160e09c9ffda0bef1e8"
                    "fff45a536959c470606248a060f2bb5e"
                ),
            },
        },
        "FN-4 law-admission evidence",
        enforce_pins=enforce_pins,
    )
    evidence["protected_case"] = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "fn4-neg-law-undischarged",
    )
    evidence["missing_contract"] = (
        "universal-law-proof-schema-scope-and-declaration-acceptance"
    )
    return Observation(True, evidence)


def observe_contract_member_semantics(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record declared contracts without a complete member checking relation."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "fn3_conformance": (
                b"a `contract` declares fn signatures and laws; "
                b"`conform T : C { member = fn; }` declares conformance, "
                b"checked per member"
            ),
            "fn5_behavior": (
                b"Behavior parameterization is generics over contract-conforming "
                b"types (env-struct pattern)"
            ),
        },
        {
            "fn3_conformance": {
                "byte_end": 50843,
                "byte_start": 50724,
                "sha256": (
                    "e047d27e9a2bc0feef8cf97911c9eba2"
                    "723057e8bcdfb30c195a13ac237e8b89"
                ),
            },
            "fn5_behavior": {
                "byte_end": 51944,
                "byte_start": 51855,
                "sha256": (
                    "19bf72318a692d1ef8ddd3f9b83334e3"
                    "e2c3456e1a9ed644f0b360be29420c13"
                ),
            },
        },
        "contract-member semantics evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = (
        "member-set-signature-effect-substitution-law-and-call-resolution"
    )
    return Observation(True, evidence)


def observe_fn8_reserved_rule_attribution(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Compare FN-8's reservation rule with the protected cited rule."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "fn8_reservation": (
                b"`requires` is RESERVED and cannot bind any IDENT declaration"
            ),
        },
        {
            "fn8_reservation": {
                "byte_end": 53852,
                "byte_start": 53792,
                "sha256": (
                    "bc4d0e65117de04cde9031c32d327959"
                    "77189aee2d061acb110325337127d854"
                ),
            },
        },
        "FN-8 reserved-word evidence",
        enforce_pins=enforce_pins,
    )
    protected = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "form3-neg-requires-binding",
    )
    evidence["protected_case"] = protected
    expected = protected["manifest"].get("expect")
    cited_rule = expected.get("rule") if isinstance(expected, dict) else None
    evidence["protected_cited_rule"] = cited_rule
    evidence["specification_owner_rule"] = "FN-8"
    return Observation(cited_rule != "FN-8", evidence)


def observe_diag3_retained_proof_ref(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record an all-required field whose retained-check value is undefined."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "check_report_row": (
                b"| check | function; per check: node_path, fact_class "
                b"(bounds/overflow/alias/user), status (retained/eliminated), "
                b"proof_ref (for eliminated: checker-derivation id) |"
            ),
            "report_header": b"| report | fields (all required) |",
        },
        {
            "check_report_row": {
                "byte_end": 59549,
                "byte_start": 59385,
                "sha256": (
                    "54c2c2e96227c74da8c6cefd61029eca"
                    "7146a57a58363622d6d46a821f90e17c"
                ),
            },
            "report_header": {
                "byte_end": 59257,
                "byte_start": 59223,
                "sha256": (
                    "f403fac660ee6442a1a568edd22055cc8"
                    "c476fe9e829de352d1eebf9471e8518"
                ),
            },
        },
        "DIAG-3 retained-proof-ref evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = "proof_ref-value-for-retained-check"
    return Observation(True, evidence)


def observe_diag1_pre_tree_node_path(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record the missing location contract for rejection before a tree exists."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.8.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "diag1_universal_path": (
                b"Every rejection cites exactly one rule ID, the node path in the "
                b"canonical tree, and where applicable a mechanical fix or restructuring."
            ),
            "form1_noncanonical_hard_error": (
                b"Non-canonical input is a hard error; "
            ),
            "form1_unknown_hard_error": (
                b"Unknown constructs are hard errors (conservative extension).\n"
            ),
            "form2_utf8_requirement": b"Formatting, exhaustively: UTF-8;",
            "gram1_node_contract": (
                b"Every production maps 1:1 to one core-tree node kind; "
                b"there is no desugaring."
            ),
            "scope2_parse_requirement": (
                b"[SCOPE-2] A program is accepted iff it parses under the "
                b"canonical grammar, "
            ),
        },
        {
            "diag1_universal_path": {
                "byte_end": 58633,
                "byte_start": 58498,
                "sha256": (
                    "ea4b9d5fe33924f33dda38e02b906c5c"
                    "14d76250014f31c2af299f626eb4a9f5"
                ),
            },
            "form1_noncanonical_hard_error": {
                "byte_end": 7696,
                "byte_start": 7659,
                "sha256": (
                    "be6129f4b4e6c6781688d5b56718ec57"
                    "99b804ccd47a63c9f498a5697c63f6f5"
                ),
            },
            "form1_unknown_hard_error": {
                "byte_end": 7791,
                "byte_start": 7730,
                "sha256": (
                    "a6db94e026f58654700c92a544473ed2"
                    "0d04796bd1717e277d5206c28807ff53"
                ),
            },
            "form2_utf8_requirement": {
                "byte_end": 7833,
                "byte_start": 7801,
                "sha256": (
                    "7449c18148efd9b3fc9c7e16f0977a9b"
                    "409af9e8345871cddd00f694d05f10d0"
                ),
            },
            "gram1_node_contract": {
                "byte_end": 12345,
                "byte_start": 12268,
                "sha256": (
                    "0f10dd0af3c839004ed9fae81ee611804"
                    "3a10ed9cf48476a0c406f6a3386cfe2"
                ),
            },
            "scope2_parse_requirement": {
                "byte_end": 6696,
                "byte_start": 6621,
                "sha256": (
                    "b3cc1e52a8c187dccd3f3f8d21b430eb"
                    "090a37d52e541f3cfe5cf5e092eab6af"
                ),
            },
        },
        "DIAG-1 pre-tree node-path evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = (
        "pre-canonical-tree-rejection-location-representation"
    )
    return Observation(True, evidence)


def validate_registry(
    observations: Mapping[str, Observation],
) -> dict[str, Registration]:
    """Require a bijection between registrations and recomputed predicates."""
    registrations: dict[str, Registration] = {}
    predicate_ids: set[str] = set()
    for registration in REGISTRATIONS:
        if registration.identifier in registrations:
            raise DiscrepancyError(
                f"duplicate discrepancy registration: {registration.identifier}"
            )
        if registration.predicate_identifier in predicate_ids:
            raise DiscrepancyError(
                f"duplicate discrepancy predicate: {registration.predicate_identifier}"
            )
        if not registration.affected_facet_ids or list(
            registration.affected_facet_ids
        ) != sorted(set(registration.affected_facet_ids)):
            raise DiscrepancyError(
                f"affected facets are empty, duplicated, or unsorted: {registration.identifier}"
            )
        if not registration.resolution_authorities or list(
            registration.resolution_authorities
        ) != sorted(set(registration.resolution_authorities)):
            raise DiscrepancyError(
                "resolution authorities are empty, duplicated, or unsorted: "
                f"{registration.identifier}"
            )
        registrations[registration.identifier] = registration
        predicate_ids.add(registration.predicate_identifier)
    registered_ids = set(registrations)
    observed_ids = set(observations)
    if registered_ids != observed_ids:
        raise DiscrepancyError(
            "discrepancy registrations and predicate observations differ; "
            f"unobserved={sorted(registered_ids - observed_ids)}, "
            f"unregistered={sorted(observed_ids - registered_ids)}"
        )
    return registrations


def recompute(authorities: inputs.AuthorityInputs) -> dict[str, Observation]:
    """Recompute every registered predicate from one authority snapshot."""
    observations = {
        "discrepancy:v0.8/affine-deref-storage-lifecycle": (
            observe_affine_deref_lifecycle(authorities.specification)
        ),
        "discrepancy:v0.8/diag1-pre-tree-node-path": (
            observe_diag1_pre_tree_node_path(authorities.specification)
        ),
        "discrepancy:v0.8/diag3-retained-proof-ref": (
            observe_diag3_retained_proof_ref(authorities.specification)
        ),
        "discrepancy:v0.8/eff1-row-canonicality": observe_eff1_row_canonicality(
            authorities.specification,
            authorities.manifest,
            authorities.case_sources,
            authorities.protected_conformance,
        ),
        "discrepancy:v0.8/eff2-local-region-effects": (
            observe_eff2_local_region_effects(
                authorities.specification,
                authorities.manifest,
                authorities.case_sources,
                authorities.protected_conformance,
            )
        ),
        "discrepancy:v0.8/fn3-contract-member-semantics": (
            observe_contract_member_semantics(authorities.specification)
        ),
        "discrepancy:v0.8/fn4-law-admission": observe_fn4_law_admission(
            authorities.specification,
            authorities.manifest,
            authorities.case_sources,
            authorities.protected_conformance,
        ),
        "discrepancy:v0.8/fn8-reserved-rule-attribution": (
            observe_fn8_reserved_rule_attribution(
                authorities.specification,
                authorities.manifest,
                authorities.case_sources,
                authorities.protected_conformance,
            )
        ),
        "discrepancy:v0.8/op1-dotless-reservation": observe_op1(
            authorities.specification
        ),
        "discrepancy:v0.8/form2-protected-conformance-spacing": observe_form2(
            authorities.manifest,
            authorities.case_sources,
            protected_surface=authorities.protected_conformance,
        ),
        "discrepancy:v0.8/form4-doc-cross-reference": observe_form4(
            authorities.specification, authorities.source_index
        ),
        "discrepancy:v0.8/form5-form7-float-canonical-spelling": observe_form5_form7(
            authorities.specification
        ),
        "discrepancy:v0.8/fn7-main-return-spelling": observe_fn7(
            authorities.specification
        ),
        "discrepancy:v0.8/gram1-gram7-match-node-bijection": observe_gram1_gram7(
            authorities.specification
        ),
        "discrepancy:v0.8/gram-terminal-ident-partition": (
            observe_gram_terminal_ident_partition(
                authorities.specification,
                authorities.source_index,
            )
        ),
    }
    validate_registry(observations)
    return observations
