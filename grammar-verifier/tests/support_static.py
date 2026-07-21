"""Canonical static-report fixtures for runner-boundary tests."""

from __future__ import annotations

import hashlib

from runner_inputs import Inputs
from support_common import common_lines


def static_report(value: Inputs) -> bytes:
    encode = lambda descriptor: descriptor.hex().encode("ascii")
    lhs = encode(b"expr")
    predicates = (
        b"end",
        b"fixed:28",
        b"fixed:29",
        b"fixed:6465726566",
        b"lex:4944454e54",
        b"lex:545950454944",
    )
    fixed_predicate = encode(b"fixed:6465726566")
    ident_predicate = encode(b"lex:4944454e54")
    fixed_word = encode(b"fixed:6465726566,fixed:28")
    ident_word = encode(b"lex:4944454e54,fixed:28")
    end_word = encode(b"end,end")
    witness_stream = b"6465726566,28".hex().encode("ascii")
    specific: list[bytes] = []
    for document in (b"current", b"proposal"):
        for symbol, first_word in (
            (b"call", ident_word),
            (b"expr", fixed_word),
            (b"fixed_call", fixed_word),
        ):
            symbol_hex = encode(symbol)
            specific.append(
                b"\t".join((b"STATIC-NULLABLE", document, symbol_hex, b"0"))
            )
            specific.append(
                b"\t".join((b"STATIC-FIRST", document, symbol_hex, first_word))
            )
            specific.append(
                b"\t".join((b"STATIC-FOLLOW", document, symbol_hex, end_word))
            )
        specific.extend(
            (
                b"\t".join(
                    (b"STATIC-DECISION", document, lhs, b"0", b"choice", b"0", fixed_word)
                ),
                b"\t".join(
                    (b"STATIC-DECISION", document, lhs, b"0", b"choice", b"1", ident_word)
                ),
            )
        )
        witnesses = {
            b"end": b"-",
            b"fixed:28": b"28",
            b"fixed:29": b"29",
            b"fixed:6465726566": b"6465726566",
            b"lex:4944454e54": b"61",
            b"lex:545950454944": b"41",
        }
        for predicate in predicates:
            encoded = encode(predicate)
            specific.append(
                b"\t".join(
                    (b"STATIC-INTERSECTION", document, encoded, encoded, witnesses[predicate])
                )
            )
    specific.extend(
        (
            b"\t".join(
                (
                    b"STATIC-INTERSECTION",
                    b"current",
                    fixed_predicate,
                    ident_predicate,
                    b"deref".hex().encode("ascii"),
                )
            ),
            b"\t".join(
                (
                    b"STATIC-CONFLICT",
                    b"current",
                    lhs,
                    b"0",
                    b"choice",
                    b"0",
                    b"1",
                    fixed_word,
                    ident_word,
                    witness_stream,
                )
            ),
        )
    )
    intersection_key = b"\t".join((fixed_predicate, ident_predicate))
    conflict_key = b"\t".join(
        (lhs, b"0", b"choice", b"0", b"1", fixed_word, ident_word)
    )
    specific.append(
        b"\t".join(
            (b"STATIC-DELTA", b"conflict", b"removed", conflict_key.hex().encode("ascii"))
        )
    )
    specific.append(
        b"\t".join(
            (
                b"STATIC-DELTA",
                b"intersection",
                b"removed",
                intersection_key.hex().encode("ascii"),
            )
        )
    )
    for predicate in predicates:
        key = b"\t".join((encode(predicate), encode(predicate)))
        specific.append(
            b"\t".join(
                (b"STATIC-DELTA", b"intersection", b"retained", key.hex().encode("ascii"))
            )
        )
    for document in (b"current", b"proposal"):
        for identifier, source in (
            (b"deref-p", b"deref(p)"),
            (b"deref-x", b"deref(x)"),
        ):
            specific.append(
                b"\t".join(
                    (
                        b"STATIC-CASE",
                        document,
                        identifier,
                        b"expr".hex().encode("ascii"),
                        source.hex().encode("ascii"),
                        b"0" if document == b"proposal" else b"1",
                    )
                )
            )
    source = b"deref(x)"
    digest = hashlib.sha256(
        len(source).to_bytes(8, "big") + source
    ).hexdigest().encode("ascii")
    for document in (b"current", b"proposal"):
        specific.append(
            b"\t".join(
                (
                    b"STATIC-DOMAIN",
                    document,
                    b"fixed-lowerword-calls".hex().encode("ascii"),
                    b"expr".hex().encode("ascii"),
                    b"x".hex().encode("ascii"),
                    b"1",
                    digest,
                )
            )
        )
    specific.append(
        b"STATIC-TRANSITION\tfixed-ident-partition\t1\t0\t"
        b"removes-call-through-ident\t6465726566287829"
    )
    tag_order = {
        tag: index
        for index, tag in enumerate(
            (
                b"STATIC-NULLABLE",
                b"STATIC-FIRST",
                b"STATIC-FOLLOW",
                b"STATIC-INTERSECTION",
                b"STATIC-DECISION",
                b"STATIC-CONFLICT",
                b"STATIC-DELTA",
                b"STATIC-CASE",
                b"STATIC-DOMAIN",
                b"STATIC-TRANSITION",
            )
        )
    }
    specific.sort(key=lambda line: (tag_order[line.split(b"\t", 1)[0]], line))
    lines = [b"WFGRREPORT1", b"ENGINE\tstatic", b"COMMON-BEGIN"]
    lines.extend(common_lines(value))
    lines.extend((b"COMMON-END", b"STATIC-BEGIN"))
    lines.extend(specific)
    lines.extend((b"STATIC-END", b"END"))
    return b"\n".join(lines) + b"\n"
