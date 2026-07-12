#!/usr/bin/env python3
"""Exercise atomic function-region and parameter symbol indexing."""

import ctypes
import tempfile
from pathlib import Path

from test_ast_validate import AST_VALIDATION_CLEAN, AstValidationReport, validate
from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_ALLOCATES_EFFECT, AST_NONE, AstTape, children_of, parse
from test_semantic_globals import (
    SEMANTIC_GLOBALS_CLEAN,
    SemanticGlobalsReport,
)
from test_symbols import (
    SYMBOL_CLEAN,
    SYMBOL_NONE,
    SYMBOL_REGION,
    SYMBOL_VALUE,
    SymbolTape,
    assert_guards,
    column_snapshot,
    find,
    insert,
    make_symbols,
    reset,
)


SEMANTIC_FUNCTION_CLEAN = 0
SEMANTIC_FUNCTION_INVALID_TOKEN_TAPE = 1
SEMANTIC_FUNCTION_INVALID_AST_TAPE = 2
SEMANTIC_FUNCTION_EXPECTED_FUNCTION = 3
SEMANTIC_FUNCTION_MALFORMED_FUNCTION = 4
SEMANTIC_FUNCTION_INVALID_NAME_TOKEN = 5
SEMANTIC_FUNCTION_DUPLICATE = 6
SEMANTIC_FUNCTION_CAPACITY = 7
SEMANTIC_FUNCTION_INVALID_SYMBOL_TAPE = 8

AST_FUNCTION = 1
AST_BLOCK = 6
AST_PARAMETER = 9
AST_REGION = 22
AST_REGION_PARAMETERS = 25


class SemanticFunctionSymbolsReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("declaration", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
    ]


def mixed_fixture():
    return (
        b"fn alpha ['r, 'w] (input: &'r buffer<u8>, "
        b"out: &uniq 'w TokenTape, flag: own Bool) -> own unit "
        b"reads('r 'w), writes('w), allocates(heap arena 'r), traps "
        b"{ return unit; }\n"
        b"fn beta ['r] (input: &'r buffer<u8>) -> own unit "
        b"reads('r) { return unit; }\n"
    )


def source_buffer(storage, length):
    return Buffer(ctypes.cast(storage, ctypes.c_void_p), length)


def configure(library):
    library.semantic_index_function_symbols.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(SemanticFunctionSymbolsReport),
    ]
    library.semantic_index_function_symbols.restype = None
    library.ast_validate.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
    ]
    library.ast_validate.restype = None
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


def parsed(library, data):
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    source = source_buffer(source_storage, len(data))
    return source_storage, source, token_storage, tokens, columns, ast


def empty_symbols(library, capacities=(8, 8, 8, 8)):
    storage, physical, symbols = make_symbols(capacities)
    reset(library, symbols)
    return storage, physical, symbols


def invoke(library, source, tokens, ast, function, symbols):
    report = SemanticFunctionSymbolsReport(99, 99, 99)
    library.semantic_index_function_symbols(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        function,
        ctypes.byref(symbols),
        ctypes.byref(report),
    )
    return report


def functions_of(columns, ast):
    functions = children_of(columns, ast.root)
    assert all(columns[0][node] == AST_FUNCTION for node in functions)
    return functions


def declared_symbols(columns, function):
    kinds = columns[0]
    direct = children_of(columns, function)
    regions = []
    if len(direct) > 1 and kinds[direct[1]] == AST_REGION_PARAMETERS:
        regions = children_of(columns, direct[1])
    parameters = [node for node in direct if kinds[node] == AST_PARAMETER]
    return regions, parameters


def logical_symbol_snapshot(storage, symbols):
    return tuple(tuple(column[: symbols.count]) for column in storage)


def index_compiler_symbols(library, source, tokens, columns, ast, functions):
    storage, physical, symbols = empty_symbols(
        library, (ast.count, ast.count, ast.count, ast.count)
    )
    globals_report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(globals_report),
    )
    assert (
        globals_report.status,
        globals_report.declaration,
        globals_report.related,
    ) == (SEMANTIC_GLOBALS_CLEAN, AST_NONE, SYMBOL_NONE)
    global_count = symbols.count

    reports = []
    for function in functions:
        report = invoke(library, source, tokens, ast, function, symbols)
        observed = (report.status, report.declaration, report.related)
        assert observed == (
            SEMANTIC_FUNCTION_CLEAN,
            AST_NONE,
            SYMBOL_NONE,
        ), (function, observed)
        reports.append(observed)

    expected_local_symbols = []
    for function in functions:
        regions, parameters = declared_symbols(columns, function)
        expected_local_symbols.extend(
            (SYMBOL_REGION, function, columns[1][declaration], declaration)
            for declaration in regions
        )
        expected_local_symbols.extend(
            (SYMBOL_VALUE, function, columns[1][declaration], declaration)
            for declaration in parameters
        )
    observed_local_symbols = list(
        zip(
            storage[0][global_count : symbols.count],
            storage[1][global_count : symbols.count],
            storage[2][global_count : symbols.count],
            storage[3][global_count : symbols.count],
        )
    )
    assert observed_local_symbols == expected_local_symbols
    expected_scopes = {
        function for function in functions if any(declared_symbols(columns, function))
    }
    observed_scopes = set(storage[1][global_count : symbols.count])
    assert observed_scopes == expected_scopes
    assert len(observed_scopes) > 1
    assert (symbols.status, symbols.related) == (SYMBOL_CLEAN, SYMBOL_NONE)
    assert_guards(storage, physical)
    return (
        global_count,
        symbols.count,
        symbols.status,
        symbols.related,
        tuple(reports),
        logical_symbol_snapshot(storage, symbols),
    )


def token_bytes(data, token_storage, token):
    starts, ends = token_storage[1], token_storage[2]
    return data[starts[token] : ends[token]]


def assert_mixed_modes_scopes_and_lookup(library):
    data = mixed_fixture()
    source_storage, source, token_storage, tokens, columns, ast = parsed(
        library, data
    )
    functions = functions_of(columns, ast)
    alpha_regions, alpha_parameters = declared_symbols(columns, functions[0])
    beta_regions, beta_parameters = declared_symbols(columns, functions[1])
    assert len(alpha_regions) == 2
    assert len(alpha_parameters) == 3
    assert len(beta_regions) == 1
    assert len(beta_parameters) == 1
    assert any(columns[0][node] == AST_ALLOCATES_EFFECT for node in children_of(columns, functions[0]))

    storage, physical, symbols = empty_symbols(library, (7, 7, 7, 7))
    report = invoke(library, source, tokens, ast, functions[0], symbols)
    assert (report.status, report.declaration, report.related) == (
        SEMANTIC_FUNCTION_CLEAN,
        AST_NONE,
        SYMBOL_NONE,
    )
    alpha_declarations = alpha_regions + alpha_parameters
    assert symbols.count == 5
    assert list(storage[0][:5]) == [
        SYMBOL_REGION,
        SYMBOL_REGION,
        SYMBOL_VALUE,
        SYMBOL_VALUE,
        SYMBOL_VALUE,
    ]
    assert list(storage[1][:5]) == [functions[0]] * 5
    assert list(storage[2][:5]) == [columns[1][node] for node in alpha_declarations]
    assert list(storage[3][:5]) == alpha_declarations

    for slot, declaration in enumerate(alpha_declarations):
        namespace = SYMBOL_REGION if declaration in alpha_regions else SYMBOL_VALUE
        assert find(
            library,
            source,
            tokens,
            symbols,
            namespace,
            functions[0],
            columns[1][declaration],
        ) == (SYMBOL_CLEAN, slot)

    report = invoke(library, source, tokens, ast, functions[1], symbols)
    assert report.status == SEMANTIC_FUNCTION_CLEAN
    assert symbols.count == 7
    assert list(storage[0][5:7]) == [SYMBOL_REGION, SYMBOL_VALUE]
    assert list(storage[1][5:7]) == [functions[1], functions[1]]
    assert list(storage[3][5:7]) == beta_regions + beta_parameters
    assert token_bytes(data, token_storage, columns[1][alpha_regions[0]]) == b"'r"
    assert token_bytes(data, token_storage, columns[1][beta_regions[0]]) == b"'r"
    assert token_bytes(data, token_storage, columns[1][alpha_parameters[0]]) == b"input"
    assert token_bytes(data, token_storage, columns[1][beta_parameters[0]]) == b"input"
    assert find(
        library,
        source,
        tokens,
        symbols,
        SYMBOL_REGION,
        functions[1],
        columns[1][beta_regions[0]],
    ) == (SYMBOL_CLEAN, 5)
    assert find(
        library,
        source,
        tokens,
        symbols,
        SYMBOL_VALUE,
        functions[1],
        columns[1][beta_parameters[0]],
    ) == (SYMBOL_CLEAN, 6)
    assert_guards(storage, physical)
    assert source_storage


def assert_exact_compiler_source(library):
    data = compiler_source().encode("ascii")
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    validation = validate(library, len(data), tokens.count, ast)
    assert (
        validation.status,
        validation.node,
        validation.related,
    ) == (AST_VALIDATION_CLEAN, AST_NONE, AST_NONE)

    top_level = children_of(columns, ast.root)
    functions = [
        node for node in range(ast.count) if columns[0][node] == AST_FUNCTION
    ]
    assert functions == [
        node for node in top_level if columns[0][node] == AST_FUNCTION
    ]
    assert len(functions) > 1
    first = index_compiler_symbols(
        library, source, tokens, columns, ast, functions
    )
    second = index_compiler_symbols(
        library, source, tokens, columns, ast, functions
    )
    assert first == second
    global_count, total_count, _, _, reports, _ = first
    assert global_count == len(top_level)
    assert total_count > global_count
    assert len(reports) == len(functions)
    assert source_storage
    return len(functions), global_count, total_count


def assert_region_value_namespaces(library):
    data = b"fn names ['r] (r: own u64) -> own unit pure { return unit; }"
    source_storage, source, token_storage, tokens, columns, ast = parsed(
        library, data
    )
    function = functions_of(columns, ast)[0]
    regions, parameters = declared_symbols(columns, function)
    storage, physical, symbols = empty_symbols(library, (2, 2, 2, 2))
    report = invoke(library, source, tokens, ast, function, symbols)
    assert report.status == SEMANTIC_FUNCTION_CLEAN
    assert list(storage[0][:2]) == [SYMBOL_REGION, SYMBOL_VALUE]
    assert list(storage[1][:2]) == [function, function]
    assert list(storage[3][:2]) == regions + parameters
    assert token_bytes(data, token_storage, storage[2][0]) == b"'r"
    assert token_bytes(data, token_storage, storage[2][1]) == b"r"
    assert_guards(storage, physical)
    assert source_storage


def assert_duplicate_atomic(library, data, namespace):
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    function = functions_of(columns, ast)[0]
    regions, parameters = declared_symbols(columns, function)
    declarations = regions if namespace == SYMBOL_REGION else parameters
    assert len(declarations) == 2
    storage, physical, symbols = empty_symbols(library, (4, 4, 4, 4))
    before = column_snapshot(storage, physical)
    for _ in range(2):
        report = invoke(library, source, tokens, ast, function, symbols)
        assert (report.status, report.declaration, report.related) == (
            SEMANTIC_FUNCTION_DUPLICATE,
            declarations[1],
            declarations[0],
        )
        assert symbols.count == 0
        assert column_snapshot(storage, physical) == before
        assert_guards(storage, physical)
    assert source_storage


def assert_duplicates(library):
    assert_duplicate_atomic(
        library,
        b"fn duplicate ['r, 'r] () -> own unit pure { return unit; }",
        SYMBOL_REGION,
    )
    assert_duplicate_atomic(
        library,
        b"fn duplicate (x: own u64, x: own u64) -> own unit pure "
        b"{ return unit; }",
        SYMBOL_VALUE,
    )


def assert_existing_conflict_is_atomic(library):
    data = mixed_fixture()
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    function = functions_of(columns, ast)[0]
    _, parameters = declared_symbols(columns, function)
    conflict = parameters[1]
    storage, physical, symbols = empty_symbols(library, (8, 8, 8, 8))
    assert insert(
        library,
        source,
        tokens,
        symbols,
        SYMBOL_VALUE,
        function,
        columns[1][conflict],
        777,
    ) == (SYMBOL_CLEAN, 0)
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, function, symbols)
    assert (report.status, report.declaration, report.related) == (
        SEMANTIC_FUNCTION_DUPLICATE,
        conflict,
        777,
    )
    assert symbols.count == 1
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_asymmetric_capacity_and_retry(library):
    data = mixed_fixture()
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    function = functions_of(columns, ast)[0]
    fields = ("namespaces", "scopes", "name_tokens", "declarations")
    for short_index, field in enumerate(fields):
        capacities = [5, 5, 5, 5]
        capacities[short_index] = 4
        storage, physical, symbols = empty_symbols(library, tuple(capacities))
        before = column_snapshot(storage, physical)
        for _ in range(2):
            report = invoke(library, source, tokens, ast, function, symbols)
            assert (report.status, report.declaration, report.related) == (
                SEMANTIC_FUNCTION_CAPACITY,
                AST_NONE,
                5,
            )
            assert symbols.count == 0
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)
        short = getattr(symbols, field)
        setattr(symbols, field, Buffer(short.data, 5))
        report = invoke(library, source, tokens, ast, function, symbols)
        assert report.status == SEMANTIC_FUNCTION_CLEAN
        assert symbols.count == 5
        assert_guards(storage, physical)
    assert source_storage


def assert_failure_atomic(library, mutate, expected_status):
    data = mixed_fixture()
    source_storage, source, token_storage, tokens, columns, ast = parsed(
        library, data
    )
    function = functions_of(columns, ast)[0]
    regions, parameters = declared_symbols(columns, function)
    mutate(token_storage, tokens, columns, ast, function, regions, parameters)
    storage, physical, symbols = empty_symbols(library, (8, 8, 8, 8))
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, function, symbols)
    assert report.status == expected_status, (
        report.status,
        expected_status,
        report.declaration,
        report.related,
    )
    assert symbols.count == 0
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_hostile_inputs(library):
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: setattr(
            tokens, "kinds", Buffer(tokens.kinds.data, tokens.count - 1)
        ),
        SEMANTIC_FUNCTION_INVALID_TOKEN_TAPE,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: setattr(
            ast, "heads", Buffer(ast.heads.data, ast.count - 1)
        ),
        SEMANTIC_FUNCTION_INVALID_AST_TAPE,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: columns[0].__setitem__(
            function, AST_BLOCK
        ),
        SEMANTIC_FUNCTION_EXPECTED_FUNCTION,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: columns[0].__setitem__(
            regions[0], AST_PARAMETER
        ),
        SEMANTIC_FUNCTION_MALFORMED_FUNCTION,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: columns[4].__setitem__(
            parameters[0], ast.count
        ),
        SEMANTIC_FUNCTION_MALFORMED_FUNCTION,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast, function, regions, parameters: token_storage[0].__setitem__(
            columns[1][parameters[0]], 3
        ),
        SEMANTIC_FUNCTION_INVALID_NAME_TOKEN,
    )

    def malformed_allocation(
        token_storage, tokens, columns, ast, function, regions, parameters
    ):
        allocation = next(
            node
            for node in children_of(columns, function)
            if columns[0][node] == AST_ALLOCATES_EFFECT
        )
        storage = children_of(columns, allocation)[0]
        columns[0][storage] = AST_REGION

    assert_failure_atomic(
        library,
        malformed_allocation,
        SEMANTIC_FUNCTION_MALFORMED_FUNCTION,
    )

    data = mixed_fixture()
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    function = functions_of(columns, ast)[0]
    storage, physical, symbols = empty_symbols(library, (1, 1, 1, 1))
    symbols.count = 2
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, function, symbols)
    assert report.status == SEMANTIC_FUNCTION_INVALID_SYMBOL_TAPE
    assert symbols.count == 2
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_mixed_modes_scopes_and_lookup(library)
        assert_region_value_namespaces(library)
        assert_duplicates(library)
        assert_existing_conflict_is_atomic(library)
        assert_asymmetric_capacity_and_retry(library)
        assert_hostile_inputs(library)
        functions, globals_count, symbols_count = assert_exact_compiler_source(library)
        print(
            "semantic function symbols: mixed modes, exact scoped lookup, "
            "namespace separation, duplicates, malformed tapes/shapes, "
            "allocates effects, atomic capacity and retry; "
            f"compiler={functions} functions/{globals_count} globals/"
            f"{symbols_count} symbols deterministic"
        )


if __name__ == "__main__":
    main()
