#!/usr/bin/env python3
"""Exercise xlc's token-anchored compiler-body expression grammar."""

import ctypes
import re
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library
from test_parser import AST_NONE, AstTape, lex, make_ast, make_source


HERE = Path(__file__).resolve().parent


class ParserNodeResult(ctypes.Structure):
    _fields_ = [("node", ctypes.c_uint64), ("next", ctypes.c_uint64)]


def enum_ordinals(enum_name):
    text = (HERE / "src" / "ast.xl").read_text()
    body = re.search(rf"enum {enum_name} \{{(.*?)\n\}}", text, re.S).group(1)
    return {
        name: ordinal
        for ordinal, name in enumerate(re.findall(r"\b([A-Z][A-Za-z0-9_]*)\(\);", body))
    }


AST = enum_ordinals("AstKind")
STATUS = enum_ordinals("ParseStatus")


def configure(library):
    signature = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.parser_parse_expression_v2.argtypes = signature
    library.parser_parse_expression_v2.restype = ParserNodeResult
    library.parser_parse_place_v2.argtypes = signature
    library.parser_parse_place_v2.restype = ParserNodeResult


def parse_entry(library, data, entry="expression", capacity=None):
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    if capacity is None:
        capacity = tokens.count
    ast_storage, guards, ast = make_ast(capacity)
    ast.count = 0
    ast.root = AST_NONE
    ast.status = STATUS["ParseClean"]
    ast.error_start = 0
    ast.error_end = 0
    function = (
        library.parser_parse_expression_v2
        if entry == "expression"
        else library.parser_parse_place_v2
    )
    result = function(source, ctypes.byref(tokens), ctypes.byref(ast), 0)
    observed_guards = tuple(column[capacity] for column in ast_storage)
    assert observed_guards == guards
    return source_storage, token_storage, tokens, ast_storage, ast, result


def children_of(columns, node):
    first = columns[4]
    next_sibling = columns[6]
    result = []
    child = first[node]
    while child != AST_NONE:
        result.append(child)
        child = next_sibling[child]
        assert len(result) < 1000
    return result


def assert_success_invariants(data, token_storage, tokens, columns, ast, result):
    kinds, heads, starts, ends, _, _, _ = columns
    _, token_starts, token_ends = token_storage
    assert ast.status == STATUS["ParseClean"]
    assert result.node != AST_NONE
    assert result.next == tokens.count - 1
    assert ast.count <= tokens.count
    observed_heads = list(heads[: ast.count])
    assert len(observed_heads) == len(set(observed_heads))
    for node in range(ast.count):
        head = heads[node]
        assert head < tokens.count
        assert starts[node] <= token_starts[head]
        assert token_ends[head] <= ends[node] <= len(data)
        assert 0 <= kinds[node] < len(AST)


def fixture_cases():
    path = HERE / "examples" / "expression_slice.cases"
    for line in path.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        source, expected_kind = line.split("\t")
        yield source.encode("ascii"), expected_kind


def assert_fixture_cases(library):
    for data, expected_kind in fixture_cases():
        _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
        assert_success_invariants(data, token_storage, tokens, columns, ast, result)
        assert columns[0][result.node] == AST[expected_kind], (data, expected_kind)


def assert_place_tree(library):
    data = b"&uniq 'a index<u64>(deref(out).kinds, slot)"
    _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
    assert_success_invariants(data, token_storage, tokens, columns, ast, result)
    kinds, heads, starts, ends, _, _, _ = columns
    assert ast.count == 8
    assert list(kinds[: ast.count]) == [
        AST["AstUniqBorrow"],
        AST["AstRegion"],
        AST["AstIndexPlace"],
        AST["AstTypeArgument"],
        AST["AstDerefPlace"],
        AST["AstPlaceUse"],
        AST["AstFieldPlace"],
        AST["AstPlaceUse"],
    ]
    token_text = [
        data[token_storage[1][token] : token_storage[2][token]]
        for token in heads[: ast.count]
    ]
    assert token_text == [
        b"&uniq",
        b"'a",
        b"index",
        b"u64",
        b"deref",
        b"out",
        b"kinds",
        b"slot",
    ]
    assert children_of(columns, 0) == [1, 2]
    assert children_of(columns, 2) == [3, 6, 7]
    assert children_of(columns, 6) == [4]
    assert children_of(columns, 4) == [5]
    assert (starts[0], ends[0]) == (0, len(data))
    deref_start = data.index(b"deref")
    deref_end = data.index(b")", deref_start) + 1
    field_end = data.index(b"kinds") + len(b"kinds")
    assert (starts[4], ends[4]) == (deref_start, deref_end)
    assert (starts[6], ends[6]) == (deref_start, field_end)


def assert_constructor_tree(library):
    data = b"ParserNodeResult(node: value, next: True())"
    _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
    assert_success_invariants(data, token_storage, tokens, columns, ast, result)
    assert list(columns[0][: ast.count]) == [
        AST["AstConstructor"],
        AST["AstNamedArgument"],
        AST["AstPlaceUse"],
        AST["AstNamedArgument"],
        AST["AstConstructor"],
    ]
    assert children_of(columns, 0) == [1, 3]
    assert children_of(columns, 1) == [2]
    assert children_of(columns, 3) == [4]


def assert_move_tree(library):
    data = b"ParserNodeResult(node: move value, next: 1_u64)"
    _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
    assert_success_invariants(data, token_storage, tokens, columns, ast, result)
    assert list(columns[0][: ast.count]) == [
        AST["AstConstructor"],
        AST["AstNamedArgument"],
        AST["AstMove"],
        AST["AstPlaceUse"],
        AST["AstNamedArgument"],
        AST["AstNumericLiteral"],
    ]
    assert children_of(columns, 0) == [1, 4]
    assert children_of(columns, 1) == [2]
    assert children_of(columns, 2) == [3]
    assert data[columns[2][2] : columns[3][2]] == b"move value"


def assert_settable_places(library):
    good = [
        b"value",
        b"source.length",
        b"deref(out).count",
        b"index<u64>(deref(out).heads, slot)",
    ]
    for data in good:
        _, token_storage, tokens, columns, ast, result = parse_entry(
            library, data, entry="place"
        )
        assert_success_invariants(data, token_storage, tokens, columns, ast, result)

    for data in (
        b"unit",
        b"1_u64",
        b"move value",
        b"call(value: x)",
        b"len<u64>(x)",
    ):
        _, _, _, _, ast, result = parse_entry(library, data, entry="place")
        assert result.node == AST_NONE
        assert ast.status == STATUS["ParseExpectedPlace"]


def assert_malformed(library):
    cases = {
        b"deref(x": "ParseExpectedRightParen",
        b"deref(1_u64)": "ParseExpectedPlace",
        b"x.": "ParseExpectedName",
        b"index<u64>(x,)": "ParseExpectedAtom",
        b"index<u64>(x, y,)": "ParseExpectedRightParen",
        b"index<u64>(x y)": "ParseExpectedCommaOrRightParen",
        b"&'r": "ParseExpectedPlace",
        b"& x": "ParseExpectedRegion",
        b"move 1_u64": "ParseExpectedPlace",
        b"move call(value: x)": "ParseExpectedPlace",
        b"True(": "ParseExpectedName",
        b"Pair(left x)": "ParseExpectedColon",
        b"Pair(left: f(x: y))": "ParseExpectedAtom",
        b"op<u64>(f(x: y))": "ParseExpectedAtom",
        b"op<u64>(x,)": "ParseExpectedAtom",
        b"f(value: x,)": "ParseExpectedName",
    }
    for data, expected_status in cases.items():
        _, _, _, _, ast, result = parse_entry(library, data)
        assert result.node == AST_NONE, data
        assert ast.status == STATUS[expected_status], (data, ast.status, expected_status)


def assert_capacity_guard(library):
    data = b"&uniq 'a index<u64>(deref(out).kinds, slot)"
    _, _, _, _, ast, result = parse_entry(library, data, capacity=7)
    assert result.node == AST_NONE
    assert ast.status == STATUS["ParseCapacity"]
    assert ast.count == 7


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_fixture_cases(library)
        assert_place_tree(library)
        assert_constructor_tree(library)
        assert_move_tree(library)
        assert_settable_places(library)
        assert_malformed(library)
        assert_capacity_guard(library)
    print("compiler expression parser: places, borrows, constructors, calls, malformed shapes, and capacity pass")


if __name__ == "__main__":
    main()
