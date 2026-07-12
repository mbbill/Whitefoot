#!/usr/bin/env python3
"""Exercise the first global-declaration semantic indexing pass."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library
from test_parser import AST_NONE, AST_PROGRAM, AstTape, children_of, parse
from test_symbols import (
    SYMBOL_CLEAN,
    SYMBOL_NONE,
    SYMBOL_TYPE,
    SYMBOL_VALUE,
    SymbolTape,
    assert_guards,
    column_snapshot,
    find,
    insert,
    make_symbols,
    reset,
)


SEMANTIC_GLOBALS_CLEAN = 0
SEMANTIC_GLOBALS_INVALID_TOKEN_TAPE = 1
SEMANTIC_GLOBALS_INVALID_AST_TAPE = 2
SEMANTIC_GLOBALS_EXPECTED_PROGRAM = 3
SEMANTIC_GLOBALS_MALFORMED_DECLARATION = 4
SEMANTIC_GLOBALS_INVALID_NAME_TOKEN = 5
SEMANTIC_GLOBALS_DUPLICATE = 6
SEMANTIC_GLOBALS_CAPACITY = 7
SEMANTIC_GLOBALS_INVALID_SYMBOL_TAPE = 8

AST_FUNCTION = 1
AST_FUNCTION_NAME = 2
AST_BLOCK = 6
AST_BINDING_NAME = 12
AST_ENUM_DECL = 29
AST_ENUM_NAME = 30
AST_STRUCT_DECL = 32
AST_STRUCT_NAME = 33
AST_CONST_DECL = 36
AST_CONST_NAME = 37


class SemanticGlobalsReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("declaration", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
    ]


def mixed_fixture():
    return (
        b"fn alpha () -> own unit pure { return unit; }\n"
        b"enum Gamma { Variant(); }\n"
        b"const beta: u64 = 7_u64;\n"
        b"struct Delta { field: u64; }\n"
    )


def source_buffer(storage, length):
    return Buffer(ctypes.cast(storage, ctypes.c_void_p), length)


def configure(library):
    library.semantic_index_globals.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(SemanticGlobalsReport),
    ]
    library.semantic_index_globals.restype = None
    library.symbol_tape_reset.argtypes = [ctypes.POINTER(SymbolTape)]
    library.symbol_tape_reset.restype = None
    library.symbol_find_in_scope.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(SymbolTape),
        ctypes.c_int32,
        ctypes.c_uint64,
        ctypes.c_uint64,
    ]
    library.symbol_find_in_scope.restype = None
    library.symbol_insert_unique.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(SymbolTape),
        ctypes.c_int32,
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.c_uint64,
    ]
    library.symbol_insert_unique.restype = None


def invoke(library, source, tokens, ast, symbols):
    report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(report),
    )
    return report


def parsed(library, data):
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    source = source_buffer(source_storage, len(data))
    assert ast.status == 0
    return source_storage, source, token_storage, tokens, columns, ast


def empty_symbols(library, capacities=(8, 8, 8, 8)):
    storage, physical, symbols = make_symbols(capacities)
    reset(library, symbols)
    assert (symbols.count, symbols.status, symbols.related) == (
        0,
        SYMBOL_CLEAN,
        SYMBOL_NONE,
    )
    return storage, physical, symbols


def declaration_names(columns, ast):
    declarations = children_of(columns, ast.root)
    names = [children_of(columns, declaration)[0] for declaration in declarations]
    return declarations, names


def assert_mixed_index_and_lookup(library):
    data = mixed_fixture()
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    kinds, heads, _, _, _, _, _ = columns
    declarations, names = declaration_names(columns, ast)
    assert [kinds[node] for node in declarations] == [
        AST_FUNCTION,
        AST_ENUM_DECL,
        AST_CONST_DECL,
        AST_STRUCT_DECL,
    ]
    assert [kinds[node] for node in names] == [
        AST_FUNCTION_NAME,
        AST_ENUM_NAME,
        AST_CONST_NAME,
        AST_STRUCT_NAME,
    ]

    storage, physical, symbols = empty_symbols(library, (4, 4, 4, 4))
    report = invoke(library, source, tokens, ast, symbols)
    assert (report.status, report.declaration, report.related) == (
        SEMANTIC_GLOBALS_CLEAN,
        AST_NONE,
        SYMBOL_NONE,
    )
    assert symbols.count == 4
    assert list(storage[0][:4]) == [
        SYMBOL_VALUE,
        SYMBOL_TYPE,
        SYMBOL_VALUE,
        SYMBOL_TYPE,
    ]
    assert list(storage[1][:4]) == [SYMBOL_NONE] * 4
    assert list(storage[2][:4]) == [heads[name] for name in names]
    assert list(storage[3][:4]) == declarations
    assert_guards(storage, physical)

    for slot, (declaration, name) in enumerate(zip(declarations, names)):
        namespace = SYMBOL_TYPE if slot in (1, 3) else SYMBOL_VALUE
        assert find(
            library,
            source,
            tokens,
            symbols,
            namespace,
            SYMBOL_NONE,
            heads[name],
        ) == (SYMBOL_CLEAN, slot)
        assert storage[3][slot] == declaration
    assert source_storage


def assert_duplicate_is_atomic(library, data):
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    declarations = children_of(columns, ast.root)
    assert len(declarations) == 2
    storage, physical, symbols = empty_symbols(library, (2, 2, 2, 2))
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert (report.status, report.declaration, report.related) == (
        SEMANTIC_GLOBALS_DUPLICATE,
        declarations[1],
        declarations[0],
    )
    assert symbols.count == 0
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_duplicate_namespaces(library):
    assert_duplicate_is_atomic(
        library,
        b"fn same () -> own unit pure { return unit; }\n"
        b"const same: u64 = 1_u64;\n",
    )
    assert_duplicate_is_atomic(
        library,
        b"enum Same { A(); }\n"
        b"struct Same { field: u64; }\n",
    )
    assert_duplicate_is_atomic(
        library,
        b"fn repeat () -> own unit pure { return unit; }\n"
        b"fn repeat () -> own unit pure { return unit; }\n",
    )


def assert_prelude_type_collisions(library):
    for name in (b"Bool", b"Option", b"Result", b"Overflow", b"DivError", b"NarrowError"):
        data = b"struct " + name + b" { value: u64; }\n"
        source_storage, source, _, tokens, columns, ast = parsed(library, data)
        declarations = children_of(columns, ast.root)
        assert len(declarations) == 1
        storage, physical, symbols = empty_symbols(
            library, (ast.count, ast.count, ast.count, ast.count)
        )
        before = column_snapshot(storage, physical)
        for _ in range(2):
            report = invoke(library, source, tokens, ast, symbols)
            assert (report.status, report.declaration, report.related) == (
                SEMANTIC_GLOBALS_DUPLICATE,
                declarations[0],
                SYMBOL_NONE,
            ), (name, report.status, report.declaration, report.related)
            assert symbols.count == 0
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)
        assert source_storage


def assert_existing_duplicate(library):
    data = b"fn alpha () -> own unit pure { return unit; }"
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    declarations, names = declaration_names(columns, ast)
    name_token = columns[1][names[0]]
    storage, physical, symbols = empty_symbols(library, (2, 2, 2, 2))
    assert insert(
        library,
        source,
        tokens,
        symbols,
        SYMBOL_VALUE,
        SYMBOL_NONE,
        name_token,
        700,
    ) == (SYMBOL_CLEAN, 0)
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert (report.status, report.declaration, report.related) == (
        SEMANTIC_GLOBALS_DUPLICATE,
        declarations[0],
        700,
    )
    assert symbols.count == 1
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_asymmetric_capacity_is_atomic(library):
    data = mixed_fixture()
    source_storage, source, _, tokens, _, ast = parsed(library, data)
    fields = ("namespaces", "scopes", "name_tokens", "declarations")
    for short_index, field in enumerate(fields):
        capacities = [4, 4, 4, 4]
        capacities[short_index] = 3
        storage, physical, symbols = empty_symbols(library, tuple(capacities))
        before = column_snapshot(storage, physical)
        for _ in range(2):
            report = invoke(library, source, tokens, ast, symbols)
            assert (report.status, report.declaration, report.related) == (
                SEMANTIC_GLOBALS_CAPACITY,
                AST_NONE,
                4,
            )
            assert symbols.count == 0
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)

        short = getattr(symbols, field)
        setattr(symbols, field, Buffer(short.data, 4))
        report = invoke(library, source, tokens, ast, symbols)
        assert report.status == SEMANTIC_GLOBALS_CLEAN
        assert symbols.count == 4
        assert_guards(storage, physical)
    assert source_storage


def assert_failure_without_symbol_writes(
    library,
    data,
    mutate,
    expected_status,
    expected_declaration=None,
):
    source_storage, source, token_storage, tokens, columns, ast = parsed(library, data)
    declarations, names = declaration_names(columns, ast)
    mutate(source, token_storage, tokens, columns, ast, declarations, names)
    storage, physical, symbols = empty_symbols(library, (4, 4, 4, 4))
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert report.status == expected_status, (
        report.status,
        expected_status,
        report.declaration,
        report.related,
    )
    if expected_declaration is not None:
        assert report.declaration == expected_declaration(declarations)
    assert symbols.count == 0
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_hostile_inputs(library):
    data = mixed_fixture()

    def short_token_column(source, token_storage, tokens, columns, ast, declarations, names):
        tokens.kinds = Buffer(tokens.kinds.data, tokens.count - 1)

    assert_failure_without_symbol_writes(
        library,
        data,
        short_token_column,
        SEMANTIC_GLOBALS_INVALID_TOKEN_TAPE,
    )

    def short_ast_column(source, token_storage, tokens, columns, ast, declarations, names):
        ast.heads = Buffer(ast.heads.data, ast.count - 1)

    assert_failure_without_symbol_writes(
        library,
        data,
        short_ast_column,
        SEMANTIC_GLOBALS_INVALID_AST_TAPE,
    )

    def bad_root_reference(source, token_storage, tokens, columns, ast, declarations, names):
        ast.root = ast.count

    assert_failure_without_symbol_writes(
        library,
        data,
        bad_root_reference,
        SEMANTIC_GLOBALS_INVALID_AST_TAPE,
    )

    def wrong_root_kind(source, token_storage, tokens, columns, ast, declarations, names):
        columns[0][ast.root] = AST_FUNCTION

    assert_failure_without_symbol_writes(
        library,
        data,
        wrong_root_kind,
        SEMANTIC_GLOBALS_EXPECTED_PROGRAM,
        lambda declarations: 0,
    )

    def bad_program_child(source, token_storage, tokens, columns, ast, declarations, names):
        columns[4][ast.root] = ast.count

    assert_failure_without_symbol_writes(
        library,
        data,
        bad_program_child,
        SEMANTIC_GLOBALS_INVALID_AST_TAPE,
    )

    def bad_name_reference(source, token_storage, tokens, columns, ast, declarations, names):
        columns[4][declarations[0]] = ast.count

    assert_failure_without_symbol_writes(
        library,
        data,
        bad_name_reference,
        SEMANTIC_GLOBALS_MALFORMED_DECLARATION,
        lambda declarations: declarations[0],
    )

    def wrong_name_kind(source, token_storage, tokens, columns, ast, declarations, names):
        columns[0][names[0]] = AST_BINDING_NAME

    assert_failure_without_symbol_writes(
        library,
        data,
        wrong_name_kind,
        SEMANTIC_GLOBALS_MALFORMED_DECLARATION,
        lambda declarations: declarations[0],
    )

    def name_has_child(source, token_storage, tokens, columns, ast, declarations, names):
        columns[4][names[0]] = declarations[0]
        columns[5][names[0]] = declarations[0]

    assert_failure_without_symbol_writes(
        library,
        data,
        name_has_child,
        SEMANTIC_GLOBALS_MALFORMED_DECLARATION,
        lambda declarations: declarations[0],
    )

    def wrong_name_token(source, token_storage, tokens, columns, ast, declarations, names):
        columns[1][names[0]] = tokens.count - 1

    assert_failure_without_symbol_writes(
        library,
        data,
        wrong_name_token,
        SEMANTIC_GLOBALS_INVALID_NAME_TOKEN,
        lambda declarations: declarations[0],
    )

    def unsupported_top_item(source, token_storage, tokens, columns, ast, declarations, names):
        columns[0][declarations[0]] = AST_BLOCK

    assert_failure_without_symbol_writes(
        library,
        data,
        unsupported_top_item,
        SEMANTIC_GLOBALS_MALFORMED_DECLARATION,
        lambda declarations: declarations[0],
    )

    source_storage, source, _, tokens, _, ast = parsed(library, data)
    storage, physical, symbols = empty_symbols(library, (1, 1, 1, 1))
    symbols.count = 2
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert report.status == SEMANTIC_GLOBALS_INVALID_SYMBOL_TAPE
    assert symbols.count == 2
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_mixed_index_and_lookup(library)
        assert_duplicate_namespaces(library)
        assert_prelude_type_collisions(library)
        assert_existing_duplicate(library)
        assert_asymmetric_capacity_is_atomic(library)
        assert_hostile_inputs(library)
        print(
            "semantic globals: mixed items, exact lookup, prelude type reservations, "
            "shared namespaces, malformed tapes, atomic duplicates and capacity pass"
        )


if __name__ == "__main__":
    main()
