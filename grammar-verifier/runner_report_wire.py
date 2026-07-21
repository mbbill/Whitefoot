"""Canonical field decoding shared by engine report validators."""

from __future__ import annotations

from typing import Optional

from runner_common_wire import canonical_hex
from runner_inputs import Inputs, fail


def decoded_hex(value: bytes, optional: bool = False) -> Optional[bytes]:
    canonical_hex(value, optional)
    return None if value == b"-" else bytes.fromhex(value.decode("ascii"))


def predicate_hex(value: bytes) -> bytes:
    decoded = decoded_hex(value)
    assert decoded is not None
    if decoded in (b"pattern:digits", b"end"):
        return decoded
    prefix, separator, payload = decoded.partition(b":")
    if separator != b":" or prefix not in (b"fixed", b"lex"):
        fail("static_predicate", "static emitted an unknown terminal predicate descriptor")
    canonical_hex(payload)
    return decoded


def predicate_order(descriptor: bytes) -> tuple[int, bytes]:
    rank = (
        0
        if descriptor.startswith(b"fixed:")
        else 1
        if descriptor == b"pattern:digits"
        else 2
        if descriptor.startswith(b"lex:")
        else 3
    )
    return rank, descriptor


def word_hex(value: bytes, allowed: frozenset[bytes] | None = None) -> tuple[bytes, ...]:
    decoded = decoded_hex(value)
    assert decoded is not None
    if decoded == b"empty":
        return ()
    predicates = decoded.split(b",")
    if len(predicates) not in (1, 2):
        fail("static_word", "static emitted a lookahead word outside the two-token bound")
    seen_end = False
    for predicate in predicates:
        descriptor = predicate_hex(predicate.hex().encode("ascii"))
        if allowed is not None and descriptor not in allowed:
            fail("static_word", "static emitted a predicate absent from the common terminal ledger")
        if seen_end and descriptor != b"end":
            fail("static_word", "static emitted a predicate after end of input")
        seen_end |= descriptor == b"end"
    return tuple(predicates)


def witness_stream_hex(value: bytes) -> tuple[bytes, ...]:
    decoded = decoded_hex(value)
    assert decoded is not None
    tokens = decoded.split(b",")
    if len(tokens) not in (1, 2) or any(not token for token in tokens):
        fail("static_witness", "static emitted a witness outside the two-token lookahead bound")
    seen_end = False
    for token in tokens:
        canonical_hex(token, optional=True)
        if seen_end and token != b"-":
            fail("static_witness", "static emitted a token after end of input")
        seen_end |= token == b"-"
    return tuple(tokens)


def expectations(
    inputs: Inputs,
) -> tuple[dict[tuple[bytes, bytes], bytes], dict[bytes, bytes], dict[bytes, bytes]]:
    cases: dict[tuple[bytes, bytes], bytes] = {}
    transitions: dict[bytes, bytes] = {}
    case_deltas: dict[bytes, bytes] = {}
    for line in inputs.expectations.data.splitlines()[1:]:
        fields = line.split(b"\t")
        if fields[0] == b"case":
            cases[(fields[1], fields[2])] = fields[3]
        elif fields[0] == b"case-delta":
            case_deltas[fields[1]] = fields[2]
        else:
            transitions[fields[1]] = fields[2]
    return cases, transitions, case_deltas


def case_inputs(inputs: Inputs) -> dict[bytes, tuple[bytes, bytes]]:
    return {
        fields[1]: (fields[2].hex().encode("ascii"), fields[3])
        for fields in (line.split(b"\t") for line in inputs.section("cases").data.splitlines()[1:])
    }


def domain_inputs(inputs: Inputs) -> dict[bytes, tuple[bytes, bytes]]:
    return {
        fields[1]: (fields[3].hex().encode("ascii"), fields[4])
        for fields in (line.split(b"\t") for line in inputs.section("domains").data.splitlines()[1:])
    }


def record_fields(line: bytes, fields_by_tag: dict[str, int], engine: str) -> tuple[str, list[bytes]]:
    fields = line.split(b"\t")
    try:
        tag = fields[0].decode("ascii")
    except (IndexError, UnicodeDecodeError):
        fail("report_specific", f"{engine} emitted an invalid record tag")
    if tag not in fields_by_tag or len(fields) != fields_by_tag[tag] or any(not field for field in fields):
        fail("report_specific", f"{engine} emitted an unknown or malformed record")
    if any(byte < 0x20 or byte > 0x7E for field in fields for byte in field):
        fail("report_specific", f"{engine} emitted a noncanonical record byte")
    return tag, fields
