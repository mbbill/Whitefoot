"""Deterministic common ledger and Oracle-specific derivation evidence."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import struct

from core import Inputs, Limits, ascii_hex, fail
from ebnf import walk_nodes
from extract import GrammarDocument
from generalized import (
    OracleMetrics,
    ParseResult,
    compile_grammar,
    parse_source,
)


@dataclass(frozen=True)
class CaseObservation:
    document: str
    identifier: str
    start: str
    source: bytes
    result: ParseResult


@dataclass(frozen=True)
class StreamObservation:
    ordinal: int
    source: bytes
    result: ParseResult


@dataclass(frozen=True)
class DomainObservation:
    document: str
    identifier: str
    start: str
    argument: bytes
    digest: str
    streams: tuple[StreamObservation, ...]


@dataclass(frozen=True)
class CaseDelta:
    identifier: str
    status: str
    trace: bytes


@dataclass(frozen=True)
class Evidence:
    cases: tuple[CaseObservation, ...]
    domains: tuple[DomainObservation, ...]
    deltas: tuple[CaseDelta, ...]
    metrics: dict[str, OracleMetrics]


class _Lines:
    def __init__(self, limits: Limits) -> None:
        self.limit = limits.get("max_engine_output_bytes")
        self.lines: list[bytes] = []
        self.length = 0

    def add(self, *fields: object) -> None:
        encoded: list[bytes] = []
        for field in fields:
            if isinstance(field, bytes):
                value = field
            elif isinstance(field, (str, int)):
                value = str(field).encode("ascii")
            else:
                fail("internal", "report_field_type")
            if b"\t" in value or b"\n" in value or b"\r" in value:
                fail("internal", "report_field_byte")
            encoded.append(value)
        line = b"\t".join(encoded) + b"\n"
        if self.length + len(line) > self.limit:
            fail("resource", "limit_max_engine_output_bytes")
        self.lines.append(line)
        self.length += len(line)

    def finish(self) -> bytes:
        return b"".join(self.lines)


def analyze(
    inputs: Inputs,
    current: GrammarDocument,
    proposal: GrammarDocument,
) -> Evidence:
    documents = ((current, compile_grammar(current)), (proposal, compile_grammar(proposal)))
    cases: list[CaseObservation] = []
    domains: list[DomainObservation] = []
    metrics = {"current": OracleMetrics(), "proposal": OracleMetrics()}
    for grammar, compiled in documents:
        document_metrics = metrics[grammar.name]
        for item in inputs.cases:
            result = parse_source(
                grammar,
                compiled,
                item.start,
                item.source,
                inputs.limits,
            )
            document_metrics.include(result)
            cases.append(
                CaseObservation(
                    grammar.name,
                    item.identifier,
                    item.start,
                    item.source,
                    result,
                )
            )
        generated_count = 0
        for domain in inputs.domains:
            if domain.kind != "fixed-lowerword-call":
                fail("internal", "domain_kind")
            if domain.start not in compiled.starts:
                fail("extraction", "unknown_start_nonterminal")
            generated_count += len(grammar.expanded_lowerwords)
            inputs.limits.require("max_generated_streams", generated_count)
            digest = hashlib.sha256()
            streams: list[StreamObservation] = []
            for ordinal, word in enumerate(grammar.expanded_lowerwords):
                source = word + b"(" + domain.argument + b")"
                digest.update(struct.pack(">Q", len(source)))
                digest.update(source)
                result = parse_source(
                    grammar,
                    compiled,
                    domain.start,
                    source,
                    inputs.limits,
                )
                document_metrics.include(result)
                streams.append(StreamObservation(ordinal, source, result))
            domains.append(
                DomainObservation(
                    grammar.name,
                    domain.identifier,
                    domain.start,
                    domain.argument,
                    digest.hexdigest(),
                    tuple(streams),
                )
            )
    deltas: list[CaseDelta] = []
    for case in inputs.cases:
        current_case = next(
            item for item in cases if item.document == "current" and item.identifier == case.identifier
        )
        proposal_case = next(
            item for item in cases if item.document == "proposal" and item.identifier == case.identifier
        )
        current_traces = set(current_case.result.traces)
        proposal_traces = set(proposal_case.result.traces)
        for status, traces in (
            ("retained", current_traces & proposal_traces),
            ("removed", current_traces - proposal_traces),
            ("introduced", proposal_traces - current_traces),
        ):
            for trace in sorted(traces):
                deltas.append(CaseDelta(case.identifier, status, trace))
    return Evidence(tuple(cases), tuple(domains), tuple(deltas), metrics)


def _common_records(grammar: GrammarDocument) -> list[tuple[int, int, bytes]]:
    rank = {
        "RULE": 1,
        "SURFACE": 2,
        "PROD": 3,
        "NODE": 4,
        "LEX": 5,
        "FIXED": 6,
        "REF": 7,
    }
    records: list[tuple[int, int, bytes]] = []

    def add(start: int, tag: str, fields: tuple[object, ...]) -> None:
        encoded = [tag.encode("ascii")]
        for field in fields:
            encoded.append(str(field).encode("ascii"))
        line = b"\t".join(encoded)
        records.append((start, rank[tag], line))

    for rule in grammar.rules:
        add(rule.start, "RULE", (grammar.name, ascii_hex(rule.owner), rule.start, rule.end))
    for surface in grammar.surfaces:
        add(
            surface.start,
            "SURFACE",
            (
                grammar.name,
                surface.kind,
                surface.start,
                surface.end,
                ascii_hex(surface.owner),
            ),
        )
    for production in grammar.productions:
        add(
            production.definition_start,
            "PROD",
            (
                grammar.name,
                ascii_hex(production.owner),
                ascii_hex(production.lhs),
                production.definition_start,
                production.definition_end,
                production.rhs_start,
                production.rhs_end,
            ),
        )
        for path, node in walk_nodes(production.root):
            value = "-" if node.value is None else ascii_hex(node.value)
            add(
                node.start,
                "NODE",
                (
                    grammar.name,
                    ascii_hex(production.lhs),
                    path,
                    node.kind,
                    node.start,
                    node.end,
                    value,
                ),
            )
    for lexical in grammar.lexical:
        add(
            lexical.start,
            "LEX",
            (
                grammar.name,
                ascii_hex(lexical.owner),
                ascii_hex(lexical.name),
                lexical.kind,
                lexical.start,
                lexical.end,
                lexical.predicate.hex(),
            ),
        )
    for fixed in grammar.fixed:
        add(
            fixed.start,
            "FIXED",
            (
                grammar.name,
                ascii_hex(fixed.lhs),
                fixed.path,
                fixed.start,
                fixed.end,
                fixed.spelling.hex(),
                fixed.descriptor.hex(),
            ),
        )
    for reference in grammar.references:
        add(
            reference.start,
            "REF",
            (
                grammar.name,
                ascii_hex(reference.lhs),
                reference.path,
                reference.start,
                reference.end,
                ascii_hex(reference.name),
            ),
        )
    records.sort(key=lambda item: (item[0], item[1], item[2]))
    return records


def render_success(
    inputs: Inputs,
    current: GrammarDocument,
    proposal: GrammarDocument,
    evidence: Evidence,
) -> bytes:
    lines = _Lines(inputs.limits)
    lines.add("WFGRREPORT1")
    lines.add("ENGINE", "oracle")
    lines.add("COMMON-BEGIN")
    for section in inputs.bound_sections():
        lines.add("BIND", section.name, len(section.data), section.digest)
    for grammar in (current, proposal):
        for _start, _rank, record in _common_records(grammar):
            fields = record.split(b"\t")
            lines.add(*fields)
        coverage = grammar.coverage
        lines.add(
            "COVERAGE",
            grammar.name,
            coverage.assignments,
            coverage.fences,
            coverage.inline,
            coverage.lexical_cues,
            coverage.unclassified,
        )
    lines.add("COMMON-END")
    lines.add("ORACLE-BEGIN")
    for case_input in inputs.cases:
        for document in ("current", "proposal"):
            case = next(
                item
                for item in evidence.cases
                if item.document == document and item.identifier == case_input.identifier
            )
            lines.add(
                "CASE",
                document,
                ascii_hex(case.identifier),
                ascii_hex(case.start),
                case.source.hex(),
                case.result.classification,
                len(case.result.traces),
            )
            for ordinal, trace in enumerate(case.result.traces):
                lines.add("CASE-TRACE", document, ascii_hex(case.identifier), ordinal, trace.hex())
    delta_rank = {"removed": 0, "retained": 1, "introduced": 2}
    for delta in sorted(
        evidence.deltas,
        key=lambda item: (item.identifier, delta_rank[item.status], item.trace),
    ):
        lines.add("CASE-DELTA", ascii_hex(delta.identifier), delta.status, delta.trace.hex())
    for domain_input in inputs.domains:
        for document in ("current", "proposal"):
            domain = next(
                item
                for item in evidence.domains
                if item.document == document and item.identifier == domain_input.identifier
            )
            lines.add(
                "DOMAIN",
                document,
                ascii_hex(domain.identifier),
                ascii_hex(domain.start),
                domain.argument.hex(),
                len(domain.streams),
                domain.digest,
            )
            for stream in domain.streams:
                lines.add(
                    "STREAM",
                    document,
                    ascii_hex(domain.identifier),
                    stream.ordinal,
                    stream.source.hex(),
                    stream.result.classification,
                    len(stream.result.traces),
                )
                for trace_ordinal, trace in enumerate(stream.result.traces):
                    lines.add(
                        "STREAM-TRACE",
                        document,
                        ascii_hex(domain.identifier),
                        stream.ordinal,
                        trace_ordinal,
                        trace.hex(),
                    )
    for document in ("current", "proposal"):
        metric = evidence.metrics[document]
        lines.add(
            "METRIC",
            document,
            metric.parsed_streams,
            metric.source_tokens,
            metric.chart_items,
            metric.packed_edges,
            metric.proof_nodes,
        )
    lines.add("ORACLE-END")
    lines.add("END")
    return lines.finish()


def render_failure(family: str, code: str) -> bytes:
    return (
        b"WFGRREPORT1\n"
        b"ENGINE\toracle\n"
        + b"FAIL\t"
        + family.encode("ascii")
        + b"\t"
        + code.encode("ascii")
        + b"\nEND\n"
    )
