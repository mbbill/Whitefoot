"""Closed binary framing and text manifests for the grammar Oracle."""

from __future__ import annotations

import re
import struct

from core import BoundInput, Case, CURRENT_SHA256, Domain, Inputs, Limits, fail, sha256


MAGIC = b"WFGRAMV1"
HEADER_BYTES = 48
OUTER_LIMITS_BYTES = 8_192
OUTER_DOCUMENT_BYTES = 1_048_576
OUTER_CASE_DOMAIN_BYTES = 262_144
OUTER_FRAME_BYTES = (
    HEADER_BYTES
    + OUTER_LIMITS_BYTES
    + 2 * OUTER_DOCUMENT_BYTES
    + 2 * OUTER_CASE_DOMAIN_BYTES
)

_LIMIT_CAPS = {
    "cpu_timeout_seconds": 60,
    "max_case_bytes": 131_072,
    "max_cases": 1_024,
    "max_definitions": 1_024,
    "max_document_bytes": 524_288,
    "max_domain_bytes": 131_072,
    "max_domains": 64,
    "max_ebnf_depth": 128,
    "max_engine_output_bytes": 8_388_608,
    "max_final_report_bytes": 16_777_216,
    "max_generated_streams": 100_000,
    "max_grammar_nodes": 65_536,
    "max_lexical_definitions": 128,
    "max_line_bytes": 16_384,
    "max_lines": 4_096,
    "max_rules": 1_024,
    "max_symbol_bytes": 256,
    "max_terminal_occurrences": 8_192,
    "oracle_max_chart_items": 1_000_000,
    "oracle_max_packed_edges": 1_000_000,
    "oracle_max_proof_nodes": 1_000_000,
    "oracle_max_source_tokens": 256,
    "static_max_lookahead_words": 262_144,
    "static_max_product_states": 1_000_000,
    "static_max_work": 10_000_000,
    "wall_timeout_seconds": 60,
}
_LIMIT_LINE = re.compile(rb"([a-z][a-z0-9_]*)=([1-9][0-9]*)")
_ID = re.compile(rb"[a-z][a-z0-9-]*")
_NONTERMINAL = re.compile(rb"[a-z][a-z0-9_]*")
_LOWER_HEX = re.compile(rb"(?:[0-9a-f]{2})+")


def read_frame(stream: object) -> Inputs:
    """Read one bounded frame and parse all non-semantic input manifests."""

    raw = stream.read(OUTER_FRAME_BYTES + 1)
    if not isinstance(raw, bytes):
        fail("internal", "stdin_not_bytes")
    if len(raw) > OUTER_FRAME_BYTES:
        fail("input", "frame_outer_limit")
    if len(raw) < HEADER_BYTES or raw[:8] != MAGIC:
        fail("input", "frame_header")
    lengths = struct.unpack(">QQQQQ", raw[8:HEADER_BYTES])
    outer_caps = (
        OUTER_LIMITS_BYTES,
        OUTER_DOCUMENT_BYTES,
        OUTER_DOCUMENT_BYTES,
        OUTER_CASE_DOMAIN_BYTES,
        OUTER_CASE_DOMAIN_BYTES,
    )
    if any(length > cap for length, cap in zip(lengths, outer_caps)):
        fail("input", "section_outer_limit")
    total = HEADER_BYTES + sum(lengths)
    if total != len(raw):
        fail("input", "frame_length")
    sections: list[bytes] = []
    cursor = HEADER_BYTES
    for length in lengths:
        end = cursor + length
        sections.append(raw[cursor:end])
        cursor = end
    limits_raw, current, proposal, cases_raw, domains_raw = sections
    limits = parse_limits(limits_raw)
    limits.require("max_document_bytes", len(current))
    limits.require("max_document_bytes", len(proposal))
    limits.require("max_case_bytes", len(cases_raw))
    limits.require("max_domain_bytes", len(domains_raw))
    if sha256(current) != CURRENT_SHA256:
        fail("input", "current_document_hash")
    cases = parse_cases(cases_raw, limits)
    domains = parse_domains(domains_raw, limits)
    return Inputs(
        BoundInput("limits", limits_raw),
        BoundInput("current", current),
        BoundInput("proposal", proposal),
        BoundInput("cases", cases_raw),
        BoundInput("domains", domains_raw),
        limits,
        cases,
        domains,
    )


def parse_limits(raw: bytes) -> Limits:
    if not raw.endswith(b"\n") or b"\r" in raw:
        fail("input", "limits_layout")
    lines = raw[:-1].split(b"\n")
    if len(lines) != len(_LIMIT_CAPS):
        fail("input", "limits_fields")
    observed_names: list[str] = []
    values: dict[str, int] = {}
    for line in lines:
        match = _LIMIT_LINE.fullmatch(line)
        if match is None:
            fail("input", "limits_line")
        name = match.group(1).decode("ascii")
        if name in values:
            fail("input", "limits_duplicate")
        maximum = _LIMIT_CAPS.get(name)
        spelling = match.group(2)
        if maximum is None:
            fail("input", "limits_value")
        maximum_spelling = str(maximum).encode("ascii")
        if len(spelling) > len(maximum_spelling) or (
            len(spelling) == len(maximum_spelling) and spelling > maximum_spelling
        ):
            fail("input", "limits_value")
        value = int(spelling)
        observed_names.append(name)
        values[name] = value
    if observed_names != sorted(_LIMIT_CAPS) or set(values) != set(_LIMIT_CAPS):
        fail("input", "limits_order")
    return Limits(values)


def _manifest_lines(
    raw: bytes,
    header: bytes,
    byte_limit: int,
    record_limit: str,
    limits: Limits,
    code: str,
) -> tuple[bytes, ...]:
    if len(raw) > byte_limit or not raw.endswith(b"\n") or b"\r" in raw:
        fail("input", f"{code}_layout")
    if not raw.isascii():
        fail("input", f"{code}_ascii")
    records: list[bytes] = []
    start = 0
    line_index = 0
    while start < len(raw):
        newline = raw.find(b"\n", start)
        if newline < 0:
            fail("internal", "manifest_line_coverage")
        if line_index == 0:
            if raw[start:newline] != header:
                fail("input", f"{code}_header")
        else:
            limits.require(record_limit, len(records) + 1)
            records.append(raw[start:newline])
        start = newline + 1
        line_index += 1
    if line_index == 0:
        fail("input", f"{code}_header")
    return tuple(records)


def _text_field(raw: bytes, pattern: re.Pattern[bytes], code: str, limit: int) -> str:
    if len(raw) > limit or pattern.fullmatch(raw) is None:
        fail("input", code)
    return raw.decode("ascii")


def _hex_field(raw: bytes, code: str, limit: int) -> bytes:
    if _LOWER_HEX.fullmatch(raw) is None:
        fail("input", code)
    value = bytes.fromhex(raw.decode("ascii"))
    if not value or len(value) > limit:
        fail("resource", f"limit_{code}_bytes")
    return value


def parse_cases(raw: bytes, limits: Limits) -> tuple[Case, ...]:
    lines = _manifest_lines(
        raw,
        b"whitefoot.grammar-cases.v1",
        limits.get("max_case_bytes"),
        "max_cases",
        limits,
        "cases",
    )
    result: list[Case] = []
    for line in lines:
        fields = line.split(b"\t")
        if len(fields) != 4 or fields[0] != b"case":
            fail("input", "case_record")
        identifier = _text_field(
            fields[1], _ID, "case_identifier", limits.get("max_symbol_bytes")
        )
        start = _text_field(
            fields[2], _NONTERMINAL, "case_start", limits.get("max_symbol_bytes")
        )
        source = _hex_field(fields[3], "case", limits.get("max_case_bytes"))
        result.append(Case(identifier, start, source))
    identifiers = [item.identifier for item in result]
    if identifiers != sorted(identifiers) or len(identifiers) != len(set(identifiers)):
        fail("input", "case_order")
    return tuple(result)


def parse_domains(raw: bytes, limits: Limits) -> tuple[Domain, ...]:
    lines = _manifest_lines(
        raw,
        b"whitefoot.grammar-domains.v1",
        limits.get("max_domain_bytes"),
        "max_domains",
        limits,
        "domains",
    )
    result: list[Domain] = []
    for line in lines:
        fields = line.split(b"\t")
        if len(fields) != 5 or fields[0] != b"domain":
            fail("input", "domain_record")
        identifier = _text_field(
            fields[1], _ID, "domain_identifier", limits.get("max_symbol_bytes")
        )
        if fields[2] != b"fixed-lowerword-call":
            fail("input", "domain_kind")
        start = _text_field(
            fields[3], _NONTERMINAL, "domain_start", limits.get("max_symbol_bytes")
        )
        argument = _hex_field(fields[4], "domain", limits.get("max_domain_bytes"))
        result.append(Domain(identifier, "fixed-lowerword-call", start, argument))
    identifiers = [item.identifier for item in result]
    if identifiers != sorted(identifiers) or len(identifiers) != len(set(identifiers)):
        fail("input", "domain_order")
    return tuple(result)
