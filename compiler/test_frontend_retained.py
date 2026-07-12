#!/usr/bin/env python3
"""Retain and reuse every front-end tape across success and failure runs."""

import ctypes
import tempfile
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_frontend import (
    FRONTEND_CLEAN,
    FRONTEND_GLOBALS,
    FRONTEND_LEX,
    FRONTEND_PARSE,
    GLOBALS_DUPLICATE,
    LEX_UNKNOWN_BYTE,
    PARSE_EXPECTED_NAME,
    FrontendReport,
    clean_counts,
    expected_counts,
    mixed_fixture,
    report_tuple,
    run_frontend,
)
from test_lexer import Buffer, TokenTape, build_library
from test_parser import AST_NONE, AstTape
from test_semantic_facts import NodeFacts, TypeTape
from test_symbols import SymbolTape


ENUM_POISON = 0x55555555
ENUM_GUARD = 0x66666666
U64_POISON = 0x7777777777777777
U64_GUARD = 0x8888888888888888
FACTS_CLEAN = 0
AST_VALIDATION_EMPTY = 4


class AnalyzedUnit(ctypes.Structure):
    _fields_ = [
        ("tokens", TokenTape),
        ("ast", AstTape),
        ("validation", AstValidationReport),
        ("symbols", SymbolTape),
        ("types", TypeTape),
        ("facts", NodeFacts),
    ]


def guarded_column(ctype, capacity, poison, guard):
    column = (ctype * (capacity + 1))()
    for index in range(capacity):
        column[index] = poison
    column[capacity] = guard
    return column, Buffer(ctypes.cast(column, ctypes.c_void_p), capacity)


def guarded_tape_columns(capacity, specs):
    storage = []
    buffers = []
    guards = []
    for ctype, poison, guard in specs:
        column, buffer = guarded_column(ctype, capacity, poison, guard)
        storage.append(column)
        buffers.append(buffer)
        guards.append((column, capacity, guard))
    return tuple(storage), tuple(buffers), tuple(guards)


def make_guarded_unit(capacity):
    enum = (ctypes.c_int32, ENUM_POISON, ENUM_GUARD)
    u64 = (ctypes.c_uint64, U64_POISON, U64_GUARD)
    all_storage = []
    all_guards = []

    token_storage, token_buffers, token_guards = guarded_tape_columns(
        capacity, (enum, u64, u64)
    )
    tokens = TokenTape(*token_buffers, 99, 99, 99, 99)
    all_storage.append(token_storage)
    all_guards.extend(token_guards)

    ast_storage, ast_buffers, ast_guards = guarded_tape_columns(
        capacity, (enum, u64, u64, u64, u64, u64, u64)
    )
    ast = AstTape(*ast_buffers, 99, 99, 99, 99, 99)
    all_storage.append(ast_storage)
    all_guards.extend(ast_guards)

    validation_storage, validation_buffers, validation_guards = (
        guarded_tape_columns(capacity, (u64, u64))
    )
    validation = AstValidationReport(*validation_buffers, 99, 99, 99)
    all_storage.append(validation_storage)
    all_guards.extend(validation_guards)

    symbol_storage, symbol_buffers, symbol_guards = guarded_tape_columns(
        capacity, (enum, u64, u64, u64)
    )
    symbols = SymbolTape(*symbol_buffers, 99, 99, 99)
    all_storage.append(symbol_storage)
    all_guards.extend(symbol_guards)

    type_storage, type_buffers, type_guards = guarded_tape_columns(
        capacity, (enum, u64, u64, u64, u64, enum)
    )
    types = TypeTape(*type_buffers, 99, 99, 99, 99)
    all_storage.append(type_storage)
    all_guards.extend(type_guards)

    fact_storage, fact_buffers, fact_guards = guarded_tape_columns(
        capacity, (u64, u64, u64, enum, u64, u64, enum, enum)
    )
    facts = NodeFacts(*fact_buffers, 99, 99, 99, 99)
    all_storage.append(fact_storage)
    all_guards.extend(fact_guards)

    unit = AnalyzedUnit(tokens, ast, validation, symbols, types, facts)
    return tuple(all_storage), tuple(all_guards), unit


def source_buffer(data):
    storage = (ctypes.c_uint8 * max(1, len(data)))(*data)
    return storage, Buffer(ctypes.cast(storage, ctypes.c_void_p), len(data))


def configure(library):
    library.frontend_unit_new.argtypes = [ctypes.c_uint64]
    library.frontend_unit_new.restype = AnalyzedUnit
    library.frontend_analyze_into.argtypes = [
        Buffer,
        ctypes.POINTER(AnalyzedUnit),
        ctypes.POINTER(FrontendReport),
    ]
    library.frontend_analyze_into.restype = None


def analyze(library, unit, data):
    source_storage, source = source_buffer(data)
    report = FrontendReport(*([0x5A] * 11))
    library.frontend_analyze_into(
        source, ctypes.byref(unit), ctypes.byref(report)
    )
    assert source_storage
    return report


def assert_guards(guards):
    for column, capacity, guard in guards:
        assert column[capacity] == guard


def buffer_lengths(unit):
    groups = (
        (unit.tokens, ("kinds", "starts", "ends")),
        (
            unit.ast,
            (
                "kinds",
                "heads",
                "starts",
                "ends",
                "first_children",
                "last_children",
                "next_siblings",
            ),
        ),
        (unit.validation, ("marks", "stack")),
        (
            unit.symbols,
            ("namespaces", "scopes", "name_tokens", "declarations"),
        ),
        (
            unit.types,
            (
                "kinds",
                "declarations",
                "element_types",
                "array_lengths",
                "source_nodes",
                "prelude_types",
            ),
        ),
        (
            unit.facts,
            (
                "type_ids",
                "resolved_declarations",
                "ordinals",
                "operations",
                "constant_values",
                "control_targets",
                "modes",
                "prelude_constructors",
            ),
        ),
    )
    return tuple(
        getattr(tape, field).length
        for tape, fields in groups
        for field in fields
    )


def assert_empty_facts(unit):
    assert (
        unit.types.count,
        unit.types.status,
        unit.types.node,
        unit.types.related,
    ) == (0, FACTS_CLEAN, AST_NONE, AST_NONE)
    assert (
        unit.facts.count,
        unit.facts.status,
        unit.facts.node,
        unit.facts.related,
    ) == (0, FACTS_CLEAN, AST_NONE, AST_NONE)


def logical_snapshot(storage, unit):
    token_storage, ast_storage, _, symbol_storage, _, _ = storage
    return (
        tuple(tuple(column[: unit.tokens.count]) for column in token_storage),
        tuple(tuple(column[: unit.ast.count]) for column in ast_storage),
        tuple(tuple(column[: unit.symbols.count]) for column in symbol_storage),
    )


def assert_allocated_unit(library, data):
    capacity = len(data) + 1
    allocated = library.frontend_unit_new(capacity)
    assert buffer_lengths(allocated) == (capacity,) * 30
    assert (
        allocated.tokens.count,
        allocated.ast.count,
        allocated.symbols.count,
        allocated.types.count,
        allocated.facts.count,
    ) == (0, 0, 0, 0, 0)
    report = analyze(library, allocated, data)
    assert report.stage == FRONTEND_CLEAN
    assert allocated.tokens.count == report.token_count
    assert allocated.ast.count == report.node_count
    assert allocated.symbols.count == report.symbol_count
    assert_empty_facts(allocated)


def assert_reuse_and_failures(library, data):
    capacity = len(data) + 1
    storage, guards, unit = make_guarded_unit(capacity)
    original_lengths = buffer_lengths(unit)
    assert original_lengths == (capacity,) * 30

    first = analyze(library, unit, data)
    expected = expected_counts(library, data)
    assert clean_counts(first) == expected
    assert unit.tokens.count == first.token_count
    assert unit.ast.count == first.node_count
    assert unit.validation.status == 0
    assert unit.symbols.count == first.symbol_count
    assert unit.symbols.status == 0
    assert unit.ast.root < unit.ast.count
    assert storage[0][0][unit.tokens.count - 1] == 0
    assert storage[1][0][unit.ast.root] == 0
    assert_empty_facts(unit)
    first_snapshot = logical_snapshot(storage, unit)
    assert_guards(guards)

    lex_error = analyze(library, unit, b"#")
    assert (
        lex_error.stage,
        lex_error.status,
        lex_error.error_start,
        lex_error.error_end,
    ) == (FRONTEND_LEX, LEX_UNKNOWN_BYTE, 0, 1)
    assert unit.tokens.count == 0
    assert (unit.ast.count, unit.ast.root, unit.ast.status) == (0, AST_NONE, 0)
    assert unit.validation.status == AST_VALIDATION_EMPTY
    assert (unit.symbols.count, unit.symbols.status) == (0, 0)
    assert_empty_facts(unit)
    assert_guards(guards)

    parse_error = analyze(library, unit, b"fn")
    assert (parse_error.stage, parse_error.status) == (
        FRONTEND_PARSE,
        PARSE_EXPECTED_NAME,
    )
    assert unit.tokens.count == 2
    assert unit.ast.count == parse_error.node_count > 0
    assert unit.validation.status == AST_VALIDATION_EMPTY
    assert unit.symbols.count == 0
    assert_empty_facts(unit)
    assert_guards(guards)

    semantic_error = analyze(library, unit, b"struct Bool { value: u64; }\n")
    assert (semantic_error.stage, semantic_error.status) == (
        FRONTEND_GLOBALS,
        GLOBALS_DUPLICATE,
    )
    assert semantic_error.related == AST_NONE
    assert unit.validation.status == 0
    assert unit.symbols.count == 0
    assert_empty_facts(unit)
    assert_guards(guards)

    second = analyze(library, unit, data)
    assert report_tuple(second) == report_tuple(first)
    assert logical_snapshot(storage, unit) == first_snapshot
    assert buffer_lengths(unit) == original_lengths
    assert_empty_facts(unit)
    assert_guards(guards)


def assert_wrapper_determinism(library, data):
    first = run_frontend(library, data)
    second = run_frontend(library, data)
    assert report_tuple(first) == report_tuple(second)


def main():
    data = mixed_fixture()
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_allocated_unit(library, data)
        assert_reuse_and_failures(library, data)
        assert_wrapper_determinism(library, data)
        print(
            "retained frontend: allocated and guarded units preserve token/AST/"
            "symbol tapes across deterministic success/failure reuse; fact tapes reset empty"
        )


if __name__ == "__main__":
    main()
