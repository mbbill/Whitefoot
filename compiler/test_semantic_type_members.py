#!/usr/bin/env python3
"""Exercise atomic enum-constructor and struct-field symbol indexing."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_NONE, AST_PROGRAM, AstTape, children_of, parse
from test_parser_items import (
    AST_ENUM_DECL,
    AST_FIELD,
    AST_STRUCT_DECL,
    AST_VARIANT,
)
from test_semantic_globals import (
    SEMANTIC_GLOBALS_CLEAN,
    SemanticGlobalsReport,
    configure as configure_globals,
)
from test_symbols import (
    SYMBOL_CLEAN,
    SYMBOL_CONSTRUCTOR,
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


MEMBERS_CLEAN = 0
MEMBERS_INVALID_TOKEN_TAPE = 1
MEMBERS_INVALID_AST_TAPE = 2
MEMBERS_EXPECTED_PROGRAM = 3
MEMBERS_MALFORMED_DECLARATION = 4
MEMBERS_INVALID_NAME_TOKEN = 5
MEMBERS_DUPLICATE = 6
MEMBERS_CAPACITY = 7
MEMBERS_INVALID_SYMBOL_TAPE = 8


class SemanticTypeMembersReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("declaration", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("variant_count", ctypes.c_uint64),
        ("field_count", ctypes.c_uint64),
    ]


def fixture():
    return (
        b"enum Color { Red(); Green(); }\n"
        b"struct Left { value: u64; shared: u8; }\n"
        b"struct Right { value: u64; }\n"
        b"fn inspect () -> own unit pure { return unit; }\n"
    )


def source_buffer(storage, length):
    return Buffer(ctypes.cast(storage, ctypes.c_void_p), length)


def configure(library):
    configure_globals(library)
    library.semantic_index_type_members.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(SemanticTypeMembersReport),
    ]
    library.semantic_index_type_members.restype = None


def parsed(library, data):
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    source = source_buffer(source_storage, len(data))
    return source_storage, source, token_storage, tokens, columns, ast


def index_globals(library, source, tokens, ast, capacities):
    storage, physical, symbols = make_symbols(capacities)
    reset(library, symbols)
    report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(report),
    )
    assert report.status == SEMANTIC_GLOBALS_CLEAN, (
        report.status,
        report.declaration,
        report.related,
    )
    return storage, physical, symbols


def ready(library, data, capacities=None):
    parsed_state = parsed(library, data)
    _, source, _, tokens, _, ast = parsed_state
    if capacities is None:
        capacities = (ast.count,) * 4
    symbol_state = index_globals(library, source, tokens, ast, capacities)
    return parsed_state + symbol_state


def invoke(library, source, tokens, ast, symbols):
    report = SemanticTypeMembersReport(99, 99, 99, 99, 99)
    library.semantic_index_type_members(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(report),
    )
    return report


def report_tuple(report):
    return (
        report.status,
        report.declaration,
        report.related,
        report.variant_count,
        report.field_count,
    )


def top_declarations(columns, ast):
    return children_of(columns, ast.root)


def source_members(columns, ast):
    kinds = columns[0]
    variants = []
    fields = []
    ordered = []
    structs = []
    for declaration in top_declarations(columns, ast):
        direct = children_of(columns, declaration)
        if kinds[declaration] == AST_ENUM_DECL:
            enum_variants = direct[1:]
            assert all(kinds[node] == AST_VARIANT for node in enum_variants)
            variants.extend(enum_variants)
            ordered.extend(enum_variants)
        elif kinds[declaration] == AST_STRUCT_DECL:
            struct_fields = direct[1:]
            assert all(kinds[node] == AST_FIELD for node in struct_fields)
            structs.append((declaration, struct_fields))
            fields.extend(struct_fields)
            ordered.extend(struct_fields)
    return variants, fields, ordered, structs


def token_bytes(data, token_storage, token):
    starts, ends = token_storage[1], token_storage[2]
    return data[starts[token] : ends[token]]


def assert_clean_members_and_lookup(library):
    data = fixture()
    state = ready(library, data)
    (
        source_storage,
        source,
        token_storage,
        tokens,
        columns,
        ast,
        storage,
        physical,
        symbols,
    ) = state
    variants, fields, ordered, structs = source_members(columns, ast)
    assert (len(variants), len(fields)) == (2, 3)
    base = symbols.count
    report = invoke(library, source, tokens, ast, symbols)
    assert report_tuple(report) == (
        MEMBERS_CLEAN,
        AST_NONE,
        SYMBOL_NONE,
        2,
        3,
    )
    assert symbols.count == base + 5
    assert list(storage[0][base : base + 5]) == [
        SYMBOL_CONSTRUCTOR,
        SYMBOL_CONSTRUCTOR,
        SYMBOL_VALUE,
        SYMBOL_VALUE,
        SYMBOL_VALUE,
    ]
    expected_scopes = [SYMBOL_NONE, SYMBOL_NONE]
    for struct, struct_fields in structs:
        expected_scopes.extend([struct] * len(struct_fields))
    assert list(storage[1][base : base + 5]) == expected_scopes
    assert list(storage[2][base : base + 5]) == [columns[1][node] for node in ordered]
    assert list(storage[3][base : base + 5]) == ordered

    for offset, member in enumerate(ordered):
        slot = base + offset
        namespace = storage[0][slot]
        scope = storage[1][slot]
        assert find(
            library,
            source,
            tokens,
            symbols,
            namespace,
            scope,
            columns[1][member],
        ) == (SYMBOL_CLEAN, slot)
    assert token_bytes(data, token_storage, columns[1][fields[0]]) == b"value"
    assert token_bytes(data, token_storage, columns[1][fields[2]]) == b"value"
    assert storage[1][base + 2] != storage[1][base + 4]
    assert_guards(storage, physical)
    assert source_storage


def assert_duplicate_atomic(
    library,
    data,
    expected_counts,
    select_current,
    select_related,
):
    state = ready(library, data)
    (
        source_storage,
        source,
        _,
        tokens,
        columns,
        ast,
        storage,
        physical,
        symbols,
    ) = state
    current = select_current(columns, ast)
    related = select_related(columns, ast)
    before = column_snapshot(storage, physical)
    before_count = symbols.count
    for _ in range(2):
        report = invoke(library, source, tokens, ast, symbols)
        assert report_tuple(report) == (
            MEMBERS_DUPLICATE,
            current,
            related,
            expected_counts[0],
            expected_counts[1],
        )
        assert symbols.count == before_count
        assert column_snapshot(storage, physical) == before
        assert_guards(storage, physical)
    assert source_storage


def assert_constructor_and_member_duplicates(library):
    coexistence = (
        b"enum Marker { Marker(); }\n"
        b"struct Taken { value: u64; }\n"
        b"enum Maker { Taken(); }\n"
    )
    state = ready(library, coexistence)
    (
        coexistence_source_storage,
        coexistence_source,
        _,
        coexistence_tokens,
        coexistence_columns,
        coexistence_ast,
        coexistence_storage,
        coexistence_physical,
        coexistence_symbols,
    ) = state
    coexistence_tops = top_declarations(coexistence_columns, coexistence_ast)
    coexistence_variants = source_members(
        coexistence_columns, coexistence_ast
    )[0]
    coexistence_base = coexistence_symbols.count
    coexistence_report = invoke(
        library,
        coexistence_source,
        coexistence_tokens,
        coexistence_ast,
        coexistence_symbols,
    )
    assert report_tuple(coexistence_report) == (
        MEMBERS_CLEAN,
        AST_NONE,
        SYMBOL_NONE,
        2,
        1,
    )
    assert coexistence_symbols.count == coexistence_base + 3
    marker_name = children_of(
        coexistence_columns, coexistence_tops[0]
    )[0]
    taken_name = children_of(
        coexistence_columns, coexistence_tops[1]
    )[0]
    assert find(
        library,
        coexistence_source,
        coexistence_tokens,
        coexistence_symbols,
        SYMBOL_TYPE,
        SYMBOL_NONE,
        coexistence_columns[1][marker_name],
    ) == (SYMBOL_CLEAN, 0)
    assert find(
        library,
        coexistence_source,
        coexistence_tokens,
        coexistence_symbols,
        SYMBOL_CONSTRUCTOR,
        SYMBOL_NONE,
        coexistence_columns[1][coexistence_variants[0]],
    ) == (SYMBOL_CLEAN, coexistence_base)
    assert find(
        library,
        coexistence_source,
        coexistence_tokens,
        coexistence_symbols,
        SYMBOL_TYPE,
        SYMBOL_NONE,
        coexistence_columns[1][taken_name],
    ) == (SYMBOL_CLEAN, 1)
    assert find(
        library,
        coexistence_source,
        coexistence_tokens,
        coexistence_symbols,
        SYMBOL_CONSTRUCTOR,
        SYMBOL_NONE,
        coexistence_columns[1][coexistence_variants[1]],
    ) == (SYMBOL_CLEAN, coexistence_base + 2)
    assert_guards(coexistence_storage, coexistence_physical)
    assert coexistence_source_storage

    variants = b"enum First { Shared(); }\nenum Second { Shared(); }\n"

    def second_variant(columns, ast):
        return source_members(columns, ast)[0][1]

    def first_variant(columns, ast):
        return source_members(columns, ast)[0][0]

    assert_duplicate_atomic(
        library,
        variants,
        (1, 0),
        second_variant,
        first_variant,
    )

    fields = b"struct Bad { same: u64; same: u8; }\n"

    def second_field(columns, ast):
        return source_members(columns, ast)[1][1]

    def first_field(columns, ast):
        return source_members(columns, ast)[1][0]

    assert_duplicate_atomic(
        library,
        fields,
        (0, 1),
        second_field,
        first_field,
    )


def assert_prelude_constructor_collisions(library):
    constructors = (
        b"True",
        b"False",
        b"None",
        b"Some",
        b"Ok",
        b"Err",
        b"Overflow",
        b"DivideByZero",
        b"DivOverflow",
        b"NarrowError",
    )
    for constructor in constructors:
        data = b"enum UserKind { " + constructor + b"(); }\n"
        state = ready(library, data)
        (
            source_storage,
            source,
            _,
            tokens,
            columns,
            ast,
            storage,
            physical,
            symbols,
        ) = state
        variant = source_members(columns, ast)[0][0]
        before = column_snapshot(storage, physical)
        before_count = symbols.count
        for _ in range(2):
            report = invoke(library, source, tokens, ast, symbols)
            assert report_tuple(report) == (
                MEMBERS_DUPLICATE,
                variant,
                SYMBOL_NONE,
                0,
                0,
            ), (constructor, report_tuple(report))
            assert symbols.count == before_count
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)
        assert source_storage


def assert_preexisting_collision(library):
    data = b"enum ExternalKind { External(); }\n"
    state = ready(library, data)
    (
        source_storage,
        source,
        _,
        tokens,
        columns,
        ast,
        storage,
        physical,
        symbols,
    ) = state
    variant = source_members(columns, ast)[0][0]
    assert insert(
        library,
        source,
        tokens,
        symbols,
        SYMBOL_CONSTRUCTOR,
        SYMBOL_NONE,
        columns[1][variant],
        777,
    ) == (SYMBOL_CLEAN, 1)
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert report_tuple(report) == (
        MEMBERS_DUPLICATE,
        variant,
        777,
        0,
        0,
    )
    assert symbols.count == 2
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage

    field_data = b"struct ExternalFields { item: u64; }\n"
    field_state = ready(library, field_data)
    (
        field_source_storage,
        field_source,
        _,
        field_tokens,
        field_columns,
        field_ast,
        field_storage,
        field_physical,
        field_symbols,
    ) = field_state
    field_struct, struct_fields = source_members(field_columns, field_ast)[3][0]
    field = struct_fields[0]
    assert insert(
        library,
        field_source,
        field_tokens,
        field_symbols,
        SYMBOL_VALUE,
        field_struct,
        field_columns[1][field],
        888,
    ) == (SYMBOL_CLEAN, 1)
    field_before = column_snapshot(field_storage, field_physical)
    field_report = invoke(
        library, field_source, field_tokens, field_ast, field_symbols
    )
    assert report_tuple(field_report) == (
        MEMBERS_DUPLICATE,
        field,
        888,
        0,
        0,
    )
    assert field_symbols.count == 2
    assert column_snapshot(field_storage, field_physical) == field_before
    assert_guards(field_storage, field_physical)
    assert field_source_storage


def assert_asymmetric_capacity_and_retry(library):
    data = fixture()
    fields = ("namespaces", "scopes", "name_tokens", "declarations")
    for short_index, field in enumerate(fields):
        capacities = [9, 9, 9, 9]
        capacities[short_index] = 8
        state = ready(library, data, tuple(capacities))
        (
            source_storage,
            source,
            _,
            tokens,
            _,
            ast,
            storage,
            physical,
            symbols,
        ) = state
        assert symbols.count == 4
        before = column_snapshot(storage, physical)
        for _ in range(2):
            report = invoke(library, source, tokens, ast, symbols)
            assert report_tuple(report) == (
                MEMBERS_CAPACITY,
                AST_NONE,
                5,
                2,
                3,
            )
            assert symbols.count == 4
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)
        short = getattr(symbols, field)
        setattr(symbols, field, Buffer(short.data, 9))
        report = invoke(library, source, tokens, ast, symbols)
        assert report.status == MEMBERS_CLEAN
        assert symbols.count == 9
        assert_guards(storage, physical)
        assert source_storage


def assert_failure_atomic(library, mutate, expected_status):
    data = fixture()
    state = ready(library, data)
    (
        source_storage,
        source,
        token_storage,
        tokens,
        columns,
        ast,
        storage,
        physical,
        symbols,
    ) = state
    mutate(token_storage, tokens, columns, ast)
    before = column_snapshot(storage, physical)
    before_count = symbols.count
    report = invoke(library, source, tokens, ast, symbols)
    assert report.status == expected_status, report_tuple(report)
    assert symbols.count == before_count
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def assert_hostile_inputs(library):
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast: setattr(
            tokens, "kinds", Buffer(tokens.kinds.data, tokens.count - 1)
        ),
        MEMBERS_INVALID_TOKEN_TAPE,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast: setattr(
            ast, "heads", Buffer(ast.heads.data, ast.count - 1)
        ),
        MEMBERS_INVALID_AST_TAPE,
    )
    assert_failure_atomic(
        library,
        lambda token_storage, tokens, columns, ast: columns[0].__setitem__(
            ast.root, AST_ENUM_DECL
        ),
        MEMBERS_EXPECTED_PROGRAM,
    )

    def bad_variant_kind(token_storage, tokens, columns, ast):
        variant = source_members(columns, ast)[0][0]
        columns[0][variant] = AST_FIELD

    assert_failure_atomic(
        library,
        bad_variant_kind,
        MEMBERS_MALFORMED_DECLARATION,
    )

    def bad_variant_token(token_storage, tokens, columns, ast):
        variant = source_members(columns, ast)[0][0]
        token_storage[0][columns[1][variant]] = 1

    assert_failure_atomic(
        library,
        bad_variant_token,
        MEMBERS_INVALID_NAME_TOKEN,
    )

    def bad_field_type_ref(token_storage, tokens, columns, ast):
        field = source_members(columns, ast)[1][0]
        columns[4][field] = ast.count
        columns[5][field] = ast.count

    assert_failure_atomic(
        library,
        bad_field_type_ref,
        MEMBERS_MALFORMED_DECLARATION,
    )

    def bad_future_type_name_ref(token_storage, tokens, columns, ast):
        struct = next(
            node
            for node in top_declarations(columns, ast)
            if columns[0][node] == AST_STRUCT_DECL
        )
        columns[4][struct] = ast.count

    assert_failure_atomic(
        library,
        bad_future_type_name_ref,
        MEMBERS_MALFORMED_DECLARATION,
    )

    data = fixture()
    state = ready(library, data, (4, 4, 4, 4))
    source_storage, source, _, tokens, _, ast, storage, physical, symbols = state
    symbols.count = 5
    before = column_snapshot(storage, physical)
    report = invoke(library, source, tokens, ast, symbols)
    assert report.status == MEMBERS_INVALID_SYMBOL_TAPE
    assert symbols.count == 5
    assert column_snapshot(storage, physical) == before
    assert_guards(storage, physical)
    assert source_storage


def logical_snapshot(storage, count):
    return tuple(tuple(column[:count]) for column in storage)


def assert_compiler_source_clean_and_deterministic(library):
    data = compiler_source().encode("ascii")
    source_storage, source, _, tokens, columns, ast = parsed(library, data)
    first_storage, first_physical, first_symbols = index_globals(
        library, source, tokens, ast, (ast.count,) * 4
    )
    first_base = first_symbols.count
    first_report = invoke(library, source, tokens, ast, first_symbols)
    assert first_report.status == MEMBERS_CLEAN, report_tuple(first_report)
    assert first_report.variant_count > 0
    assert first_report.field_count > 0
    assert first_symbols.count == (
        first_base + first_report.variant_count + first_report.field_count
    )
    first_snapshot = logical_snapshot(first_storage, first_symbols.count)
    assert_guards(first_storage, first_physical)

    second_storage, second_physical, second_symbols = index_globals(
        library, source, tokens, ast, (ast.count,) * 4
    )
    second_report = invoke(library, source, tokens, ast, second_symbols)
    assert report_tuple(second_report) == report_tuple(first_report)
    assert second_symbols.count == first_symbols.count
    assert logical_snapshot(second_storage, second_symbols.count) == first_snapshot
    assert_guards(second_storage, second_physical)
    assert columns[0][ast.root] == AST_PROGRAM
    assert source_storage


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_clean_members_and_lookup(library)
        assert_constructor_and_member_duplicates(library)
        assert_prelude_constructor_collisions(library)
        assert_preexisting_collision(library)
        assert_asymmetric_capacity_and_retry(library)
        assert_hostile_inputs(library)
        assert_compiler_source_clean_and_deterministic(library)
        print(
            "semantic type members: constructors and fields, prelude reservations, "
            "generic same-spelling namespaces, malformed tapes, atomic capacity, "
            "retry, and deterministic compiler-source indexing pass"
        )


if __name__ == "__main__":
    main()
