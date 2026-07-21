"""Canonical Oracle-report fixtures for runner-boundary tests."""

from __future__ import annotations

import hashlib

from runner_inputs import Inputs
from support_common import common_lines


def _tree(label: bytes, *children: bytes) -> bytes:
    result = (
        b"T"
        + len(label).to_bytes(4, "big")
        + label
        + len(children).to_bytes(4, "big")
    )
    for child in children:
        result += len(child).to_bytes(8, "big") + child
    return result


def _trace(source: bytes, route: bytes) -> bytes:
    argument = source[6:7]
    fixed_token = lambda spelling, start, end: _tree(
        b"token:fixed:"
        + spelling.hex().encode("ascii")
        + b":"
        + str(start).encode("ascii")
        + b":"
        + str(end).encode("ascii")
    )
    lexical_token = lambda spelling, start, end: _tree(
        b"token:lexical:4944454e54:"
        + str(start).encode("ascii")
        + b":"
        + str(end).encode("ascii")
    )
    left = fixed_token(b"(", 5, 6)
    value = lexical_token(argument, 6, 7)
    right = fixed_token(b")", 7, 8)
    if route == b"fixed":
        callee = _tree(
            b"node:66697865645f63616c6c:0.0:fixed:6465726566",
            fixed_token(b"deref", 0, 5),
        )
        body = _tree(
            b"node:66697865645f63616c6c:0:sequence:-",
            callee,
            _tree(b"node:66697865645f63616c6c:0.1:fixed:28", left),
            _tree(b"node:66697865645f63616c6c:0.2:ref:4944454e54", value),
            _tree(b"node:66697865645f63616c6c:0.3:fixed:29", right),
        )
        selected = _tree(
            b"node:65787072:0.0:ref:66697865645f63616c6c",
            _tree(b"production:66697865645f63616c6c", body),
        )
        node = _tree(b"node:65787072:0:choice:0", selected)
    else:
        body = _tree(
            b"node:63616c6c:0:sequence:-",
            _tree(
                b"node:63616c6c:0.0:ref:4944454e54",
                lexical_token(b"deref", 0, 5),
            ),
            _tree(b"node:63616c6c:0.1:fixed:28", left),
            _tree(b"node:63616c6c:0.2:ref:4944454e54", value),
            _tree(b"node:63616c6c:0.3:fixed:29", right),
        )
        selected = _tree(
            b"node:65787072:0.1:ref:63616c6c",
            _tree(b"production:63616c6c", body),
        )
        node = _tree(b"node:65787072:0:choice:1", selected)
    return _tree(b"production:65787072", node).hex().encode("ascii")


def oracle_report(value: Inputs) -> bytes:
    records: list[bytes] = []
    deltas: list[bytes] = []
    for identifier, source in (
        (b"deref-p", b"deref(p)"),
        (b"deref-x", b"deref(x)"),
    ):
        identifier_hex = identifier.hex().encode("ascii")
        start_hex = b"expr".hex().encode("ascii")
        source_hex = source.hex().encode("ascii")
        records.append(
            b"\t".join(
                (b"CASE", b"current", identifier_hex, start_hex, source_hex, b"many", b"2")
            )
        )
        fixed_trace = _trace(source, b"fixed")
        call_trace = _trace(source, b"call")
        records.append(
            b"\t".join((b"CASE-TRACE", b"current", identifier_hex, b"0", fixed_trace))
        )
        records.append(
            b"\t".join((b"CASE-TRACE", b"current", identifier_hex, b"1", call_trace))
        )
        records.append(
            b"\t".join(
                (b"CASE", b"proposal", identifier_hex, start_hex, source_hex, b"one", b"1")
            )
        )
        records.append(
            b"\t".join((b"CASE-TRACE", b"proposal", identifier_hex, b"0", fixed_trace))
        )
        deltas.extend(
            (
                b"\t".join((b"CASE-DELTA", identifier_hex, b"removed", call_trace)),
                b"\t".join((b"CASE-DELTA", identifier_hex, b"retained", fixed_trace)),
            )
        )
    records.extend(deltas)
    domain_hex = b"fixed-lowerword-calls".hex().encode("ascii")
    start_hex = b"expr".hex().encode("ascii")
    argument_hex = b"x".hex().encode("ascii")
    source = b"deref(x)"
    digest = hashlib.sha256(
        len(source).to_bytes(8, "big") + source
    ).hexdigest().encode("ascii")
    for document in (b"current", b"proposal"):
        records.append(
            b"\t".join(
                (b"DOMAIN", document, domain_hex, start_hex, argument_hex, b"1", digest)
            )
        )
        records.append(
            b"\t".join(
                (
                    b"STREAM",
                    document,
                    domain_hex,
                    b"0",
                    source.hex().encode("ascii"),
                    b"one",
                    b"1",
                )
            )
        )
        records.append(
            b"\t".join(
                (
                    b"STREAM-TRACE",
                    document,
                    domain_hex,
                    b"0",
                    b"0",
                    _trace(source, b"fixed"),
                )
            )
        )
    records.extend(
        (
            b"METRIC\tcurrent\t3\t8\t32\t16\t4",
            b"METRIC\tproposal\t3\t8\t32\t16\t4",
        )
    )
    lines = [b"WFGRREPORT1", b"ENGINE\toracle", b"COMMON-BEGIN"]
    lines.extend(common_lines(value))
    lines.extend((b"COMMON-END", b"ORACLE-BEGIN"))
    lines.extend(records)
    lines.extend((b"ORACLE-END", b"END"))
    return b"\n".join(lines) + b"\n"
