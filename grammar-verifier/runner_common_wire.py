"""Canonical wire decoding and ordering for common report records."""

from __future__ import annotations

import re

from runner_inputs import fail


COMMON_FIELDS = {
    "BIND": 4,
    "RULE": 5,
    "SURFACE": 6,
    "PROD": 8,
    "NODE": 8,
    "LEX": 8,
    "FIXED": 8,
    "REF": 7,
    "COVERAGE": 7,
}
COMMON_RANK = {name: rank for rank, name in enumerate(COMMON_FIELDS)}
CANONICAL_INTEGER = re.compile(rb"0|[1-9][0-9]*\Z")
HEX = re.compile(rb"(?:[0-9a-f]{2})+\Z")
SHA256 = re.compile(rb"[0-9a-f]{64}\Z")
IDENTIFIER = re.compile(rb"[a-z][a-z0-9-]*\Z")
KIND = re.compile(rb"[a-z][a-z0-9_-]*\Z")
PATH = re.compile(rb"0(?:\.(?:0|[1-9][0-9]*))*\Z")
PRODUCTION_NAME = re.compile(rb"[a-z][a-z0-9_]*\Z")
REFERENCE_NAME = re.compile(rb"[A-Za-z_][A-Za-z0-9_]*\Z")
CLOSED_TABLE_NAME = re.compile(rb"[A-Z][A-Z0-9_]*\Z")
SURFACE_KINDS = (b"assignment", b"grammar-fence", b"grammar-inline", b"lexical-cue")
NODE_KINDS = (
    b"choice",
    b"fixed",
    b"group",
    b"optional",
    b"pattern",
    b"ref",
    b"repeat0",
    b"repeat1",
    b"sequence",
)
LEX_KINDS = (b"byte-string", b"closed-table", b"literal-union", b"regex")


def canonical_uint(value: bytes) -> int:
    if not CANONICAL_INTEGER.fullmatch(value):
        fail("report_integer", "an engine report integer is not canonical")
    # Every report integer is an unsigned 64-bit wire value. Check the
    # spelling before conversion so an untrusted line-sized run of digits
    # cannot escape as a Python conversion error or consume excessive work.
    if len(value) > 20 or (len(value) == 20 and value > b"18446744073709551615"):
        fail("report_integer", "an engine report integer exceeds u64")
    return int(value)


def canonical_hex(value: bytes, optional: bool = False) -> None:
    if optional and value == b"-":
        return
    if not HEX.fullmatch(value):
        fail("report_hex", "an engine report byte field is not canonical hex")


def decoded_bytes(value: bytes) -> bytes:
    canonical_hex(value)
    return bytes.fromhex(value.decode("ascii"))


def record_primary(tag: str, fields: list[bytes], document_size: int) -> int:
    """Validate one common record shape and return its source-order offset."""

    if tag == "RULE":
        integer_fields, hex_fields, start = (3, 4), ((2, False),), 3
    elif tag == "SURFACE":
        integer_fields, hex_fields, start = (3, 4), ((5, True),), 3
        if fields[2] not in SURFACE_KINDS:
            fail("report_kind", "a surface kind is not in the closed vocabulary")
    elif tag == "PROD":
        integer_fields = (4, 5, 6, 7)
        hex_fields, start = ((2, True), (3, False)), 4
    elif tag == "NODE":
        integer_fields = (5, 6)
        hex_fields, start = ((2, False), (7, True)), 5
        if not PATH.fullmatch(fields[3]) or fields[4] not in NODE_KINDS:
            fail(
                "report_node",
                "an EBNF node path or kind is not in the closed vocabulary",
            )
    elif tag == "LEX":
        integer_fields = (5, 6)
        hex_fields, start = ((2, True), (3, False), (7, False)), 5
        if fields[4] not in LEX_KINDS:
            fail("report_kind", "a lexical predicate kind is not in the closed vocabulary")
    elif tag == "FIXED":
        integer_fields = (4, 5)
        hex_fields, start = ((2, False), (6, False), (7, False)), 4
        if not PATH.fullmatch(fields[3]):
            fail("report_path", "a fixed-terminal path is not canonical")
    elif tag == "REF":
        integer_fields = (4, 5)
        hex_fields, start = ((2, False), (6, False)), 4
        if not PATH.fullmatch(fields[3]):
            fail("report_path", "a reference path is not canonical")
    else:
        integer_fields, hex_fields, start = (2, 3, 4, 5, 6), (), -1
    integers = {index: canonical_uint(fields[index]) for index in integer_fields}
    for index, optional in hex_fields:
        canonical_hex(fields[index], optional)
    if start < 0:
        return document_size + 1
    begin, end = integers[start], integers[start + 1]
    if begin >= end or end > document_size:
        fail("report_span", "a common record span is outside its document")
    if tag == "PROD":
        rhs_begin, rhs_end = integers[6], integers[7]
        if (
            rhs_begin >= rhs_end
            or rhs_end > document_size
            or not (begin <= rhs_begin < rhs_end <= end)
        ):
            fail("report_span", "a production RHS span is outside its definition")
    return begin


def semantic_key(
    tag: str,
    document: str,
    fields: list[bytes],
) -> tuple[object, ...]:
    if tag == "RULE":
        return (tag, document, fields[2])
    if tag == "SURFACE":
        return (tag, document, fields[2], fields[3], fields[4])
    if tag in ("NODE", "FIXED", "REF"):
        return (tag, document, fields[2], fields[3])
    if tag in ("PROD", "LEX"):
        return (tag, document, fields[3])
    return (tag, document)
