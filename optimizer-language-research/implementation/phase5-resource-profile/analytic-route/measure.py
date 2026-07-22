"""Checked fold from private analytic relations to ResourceProfile v1 actuals."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from manifest import WorkloadManifest
from relation import (
    MODE_WORDS,
    OPERATION_SPELLINGS,
    PRELUDE_SPELLINGS,
    RESERVATION_SPELLINGS,
    PrivateRelation,
    build_relation,
)
from selection import select_diagnostic


U64_MAX = (1 << 64) - 1
FIELD_NAMES = (
    "max_sources", "max_logical_path_bytes", "max_source_bytes",
    "max_total_source_bytes", "max_binding_bytes", "max_token_bytes",
    "max_tokens", "max_lexemes", "max_lexical_scan_work",
    "max_classified_tokens", "max_production_nodes", "max_mixed_elements",
    "max_tree_depth", "max_parser_stack_entries", "max_list_members",
    "max_expected_terminals", "max_syntax_work", "max_tree_bytes",
    "max_declarations", "max_scopes", "max_scope_depth",
    "max_declaration_events", "max_lexical_uses", "max_deferred_uses",
    "max_spelling_bytes", "max_lookup_entries", "max_ancestry_steps",
    "max_node_path_depth", "max_diagnostic_origins", "max_diagnostic_paths",
    "max_diagnostic_path_components", "max_coverage_records",
    "max_resolution_work",
)
DERIVED_NAMES = (
    "terminals", "private_derivation_elements", "gaps", "source_extents",
    "source_role_occurrences", "prelude_declarations", "prelude_lookup_entries",
    "operation_lookup_entries", "source_lookup_entries",
    "ordering_scratch_elements", "source_spelling_bytes",
    "prelude_spelling_bytes", "operation_spelling_bytes",
    "dotless_reservation_spelling_bytes", "mode_word_spelling_bytes",
    "derivation_tree_bytes", "node_tree_bytes", "mixed_tree_bytes",
    "terminal_tree_bytes", "source_extent_tree_bytes",
    "source_diagnostic_origins", "prelude_diagnostic_origins",
    "diagnostic_issue_elements",
)


TRACE_GAP_FIELDS = (
    "max_lexical_scan_work",
    "max_parser_stack_entries",
    "max_list_members",
    "max_expected_terminals",
    "max_syntax_work",
    "max_resolution_work",
)


class MeasurementError(ValueError):
    """A private relation or checked derived quantity is impossible."""


@dataclass(frozen=True)
class Measurement:
    actuals: tuple[Optional[int], ...]
    derived: tuple[tuple[str, int], ...]
    trace_gaps: tuple[tuple[str, str], ...]
    expected_diagnostic: tuple[str, ...]

    def by_name(self) -> dict[str, Optional[int]]:
        return dict(zip(FIELD_NAMES, self.actuals))


def validate_shape(measurement: Measurement) -> None:
    if len(measurement.actuals) != len(FIELD_NAMES):
        raise MeasurementError("measurement does not have exactly 33 actuals")
    if tuple(name for name, _ in measurement.derived) != DERIVED_NAMES:
        raise MeasurementError("derived-count names or order are not closed")
    if tuple(name for name, _ in measurement.trace_gaps) != TRACE_GAP_FIELDS:
        raise MeasurementError("trace-gap fields or order are not closed")
    gaps = set(TRACE_GAP_FIELDS)
    for name, value in zip(FIELD_NAMES, measurement.actuals):
        if (value is None) != (name in gaps):
            raise MeasurementError("availability bits disagree with the trace-gap ledger")
        if value is not None:
            _u64(value, name)
    if not measurement.expected_diagnostic or any(
        not component or not component.isascii()
        for component in measurement.expected_diagnostic
    ):
        raise MeasurementError("expected diagnostic is not closed ASCII data")


def _u64(value: int, family: str) -> int:
    if type(value) is not int or value < 0 or value > U64_MAX:
        raise MeasurementError(f"{family} is not representable as u64")
    return value


def _add(family: str, *values: int) -> int:
    total = 0
    for value in values:
        _u64(value, family)
        total += value
        _u64(total, family)
    return total


def _multiply(family: str, left: int, right: int) -> int:
    _u64(left, family)
    _u64(right, family)
    return _u64(left * right, family)


def _maximum_depth(parents: tuple[int | None, ...], family: str) -> int:
    depths: list[int] = []
    for index, parent in enumerate(parents):
        if index == 0:
            if parent is not None:
                raise MeasurementError(f"{family} root has a parent")
            depths.append(0)
            continue
        if type(parent) is not int or parent < 0 or parent >= index:
            raise MeasurementError(f"{family} parent relation is not canonical preorder")
        depths.append(_add(family, depths[parent], 1))
    if not depths:
        raise MeasurementError(f"{family} relation is empty")
    return max(depths)


def _fixed_spelling_components() -> tuple[int, int, int, int]:
    if len(PRELUDE_SPELLINGS) != 24:
        raise MeasurementError("private PRE-1 spelling relation is not closed")
    if len(OPERATION_SPELLINGS) != 83 or len(set(OPERATION_SPELLINGS)) != 83:
        raise MeasurementError("private operation spelling relation is not closed")
    if len(RESERVATION_SPELLINGS) != 56 or len(set(RESERVATION_SPELLINGS)) != 56:
        raise MeasurementError("private reservation spelling relation is not closed")
    if tuple(RESERVATION_SPELLINGS[-5:]) != MODE_WORDS:
        raise MeasurementError("private mode-word relation is not closed")
    return tuple(
        sum(len(spelling.encode("utf-8")) for spelling in spellings)
        for spellings in (
            PRELUDE_SPELLINGS,
            OPERATION_SPELLINGS,
            RESERVATION_SPELLINGS[:-5],
            MODE_WORDS,
        )
    )


def measure(manifest: WorkloadManifest) -> Measurement:
    relation: PrivateRelation = build_relation(manifest)
    sources = len(manifest.sources)
    path_lengths = tuple(len(source.logical_path.encode("ascii")) for source in manifest.sources)
    source_lengths = tuple(source.byte_length for source in manifest.sources)
    total_source_bytes = _add("total source bytes", *source_lengths)
    binding_bytes = 50
    for path, source in zip(path_lengths, source_lengths):
        binding_bytes = _add("binding bytes", binding_bytes, 16, path, source)

    tokens = len(relation.aggregate_token_length_witness)
    lexemes = _add("lexemes", tokens, relation.trivia_runs)
    nodes = len(relation.aggregate_production_parent_witness)
    terminals = tokens
    private_elements = _add("private derivation elements", nodes, terminals)
    mixed = _add("mixed elements", nodes - 1, terminals)
    tree_depth = _maximum_depth(
        relation.aggregate_production_parent_witness, "production depth"
    )
    scope_depth = _maximum_depth(relation.scopes, "scope depth")

    event_count = sum(
        role.category in ("declaration", "dependent") for role in relation.roles
    )
    lexical_uses = sum(role.category == "lexical" for role in relation.roles)
    deferred_uses = sum(role.category == "deferred" for role in relation.roles)
    if event_count + lexical_uses + deferred_uses != len(relation.roles):
        raise MeasurementError("private role categories do not form a partition")
    declarations = _add("declarations", event_count, 24)
    source_spelling = _add(
        "source spelling bytes",
        *(len(role.spelling.encode("utf-8")) for role in relation.roles),
    )
    prelude_spelling, operation_spelling, dotless_spelling, mode_spelling = (
        _fixed_spelling_components()
    )
    spelling_bytes = _add(
        "spelling bytes",
        source_spelling,
        prelude_spelling,
        operation_spelling,
        dotless_spelling,
        mode_spelling,
    )
    source_lookup = sum(role.lookup_multiplicity for role in relation.roles)
    if len(relation.lookup_entries) != _add("lookup entries", 18, 83, source_lookup):
        raise MeasurementError("private lookup relation does not match its insertions")
    ancestry_steps = len(relation.scopes) - 1
    coverage = _add("coverage records", nodes, len(relation.roles))

    derivation_bytes = _multiply("DerivationElement bytes", private_elements, 64)
    node_bytes = _multiply("NodeRecord bytes", nodes, 128)
    mixed_bytes = _multiply("MixedElement bytes", mixed, 16)
    terminal_bytes = _multiply("TerminalRecord bytes", terminals, 32)
    extent_bytes = _multiply("BundleSourceExtent bytes", sources, 24)
    tree_bytes = _add(
        "tree bytes", derivation_bytes, node_bytes, mixed_bytes, terminal_bytes, extent_bytes
    )

    actuals = (
        sources,
        max(path_lengths),
        max(source_lengths),
        total_source_bytes,
        binding_bytes,
        max(relation.aggregate_token_length_witness),
        tokens,
        lexemes,
        None,
        terminals,
        nodes,
        mixed,
        tree_depth,
        None,
        None,
        None,
        None,
        tree_bytes,
        declarations,
        len(relation.scopes),
        scope_depth,
        event_count,
        lexical_uses,
        deferred_uses,
        spelling_bytes,
        len(relation.lookup_entries),
        ancestry_steps,
        tree_depth,
        0,
        0,
        0,
        coverage,
        None,
    )
    if len(actuals) != len(FIELD_NAMES):
        raise MeasurementError("profile field relation is not complete")
    for name, value in zip(FIELD_NAMES, actuals):
        if value is not None:
            _u64(value, name)

    derived = (
        ("terminals", terminals),
        ("private_derivation_elements", private_elements),
        ("gaps", terminals),
        ("source_extents", sources),
        ("source_role_occurrences", len(relation.roles)),
        ("prelude_declarations", 24),
        ("prelude_lookup_entries", 18),
        ("operation_lookup_entries", 83),
        ("source_lookup_entries", source_lookup),
        ("ordering_scratch_elements", len(relation.lookup_entries)),
        ("source_spelling_bytes", source_spelling),
        ("prelude_spelling_bytes", prelude_spelling),
        ("operation_spelling_bytes", operation_spelling),
        ("dotless_reservation_spelling_bytes", dotless_spelling),
        ("mode_word_spelling_bytes", mode_spelling),
        ("derivation_tree_bytes", derivation_bytes),
        ("node_tree_bytes", node_bytes),
        ("mixed_tree_bytes", mixed_bytes),
        ("terminal_tree_bytes", terminal_bytes),
        ("source_extent_tree_bytes", extent_bytes),
        ("source_diagnostic_origins", 0),
        ("prelude_diagnostic_origins", 0),
        ("diagnostic_issue_elements", 0),
    )
    for name, value in derived:
        _u64(value, name)
    trace_gaps = (
        ("max_lexical_scan_work", "exact scanner probe/comparison/UTF-8 action trace is not independently replayed"),
        ("max_parser_stack_entries", "exact parser task/frame push-pop trace is not independently replayed"),
        ("max_list_members", "exact RepeatZero/RepeatOne arm-selection trace is not independently replayed"),
        ("max_expected_terminals", "exact diagnostic expected-set construction trace is not independently replayed"),
        ("max_syntax_work", "exact classifier/parser/finalizer/audit action trace is not independently replayed"),
        ("max_resolution_work", "exact four-sort/query/origin/diagnostic action trace is not independently replayed"),
    )
    result = Measurement(actuals, derived, trace_gaps, select_diagnostic(relation))
    validate_shape(result)
    return result


def require_complete(measurement: Measurement) -> tuple[int, ...]:
    """Fail closed before a partial analytic receipt can size a profile."""
    validate_shape(measurement)
    if measurement.trace_gaps:
        names = ", ".join(name for name, _ in measurement.trace_gaps)
        raise MeasurementError(f"exact algorithm trace is unavailable for: {names}")
    return tuple(value for value in measurement.actuals if value is not None)
