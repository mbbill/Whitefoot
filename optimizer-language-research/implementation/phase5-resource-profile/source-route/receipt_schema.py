"""Closed structural vocabulary shared by source-route receipt components."""

from counts import FIELD_NAMES, TRACE_FIELDS


RECEIPT_SCHEMA = "whitefoot-resource-source-route-receipt-v1"
BUNDLE_DOMAIN = b"WHITEFOOT-SOURCE-ROUTE-BUNDLE-V2\0"
U64_MAX = (1 << 64) - 1
TRACE_FIELD_ORDER = tuple(name for name in FIELD_NAMES if name in TRACE_FIELDS)
BASE_DERIVED = frozenset(
    {
        "canonical_path_depth",
        "classified_tokens",
        "diagnostic_issue_elements",
        "gaps",
        "mixed_elements",
        "private_derivation_elements",
        "source_extents",
        "terminals",
    }
)
ADMITTED_DERIVED = frozenset(
    {
        "ancestry_steps",
        "coverage_records",
        "declarations",
        "diagnostic_origins",
        "diagnostic_path_components",
        "diagnostic_paths",
        "lookup_entries",
    }
)
NOT_DERIVED_FIELDS = frozenset(
    {
        "max_declarations",
        "max_scopes",
        "max_scope_depth",
        "max_declaration_events",
        "max_lexical_uses",
        "max_deferred_uses",
        "max_spelling_bytes",
        "max_lookup_entries",
        "max_ancestry_steps",
        "max_coverage_records",
    }
)
TOP_LEVEL_KEYS = frozenset(
    {
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
)
