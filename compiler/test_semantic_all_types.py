#!/usr/bin/env python3
"""Resolve every contextual type root in one deterministic semantic gate."""

import ctypes
import tempfile
from contextlib import contextmanager
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_NONE, AstTape
from test_semantic_type_resolve import (
    RESOLVE_CLEAN,
    RESOLVE_MALFORMED_TYPE,
    RESOLVE_UNKNOWN_NAMED_TYPE,
    guards_clean,
    indexed_symbols,
    parsed,
    validate_once,
)
from test_symbols import SYMBOL_NONE, SymbolTape, assert_guards


ALL_TYPES_CLEAN = 0
ALL_TYPES_RESOLVE_FAILED = 1

AST_RETURN_TYPE = 4
AST_PARAMETER_TYPE = 10
AST_BINDING_TYPE = 13
AST_TYPE_ARGUMENT = 17
AST_NESTED_TYPE = 23
AST_CONST_ARGUMENT = 24
AST_FIELD_TYPE = 35
AST_CONST_TYPE = 38

TYPE_ROOT_KINDS = {
    AST_RETURN_TYPE,
    AST_PARAMETER_TYPE,
    AST_BINDING_TYPE,
    AST_TYPE_ARGUMENT,
    AST_NESTED_TYPE,
    AST_FIELD_TYPE,
    AST_CONST_TYPE,
}


class SemanticAllTypesReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("type_node", ctypes.c_uint64),
        ("resolved_count", ctypes.c_uint64),
        ("resolve_status", ctypes.c_int32),
        ("resolve_related", ctypes.c_uint64),
    ]


def configure(library):
    from test_semantic_type_resolve import configure as configure_one_type

    configure_one_type(library)
    library.semantic_resolve_all_types.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SemanticAllTypesReport),
    ]
    library.semantic_resolve_all_types.restype = None


def mixed_fixture():
    return (
        b"enum Kind { Alpha(); Beta(); }\n"
        b"const width: u64 = 4_u64;\n"
        b"struct Row { code: u8; kind: Kind; }\n"
        b"struct Packet {\n"
        b"  flag: Bool;\n"
        b"  row: Row;\n"
        b"  bytes: buffer<u8>;\n"
        b"  rows: array<Row, width>;\n"
        b"}\n"
        b"fn inspect (packet: own Packet, rows: own buffer<Row>) -> own buffer<Row> pure {\n"
        b"  let count: own u64 = len<Row>(rows);\n"
        b"  return rows;\n"
        b"}\n"
    )


def ready(library, data):
    source_storage, source, token_storage, tokens, columns, ast = parsed(
        library, data
    )
    validated = validate_once(library, source, tokens, ast)
    assert validated[2].status == 0
    symbol_storage, symbol_physical, symbols = indexed_symbols(
        library, source, tokens, ast, capacity=ast.count
    )
    return (
        source_storage,
        source,
        token_storage,
        tokens,
        columns,
        ast,
        validated,
        symbol_storage,
        symbol_physical,
        symbols,
    )


def type_nodes(columns, ast):
    kinds = columns[0]
    return [
        node for node in range(ast.count) if kinds[node] in TYPE_ROOT_KINDS
    ]


def invoke(library, source, tokens, ast, symbols, validated):
    report = SemanticAllTypesReport(99, 99, 99, 99, 99)
    library.semantic_resolve_all_types(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(validated[2]),
        ctypes.byref(report),
    )
    guards_clean(validated[0], validated[1])
    return report


def report_tuple(report):
    return (
        report.status,
        report.type_node,
        report.resolved_count,
        report.resolve_status,
        report.resolve_related,
    )


def assert_clean_unit(library, data):
    state = ready(library, data)
    _, source, _, tokens, columns, ast, validated, symbol_storage, physical, symbols = state
    expected = len(type_nodes(columns, ast))
    assert expected > 0

    first = invoke(library, source, tokens, ast, symbols, validated)
    second = invoke(library, source, tokens, ast, symbols, validated)
    clean = (ALL_TYPES_CLEAN, AST_NONE, expected, RESOLVE_CLEAN, SYMBOL_NONE)
    assert report_tuple(first) == clean, report_tuple(first)
    assert report_tuple(second) == clean, report_tuple(second)
    assert_guards(symbol_storage, physical)
    return expected


def find_node(data, columns, kind, text):
    kinds, _, starts, ends, _, _, _ = columns
    matches = [
        node
        for node in range(len(kinds) - 1)
        if kinds[node] == kind and data[starts[node] : ends[node]] == text
    ]
    assert len(matches) == 1, (kind, text, matches)
    return matches[0]


@contextmanager
def changed(column, index, value):
    before = column[index]
    column[index] = value
    try:
        yield
    finally:
        column[index] = before


def assert_first_failure(library):
    unknown_data = (
        b"struct Known { value: u64; }\n"
        b"struct Broken { okay: Known; missing: Missing; later: u64; }\n"
    )
    state = ready(library, unknown_data)
    _, source, _, tokens, columns, ast, validated, symbols_storage, physical, symbols = state
    unknown = find_node(unknown_data, columns, AST_FIELD_TYPE, b"Missing")
    before = sum(node < unknown for node in type_nodes(columns, ast))
    report = invoke(library, source, tokens, ast, symbols, validated)
    assert report_tuple(report) == (
        ALL_TYPES_RESOLVE_FAILED,
        unknown,
        before,
        RESOLVE_UNKNOWN_NAMED_TYPE,
        columns[1][unknown],
    )
    assert_guards(symbols_storage, physical)

    malformed_data = b"struct Bad { okay: u64; broken: buffer<u8>; later: u64; }\n"
    state = ready(library, malformed_data)
    _, source, _, tokens, columns, ast, validated, symbols_storage, physical, symbols = state
    outer = find_node(
        malformed_data, columns, AST_FIELD_TYPE, b"buffer<u8>"
    )
    child = columns[4][outer]
    assert child != AST_NONE and columns[0][child] == AST_NESTED_TYPE
    before = sum(node < outer for node in type_nodes(columns, ast))
    with changed(columns[0], child, AST_CONST_ARGUMENT):
        report = invoke(library, source, tokens, ast, symbols, validated)
        assert report_tuple(report) == (
            ALL_TYPES_RESOLVE_FAILED,
            outer,
            before,
            RESOLVE_MALFORMED_TYPE,
            child,
        )
    assert_guards(symbols_storage, physical)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        mixed_count = assert_clean_unit(library, mixed_fixture())
        compiler_count = assert_clean_unit(
            library, compiler_source().encode("ascii")
        )
        assert_first_failure(library)
        print(
            "semantic all-types gate: "
            f"mixed={mixed_count}, compiler={compiler_count}, deterministic; "
            "unknown and malformed roots stop at the first ordinal"
        )


if __name__ == "__main__":
    main()
