#!/usr/bin/env python3
"""Resolve parsed type uses against exact global symbols and const sizes."""

import ctypes
import tempfile
from contextlib import contextmanager
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_lexer import Buffer, TokenTape, build_library
from test_parser import AST_NONE, AstTape, parse
from test_semantic_globals import (
    SEMANTIC_GLOBALS_CLEAN,
    SemanticGlobalsReport,
    configure as configure_globals,
)
from test_symbols import (
    SYMBOL_CLEAN,
    SYMBOL_NONE,
    SYMBOL_TYPE,
    SYMBOL_VALUE,
    SymbolTape,
    assert_guards,
    lex as lex_symbol_source,
    make_symbols,
    reset,
)


RESOLVE_CLEAN = 0
RESOLVE_INVALID_TOKEN_TAPE = 1
RESOLVE_INVALID_AST_TAPE = 2
RESOLVE_INVALID_SYMBOL_TAPE = 3
RESOLVE_INVALID_SCRATCH = 4
RESOLVE_MALFORMED_TYPE = 5
RESOLVE_UNKNOWN_NAMED_TYPE = 6
RESOLVE_INVALID_ARRAY_CONST = 7

AST_RETURN_TYPE = 4
AST_PARAMETER_TYPE = 10
AST_NESTED_TYPE = 23
AST_CONST_ARGUMENT = 24
AST_FIELD_TYPE = 35
AST_CONST_TYPE = 38
AST_FUNCTION = 1

AST_VALIDATION_POISON = 99
U64_GUARD = 0xA9A9A9A9A9A9A9A9


class SemanticTypeResolveReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("node", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
    ]


def fixture():
    return (
        b"enum TokenKind { Word(); End(); }\n"
        b"struct TokenTape { count: u64; }\n"
        b"const width: u64 = 4_u64;\n"
        b"const float_width: f64 = 4.0_f64;\n"
        b"const table_width: array<u8, 1> = [4_u8];\n"
        b"const self_width: array<u8, self_width> = [1_u8];\n"
        b"fn capacity () -> own u64 pure { return 4_u64; }\n"
        b"struct Uses {\n"
        b"  primitive: u64;\n"
        b"  boolean: Bool;\n"
        b"  overflow_error: Overflow;\n"
        b"  div_error: DivError;\n"
        b"  narrow_error: NarrowError;\n"
        b"  bare_option: Option;\n"
        b"  bare_result: Result;\n"
        b"  token: TokenKind;\n"
        b"  tape: TokenTape;\n"
        b"  bytes: buffer<TokenKind>;\n"
        b"  fixed: array<TokenTape, width>;\n"
        b"  literal: array<u8, 10>;\n"
        b"  missing: Missing;\n"
        b"  wrong_namespace: Capacity;\n"
        b"  forward_type: FutureType;\n"
        b"  bad_function_size: array<u8, capacity>;\n"
        b"  bad_float_size: array<u8, float_width>;\n"
        b"  bad_array_size: array<u8, table_width>;\n"
        b"}\n"
        b"fn inspect (tokens: own buffer<TokenKind>, tapes: own array<TokenTape, width>, flag: own Bool) -> own buffer<TokenKind> pure {\n"
        b"  return tokens;\n"
        b"}\n"
        b"const later_width: u64 = 8_u64;\n"
        b"struct TooEarly { values: array<u8, after_width>; }\n"
        b"const after_width: u64 = 8_u64;\n"
        b"struct FutureType { value: u64; }\n"
    )


def source_buffer(storage, length):
    return Buffer(ctypes.cast(storage, ctypes.c_void_p), length)


def configure(library):
    configure_globals(library)
    library.semantic_resolve_type.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.c_uint64,
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SemanticTypeResolveReport),
    ]
    library.semantic_resolve_type.restype = None
    library.ast_validate.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
    ]
    library.ast_validate.restype = None
    library.semantic_type_resolve_decimal_const.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.c_uint64,
    ]
    library.semantic_type_resolve_decimal_const.restype = ctypes.c_bool


def parsed(library, data):
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    return (
        source_storage,
        source_buffer(source_storage, len(data)),
        token_storage,
        tokens,
        columns,
        ast,
    )


def indexed_symbols(library, source, tokens, ast, capacity=32):
    storage, physical, symbols = make_symbols((capacity,) * 4)
    reset(library, symbols)
    report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(report),
    )
    assert report.status == SEMANTIC_GLOBALS_CLEAN
    assert symbols.status == SYMBOL_CLEAN
    assert_guards(storage, physical)
    return storage, physical, symbols


def make_validation(tokens, ast, marks_length=None, stack_length=None):
    if marks_length is None:
        marks_length = tokens.count
    if stack_length is None:
        stack_length = ast.count
    marks_physical = max(1, tokens.count)
    stack_physical = max(1, ast.count)
    marks = (ctypes.c_uint64 * (marks_physical + 1))()
    stack = (ctypes.c_uint64 * (stack_physical + 1))()
    marks[marks_physical] = U64_GUARD
    stack[stack_physical] = U64_GUARD
    validation = AstValidationReport(
        Buffer(ctypes.cast(marks, ctypes.c_void_p), marks_length),
        Buffer(ctypes.cast(stack, ctypes.c_void_p), stack_length),
        AST_VALIDATION_POISON,
        99,
        99,
    )
    return (marks, stack), (marks_physical, stack_physical), validation


def validate_once(
    library,
    source,
    tokens,
    ast,
    marks_length=None,
    stack_length=None,
):
    storage, physical, validation = make_validation(
        tokens, ast, marks_length, stack_length
    )
    library.ast_validate(
        source.length,
        tokens.count,
        ctypes.byref(ast),
        ctypes.byref(validation),
    )
    guards_clean(storage, physical)
    return storage, physical, validation


def guards_clean(storage, physical):
    marks, stack = storage
    marks_physical, stack_physical = physical
    assert marks[marks_physical] == U64_GUARD
    assert stack[stack_physical] == U64_GUARD


def invoke(
    library,
    source,
    tokens,
    ast,
    symbols,
    node,
    validated=None,
):
    if validated is None:
        validated = validate_once(library, source, tokens, ast)
    scratch_storage, scratch_physical, validation = validated
    report = SemanticTypeResolveReport(99, 99, 99)
    library.semantic_resolve_type(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        node,
        ctypes.byref(validation),
        ctypes.byref(report),
    )
    guards_clean(scratch_storage, scratch_physical)
    return report


def find_node(data, columns, kind, text, occurrence=0):
    kinds, _, starts, ends, _, _, _ = columns
    matches = [
        node
        for node in range(len(kinds) - 1)
        if kinds[node] == kind and data[starts[node] : ends[node]] == text
    ]
    assert len(matches) > occurrence, (kind, text, matches)
    return matches[occurrence]


@contextmanager
def changed(column, index, value):
    before = column[index]
    column[index] = value
    try:
        yield
    finally:
        column[index] = before


def assert_valid_and_resolution_errors(library):
    data = fixture()
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    symbol_storage, symbol_physical, symbols = indexed_symbols(
        library, source, tokens, ast
    )
    validated = validate_once(library, source, tokens, ast)
    assert validated[2].status == 0

    valid = (
        (AST_FIELD_TYPE, b"u64", 0),
        (AST_FIELD_TYPE, b"Bool", 0),
        (AST_FIELD_TYPE, b"Overflow", 0),
        (AST_FIELD_TYPE, b"DivError", 0),
        (AST_FIELD_TYPE, b"NarrowError", 0),
        (AST_FIELD_TYPE, b"TokenKind", 0),
        (AST_FIELD_TYPE, b"TokenTape", 0),
        (AST_FIELD_TYPE, b"buffer<TokenKind>", 0),
        (AST_FIELD_TYPE, b"array<TokenTape, width>", 0),
        (AST_FIELD_TYPE, b"array<u8, 10>", 0),
        (AST_PARAMETER_TYPE, b"buffer<TokenKind>", 0),
        (AST_PARAMETER_TYPE, b"array<TokenTape, width>", 0),
        (AST_PARAMETER_TYPE, b"Bool", 0),
        (AST_RETURN_TYPE, b"buffer<TokenKind>", 0),
        (AST_CONST_TYPE, b"u64", 0),
        (AST_CONST_TYPE, b"array<u8, 1>", 0),
    )
    for kind, text, occurrence in valid:
        node = find_node(data, columns, kind, text, occurrence)
        report = invoke(library, source, tokens, ast, symbols, node, validated)
        assert (report.status, report.node, report.related) == (
            RESOLVE_CLEAN,
            AST_NONE,
            SYMBOL_NONE,
        ), (kind, text, report.status, report.node, report.related)

    for text in (b"Option", b"Result"):
        bare_generic = find_node(data, columns, AST_FIELD_TYPE, text)
        report = invoke(
            library, source, tokens, ast, symbols, bare_generic, validated
        )
        assert (report.status, report.node, report.related) == (
            RESOLVE_MALFORMED_TYPE,
            bare_generic,
            columns[1][bare_generic],
        ), (text, report.status, report.node, report.related)

    unknown = find_node(data, columns, AST_FIELD_TYPE, b"Missing")
    report = invoke(library, source, tokens, ast, symbols, unknown, validated)
    assert (report.status, report.node) == (RESOLVE_UNKNOWN_NAMED_TYPE, unknown)

    # Value symbols never satisfy a TypeID query. Capitalization makes equal-byte
    # cross-namespace declarations unspellable, so Capacity is the hostile edge:
    # only the lowercase value `capacity` exists.
    wrong_namespace = find_node(data, columns, AST_FIELD_TYPE, b"Capacity")
    report = invoke(
        library, source, tokens, ast, symbols, wrong_namespace, validated
    )
    assert (report.status, report.node) == (
        RESOLVE_UNKNOWN_NAMED_TYPE,
        wrong_namespace,
    )

    forward_type = find_node(data, columns, AST_FIELD_TYPE, b"FutureType")
    report = invoke(library, source, tokens, ast, symbols, forward_type, validated)
    assert (report.status, report.node) == (
        RESOLVE_UNKNOWN_NAMED_TYPE,
        forward_type,
    )

    for text in (
        b"array<u8, capacity>",
        b"array<u8, float_width>",
        b"array<u8, table_width>",
        b"array<u8, after_width>",
    ):
        node = find_node(data, columns, AST_FIELD_TYPE, text)
        report = invoke(library, source, tokens, ast, symbols, node, validated)
        assert report.status == RESOLVE_INVALID_ARRAY_CONST, (
            text,
            report.status,
            report.node,
            report.related,
        )

    self_size = find_node(
        data, columns, AST_CONST_TYPE, b"array<u8, self_width>"
    )
    report = invoke(library, source, tokens, ast, symbols, self_size, validated)
    assert report.status == RESOLVE_INVALID_ARRAY_CONST

    # The structural AST remains a valid tree, but the generic child role is
    # wrong. That is a malformed type, not an invalid tape.
    buffer_node = find_node(data, columns, AST_FIELD_TYPE, b"buffer<TokenKind>")
    nested = columns[4][buffer_node]
    assert columns[0][nested] == AST_NESTED_TYPE
    with changed(columns[0], nested, AST_CONST_ARGUMENT):
        report = invoke(
            library, source, tokens, ast, symbols, buffer_node, validated
        )
        assert (report.status, report.node) == (
            RESOLVE_MALFORMED_TYPE,
            buffer_node,
        )

    assert_guards(symbol_storage, symbol_physical)
    assert source_storage


def assert_canonical_numeric_sizes(library):
    cases = (
        (b"0", RESOLVE_CLEAN),
        (b"10", RESOLVE_CLEAN),
        (b"18446744073709551615", RESOLVE_CLEAN),
        (b"00", RESOLVE_INVALID_ARRAY_CONST),
        (b"01", RESOLVE_INVALID_ARRAY_CONST),
        (b"18446744073709551616", RESOLVE_INVALID_ARRAY_CONST),
        (b"999999999999999999999", RESOLVE_INVALID_ARRAY_CONST),
    )
    for size, expected in cases:
        data = b"struct Numbers { value: array<u8, " + size + b">; }"
        source_storage, source, _, tokens, columns, ast = parsed(library, data)
        symbol_storage, symbol_physical, symbols = indexed_symbols(
            library, source, tokens, ast
        )
        validated = validate_once(library, source, tokens, ast)
        assert validated[2].status == 0
        node = find_node(
            data, columns, AST_FIELD_TYPE, b"array<u8, " + size + b">"
        )
        report = invoke(library, source, tokens, ast, symbols, node, validated)
        assert report.status == expected, (size, report.status, report.related)
        assert_guards(symbol_storage, symbol_physical)
        assert source_storage

    # TokNumber deliberately has a broad lexical tail. The semantic decimal
    # check must scan every byte before an early lexicographic range decision.
    malformed = b"1000000000000000000x"
    assert len(malformed) == 20
    malformed_storage, malformed_source, _, malformed_tokens = lex_symbol_source(
        library, malformed
    )
    assert not library.semantic_type_resolve_decimal_const(
        malformed_source, ctypes.byref(malformed_tokens), 0
    )
    assert malformed_storage


def assert_fail_closed_inputs(library):
    data = fixture()

    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    symbol_storage, symbol_physical, symbols = indexed_symbols(
        library, source, tokens, ast
    )
    node = find_node(data, columns, AST_FIELD_TYPE, b"buffer<TokenKind>")
    validated = validate_once(library, source, tokens, ast)
    assert validated[2].status == 0

    kinds_length = tokens.kinds.length
    tokens.kinds.length = tokens.count - 1
    report = invoke(library, source, tokens, ast, symbols, node, validated)
    assert report.status == RESOLVE_INVALID_TOKEN_TAPE
    tokens.kinds.length = kinds_length

    heads_length = ast.heads.length
    ast.heads.length = ast.count - 1
    report = invoke(library, source, tokens, ast, symbols, node, validated)
    assert report.status == RESOLVE_INVALID_AST_TAPE
    ast.heads.length = heads_length

    old_stack_length = validated[2].stack.length
    validated[2].stack.length = ast.count - 1
    report = invoke(library, source, tokens, ast, symbols, node, validated)
    assert report.status == RESOLVE_INVALID_SCRATCH
    validated[2].stack.length = old_stack_length

    old_count = symbols.count
    symbols.count = symbols.namespaces.length + 1
    report = invoke(library, source, tokens, ast, symbols, node, validated)
    assert report.status == RESOLVE_INVALID_SYMBOL_TAPE
    symbols.count = old_count

    # A legal value-namespace function entry is found for a named array size,
    # but it is rejected because only integer-typed const declarations qualify.
    function_size = find_node(
        data, columns, AST_FIELD_TYPE, b"array<u8, capacity>"
    )
    report = invoke(
        library, source, tokens, ast, symbols, function_size, validated
    )
    assert report.status == RESOLVE_INVALID_ARRAY_CONST

    # A caller cannot pass an arbitrary AST node as a type root.
    function = next(
        index for index in range(ast.count) if columns[0][index] == AST_FUNCTION
    )
    report = invoke(library, source, tokens, ast, symbols, function, validated)
    assert report.status == RESOLVE_MALFORMED_TYPE

    assert_guards(symbol_storage, symbol_physical)
    assert source_storage


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_valid_and_resolution_errors(library)
        assert_canonical_numeric_sizes(library)
        assert_fail_closed_inputs(library)
        print(
            "semantic type resolution: primitives, leaf prelude types, bare generic rejection, exact global types, "
            "nested buffer/array shapes, canonical u64 sizes, named integer "
            "consts, namespaces, malformed trees, and hostile tapes pass"
        )


if __name__ == "__main__":
    main()
