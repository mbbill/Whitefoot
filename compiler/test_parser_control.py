#!/usr/bin/env python3
"""Exercise xlc's recursive compiler-body statement and control grammar."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library
from test_parser import (
    AST_NONE,
    AstTape,
    assert_head_invariant,
    children_of,
    lex,
    make_ast,
    make_source,
    span,
)
from test_parser_expressions import enum_ordinals


AST = enum_ordinals("AstKind")
STATUS = enum_ordinals("ParseStatus")


def parse_control(library, data, capacity=None):
    source_storage, source = make_source(data)
    token_storage, tokens = lex(library, source)
    if capacity is None:
        capacity = tokens.count
    ast_storage, guards, ast = make_ast(capacity)
    library.parser_control_run(source, ctypes.byref(tokens), ctypes.byref(ast))
    observed_guards = tuple(column[capacity] for column in ast_storage)
    assert observed_guards == guards
    return source_storage, token_storage, tokens, ast_storage, ast


def token_text(data, token_storage, token):
    _, starts, ends = token_storage
    return data[starts[token] : ends[token]]


def nodes_of(columns, ast, kind):
    return [node for node in range(ast.count) if columns[0][node] == kind]


def only_node(columns, ast, kind):
    nodes = nodes_of(columns, ast, kind)
    assert len(nodes) == 1, (kind, nodes)
    return nodes[0]


def control_fixture():
    return (
        b"{\n"
        b"  let slot: own u64 = 1_u64;\n"
        b"  set slot = iadd.trap<u64>(slot, 1_u64);\n"
        b"  consume(value: slot);\n"
        b"  loop @again {\n"
        b"    match ieq<u64>(slot, 0_u64) {\n"
        b"      True() => {\n"
        b"        break @again;\n"
        b"      }\n"
        b"      False() => {\n"
        b"        region 'write {\n"
        b"          set deref(out).count = slot;\n"
        b"        }\n"
        b"      }\n"
        b"    }\n"
        b"  }\n"
        b"  return unit;\n"
        b"}\n"
    )


def assert_control_tree(library):
    data = control_fixture()
    _, token_storage, tokens, columns, ast = parse_control(library, data)
    kinds, heads, starts, ends, _, _, _ = columns
    _, token_starts, token_ends = token_storage
    assert ast.status == STATUS["ParseClean"]
    assert ast.root != AST_NONE
    assert kinds[ast.root] == AST["AstBlock"]
    assert (starts[ast.root], ends[ast.root]) == (0, data.rfind(b"}") + 1)
    assert_head_invariant(tokens, columns, ast)
    for node in range(ast.count):
        head = heads[node]
        assert starts[node] <= token_starts[head]
        assert token_ends[head] <= ends[node] <= len(data)

    assert len(nodes_of(columns, ast, AST["AstBlock"])) == 5
    assert len(nodes_of(columns, ast, AST["AstSet"])) == 2
    assert len(nodes_of(columns, ast, AST["AstMatchArm"])) == 2
    for kind in (
        "AstLet",
        "AstExpressionStatement",
        "AstLoop",
        "AstLoopLabel",
        "AstBreak",
        "AstBreakLabel",
        "AstRegionBlock",
        "AstMatch",
        "AstReturn",
    ):
        assert len(nodes_of(columns, ast, AST[kind])) == 1, kind

    root_children = children_of(columns, ast.root)
    assert [kinds[node] for node in root_children] == [
        AST["AstLet"],
        AST["AstSet"],
        AST["AstExpressionStatement"],
        AST["AstLoop"],
        AST["AstReturn"],
    ]

    loop_node = only_node(columns, ast, AST["AstLoop"])
    loop_children = children_of(columns, loop_node)
    assert [kinds[node] for node in loop_children] == [
        AST["AstLoopLabel"],
        AST["AstBlock"],
    ]
    assert token_text(data, token_storage, heads[loop_children[0]]) == b"@again"
    loop_block = loop_children[1]
    assert [kinds[node] for node in children_of(columns, loop_block)] == [
        AST["AstMatch"]
    ]

    match_node = only_node(columns, ast, AST["AstMatch"])
    match_children = children_of(columns, match_node)
    assert [kinds[node] for node in match_children] == [
        AST["AstTableCall"],
        AST["AstMatchArm"],
        AST["AstMatchArm"],
    ]
    arm_names = [
        token_text(data, token_storage, heads[node]) for node in match_children[1:]
    ]
    assert arm_names == [b"True", b"False"]
    for arm in match_children[1:]:
        arm_children = children_of(columns, arm)
        assert len(arm_children) == 1
        assert kinds[arm_children[0]] == AST["AstBlock"]

    expression_statement = only_node(
        columns, ast, AST["AstExpressionStatement"]
    )
    assert token_text(data, token_storage, heads[expression_statement]) == b";"
    assert data[starts[expression_statement] : ends[expression_statement]] == (
        b"consume(value: slot);"
    )

    region_node = only_node(columns, ast, AST["AstRegionBlock"])
    region_children = children_of(columns, region_node)
    assert [kinds[node] for node in region_children] == [
        AST["AstRegion"],
        AST["AstBlock"],
    ]
    assert token_text(data, token_storage, heads[region_children[0]]) == b"'write"

    break_node = only_node(columns, ast, AST["AstBreak"])
    break_children = children_of(columns, break_node)
    assert len(break_children) == 1
    assert kinds[break_children[0]] == AST["AstBreakLabel"]
    assert token_text(data, token_storage, heads[break_children[0]]) == b"@again"


def assert_empty_block(library):
    data = b"{}"
    _, _, tokens, columns, ast = parse_control(library, data)
    assert ast.status == STATUS["ParseClean"]
    assert ast.count == 1
    assert ast.root == 0
    assert children_of(columns, ast.root) == []
    assert_head_invariant(tokens, columns, ast)


def assert_expression_scrutinee(library):
    data = b"{ match call(value: x) { True() => {} } }"
    _, _, tokens, columns, ast = parse_control(library, data)
    assert ast.status == STATUS["ParseClean"]
    match_node = only_node(columns, ast, AST["AstMatch"])
    match_children = children_of(columns, match_node)
    assert columns[0][match_children[0]] == AST["AstUserCall"]
    assert columns[0][match_children[1]] == AST["AstMatchArm"]
    assert_head_invariant(tokens, columns, ast)


def assert_failures(library):
    cases = []
    data = b"{ loop {} }"
    cases.append((data, "ParseExpectedLabel", span(data, b"{" , 2)))
    data = b"{ break @done }"
    cases.append((data, "ParseExpectedSemicolon", span(data, b"}")))
    data = b"{ region name {} }"
    cases.append((data, "ParseExpectedRegion", span(data, b"name")))
    data = b"{ match value {} }"
    cases.append((data, "ParseExpectedTypeName", span(data, b"}")))
    data = b"{ match value { () => {} } }"
    cases.append((data, "ParseExpectedTypeName", span(data, b"(")))
    data = b"{ match value { True => {} } }"
    cases.append((data, "ParseExpectedLeftParen", span(data, b"=>")))
    data = b"{ match value { True( => {} } }"
    cases.append((data, "ParseExpectedRightParen", span(data, b"=>")))
    data = b"{ match value { True() {} } }"
    cases.append((data, "ParseExpectedFatArrow", span(data, b"{", 15)))
    data = b"{ match value { True() => {}, } }"
    cases.append((data, "ParseExpectedTypeName", span(data, b",")))
    data = b"{ set = value; }"
    cases.append((data, "ParseExpectedPlace", span(data, b"=")))
    data = b"{ set value other; }"
    cases.append((data, "ParseExpectedEquals", span(data, b"other")))
    data = b"{ value }"
    cases.append((data, "ParseExpectedSemicolon", span(data, b"}")))
    data = b"{ loop @again {"
    cases.append((data, "ParseExpectedRightBrace", (len(data), len(data))))

    for data, expected_status, expected_span in cases:
        _, _, tokens, columns, ast = parse_control(library, data)
        assert ast.status == STATUS[expected_status], (
            data,
            ast.status,
            STATUS[expected_status],
        )
        assert (ast.error_start, ast.error_end) == expected_span, data
        assert_head_invariant(tokens, columns, ast)

    trailing = b"{} junk"
    _, _, _, _, ast = parse_control(library, trailing)
    assert ast.status == STATUS["ParseTrailingToken"]
    assert (ast.error_start, ast.error_end) == span(trailing, b"junk")


def assert_capacity(library):
    data = control_fixture()
    _, _, _, _, full = parse_control(library, data)
    assert full.status == STATUS["ParseClean"]
    assert full.count > 1
    _, _, tokens, columns, short = parse_control(
        library, data, capacity=full.count - 1
    )
    assert short.status == STATUS["ParseCapacity"]
    assert short.count == full.count - 1
    assert_head_invariant(tokens, columns, short)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        library.parser_control_run.argtypes = [
            Buffer,
            ctypes.POINTER(TokenTape),
            ctypes.POINTER(AstTape),
        ]
        library.parser_control_run.restype = None
        assert_control_tree(library)
        assert_empty_block(library)
        assert_expression_scrutinee(library)
        assert_failures(library)
        assert_capacity(library)
    print(
        "compiler control parser: recursive blocks, let/return/set, expression "
        "statements, loops, regions, matches, diagnostics, and capacity pass"
    )


if __name__ == "__main__":
    main()
