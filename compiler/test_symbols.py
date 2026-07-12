#!/usr/bin/env python3
"""Exercise the exact-name, scoped, fixed-capacity symbol primitive."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library


TOK_WORD = 1
TOK_TYPE_ID = 3
TOK_REGION_ID = 4
TOK_LABEL = 5
LEX_CLEAN = 0

SYMBOL_VALUE = 0
SYMBOL_TYPE = 1
SYMBOL_REGION = 2
SYMBOL_LABEL = 3
SYMBOL_CONSTRUCTOR = 4

SYMBOL_CLEAN = 0
SYMBOL_NOT_FOUND = 1
SYMBOL_INVALID_TAPE = 2
SYMBOL_INVALID_TOKEN = 3
SYMBOL_CAPACITY = 4
SYMBOL_DUPLICATE = 5
SYMBOL_NONE = (1 << 64) - 1

KIND_POISON = 0x55555555
KIND_GUARD = 0x66666666
U64_POISON = 0x7777777777777777
U64_GUARD = 0x8888888888888888


class SymbolTape(ctypes.Structure):
    _fields_ = [
        ("namespaces", Buffer),
        ("scopes", Buffer),
        ("name_tokens", Buffer),
        ("declarations", Buffer),
        ("count", ctypes.c_uint64),
        ("status", ctypes.c_int32),
        ("related", ctypes.c_uint64),
    ]


def lex(library, data):
    source_storage = (ctypes.c_uint8 * max(1, len(data)))(*data)
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
    capacity = len(data) + 1
    kinds = (ctypes.c_int32 * capacity)()
    starts = (ctypes.c_uint64 * capacity)()
    ends = (ctypes.c_uint64 * capacity)()
    tokens = TokenTape(
        Buffer(ctypes.cast(kinds, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(starts, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(ends, ctypes.c_void_p), capacity),
        0,
        LEX_CLEAN,
        0,
        0,
    )
    library.lexer_run(source, ctypes.byref(tokens))
    assert tokens.status == LEX_CLEAN
    return source_storage, source, (kinds, starts, ends), tokens


def make_symbols(capacities=(8, 8, 8, 8)):
    physical = max(1, *capacities)
    namespaces = (ctypes.c_int32 * (physical + 1))()
    scopes = (ctypes.c_uint64 * (physical + 1))()
    names = (ctypes.c_uint64 * (physical + 1))()
    declarations = (ctypes.c_uint64 * (physical + 1))()
    for index in range(physical):
        namespaces[index] = KIND_POISON
        scopes[index] = U64_POISON
        names[index] = U64_POISON
        declarations[index] = U64_POISON
    namespaces[physical] = KIND_GUARD
    scopes[physical] = U64_GUARD
    names[physical] = U64_GUARD
    declarations[physical] = U64_GUARD
    tape = SymbolTape(
        Buffer(ctypes.cast(namespaces, ctypes.c_void_p), capacities[0]),
        Buffer(ctypes.cast(scopes, ctypes.c_void_p), capacities[1]),
        Buffer(ctypes.cast(names, ctypes.c_void_p), capacities[2]),
        Buffer(ctypes.cast(declarations, ctypes.c_void_p), capacities[3]),
        99,
        99,
        99,
    )
    return (namespaces, scopes, names, declarations), physical, tape


def column_snapshot(storage, physical):
    return tuple(tuple(column[:physical]) for column in storage)


def assert_guards(storage, physical):
    assert storage[0][physical] == KIND_GUARD
    assert storage[1][physical] == U64_GUARD
    assert storage[2][physical] == U64_GUARD
    assert storage[3][physical] == U64_GUARD


def reset(library, tape):
    library.symbol_tape_reset(ctypes.byref(tape))


def find(library, source, tokens, tape, space, scope, name_token):
    library.symbol_find_in_scope(
        source,
        ctypes.byref(tokens),
        ctypes.byref(tape),
        space,
        scope,
        name_token,
    )
    return tape.status, tape.related


def insert(library, source, tokens, tape, space, scope, name_token, declaration):
    library.symbol_insert_unique(
        source,
        ctypes.byref(tokens),
        ctypes.byref(tape),
        space,
        scope,
        name_token,
        declaration,
    )
    return tape.status, tape.related


def assert_basic_semantics(library, source, tokens):
    storage, physical, symbols = make_symbols()
    before_reset = column_snapshot(storage, physical)
    reset(library, symbols)
    assert (symbols.count, symbols.status, symbols.related) == (
        0,
        SYMBOL_CLEAN,
        SYMBOL_NONE,
    )
    assert column_snapshot(storage, physical) == before_reset

    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 10, 0, 100) == (
        SYMBOL_CLEAN,
        0,
    )
    assert symbols.count == 1
    assert tuple(column[0] for column in storage) == (SYMBOL_VALUE, 10, 0, 100)

    # The second token has a different span but exactly the same bytes.
    assert find(library, source, tokens, symbols, SYMBOL_VALUE, 10, 1) == (
        SYMBOL_CLEAN,
        0,
    )
    assert find(library, source, tokens, symbols, SYMBOL_VALUE, 10, 5) == (
        SYMBOL_NOT_FOUND,
        SYMBOL_NONE,
    )

    before_duplicate = column_snapshot(storage, physical)
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 10, 1, 999) == (
        SYMBOL_DUPLICATE,
        0,
    )
    assert symbols.count == 1
    assert column_snapshot(storage, physical) == before_duplicate
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 10, 1, 999) == (
        SYMBOL_DUPLICATE,
        0,
    )

    # Scope and namespace are explicit parts of the key.
    cases = (
        (SYMBOL_VALUE, 11, 0, 101),
        (SYMBOL_TYPE, 10, 2, 102),
        (SYMBOL_REGION, 10, 3, 103),
        (SYMBOL_LABEL, 10, 4, 104),
        (SYMBOL_CONSTRUCTOR, 10, 2, 105),
    )
    for slot, case in enumerate(cases, 1):
        assert insert(library, source, tokens, symbols, *case) == (
            SYMBOL_CLEAN,
            slot,
        )
    assert symbols.count == 6
    for slot, (space, scope, name_token, _) in enumerate(cases, 1):
        assert find(library, source, tokens, symbols, space, scope, name_token) == (
            SYMBOL_CLEAN,
            slot,
        )

    before_invalid = column_snapshot(storage, physical)
    for space, wrong_token in (
        (SYMBOL_VALUE, 2),
        (SYMBOL_TYPE, 0),
        (SYMBOL_REGION, 4),
        (SYMBOL_LABEL, 3),
        (SYMBOL_CONSTRUCTOR, 0),
        (SYMBOL_VALUE, 6),
        (SYMBOL_VALUE, 99),
    ):
        assert find(library, source, tokens, symbols, space, 10, wrong_token) == (
            SYMBOL_INVALID_TOKEN,
            wrong_token,
        )
    assert symbols.count == 6
    assert column_snapshot(storage, physical) == before_invalid
    assert_guards(storage, physical)

    # Reset changes only the logical tape; a subsequent insert deterministically
    # overwrites slot zero.
    reset(library, symbols)
    assert symbols.count == 0
    assert find(library, source, tokens, symbols, SYMBOL_VALUE, 10, 0) == (
        SYMBOL_NOT_FOUND,
        SYMBOL_NONE,
    )
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 7, 5, 700) == (
        SYMBOL_CLEAN,
        0,
    )
    assert tuple(column[0] for column in storage) == (SYMBOL_VALUE, 7, 5, 700)
    assert_guards(storage, physical)


def assert_atomic_capacity(library, source, tokens):
    fields = ("namespaces", "scopes", "name_tokens", "declarations")
    for short_index, field in enumerate(fields):
        capacities = [1, 1, 1, 1]
        capacities[short_index] = 0
        storage, physical, symbols = make_symbols(tuple(capacities))
        reset(library, symbols)
        before = column_snapshot(storage, physical)
        for _ in range(2):
            assert insert(
                library, source, tokens, symbols, SYMBOL_VALUE, 1, 0, 10
            ) == (SYMBOL_CAPACITY, 0)
            assert symbols.count == 0
            assert column_snapshot(storage, physical) == before
            assert_guards(storage, physical)

        # Repair only the advertised short column and retry the same operation.
        column = getattr(symbols, field)
        setattr(symbols, field, Buffer(column.data, 1))
        assert insert(
            library, source, tokens, symbols, SYMBOL_VALUE, 1, 0, 10
        ) == (SYMBOL_CLEAN, 0)
        assert symbols.count == 1
        assert tuple(column[0] for column in storage) == (
            SYMBOL_VALUE,
            1,
            0,
            10,
        )
        assert_guards(storage, physical)

    # A duplicate is reported even when the tape has no free slot; a new key
    # reports capacity, and neither path writes a column.
    storage, physical, symbols = make_symbols((1, 1, 1, 1))
    reset(library, symbols)
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 1, 0, 10) == (
        SYMBOL_CLEAN,
        0,
    )
    full = column_snapshot(storage, physical)
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 1, 1, 99) == (
        SYMBOL_DUPLICATE,
        0,
    )
    assert column_snapshot(storage, physical) == full
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 1, 5, 99) == (
        SYMBOL_CAPACITY,
        1,
    )
    assert column_snapshot(storage, physical) == full
    assert_guards(storage, physical)


def assert_hostile_tapes(library, source, token_storage, tokens):
    # A malformed token-tape column is a tape failure, not a query failure.
    storage, physical, symbols = make_symbols((1, 1, 1, 1))
    reset(library, symbols)
    before = column_snapshot(storage, physical)
    original_kinds = (tokens.kinds.data, tokens.kinds.length)
    tokens.kinds = Buffer(tokens.kinds.data, tokens.count - 1)
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 0, 0, 1) == (
        SYMBOL_INVALID_TAPE,
        SYMBOL_NONE,
    )
    assert symbols.count == 0
    assert column_snapshot(storage, physical) == before
    tokens.kinds = Buffer(*original_kinds)

    original_status = tokens.status
    tokens.status = 7
    assert find(library, source, tokens, symbols, SYMBOL_VALUE, 0, 0) == (
        SYMBOL_INVALID_TAPE,
        SYMBOL_NONE,
    )
    tokens.status = original_status

    # A malformed queried span is an invalid token when the symbol tape is empty.
    original_end = token_storage[2][0]
    token_storage[2][0] = source.length + 1
    observed = insert(library, source, tokens, symbols, SYMBOL_VALUE, 0, 0, 1)
    assert observed == (
        SYMBOL_INVALID_TOKEN,
        0,
    ), observed
    assert column_snapshot(storage, physical) == before
    token_storage[2][0] = original_end

    # count beyond any column, or a corrupt stored token reference, makes the
    # symbol tape itself invalid and never causes a partial insert.
    symbols.count = 2
    assert insert(library, source, tokens, symbols, SYMBOL_VALUE, 0, 0, 1) == (
        SYMBOL_INVALID_TAPE,
        SYMBOL_NONE,
    )
    assert column_snapshot(storage, physical) == before

    symbols.count = 1
    storage[0][0] = SYMBOL_VALUE
    storage[1][0] = 0
    storage[2][0] = 999
    storage[3][0] = 1
    corrupt = column_snapshot(storage, physical)
    assert find(library, source, tokens, symbols, SYMBOL_VALUE, 0, 0) == (
        SYMBOL_INVALID_TAPE,
        SYMBOL_NONE,
    )
    assert column_snapshot(storage, physical) == corrupt
    assert_guards(storage, physical)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
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

        data = b"alpha alpha Alpha 'r @loop beta"
        source_storage, source, token_storage, tokens = lex(library, data)
        observed = [
            (token_storage[0][ordinal], data[token_storage[1][ordinal] : token_storage[2][ordinal]])
            for ordinal in range(tokens.count)
        ]
        assert observed[:6] == [
            (TOK_WORD, b"alpha"),
            (TOK_WORD, b"alpha"),
            (TOK_TYPE_ID, b"Alpha"),
            (TOK_REGION_ID, b"'r"),
            (TOK_LABEL, b"@loop"),
            (TOK_WORD, b"beta"),
        ]

        assert_basic_semantics(library, source, tokens)
        assert_atomic_capacity(library, source, tokens)
        assert_hostile_tapes(library, source, token_storage, tokens)

        # Keep the source allocation alive until every native call returns.
        assert source_storage
        print(
            "symbols: exact names, namespaces, scopes, duplicate slots, "
            "hostile tapes, atomic capacity, guards, reset, and retry pass"
        )


if __name__ == "__main__":
    main()
