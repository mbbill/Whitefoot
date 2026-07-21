"""Closed record replay for the neutral source-coverage ledger."""

from __future__ import annotations

import hashlib
from typing import Optional

from runner_common_schema import (
    DocumentSchema,
    FixedRecord,
    LexicalRecord,
    NodeRecord,
    ProductionRecord,
    ReferenceRecord,
    finish_schema,
    fixed_expansion,
)
from runner_common_wire import (
    COMMON_FIELDS,
    COMMON_RANK,
    PRODUCTION_NAME,
    SURFACE_KINDS,
    canonical_uint,
    decoded_bytes,
    record_primary,
    semantic_key,
)
from runner_inputs import Inputs, fail


BIND_PREFIX = b"BIND"


def validate_common(lines: list[bytes], inputs: Inputs) -> dict[bytes, DocumentSchema]:
    expected_binds = [
        b"\t".join(
            (
                BIND_PREFIX,
                section.name.encode("ascii"),
                str(len(section.data)).encode("ascii"),
                hashlib.sha256(section.data).hexdigest().encode("ascii"),
            )
        )
        for section in inputs.sections
    ]
    if lines[: len(expected_binds)] != expected_binds:
        fail("report_binding", "an engine report does not bind the exact frame sections")
    sources = {name: inputs.section(name).data for name in ("current", "proposal")}
    sizes = {name: len(source) for name, source in sources.items()}
    previous: Optional[tuple[object, ...]] = None
    coverage: set[str] = set()
    surface_counts = {
        document: {kind: 0 for kind in SURFACE_KINDS}
        for document in ("current", "proposal")
    }
    coverage_claims: dict[str, tuple[int, int, int, int, int]] = {}
    rule_counts = {document: 0 for document in sizes}
    productions: dict[str, dict[bytes, ProductionRecord]] = {
        document: {} for document in sizes
    }
    nodes: dict[str, dict[tuple[bytes, bytes], NodeRecord]] = {
        document: {} for document in sizes
    }
    lexical: dict[str, dict[bytes, LexicalRecord]] = {
        document: {} for document in sizes
    }
    fixed: dict[str, dict[tuple[bytes, bytes], FixedRecord]] = {
        document: {} for document in sizes
    }
    references: dict[str, dict[tuple[bytes, bytes], ReferenceRecord]] = {
        document: {} for document in sizes
    }
    exact_seen: set[bytes] = set()
    semantic_seen: set[tuple[object, ...]] = set()
    for line in lines[len(expected_binds) :]:
        fields = line.split(b"\t")
        try:
            tag = fields[0].decode("ascii")
            document = fields[1].decode("ascii")
        except (IndexError, UnicodeDecodeError):
            fail("report_common", "a common record has an invalid tag or document")
        if (
            tag == "BIND"
            or tag not in COMMON_FIELDS
            or len(fields) != COMMON_FIELDS[tag]
            or line in exact_seen
        ):
            fail("report_common", "a common record has an unknown shape or duplicate")
        if document not in sizes:
            fail("report_document", "a common record names an unknown document")
        primary = record_primary(tag, fields, sizes[document])
        record_key = semantic_key(tag, document, fields)
        if record_key in semantic_seen:
            fail("report_duplicate", "common records repeat a semantic key")
        exact_seen.add(line)
        semantic_seen.add(record_key)
        if tag == "RULE":
            rule_counts[document] += 1
        elif tag == "COVERAGE":
            if document in coverage or fields[6] != b"0":
                fail(
                    "report_coverage",
                    "coverage is duplicate or leaves an unclassified surface",
                )
            coverage.add(document)
            coverage_claims[document] = tuple(
                canonical_uint(field) for field in fields[2:]
            )
        elif tag == "SURFACE":
            surface_counts[document][fields[2]] += 1
        elif tag == "PROD":
            lhs = decoded_bytes(fields[3])
            if not PRODUCTION_NAME.fullmatch(lhs):
                fail(
                    "report_schema",
                    "a production name is outside the closed grammar form",
                )
            productions[document][fields[3]] = ProductionRecord(
                (canonical_uint(fields[4]), canonical_uint(fields[5])),
                (canonical_uint(fields[6]), canonical_uint(fields[7])),
            )
        elif tag == "NODE":
            nodes[document][(fields[2], fields[3])] = NodeRecord(
                fields[4],
                fields[7],
                (canonical_uint(fields[5]), canonical_uint(fields[6])),
            )
        elif tag == "LEX":
            lexical[document][fields[3]] = LexicalRecord(
                fields[4],
                (canonical_uint(fields[5]), canonical_uint(fields[6])),
                fields[7],
            )
        elif tag == "FIXED":
            fixed[document][(fields[2], fields[3])] = FixedRecord(
                fields[6],
                fixed_expansion(fields[7], fields[6]),
                (canonical_uint(fields[4]), canonical_uint(fields[5])),
            )
        elif tag == "REF":
            references[document][(fields[2], fields[3])] = ReferenceRecord(
                fields[6],
                (canonical_uint(fields[4]), canonical_uint(fields[5])),
            )
        key = (0 if document == "current" else 1, primary, COMMON_RANK[tag], line)
        if previous is not None and key < previous:
            fail("report_order", "common records are not in canonical order")
        previous = key
    if coverage != {"current", "proposal"}:
        fail("report_coverage", "a successful report omits complete document coverage")
    for document, counts in surface_counts.items():
        observed = tuple(counts[kind] for kind in SURFACE_KINDS) + (0,)
        if coverage_claims[document] != observed:
            fail(
                "report_coverage",
                "coverage counters do not equal the complete surface ledger",
            )
    return {
        document.encode("ascii"): finish_schema(
            sources[document],
            inputs,
            rule_counts[document],
            productions[document],
            nodes[document],
            lexical[document],
            fixed[document],
            references[document],
        )
        for document in sizes
    }
