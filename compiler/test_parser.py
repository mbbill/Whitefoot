#!/usr/bin/env python3
"""Compile and exercise xlc's first token-anchored AST parser slice."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library


AST_NONE = 18446744073709551615

AST_PROGRAM = 0
AST_FUNCTION = 1
AST_FUNCTION_NAME = 2
AST_OWN_MODE = 3
AST_RETURN_TYPE = 4
AST_PURE_EFFECT = 5
AST_BLOCK = 6
AST_RETURN = 7
AST_UNIT_LITERAL = 8
AST_PARAMETER = 9
AST_PARAMETER_TYPE = 10
AST_LET = 11
AST_BINDING_NAME = 12
AST_BINDING_TYPE = 13
AST_NUMERIC_LITERAL = 14
AST_PLACE_USE = 15
AST_TABLE_CALL = 16
AST_TYPE_ARGUMENT = 17
AST_USER_CALL = 18
AST_NAMED_ARGUMENT = 19
AST_SHARED_MODE = 20
AST_UNIQ_MODE = 21
AST_REGION = 22
AST_NESTED_TYPE = 23
AST_CONST_ARGUMENT = 24
AST_REGION_PARAMETERS = 25
AST_READS_EFFECT = 26
AST_WRITES_EFFECT = 27
AST_TRAPS_EFFECT = 28
AST_ALLOCATES_EFFECT = 55
AST_HEAP_STORAGE = 56
AST_ARENA_STORAGE = 57

PARSE_CLEAN = 0
PARSE_LEX_ERROR = 1
PARSE_EXPECTED_FUNCTION = 2
PARSE_EXPECTED_NAME = 3
PARSE_EXPECTED_RIGHT_PAREN = 5
PARSE_EXPECTED_ARROW = 6
PARSE_EXPECTED_RETURN_TYPE = 8
PARSE_EXPECTED_SEMICOLON = 13
PARSE_TRAILING_TOKEN = 15
PARSE_CAPACITY = 17
PARSE_INVALID_TOKEN_TAPE = 18
PARSE_EXPECTED_COLON = 19
PARSE_EXPECTED_COMMA_OR_RIGHT_PAREN = 20
PARSE_EXPECTED_STATEMENT = 21
PARSE_EXPECTED_EQUALS = 22
PARSE_EXPECTED_EXPRESSION = 23
PARSE_EXPECTED_ATOM = 24
PARSE_EXPECTED_LEFT_ANGLE = 25
PARSE_EXPECTED_RIGHT_ANGLE = 26
PARSE_EXPECTED_TYPE = 27
PARSE_EXPECTED_MODE = 28
PARSE_EXPECTED_REGION = 29
PARSE_EXPECTED_CONST_ARGUMENT = 30
PARSE_EXPECTED_EFFECT = 31
PARSE_EXPECTED_RIGHT_SQUARE = 32
PARSE_EXPECTED_STORAGE = 40


class AstTape(ctypes.Structure):
    _fields_ = [
        ("kinds", Buffer),
        ("heads", Buffer),
        ("starts", Buffer),
        ("ends", Buffer),
        ("first_children", Buffer),
        ("last_children", Buffer),
        ("next_siblings", Buffer),
        ("count", ctypes.c_uint64),
        ("root", ctypes.c_uint64),
        ("status", ctypes.c_int32),
        ("error_start", ctypes.c_uint64),
        ("error_end", ctypes.c_uint64),
    ]


def make_source(data):
    storage = (ctypes.c_uint8 * max(1, len(data)))()
    for index, byte in enumerate(data):
        storage[index] = byte
    return storage, Buffer(ctypes.cast(storage, ctypes.c_void_p), len(data))


def lex(library, source):
    capacity = source.length + 1
    kinds = (ctypes.c_int32 * capacity)()
    starts = (ctypes.c_uint64 * capacity)()
    ends = (ctypes.c_uint64 * capacity)()
    tape = TokenTape(
        Buffer(ctypes.cast(kinds, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(starts, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(ends, ctypes.c_void_p), capacity),
        0,
        0,
        0,
        0,
    )
    library.lexer_run(source, ctypes.byref(tape))
    return (kinds, starts, ends), tape


def make_ast(capacity):
    guard_kind = 0x5A5A5A5A
    guard_u64 = 0xD3D3D3D3D3D3D3D3
    kinds = (ctypes.c_int32 * (capacity + 1))()
    heads = (ctypes.c_uint64 * (capacity + 1))()
    starts = (ctypes.c_uint64 * (capacity + 1))()
    ends = (ctypes.c_uint64 * (capacity + 1))()
    first = (ctypes.c_uint64 * (capacity + 1))()
    last = (ctypes.c_uint64 * (capacity + 1))()
    next_ = (ctypes.c_uint64 * (capacity + 1))()
    kinds[capacity] = guard_kind
    for column in (heads, starts, ends, first, last, next_):
        column[capacity] = guard_u64
    tape = AstTape(
        Buffer(ctypes.cast(kinds, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(heads, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(starts, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(ends, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(first, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(last, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(next_, ctypes.c_void_p), capacity),
        99,
        99,
        18,
        99,
        99,
    )
    columns = (kinds, heads, starts, ends, first, last, next_)
    guards = (guard_kind,) + (guard_u64,) * 6
    return columns, guards, tape


def parse(library, data, ast_capacity=None):
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    capacity = tokens.count if ast_capacity is None else ast_capacity
    ast_storage, guards, ast = make_ast(capacity)
    library.parser_run(source, ctypes.byref(tokens), ctypes.byref(ast))
    observed_guards = tuple(column[capacity] for column in ast_storage)
    assert observed_guards == guards, (observed_guards, guards)
    return source_storage, token_storage, tokens, ast_storage, ast


def parse_mutated(library, data, mutate_tokens=None, mutate_ast=None):
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    if mutate_tokens is not None:
        mutate_tokens(token_storage, tokens)
    ast_storage, guards, ast = make_ast(source.length + 1)
    if mutate_ast is not None:
        mutate_ast(ast_storage, ast)
    library.parser_run(source, ctypes.byref(tokens), ctypes.byref(ast))
    capacity = source.length + 1
    observed_guards = tuple(column[capacity] for column in ast_storage)
    assert observed_guards == guards
    return source_storage, token_storage, tokens, ast_storage, ast


def span(data, text, start=0):
    begin = data.index(text, start)
    return begin, begin + len(text)


def assert_head_invariant(tokens, columns, ast):
    heads = columns[1]
    observed = list(heads[:ast.count])
    assert ast.count <= tokens.count
    assert len(observed) == len(set(observed))
    assert all(head < tokens.count for head in observed)


def children_of(columns, node):
    first = columns[4]
    next_ = columns[6]
    children = []
    child = first[node]
    while child != AST_NONE:
        children.append(child)
        child = next_[child]
        assert len(children) < 1000
    return children


def assert_first_tree(library):
    data = b"fn main () -> own unit pure {\n  return unit;\n}\n"
    _, _, tokens, columns, ast = parse(library, data)
    kinds, heads, starts, ends, first, last, next_ = columns
    assert ast.status == PARSE_CLEAN
    assert ast.count == 9
    assert ast.root == 0
    assert_head_invariant(tokens, columns, ast)
    assert list(kinds[:ast.count]) == [
        AST_PROGRAM,
        AST_FUNCTION,
        AST_FUNCTION_NAME,
        AST_OWN_MODE,
        AST_RETURN_TYPE,
        AST_PURE_EFFECT,
        AST_BLOCK,
        AST_RETURN,
        AST_UNIT_LITERAL,
    ]
    fn_end = data.index(b"}") + 1
    expected_spans = [
        (0, len(data)),
        (0, fn_end),
        span(data, b"main"),
        span(data, b"own"),
        span(data, b"unit"),
        span(data, b"pure"),
        (data.index(b"{"), fn_end),
        (data.index(b"return"), data.index(b";") + 1),
        span(data, b"unit", data.index(b"return")),
    ]
    assert list(zip(starts[:ast.count], ends[:ast.count])) == expected_spans
    assert list(heads[:ast.count]) == [13, 0, 1, 5, 6, 7, 8, 9, 10]
    assert (first[0], last[0]) == (1, 1)
    assert (first[1], last[1]) == (2, 6)
    assert [next_[index] for index in (2, 3, 4, 5, 6)] == [3, 4, 5, 6, AST_NONE]
    assert (first[6], last[6], first[7], last[7]) == (7, 7, 8, 8)
    assert first[8] == AST_NONE


def assert_multiple_functions(library):
    one = b"fn a () -> own unit pure { return unit; }\n"
    two = b"fn b () -> own unit pure { return unit; }\n"
    _, _, tokens, columns, ast = parse(library, one + two)
    _, _, _, _, first, last, next_ = columns
    assert ast.status == PARSE_CLEAN
    assert ast.count == 17
    assert_head_invariant(tokens, columns, ast)
    assert (first[0], last[0]) == (1, 9)
    assert next_[1] == 9
    assert next_[9] == AST_NONE


def assert_scalar_add_tree(library):
    data = (Path(__file__).resolve().parent / "examples" / "scalar_add.xl").read_bytes()
    _, token_storage, tokens, columns, ast = parse(library, data)
    kinds, heads, starts, ends, _, _, _ = columns
    _, token_starts, token_ends = token_storage
    assert ast.status == PARSE_CLEAN
    assert ast.count == 35
    assert_head_invariant(tokens, columns, ast)
    for node in range(ast.count):
        token = heads[node]
        assert starts[node] <= token_starts[token]
        assert token_ends[token] <= ends[node]
    assert list(kinds[:ast.count]) == [
        AST_PROGRAM,
        AST_FUNCTION,
        AST_FUNCTION_NAME,
        AST_PARAMETER,
        AST_OWN_MODE,
        AST_PARAMETER_TYPE,
        AST_PARAMETER,
        AST_OWN_MODE,
        AST_PARAMETER_TYPE,
        AST_OWN_MODE,
        AST_RETURN_TYPE,
        AST_PURE_EFFECT,
        AST_BLOCK,
        AST_LET,
        AST_BINDING_NAME,
        AST_OWN_MODE,
        AST_BINDING_TYPE,
        AST_TABLE_CALL,
        AST_TYPE_ARGUMENT,
        AST_PLACE_USE,
        AST_PLACE_USE,
        AST_RETURN,
        AST_PLACE_USE,
        AST_FUNCTION,
        AST_FUNCTION_NAME,
        AST_OWN_MODE,
        AST_RETURN_TYPE,
        AST_PURE_EFFECT,
        AST_BLOCK,
        AST_RETURN,
        AST_USER_CALL,
        AST_NAMED_ARGUMENT,
        AST_NUMERIC_LITERAL,
        AST_NAMED_ARGUMENT,
        AST_NUMERIC_LITERAL,
    ]
    assert children_of(columns, 0) == [1, 23]
    assert children_of(columns, 1) == [2, 3, 6, 9, 10, 11, 12]
    assert children_of(columns, 3) == [4, 5]
    assert children_of(columns, 6) == [7, 8]
    assert children_of(columns, 12) == [13, 21]
    assert children_of(columns, 13) == [14, 15, 16, 17]
    assert children_of(columns, 17) == [18, 19, 20]
    assert children_of(columns, 21) == [22]
    assert children_of(columns, 29) == [30]
    assert children_of(columns, 30) == [31, 33]
    assert children_of(columns, 31) == [32]
    assert children_of(columns, 33) == [34]
    expected_spans = {
        3: b"x: own i32",
        6: b"y: own i32",
        13: b"let sum: own i32 = iadd.wrap<i32>(x, y);",
        17: b"iadd.wrap<i32>(x, y)",
        21: b"return sum;",
        30: b"add(x: 40_i32, y: 2_i32)",
        31: b"x: 40_i32",
        33: b"y: 2_i32",
    }
    for node, expected in expected_spans.items():
        assert data[starts[node]:ends[node]] == expected
    expected_heads = {
        0: b"",
        1: b"fn",
        3: b"x",
        12: b"{",
        13: b"let",
        17: b"iadd.wrap",
        21: b"return",
        30: b"add",
        31: b"x",
        32: b"40_i32",
    }
    for node, expected in expected_heads.items():
        token = heads[node]
        assert data[token_starts[token]:token_ends[token]] == expected


def assert_signature_tree(library):
    data = (Path(__file__).resolve().parent / "examples" / "signature_slice.xl").read_bytes()
    _, token_storage, tokens, columns, ast = parse(library, data)
    kinds, heads, starts, ends, _, _, _ = columns
    _, token_starts, token_ends = token_storage
    assert ast.status == PARSE_CLEAN
    assert ast.count == 50
    assert_head_invariant(tokens, columns, ast)
    for node in range(ast.count):
        token = heads[node]
        assert starts[node] <= token_starts[token]
        assert token_ends[token] <= ends[node]
    assert children_of(columns, 0) == [1, 34]
    assert children_of(columns, 1) == [2, 3, 6, 11, 15, 18, 23, 24, 25, 28, 30, 31]
    assert children_of(columns, 3) == [4, 5]
    assert children_of(columns, 6) == [7, 9]
    assert children_of(columns, 7) == [8]
    assert children_of(columns, 9) == [10]
    assert children_of(columns, 11) == [12, 14]
    assert children_of(columns, 12) == [13]
    assert children_of(columns, 18) == [19, 20]
    assert children_of(columns, 20) == [21, 22]
    assert children_of(columns, 25) == [26, 27]
    assert children_of(columns, 28) == [29]
    assert children_of(columns, 34) == [35, 36, 38, 42, 44, 45, 47]
    assert kinds[3] == AST_REGION_PARAMETERS
    assert kinds[7] == AST_SHARED_MODE
    assert kinds[12] == AST_UNIQ_MODE
    assert kinds[20] == AST_PARAMETER_TYPE
    assert kinds[21] == AST_NESTED_TYPE
    assert kinds[22] == AST_CONST_ARGUMENT
    assert kinds[25] == AST_READS_EFFECT
    assert kinds[28] == AST_WRITES_EFFECT
    assert kinds[30] == AST_TRAPS_EFFECT
    expected_spans = {
        3: b"['s, 'o]",
        6: b"source: &'s buffer<u8>",
        7: b"&'s",
        9: b"buffer<u8>",
        11: b"out: &uniq 'o TokenTape",
        12: b"&uniq 'o",
        20: b"array<u8, 10>",
        25: b"reads('s 'o)",
        28: b"writes('o)",
        30: b"traps",
        42: b"&'r",
        44: b"ByteTape",
    }
    for node, expected in expected_spans.items():
        assert data[starts[node]:ends[node]] == expected


def assert_allocation_effect_tree(library):
    data = (
        b"fn allocate ['r] () -> own unit "
        b"allocates(heap arena 'r), traps { return unit; }\n"
    )
    _, token_storage, tokens, columns, ast = parse(library, data)
    kinds, heads, starts, ends, _, _, _ = columns
    _, token_starts, token_ends = token_storage
    assert ast.status == PARSE_CLEAN
    assert_head_invariant(tokens, columns, ast)
    for node in range(ast.count):
        token = heads[node]
        assert starts[node] <= token_starts[token]
        assert token_ends[token] <= ends[node]

    allocation = next(
        node for node in range(ast.count) if kinds[node] == AST_ALLOCATES_EFFECT
    )
    heap = next(node for node in range(ast.count) if kinds[node] == AST_HEAP_STORAGE)
    arena = next(node for node in range(ast.count) if kinds[node] == AST_ARENA_STORAGE)
    traps = next(node for node in range(ast.count) if kinds[node] == AST_TRAPS_EFFECT)
    assert children_of(columns, allocation) == [heap, arena]
    arena_children = children_of(columns, arena)
    assert len(arena_children) == 1
    assert kinds[arena_children[0]] == AST_REGION
    assert data[starts[allocation]:ends[allocation]] == b"allocates(heap arena 'r)"
    assert data[starts[heap]:ends[heap]] == b"heap"
    assert data[starts[arena]:ends[arena]] == b"arena 'r"
    assert data[starts[traps]:ends[traps]] == b"traps"


def assert_failures(library):
    cases = [
        (
            b"fn main ( -> own unit pure { return unit; }",
            PARSE_EXPECTED_NAME,
            b"->",
        ),
        (
            b"fn main () own unit pure { return unit; }",
            PARSE_EXPECTED_ARROW,
            b"own",
        ),
        (
            b"fn main () -> own unit pure { return unit }",
            PARSE_EXPECTED_SEMICOLON,
            b"}",
        ),
        (
            b"fn main () -> own unit pure { return unit; } junk",
            PARSE_TRAILING_TOKEN,
            b"junk",
        ),
    ]
    for data, status, error_text in cases:
        _, _, _, _, ast = parse(library, data)
        assert ast.status == status, (data, ast.status, status)
        assert (ast.error_start, ast.error_end) == span(data, error_text)

    _, _, _, _, empty = parse(library, b"")
    assert empty.status == PARSE_EXPECTED_FUNCTION
    assert (empty.error_start, empty.error_end) == (0, 0)

    _, _, _, _, lex_error = parse(library, b"#")
    assert lex_error.status == PARSE_LEX_ERROR
    assert (lex_error.error_start, lex_error.error_end) == (0, 1)

    valid = b"fn main () -> own unit pure { return unit; }"
    _, _, tokens, _, short = parse(library, valid, ast_capacity=8)
    assert short.status == PARSE_CAPACITY
    assert short.count == 8
    assert short.count <= tokens.count


def assert_malformed_tapes(library):
    data = b"fn main () -> own unit pure { return unit; }\n"

    def overlap(storage, _tokens):
        _, starts, _ = storage
        starts[1] = 0

    def missing_eof(_storage, tokens):
        tokens.count -= 1

    def premature_eof(storage, _tokens):
        kinds, _, _ = storage
        kinds[1] = 0

    def misanchored_eof(storage, tokens):
        _, starts, ends = storage
        eof = tokens.count - 1
        starts[eof] = len(data) - 1
        ends[eof] = len(data) - 1

    def short_ends(_storage, tokens):
        tokens.ends.length = tokens.count - 1

    for mutate in (overlap, missing_eof, premature_eof, misanchored_eof, short_ends):
        _, _, _, _, ast = parse_mutated(library, data, mutate_tokens=mutate)
        assert ast.status == PARSE_INVALID_TOKEN_TAPE, (mutate.__name__, ast.status)
        assert ast.count == 0, mutate.__name__
        assert ast.root == AST_NONE, mutate.__name__

    def no_head_capacity(_storage, ast):
        ast.heads.length = 0

    _, _, _, _, ast = parse_mutated(library, data, mutate_ast=no_head_capacity)
    assert ast.status == PARSE_CAPACITY
    assert ast.count == 0
    assert ast.root == AST_NONE


def assert_scalar_failures(library):
    def function(body, params=b"x: own i32, y: own i32", result=b"i32"):
        return b"fn sample (" + params + b") -> own " + result + b" pure { " + body + b" }"

    cases = []
    data = function(b"return x;", params=b"x own i32")
    cases.append((data, PARSE_EXPECTED_COLON, span(data, b"own")))
    data = function(b"return x;", params=b"x: own thing")
    cases.append((data, PARSE_EXPECTED_TYPE, span(data, b"thing")))
    data = function(b"return x;", params=b"x: own i32 y: own i32")
    cases.append((data, PARSE_EXPECTED_COMMA_OR_RIGHT_PAREN, span(data, b"y")))
    data = function(b"return x;", params=b"x: own i32,")
    trailing_parameter = data.index(b",") + 1
    cases.append((data, PARSE_EXPECTED_NAME, (trailing_parameter, trailing_parameter + 1)))
    data = function(b"let value: own i32 1_i32; return value;")
    cases.append((data, PARSE_EXPECTED_EQUALS, span(data, b"1_i32")))
    data = function(b"return ;")
    cases.append((data, PARSE_EXPECTED_EXPRESSION, span(data, b";")))
    data = function(b"return iadd.wrap<i32(x, y);")
    missing_angle = data.index(b"i32(") + 3
    cases.append((data, PARSE_EXPECTED_RIGHT_ANGLE, (missing_angle, missing_angle + 1)))
    data = function(b"return iadd.wrap<i32>(iadd.wrap<i32>(x, y), y);")
    nested_table = data.index(b"iadd.wrap", data.index(b"iadd.wrap") + 1)
    cases.append((data, PARSE_EXPECTED_ATOM, (nested_table, nested_table + len(b"iadd.wrap"))))
    data = function(b"return iadd.wrap<i32>(x,);")
    trailing_table = data.index(b",)", data.index(b"return")) + 1
    cases.append((data, PARSE_EXPECTED_ATOM, (trailing_table, trailing_table + 1)))
    data = function(b"return add(x: add(x: x, y: y), y: y);")
    nested_user = data.index(b"add", data.index(b"add") + 1)
    cases.append((data, PARSE_EXPECTED_ATOM, (nested_user, nested_user + len(b"add"))))
    data = function(b"return add(x: x,);")
    trailing_user = data.index(b",)", data.index(b"return")) + 1
    cases.append((data, PARSE_EXPECTED_NAME, (trailing_user, trailing_user + 1)))

    for data, status, expected_error in cases:
        _, _, tokens, columns, ast = parse(library, data)
        assert ast.status == status, (data, ast.status, status)
        assert (ast.error_start, ast.error_end) == expected_error, data
        assert_head_invariant(tokens, columns, ast)


def assert_signature_failures(library):
    def function(regions=b"", params=b"value: own u8", result=b"own u8", effects=b"pure"):
        region_text = b" " + regions if regions else b""
        return b"fn sample" + region_text + b" (" + params + b") -> " + result + b" " + effects + b" { return value; }"

    def located(data, needle, marker=b""):
        start = data.index(needle, data.index(marker) if marker else 0)
        return start, start + len(needle)

    cases = []
    data = function(regions=b"[]")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b"]")))
    data = function(regions=b"['s,]")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b"]")))
    data = function(regions=b"['s")
    cases.append((data, PARSE_EXPECTED_RIGHT_SQUARE, located(data, b"(")))
    data = function(regions=b"['s 't]")
    cases.append((data, PARSE_EXPECTED_RIGHT_SQUARE, located(data, b"'t")))
    data = function(params=b"value: & u8")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b"u8")))
    data = function(params=b"value: &uniq u8")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b"u8")))
    data = function(params=b"value: own unknown")
    cases.append((data, PARSE_EXPECTED_TYPE, located(data, b"unknown")))
    data = function(params=b"value: own buffer<>")
    cases.append((data, PARSE_EXPECTED_TYPE, located(data, b">", b"buffer")))
    data = function(params=b"value: own buffer<u8,>")
    cases.append((data, PARSE_EXPECTED_RIGHT_ANGLE, located(data, b",", b"buffer")))
    data = function(params=b"value: own array<u8,>")
    cases.append((data, PARSE_EXPECTED_CONST_ARGUMENT, located(data, b">", b"array")))
    data = function(params=b"value: own array<u8, 10_u64>")
    cases.append((data, PARSE_EXPECTED_CONST_ARGUMENT, located(data, b"10_u64")))
    data = function(regions=b"['r]", effects=b"reads()")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b")", b"reads")))
    data = function(regions=b"['r]", effects=b"reads('r,)")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b",", b"reads")))
    data = function(effects=b"traps,")
    cases.append((data, PARSE_EXPECTED_EFFECT, located(data, b"{")))
    data = function(regions=b"['r]", effects=b"writes('r), reads('r)")
    cases.append((data, PARSE_EXPECTED_EFFECT, located(data, b"reads")))
    data = function(effects=b"pure,")
    cases.append((data, PARSE_EXPECTED_EFFECT, located(data, b"{")))
    data = function(effects=b"allocates()")
    cases.append((data, PARSE_EXPECTED_STORAGE, located(data, b")", b"allocates")))
    data = function(effects=b"allocates(arena)")
    cases.append((data, PARSE_EXPECTED_REGION, located(data, b")", b"allocates")))
    data = function(effects=b"allocates(other)")
    cases.append((data, PARSE_EXPECTED_STORAGE, located(data, b"other")))
    data = function(effects=b"allocates(heap,)")
    cases.append((data, PARSE_EXPECTED_STORAGE, located(data, b",", b"allocates")))
    data = function(effects=b"traps, allocates(heap)")
    cases.append((data, PARSE_EXPECTED_EFFECT, located(data, b"allocates")))
    data = function(effects=b"allocates(heap), allocates(heap)")
    second_allocate = data.rindex(b"allocates")
    cases.append(
        (data, PARSE_EXPECTED_EFFECT, (second_allocate, second_allocate + len(b"allocates")))
    )

    for data, status, expected_error in cases:
        _, _, tokens, columns, ast = parse(library, data)
        assert ast.status == status, (data, ast.status, status)
        assert (ast.error_start, ast.error_end) == expected_error, data
        assert_head_invariant(tokens, columns, ast)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        library.parser_run.argtypes = [Buffer, ctypes.POINTER(TokenTape), ctypes.POINTER(AstTape)]
        library.parser_run.restype = None
        assert_first_tree(library)
        assert_multiple_functions(library)
        assert_scalar_add_tree(library)
        assert_signature_tree(library)
        assert_allocation_effect_tree(library)
        assert_failures(library)
        assert_malformed_tapes(library)
        assert_scalar_failures(library)
        assert_signature_failures(library)
        print("self-hosted parser: token-anchored trees, exact spans, fail-closed diagnostics, and capacity guard pass")


if __name__ == "__main__":
    main()
