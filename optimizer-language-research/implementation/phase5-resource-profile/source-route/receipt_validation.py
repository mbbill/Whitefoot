"""Cross-field semantic equations for strictly shaped route receipts."""

from __future__ import annotations

from model import RouteError
from receipt_structure import ReceiptFacts, validate_structure


def _exact(facts: ReceiptFacts, name: str) -> int:
    value = facts.field_values[name]
    if type(value) is not int:
        raise RouteError(f"receipt relation unexpectedly needs {name}")
    return value


def _check(relations: tuple[tuple[int, int, str], ...]) -> None:
    for observed, expected, label in relations:
        if observed != expected:
            raise RouteError(f"receipt {label} relation is inconsistent")


def _validate_base_relations(facts: ReceiptFacts) -> None:
    exact = lambda name: _exact(facts, name)
    nodes = exact("max_production_nodes")
    tokens = exact("max_tokens")
    mixed = exact("max_mixed_elements")
    sources = exact("max_sources")
    if nodes == 0:
        raise RouteError("receipt complete program has no production root")
    diagnostic_origins = facts.source_origins + facts.prelude_origins
    _check(
        (
            (sources, len(facts.source_lengths), "source count"),
            (
                exact("max_logical_path_bytes"),
                max(facts.path_lengths),
                "logical-path bytes",
            ),
            (
                exact("max_source_bytes"),
                max(facts.source_lengths),
                "source bytes",
            ),
            (
                exact("max_total_source_bytes"),
                sum(facts.source_lengths),
                "total source bytes",
            ),
            (
                exact("max_binding_bytes"),
                50
                + sum(
                    16 + path + source
                    for path, source in zip(
                        facts.path_lengths, facts.source_lengths
                    )
                ),
                "binding bytes",
            ),
            (exact("max_classified_tokens"), tokens, "classified tokens"),
            (mixed, nodes - 1 + tokens, "mixed elements"),
            (facts.derived["terminals"], tokens, "terminals"),
            (
                facts.derived["classified_tokens"],
                tokens,
                "derived classified tokens",
            ),
            (
                facts.derived["private_derivation_elements"],
                nodes + tokens,
                "derivation elements",
            ),
            (facts.derived["mixed_elements"], mixed, "derived mixed elements"),
            (facts.derived["gaps"], tokens, "gaps"),
            (facts.derived["source_extents"], sources, "source extents"),
            (
                facts.derived["canonical_path_depth"],
                facts.diagnostic_depth,
                "canonical path depth",
            ),
            (
                facts.derived["diagnostic_issue_elements"],
                facts.diagnostic_issue_elements,
                "issue elements",
            ),
            (facts.projection["terminals"], tokens, "projection terminals"),
            (
                facts.projection["production_nodes"],
                nodes,
                "projection nodes",
            ),
            (
                exact("max_diagnostic_origins"),
                diagnostic_origins,
                "diagnostic origins",
            ),
            (
                exact("max_diagnostic_paths"),
                facts.diagnostic_paths,
                "diagnostic paths",
            ),
            (
                exact("max_diagnostic_path_components"),
                facts.path_components,
                "diagnostic path components",
            ),
            (
                exact("max_tree_bytes"),
                (nodes + tokens) * 64
                + nodes * 128
                + mixed * 16
                + tokens * 32
                + sources * 24,
                "charged tree bytes",
            ),
        )
    )


def _validate_admitted_relations(facts: ReceiptFacts) -> None:
    exact = lambda name: _exact(facts, name)
    agreement = facts.agreement
    assert agreement is not None
    nodes = exact("max_production_nodes")
    tokens = exact("max_tokens")
    mixed = exact("max_mixed_elements")
    sources = exact("max_sources")
    events = exact("max_declaration_events")
    lexical = exact("max_lexical_uses")
    deferred = exact("max_deferred_uses")
    roles = facts.projection["role_occurrences"]
    _check(
        (
            (agreement["terminals"], tokens, "agreement terminals"),
            (
                agreement["private_derivation_elements"],
                nodes + tokens,
                "agreement derivation elements",
            ),
            (agreement["gaps"], tokens, "agreement gaps"),
            (agreement["source_extents"], sources, "agreement extents"),
            (agreement["source_role_occurrences"], roles, "agreement roles"),
            (roles, events + lexical + deferred, "role partition"),
            (
                exact("max_declarations"),
                events + agreement["prelude_declarations"],
                "declarations",
            ),
            (
                facts.derived["declarations"],
                exact("max_declarations"),
                "derived declarations",
            ),
            (
                exact("max_lookup_entries"),
                agreement["prelude_lookup_entries"]
                + agreement["operation_lookup_entries"]
                + agreement["source_lookup_entries"],
                "lookup entries",
            ),
            (
                facts.derived["lookup_entries"],
                exact("max_lookup_entries"),
                "derived lookup entries",
            ),
            (
                facts.projection["declaration_facts_including_prelude_lookup"],
                agreement["prelude_lookup_entries"]
                + agreement["source_lookup_entries"],
                "declaration facts",
            ),
            (exact("max_scopes"), facts.projection["scopes"], "scopes"),
            (
                exact("max_ancestry_steps"),
                exact("max_scopes") - 1,
                "ancestry steps",
            ),
            (
                facts.derived["ancestry_steps"],
                exact("max_ancestry_steps"),
                "derived ancestry",
            ),
            (
                exact("max_coverage_records"),
                nodes + roles,
                "coverage records",
            ),
            (
                facts.derived["coverage_records"],
                exact("max_coverage_records"),
                "derived coverage",
            ),
            (
                exact("max_spelling_bytes"),
                agreement["source_spelling_bytes"]
                + agreement["prelude_spelling_bytes"]
                + agreement["operation_spelling_bytes"]
                + agreement["dotless_reservation_spelling_bytes"]
                + agreement["mode_word_spelling_bytes"],
                "spelling bytes",
            ),
            (
                agreement["source_spelling_bytes"],
                facts.spelling["source_roles"],
                "source spelling",
            ),
            (
                agreement["prelude_spelling_bytes"],
                facts.spelling["prelude"],
                "prelude spelling",
            ),
            (
                agreement["operation_spelling_bytes"],
                facts.spelling["operation_families"],
                "operation spelling",
            ),
            (
                agreement["dotless_reservation_spelling_bytes"],
                facts.spelling["dotless_reservations"],
                "reservation spelling",
            ),
            (
                agreement["mode_word_spelling_bytes"],
                facts.spelling["mode_words"],
                "mode spelling",
            ),
            (
                agreement["ordering_scratch_elements"],
                exact("max_lookup_entries"),
                "ordering scratch",
            ),
            (
                agreement["derivation_tree_bytes"],
                (nodes + tokens) * 64,
                "derivation bytes",
            ),
            (agreement["node_tree_bytes"], nodes * 128, "node bytes"),
            (agreement["mixed_tree_bytes"], mixed * 16, "mixed bytes"),
            (agreement["terminal_tree_bytes"], tokens * 32, "terminal bytes"),
            (
                agreement["source_extent_tree_bytes"],
                sources * 24,
                "source extent bytes",
            ),
            (
                agreement["source_diagnostic_origins"],
                facts.source_origins,
                "source origins",
            ),
            (
                agreement["prelude_diagnostic_origins"],
                facts.prelude_origins,
                "PRE-1 origins",
            ),
            (
                agreement["diagnostic_issue_elements"],
                facts.diagnostic_issue_elements,
                "agreement issue elements",
            ),
        )
    )
    if exact("max_scope_depth") > exact("max_ancestry_steps"):
        raise RouteError("receipt scope depth exceeds its ancestry capacity")
    if exact("max_node_path_depth") != max(
        exact("max_tree_depth"), facts.diagnostic_depth
    ):
        raise RouteError("receipt node-path depth relation is inconsistent")


def validate_receipt(receipt: dict[str, object]) -> None:
    """Reject any malformed, identity-stale, or semantically inconsistent receipt."""

    facts = validate_structure(receipt)
    _validate_base_relations(facts)
    if facts.resolution_not_derived:
        if (
            facts.projection["role_occurrences"] != 0
            or facts.projection["declaration_facts_including_prelude_lookup"] != 0
        ):
            raise RouteError("receipt FN-8 rejection contains resolution products")
        if _exact(facts, "max_node_path_depth") != facts.diagnostic_depth:
            raise RouteError("receipt FN-8 diagnostic depth is inconsistent")
        return
    _validate_admitted_relations(facts)
