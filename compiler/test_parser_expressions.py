#!/usr/bin/env python3
"""Exercise wfc's token-anchored compiler-body expression grammar."""

import ctypes
import re
import tempfile
from collections import Counter
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import (
    AST_NONE,
    AstTape,
    assert_head_invariant,
    lex,
    make_ast,
    make_source,
)


HERE = Path(__file__).resolve().parent


class ParserNodeResult(ctypes.Structure):
    _fields_ = [("node", ctypes.c_uint64), ("next", ctypes.c_uint64)]


def enum_ordinals(enum_name):
    text = (HERE / "src" / "ast.wf").read_text()
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
    library.parser_parse_expression.argtypes = signature
    library.parser_parse_expression.restype = ParserNodeResult
    library.parser_parse_place_v2.argtypes = signature
    library.parser_parse_place_v2.restype = ParserNodeResult
    library.parser_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
    ]
    library.parser_run.restype = None


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
    functions = {
        "expression": library.parser_parse_expression_v2,
        "legacy_expression": library.parser_parse_expression,
        "place": library.parser_parse_place_v2,
    }
    function = functions[entry]
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


def descendants_of(columns, node):
    result = []
    stack = list(reversed(children_of(columns, node)))
    while stack:
        current = stack.pop()
        result.append(current)
        stack.extend(reversed(children_of(columns, current)))
        assert len(result) < 1000000
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


def assert_partial_references(tokens, columns, ast):
    assert_head_invariant(tokens, columns, ast)
    for node in range(ast.count):
        for column in columns[4:7]:
            reference = column[node]
            assert reference == AST_NONE or reference < ast.count


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


def assert_recursive_projection_tree(library):
    data = b"index<Outer>(rows, slot).inner.value"
    _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
    assert_success_invariants(data, token_storage, tokens, columns, ast, result)
    kinds, heads, starts, ends, _, _, _ = columns
    assert ast.count == 6
    assert list(kinds[: ast.count]) == [
        AST["AstIndexPlace"],
        AST["AstTypeArgument"],
        AST["AstPlaceUse"],
        AST["AstPlaceUse"],
        AST["AstFieldPlace"],
        AST["AstFieldPlace"],
    ]
    token_text = [
        data[token_storage[1][token] : token_storage[2][token]]
        for token in heads[: ast.count]
    ]
    assert token_text == [b"index", b"Outer", b"rows", b"slot", b"inner", b"value"]
    assert children_of(columns, 0) == [1, 2, 3]
    assert children_of(columns, 4) == [0]
    assert children_of(columns, 5) == [4]
    assert result.node == 5
    assert (starts[4], ends[4]) == (0, data.index(b".value"))
    assert (starts[5], ends[5]) == (0, len(data))


def assert_constructor_tree(library):
    data = b"ParserNodeResult(node: value, next: 1_u64)"
    _, token_storage, tokens, columns, ast, result = parse_entry(library, data)
    assert_success_invariants(data, token_storage, tokens, columns, ast, result)
    assert list(columns[0][: ast.count]) == [
        AST["AstConstructor"],
        AST["AstNamedArgument"],
        AST["AstPlaceUse"],
        AST["AstNamedArgument"],
        AST["AstNumericLiteral"],
    ]
    assert children_of(columns, 0) == [1, 3]
    assert children_of(columns, 1) == [2]
    assert children_of(columns, 3) == [4]


def node_texts(data, token_storage, columns, count):
    heads = columns[1]
    _, token_starts, token_ends = token_storage
    return [
        data[token_starts[heads[node]] : token_ends[heads[node]]]
        for node in range(count)
    ]


def assert_region_argument_calls(library):
    cases = [
        (
            b"parser_word2<'s>(source: source, token: token)",
            [
                AST["AstUserCall"],
                AST["AstRegionArgument"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
            ],
            [b"parser_word2", b"'s", b"source", b"source", b"token", b"token"],
            [1, 2, 4],
        ),
        (
            b"copy_pair<'left, 'right>(left: x, right: y)",
            [
                AST["AstUserCall"],
                AST["AstRegionArgument"],
                AST["AstRegionArgument"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
            ],
            [b"copy_pair", b"'left", b"'right", b"left", b"x", b"right", b"y"],
            [1, 2, 3, 5],
        ),
        (
            b"parser_word2(source: source, token: token)",
            [
                AST["AstUserCall"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
            ],
            [b"parser_word2", b"source", b"source", b"token", b"token"],
            [1, 3],
        ),
        (
            b"arena_nex<'r>(value: x)",
            [
                AST["AstUserCall"],
                AST["AstRegionArgument"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
            ],
            [b"arena_nex", b"'r", b"value", b"x"],
            [1, 2],
        ),
        (
            b"slice_on<'r>(value: x)",
            [
                AST["AstUserCall"],
                AST["AstRegionArgument"],
                AST["AstNamedArgument"],
                AST["AstPlaceUse"],
            ],
            [b"slice_on", b"'r", b"value", b"x"],
            [1, 2],
        ),
    ]
    for entry in ("expression", "legacy_expression"):
        for data, expected_kinds, expected_texts, expected_children in cases:
            _, token_storage, tokens, columns, ast, result = parse_entry(
                library, data, entry=entry
            )
            assert_success_invariants(data, token_storage, tokens, columns, ast, result)
            assert list(columns[0][: ast.count]) == expected_kinds, (entry, data)
            assert node_texts(data, token_storage, columns, ast.count) == expected_texts
            assert children_of(columns, result.node) == expected_children
            assert (columns[2][result.node], columns[3][result.node]) == (0, len(data))
            for node, kind in enumerate(expected_kinds):
                if kind == AST["AstRegionArgument"]:
                    text = expected_texts[node]
                    assert (columns[2][node], columns[3][node]) == (
                        data.index(text),
                        data.index(text) + len(text),
                    )
                    assert token_storage[0][columns[1][node]] == 4

    for data, callee in (
        (b"arena_new<'r, u64>(value)", b"arena_new"),
        (b"slice_of<'r, u64>(value)", b"slice_of"),
    ):
        for entry in ("expression", "legacy_expression"):
            _, token_storage, tokens, columns, ast, result = parse_entry(
                library, data, entry=entry
            )
            assert_success_invariants(
                data, token_storage, tokens, columns, ast, result
            )
            assert list(columns[0][: ast.count]) == [
                AST["AstTableCall"],
                AST["AstRegionArgument"],
                AST["AstTypeArgument"],
                AST["AstPlaceUse"],
            ], (entry, data)
            assert node_texts(data, token_storage, columns, ast.count) == [
                callee,
                b"'r",
                b"u64",
                b"value",
            ]
            assert children_of(columns, result.node) == [1, 2, 3]
            assert (columns[2][1], columns[3][1]) == (
                data.index(b"'r"),
                data.index(b"'r") + 2,
            )

    data = b"ParserNodeResult<'a>(node: value, next: 1_u64)"
    _, _, _, _, ast, result = parse_entry(library, data)
    assert result.node == AST_NONE
    assert ast.status == STATUS["ParseExpectedLeftParen"]
    assert (ast.error_start, ast.error_end) == (data.index(b"<"), data.index(b"<") + 1)

    data = b"f<'a, u64>(value: x)"
    # Phase 2 retains the compiler subset's all-region user targs. Mixed user
    # targs remain outside this parser slice and must fail at that boundary.
    for entry in ("expression", "legacy_expression"):
        _, _, _, _, ast, result = parse_entry(library, data, entry=entry)
        assert result.node == AST_NONE, entry
        assert ast.status == STATUS["ParseExpectedRegion"], (entry, ast.status)
        assert (ast.error_start, ast.error_end) == (
            data.index(b"u64"),
            data.index(b"u64") + 3,
        )

    # The v2 production path also pins the type-first mixed frontier. The
    # legacy parser never supported dotless type-first table-call routing.
    data = b"f<u64, 'a>(value: x)"
    _, _, _, _, ast, result = parse_entry(library, data)
    assert result.node == AST_NONE
    assert ast.status == STATUS["ParseExpectedRightAngle"]
    assert (ast.error_start, ast.error_end) == (data.index(b","), data.index(b",") + 1)


def assert_lexer_region_migration(library):
    data = (HERE / "src" / "lexer.wf").read_bytes()
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    ast_storage, guards, ast = make_ast(tokens.count)
    ast.count = 0
    ast.root = AST_NONE
    ast.status = STATUS["ParseClean"]
    ast.error_start = 0
    ast.error_end = 0
    library.parser_run(source, ctypes.byref(tokens), ctypes.byref(ast))
    assert tuple(column[tokens.count] for column in ast_storage) == guards
    assert ast.status == STATUS["ParseClean"]
    assert ast.root != AST_NONE
    assert_head_invariant(tokens, ast_storage, ast)

    kinds = ast_storage[0]
    all_texts = node_texts(data, token_storage, ast_storage, ast.count)
    functions = {}
    for node in children_of(ast_storage, ast.root):
        if kinds[node] != AST["AstFunction"]:
            continue
        function_children = children_of(ast_storage, node)
        name_node = function_children[0]
        assert kinds[name_node] == AST["AstFunctionName"]
        name = all_texts[name_node]
        functions[name] = node

    expected = {
        b"lexer_scan_op_suffix": Counter(
            {
                b"lexer_scan_ident": 1,
                b"lexer_match3": 1,
                b"lexer_match4": 2,
                b"lexer_match6": 1,
                b"lexer_match7": 1,
            }
        ),
        b"lexer_scan_word": Counter({b"lexer_scan_op_suffix": 1}),
    }
    for function_name, expected_calls in expected.items():
        observed = Counter()
        for node in descendants_of(ast_storage, functions[function_name]):
            if kinds[node] != AST["AstUserCall"]:
                continue
            callee = all_texts[node]
            if callee not in expected_calls:
                continue
            observed[callee] += 1
            children = children_of(ast_storage, node)
            region_children = [
                child
                for child in children
                if kinds[child] == AST["AstRegionArgument"]
            ]
            assert len(region_children) == 1, (function_name, callee, children)
            assert children[0] == region_children[0]
            assert all_texts[region_children[0]] == b"'s"
            assert all(
                kinds[child] == AST["AstNamedArgument"]
                for child in children[1:]
            )
        assert observed == expected_calls, (function_name, observed, expected_calls)


def assert_global_region_migration_debt(library):
    data = compiler_source().encode("ascii")
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    ast_storage, guards, ast = make_ast(tokens.count)
    ast.count = 0
    ast.root = AST_NONE
    ast.status = STATUS["ParseClean"]
    ast.error_start = 0
    ast.error_end = 0
    library.parser_run(source, ctypes.byref(tokens), ctypes.byref(ast))
    assert tuple(column[tokens.count] for column in ast_storage) == guards
    assert ast.status == STATUS["ParseClean"]
    assert_head_invariant(tokens, ast_storage, ast)

    kinds = ast_storage[0]
    all_texts = node_texts(data, token_storage, ast_storage, ast.count)
    region_arities = {}
    for node in children_of(ast_storage, ast.root):
        if kinds[node] != AST["AstFunction"]:
            continue
        function_children = children_of(ast_storage, node)
        name_node = function_children[0]
        region_arity = 0
        for child in function_children:
            if kinds[child] == AST["AstRegionParameters"]:
                region_arity = sum(
                    kinds[region] == AST["AstRegion"]
                    for region in children_of(ast_storage, child)
                )
        region_arities[all_texts[name_node]] = region_arity

    regionful_calls = 0
    explicit_calls = 0
    omitted_calls = 0
    for node in descendants_of(ast_storage, ast.root):
        if kinds[node] != AST["AstUserCall"]:
            continue
        callee = all_texts[node]
        assert callee in region_arities, callee
        arity = region_arities[callee]
        if arity == 0:
            continue
        regionful_calls += 1
        regions = [
            child
            for child in children_of(ast_storage, node)
            if kinds[child] == AST["AstRegionArgument"]
        ]
        if regions:
            assert len(regions) == arity, (callee, arity, len(regions))
            explicit_calls += 1
        else:
            omitted_calls += 1

    census = (regionful_calls, explicit_calls, omitted_calls)
    assert census == (3466, 106, 3360), census
    return census


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
        b"Pair(left: True())": "ParseExpectedAtom",
        b"op<u64>(f(x: y))": "ParseExpectedAtom",
        b"call(value: True())": "ParseExpectedAtom",
        b"op<u64>(x,)": "ParseExpectedAtom",
        b"f(value: x,)": "ParseExpectedName",
    }
    for data, expected_status in cases.items():
        _, _, _, _, ast, result = parse_entry(library, data)
        assert result.node == AST_NONE, data
        assert ast.status == STATUS[expected_status], (data, ast.status, expected_status)

    region_cases = [
        (b"f<>(value: x)", "ParseExpectedRegion", b">"),
        (b"f<'a,>(value: x)", "ParseExpectedRegion", b">"),
        (b"f<'a 'b>(value: x)", "ParseExpectedRightAngle", b"'b"),
        (b"f<'a(value: x)", "ParseExpectedRightAngle", b"("),
        (b"f<'a> value", "ParseExpectedLeftParen", b"value"),
    ]
    for entry in ("expression", "legacy_expression"):
        for data, expected_status, error_text in region_cases:
            _, _, tokens, columns, ast, result = parse_entry(
                library, data, entry=entry
            )
            assert result.node == AST_NONE, (entry, data)
            assert ast.status == STATUS[expected_status], (
                entry,
                data,
                ast.status,
                expected_status,
            )
            error_start = data.index(error_text)
            assert (ast.error_start, ast.error_end) == (
                error_start,
                error_start + len(error_text),
            )
            assert_partial_references(tokens, columns, ast)

    region_eof_cases = [
        (b"f<'a,", "ParseExpectedRegion"),
        (b"f<'a", "ParseExpectedRightAngle"),
        (b"f<'a>", "ParseExpectedLeftParen"),
    ]
    for entry in ("expression", "legacy_expression"):
        for data, expected_status in region_eof_cases:
            _, _, tokens, columns, ast, result = parse_entry(
                library, data, entry=entry
            )
            assert result.node == AST_NONE, (entry, data)
            assert ast.status == STATUS[expected_status], (
                entry,
                data,
                ast.status,
                expected_status,
            )
            assert (ast.error_start, ast.error_end) == (len(data), len(data))
            assert_partial_references(tokens, columns, ast)

    table_region_cases = [
        (b"arena_new<'r u64>(value)", "ParseExpectedRightAngle", b"u64"),
        (b"arena_new<'r,>(value)", "ParseExpectedType", b">"),
        (b"arena_new<'r, u64(value)", "ParseExpectedRightAngle", b"("),
    ]
    for entry in ("expression", "legacy_expression"):
        for data, expected_status, error_text in table_region_cases:
            _, _, tokens, columns, ast, result = parse_entry(
                library, data, entry=entry
            )
            assert result.node == AST_NONE, (entry, data)
            assert ast.status == STATUS[expected_status], (
                entry,
                data,
                ast.status,
                expected_status,
            )
            error_start = data.index(error_text)
            assert (ast.error_start, ast.error_end) == (
                error_start,
                error_start + len(error_text),
            )
            assert_partial_references(tokens, columns, ast)

    data = b"arena_new<'r,"
    for entry in ("expression", "legacy_expression"):
        _, _, tokens, columns, ast, result = parse_entry(
            library, data, entry=entry
        )
        assert result.node == AST_NONE, entry
        assert ast.status == STATUS["ParseExpectedType"], (entry, ast.status)
        assert (ast.error_start, ast.error_end) == (len(data), len(data))
        assert_partial_references(tokens, columns, ast)

    nested = b"outer(value: inner<'a>(value: x))"
    for entry in ("expression", "legacy_expression"):
        _, _, _, _, ast, result = parse_entry(library, nested, entry=entry)
        assert result.node == AST_NONE
        assert ast.status == STATUS["ParseExpectedAtom"], (entry, ast.status)
        nested_start = nested.index(b"inner")
        assert (ast.error_start, ast.error_end) == (
            nested_start,
            nested_start + len(b"inner"),
        )


def assert_capacity_guard(library):
    data = b"&uniq 'a index<u64>(deref(out).kinds, slot)"
    _, _, _, _, ast, result = parse_entry(library, data, capacity=7)
    assert result.node == AST_NONE
    assert ast.status == STATUS["ParseCapacity"]
    assert ast.count == 7

    data = b"copy_pair<'left, 'right>(left: x, right: y)"
    for entry in ("expression", "legacy_expression"):
        _, _, _, _, full, full_result = parse_entry(library, data, entry=entry)
        assert full.status == STATUS["ParseClean"]
        assert full_result.node != AST_NONE

        _, token_storage, tokens, columns, exact, exact_result = parse_entry(
            library, data, entry=entry, capacity=full.count
        )
        assert_success_invariants(
            data, token_storage, tokens, columns, exact, exact_result
        )
        assert exact.count == full.count

        short_capacity = full.count - 1
        _, _, short_tokens, short_columns, short, short_result = parse_entry(
            library, data, entry=entry, capacity=short_capacity
        )
        assert short_result.node == AST_NONE
        assert short.status == STATUS["ParseCapacity"]
        assert short.count == short_capacity
        assert_head_invariant(short_tokens, short_columns, short)
        assert (short.error_start, short.error_end) == (
            data.rindex(b"y"),
            data.rindex(b"y") + 1,
        )
        for node in range(short.count):
            for column in short_columns[4:7]:
                reference = column[node]
                assert reference == AST_NONE or reference < short.count

    data = b"f<'a>()"
    for entry in ("expression", "legacy_expression"):
        _, _, _, _, ast, result = parse_entry(
            library, data, entry=entry, capacity=1
        )
        assert result.node == AST_NONE
        assert ast.status == STATUS["ParseCapacity"]
        assert ast.count == 1

    data = b"arena_new<'r, u64>(value)"
    expected_failure_texts = [b"arena_new", b"'r", b"u64", b"value"]
    for entry in ("expression", "legacy_expression"):
        _, _, _, _, full, full_result = parse_entry(library, data, entry=entry)
        assert full.status == STATUS["ParseClean"]
        assert full_result.node != AST_NONE
        assert full.count == len(expected_failure_texts)

        _, exact_token_storage, exact_tokens, exact_columns, exact, exact_result = (
            parse_entry(library, data, entry=entry, capacity=full.count)
        )
        assert_success_invariants(
            data,
            exact_token_storage,
            exact_tokens,
            exact_columns,
            exact,
            exact_result,
        )

        for short_capacity, error_text in enumerate(expected_failure_texts):
            _, _, short_tokens, short_columns, short, short_result = parse_entry(
                library, data, entry=entry, capacity=short_capacity
            )
            assert short_result.node == AST_NONE, (entry, short_capacity)
            assert short.status == STATUS["ParseCapacity"], (
                entry,
                short_capacity,
                short.status,
            )
            assert short.count == short_capacity
            assert_head_invariant(short_tokens, short_columns, short)
            error_start = data.index(error_text)
            assert (short.error_start, short.error_end) == (
                error_start,
                error_start + len(error_text),
            )
            for node in range(short.count):
                for column in short_columns[4:7]:
                    reference = column[node]
                    assert reference == AST_NONE or reference < short.count


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_fixture_cases(library)
        assert_place_tree(library)
        assert_recursive_projection_tree(library)
        assert_constructor_tree(library)
        assert_region_argument_calls(library)
        assert_lexer_region_migration(library)
        region_census = assert_global_region_migration_debt(library)
        assert_move_tree(library)
        assert_settable_places(library)
        assert_malformed(library)
        assert_capacity_guard(library)
    print(
        "compiler expression parser: places, borrows, explicit-region calls, "
        "malformed shapes, and capacity pass; AST-resolved compiler census "
        f"{region_census[0]} regionful = {region_census[1]} explicit + "
        f"{region_census[2]} staged omissions"
    )


if __name__ == "__main__":
    main()
