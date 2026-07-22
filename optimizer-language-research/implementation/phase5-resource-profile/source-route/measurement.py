"""Source-to-role measurement and receipt assembly for route A."""

from __future__ import annotations

from hashlib import sha256
import struct

from counts import CountProjection, project_counts
from fn8 import select_fn8
from identities import (
    CANDIDATE_SHA256,
    MEANING_DIGESTS,
    PROPOSAL_SHA256,
    meaning_file_hashes,
    verify_identities,
)
from inventory_selection import select_inventory_issue
from lexical_selection import select_resolution_issue
from manifest import decode_manifest
from model import LogicalSource
from parser_adapter import parse_bundle
from receipt_schema import BUNDLE_DOMAIN, RECEIPT_SCHEMA
from resolution import build_declarations
from roles import project_roles
from topology import build_projection_context


def _bundle_digest(sources: tuple[LogicalSource, ...]) -> str:
    output = bytearray(BUNDLE_DOMAIN)
    output.extend(struct.pack(">Q", len(sources)))
    for source in sources:
        path = source.logical_path.encode("ascii")
        output.extend(struct.pack(">Q", len(path)))
        output.extend(path)
        output.extend(struct.pack(">Q", len(source.source)))
        output.extend(sha256(source.source).digest())
    return sha256(output).hexdigest()


def _trace_gaps() -> list[dict[str, object]]:
    return [
        {
            "allowed_inputs": ["exact source bytes", "independent token spans"],
            "field": "max_lexical_scan_work",
            "required_replay": "two immutable scanner passes with every byte/probe/comparison/validation/commit/write charge",
            "tag": 9,
        },
        {
            "allowed_inputs": [
                "independent tokens",
                "independent production tree",
                "successor grammar bytes",
            ],
            "field": "max_parser_stack_entries",
            "required_replay": "prospective combined task-plus-frame occupancy at every generated-grammar push",
            "tag": 14,
        },
        {
            "allowed_inputs": [
                "independent tokens",
                "independent production tree",
                "successor grammar bytes",
            ],
            "field": "max_list_members",
            "required_replay": "every successful RepeatZero and RepeatOne member selection before scheduling",
            "tag": 15,
        },
        {
            "allowed_inputs": [
                "independent tokens",
                "successor grammar bytes",
                "selected syntax failure",
            ],
            "field": "max_expected_terminals",
            "required_replay": "DIAG-1 diagnostic descent, row revisits, overrides, and distinct expected-set construction",
            "tag": 16,
        },
        {
            "allowed_inputs": [
                "independent source bytes",
                "independent tokens",
                "independent production tree",
            ],
            "field": "max_syntax_work",
            "required_replay": "one carried meter across classification, generated parsing, finalization, and canonical audit",
            "tag": 17,
        },
        {
            "allowed_inputs": [
                "independent mixed traversal",
                "independent scopes",
                "independent roles",
                "selected semantic diagnostic",
            ],
            "field": "max_resolution_work",
            "required_replay": "R-04 FN-8/preflight/construction/four-sort/inventory/query/two-origin-scan/materialization schedule",
            "tag": 33,
        },
    ]


def measure(
    sources: tuple[LogicalSource, ...], manifest_raw: bytes | None = None
) -> dict[str, object]:
    """Produce the sound partial receipt; never fabricate an algorithm trace."""

    identities = verify_identities()
    parsed = parse_bundle(sources)
    context = build_projection_context(parsed)
    fn8_issue = select_fn8(parsed, context)
    roles = None
    declarations = 0
    if fn8_issue is None:
        roles = project_roles(parsed, context)
        declaration_facts = build_declarations(roles, context)
        diagnostic = select_inventory_issue(roles, declaration_facts, context)
        if diagnostic is None:
            diagnostic = select_resolution_issue(roles, declaration_facts, context)
        declarations = len(declaration_facts)
    else:
        diagnostic = fn8_issue
    projection: CountProjection = project_counts(parsed, context, roles, diagnostic)
    workload = (
        None if manifest_raw is None else decode_manifest(manifest_raw, sources)
    )
    return {
        "agreement_derived_counts": projection.agreement_derived,
        "counts": list(projection.fields),
        "derived_counts": projection.derived,
        "identities": {
            "candidate_sha256": CANDIDATE_SHA256,
            "meaning_files": meaning_file_hashes(),
            "meaning_sha256": MEANING_DIGESTS,
            "parser_audit_sha256": identities["audited_parser_set"],
            "profile_schema_sha256": identities["schema.py"],
            "proposal_sha256": PROPOSAL_SHA256,
            "route_code_sha256": identities["source_route_code"],
        },
        "projection_summary": {
            "declaration_facts_including_prelude_lookup": declarations,
            "production_nodes": context.production_nodes,
            "role_occurrences": 0 if roles is None else len(roles),
            "scopes": len(context.scopes),
            "terminals": context.terminals,
        },
        "schema": RECEIPT_SCHEMA,
        "selected_diagnostic": None if diagnostic is None else diagnostic.as_json(),
        "source_bundle": {
            "sha256": _bundle_digest(sources),
            "sources": [
                {
                    "byte_length": len(source.source),
                    "logical_path": source.logical_path,
                    "sha256": sha256(source.source).hexdigest(),
                }
                for source in sources
            ],
        },
        "spelling_components": projection.spelling_components,
        "status": "trace-incomplete",
        "trace_gaps": _trace_gaps(),
        "workload": workload,
    }
