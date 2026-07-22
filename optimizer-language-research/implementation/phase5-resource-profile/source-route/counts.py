"""Exact source/tree/role/scope counts and checked receipt equations."""

from __future__ import annotations

from dataclasses import dataclass

from model import ParsedSource, Role, SelectedDiagnostic
from roles import (
    DECLARATION_ROLES,
    PRELUDE_RECORDS,
    fixed_spelling_charge,
    operation_spellings,
)
from topology import ProjectionContext, lexical_partition_counts


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
TRACE_FIELDS = frozenset(
    {
        "max_lexical_scan_work",
        "max_parser_stack_entries",
        "max_list_members",
        "max_expected_terminals",
        "max_syntax_work",
        "max_resolution_work",
    }
)
AGREEMENT_DERIVED_NAMES = (
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


@dataclass(frozen=True)
class CountProjection:
    """Every field state plus independently checked derived receipts."""

    fields: tuple[dict[str, object], ...]
    derived: dict[str, int]
    agreement_derived: dict[str, int] | None
    spelling_components: dict[str, int]


def _exact(value: int) -> dict[str, object]:
    if value < 0 or value > (1 << 64) - 1:
        raise OverflowError("exact count does not fit u64")
    return {"state": "exact", "value": value}


def _trace(reason: str) -> dict[str, object]:
    return {"reason": reason, "state": "trace-required"}


def _not_derived(reason: str) -> dict[str, object]:
    return {"reason": reason, "state": "not-derived"}


def _diagnostic_counts(
    diagnostic: SelectedDiagnostic | None,
) -> tuple[int, int, int, int, int, int]:
    if diagnostic is None:
        return 0, 0, 0, 0, 0, 0
    source_origins = tuple(
        origin for origin in diagnostic.origins if origin.kind == "source"
    )
    origins = len(diagnostic.origins)
    paths = 1 + len(source_origins)
    path_components = len(diagnostic.node_path) + sum(
        len(origin.node_path) for origin in source_origins
    )
    path_depth = max(
        (len(diagnostic.node_path), *(len(origin.node_path) for origin in source_origins))
    )
    issue_elements = 1 + origins + paths + path_components
    return (
        origins,
        len(source_origins),
        paths,
        path_components,
        path_depth,
        issue_elements,
    )


def project_counts(
    parsed: tuple[ParsedSource, ...],
    context: ProjectionContext,
    roles: tuple[Role, ...] | None,
    diagnostic: SelectedDiagnostic | None,
) -> CountProjection:
    """Compute every source-derived field and mark algorithm traces unavailable."""

    sources = len(parsed)
    path_lengths = tuple(len(source.logical_path.encode("ascii")) for source in parsed)
    source_lengths = tuple(len(source.source) for source in parsed)
    total_source_bytes = sum(source_lengths)
    binding_bytes = 50 + sum(
        16 + path_bytes + source_bytes
        for path_bytes, source_bytes in zip(path_lengths, source_lengths)
    )
    all_tokens = tuple(token for source in parsed for token in source.tokens)
    tokens = len(all_tokens)
    token_bytes = max((len(token.raw) for token in all_tokens), default=0)
    lexemes = sum(
        lexical_partition_counts(source.source, source.tokens)[1] for source in parsed
    )
    nodes = context.production_nodes
    mixed = nodes - 1 + tokens
    private_elements = nodes + tokens
    tree_bytes = (
        private_elements * 64
        + nodes * 128
        + mixed * 16
        + tokens * 32
        + sources * 24
    )
    (
        origins,
        source_origins,
        paths,
        path_components,
        diagnostic_depth,
        issue_elements,
    ) = _diagnostic_counts(diagnostic)

    exact: dict[str, int] = {
        "max_sources": sources,
        "max_logical_path_bytes": max(path_lengths, default=0),
        "max_source_bytes": max(source_lengths, default=0),
        "max_total_source_bytes": total_source_bytes,
        "max_binding_bytes": binding_bytes,
        "max_token_bytes": token_bytes,
        "max_tokens": tokens,
        "max_lexemes": lexemes,
        "max_classified_tokens": tokens,
        "max_production_nodes": nodes,
        "max_mixed_elements": mixed,
        "max_tree_depth": context.tree_depth,
        "max_tree_bytes": tree_bytes,
        "max_node_path_depth": diagnostic_depth,
        "max_diagnostic_origins": origins,
        "max_diagnostic_paths": paths,
        "max_diagnostic_path_components": path_components,
    }
    spelling_components: dict[str, int] = {}
    admitted = roles is not None
    if admitted:
        prelude_declarations = len(PRELUDE_RECORDS)
        prelude_lookup_entries = sum(
            declaration_class in {"nominal-type", "enum-variant", "contract"}
            for _, declaration_class in PRELUDE_RECORDS
        )
        operation_lookup_entries = len(operation_spellings())
        exact["max_node_path_depth"] = max(context.tree_depth, diagnostic_depth)
        declaration_events = sum(role.role_id in DECLARATION_ROLES for role in roles)
        lexical_uses = sum(role.role_class == "lexical-use" for role in roles)
        deferred_uses = sum(role.role_class == "deferred-use" for role in roles)
        source_lookup_entries = sum(
            2 if role.role_id == "D02" else 1
            for role in roles
            if role.role_id.startswith("D")
        )
        fixed_charge, spelling_components = fixed_spelling_charge()
        spelling_components = {
            **spelling_components,
            "source_roles": sum(len(role.spelling) for role in roles),
        }
        exact.update(
            max_declarations=declaration_events + prelude_declarations,
            max_scopes=len(context.scopes),
            max_scope_depth=max(scope.depth for scope in context.scopes),
            max_declaration_events=declaration_events,
            max_lexical_uses=lexical_uses,
            max_deferred_uses=deferred_uses,
            max_spelling_bytes=fixed_charge + spelling_components["source_roles"],
            max_lookup_entries=(
                prelude_lookup_entries
                + operation_lookup_entries
                + source_lookup_entries
            ),
            max_ancestry_steps=len(context.scopes) - 1,
            max_coverage_records=nodes + len(roles),
        )

    trace_reasons = {
        "max_lexical_scan_work": "requires an independent two-pass metered scanner replay",
        "max_parser_stack_entries": "requires an independent generated-grammar task/frame replay",
        "max_list_members": "requires independent RepeatZero/RepeatOne arm-selection tracing",
        "max_expected_terminals": "requires independent DIAG-1 descent and expected-set tracing",
        "max_syntax_work": "requires independent classification/parse/finalize/audit schedule replay",
        "max_resolution_work": "requires independent R-04 preflight/sort/query/diagnostic schedule replay",
    }
    fields: list[dict[str, object]] = []
    resolution_not_derived = {
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
    for tag, name in enumerate(FIELD_NAMES, start=1):
        if name in TRACE_FIELDS:
            value = _trace(trace_reasons[name])
        elif not admitted and name in resolution_not_derived:
            value = _not_derived("FN-8 rejection forbids the resolution counting subpass")
        else:
            value = _exact(exact[name])
        fields.append({"name": name, "tag": tag, **value})

    derived = {
        "canonical_path_depth": diagnostic_depth if diagnostic else 0,
        "classified_tokens": tokens,
        "diagnostic_issue_elements": issue_elements,
        "gaps": tokens,
        "mixed_elements": mixed,
        "private_derivation_elements": private_elements,
        "source_extents": sources,
        "terminals": tokens,
    }
    if admitted:
        derived.update(
            ancestry_steps=len(context.scopes) - 1,
            coverage_records=exact["max_coverage_records"],
            declarations=exact["max_declarations"],
            diagnostic_origins=origins,
            diagnostic_path_components=path_components,
            diagnostic_paths=paths,
            lookup_entries=exact["max_lookup_entries"],
        )
    agreement_derived = None
    if admitted:
        agreement_derived = {
            "terminals": tokens,
            "private_derivation_elements": private_elements,
            "gaps": tokens,
            "source_extents": sources,
            "source_role_occurrences": len(roles),
            "prelude_declarations": prelude_declarations,
            "prelude_lookup_entries": prelude_lookup_entries,
            "operation_lookup_entries": operation_lookup_entries,
            "source_lookup_entries": source_lookup_entries,
            "ordering_scratch_elements": exact["max_lookup_entries"],
            "source_spelling_bytes": spelling_components["source_roles"],
            "prelude_spelling_bytes": spelling_components["prelude"],
            "operation_spelling_bytes": spelling_components["operation_families"],
            "dotless_reservation_spelling_bytes": spelling_components[
                "dotless_reservations"
            ],
            "mode_word_spelling_bytes": spelling_components["mode_words"],
            "derivation_tree_bytes": private_elements * 64,
            "node_tree_bytes": nodes * 128,
            "mixed_tree_bytes": mixed * 16,
            "terminal_tree_bytes": tokens * 32,
            "source_extent_tree_bytes": sources * 24,
            "source_diagnostic_origins": source_origins,
            "prelude_diagnostic_origins": origins - source_origins,
            "diagnostic_issue_elements": issue_elements,
        }
        if tuple(agreement_derived) != AGREEMENT_DERIVED_NAMES:
            raise RuntimeError("agreement-derived vocabulary is not closed")
    return CountProjection(
        tuple(fields), derived, agreement_derived, spelling_components
    )
