#!/usr/bin/env python3
"""Drive the owned xlang front end through its single bootstrap ABI."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, build_library, compiler_source
from test_parser import AST_NONE, children_of, parse
from test_semantic_all_types import TYPE_ROOT_KINDS


FRONTEND_CLEAN = 0
FRONTEND_LEX = 1
FRONTEND_PARSE = 2
FRONTEND_GLOBALS = 4
FRONTEND_TYPE_MEMBERS = 5
FRONTEND_TYPES = 6

LEX_UNKNOWN_BYTE = 1
PARSE_EXPECTED_NAME = 3
GLOBALS_DUPLICATE = 6
TYPE_MEMBERS_DUPLICATE = 6
TYPE_UNKNOWN_NAMED = 6

AST_FUNCTION = 1
AST_PARAMETER = 9
AST_REGION_PARAMETERS = 25
AST_VARIANT = 31
AST_FIELD = 34


class FrontendReport(ctypes.Structure):
    _fields_ = [
        ("stage", ctypes.c_int32),
        ("status", ctypes.c_uint64),
        ("error_start", ctypes.c_uint64),
        ("error_end", ctypes.c_uint64),
        ("node", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("token_count", ctypes.c_uint64),
        ("node_count", ctypes.c_uint64),
        ("type_count", ctypes.c_uint64),
        ("symbol_count", ctypes.c_uint64),
        ("function_count", ctypes.c_uint64),
    ]


def configure(library):
    library.xlc_frontend_run.argtypes = [Buffer, ctypes.POINTER(FrontendReport)]
    library.xlc_frontend_run.restype = None


def run_frontend(library, data):
    storage = (ctypes.c_uint8 * max(1, len(data)))()
    for index, byte in enumerate(data):
        storage[index] = byte
    source = Buffer(ctypes.cast(storage, ctypes.c_void_p), len(data))
    report = FrontendReport(*([0x5A] * 11))
    library.xlc_frontend_run(source, ctypes.byref(report))
    assert storage
    return report


def report_tuple(report):
    return tuple(getattr(report, field) for field, _ in report._fields_)


def expected_counts(library, data):
    _, _, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    kinds = columns[0]
    top_level = children_of(columns, ast.root)
    functions = [node for node in top_level if kinds[node] == AST_FUNCTION]
    type_count = sum(
        1 for node in range(ast.count) if kinds[node] in TYPE_ROOT_KINDS
    )
    type_member_count = sum(
        1 for node in range(ast.count) if kinds[node] in (AST_VARIANT, AST_FIELD)
    )
    local_symbol_count = 0
    for function in functions:
        direct = children_of(columns, function)
        for child in direct:
            if kinds[child] == AST_PARAMETER:
                local_symbol_count += 1
            elif kinds[child] == AST_REGION_PARAMETERS:
                local_symbol_count += len(children_of(columns, child))
    return (
        tokens.count,
        ast.count,
        type_count,
        len(top_level) + type_member_count + local_symbol_count,
        len(functions),
    )


def clean_counts(report):
    assert (
        report.stage,
        report.status,
        report.error_start,
        report.error_end,
        report.node,
        report.related,
    ) == (FRONTEND_CLEAN, 0, 0, 0, AST_NONE, AST_NONE)
    return (
        report.token_count,
        report.node_count,
        report.type_count,
        report.symbol_count,
        report.function_count,
    )


def mixed_fixture():
    return (
        b"enum Kind { Alpha(); Beta(); }\n"
        b"const width: u64 = 4_u64;\n"
        b"struct Row { code: u8; kind: Kind; }\n"
        b"fn inspect ['s] (rows: &'s buffer<Row>) -> own u64 "
        b"reads('s), traps {\n"
        b"  let count: own u64 = len<Row>(rows);\n"
        b"  return count;\n"
        b"}\n"
    )


def assert_clean_cases(library):
    cases = [
        ("mixed", mixed_fixture()),
        ("compiler", compiler_source().encode("ascii")),
    ]
    observed = {}
    for name, data in cases:
        expected = expected_counts(library, data)
        first = run_frontend(library, data)
        second = run_frontend(library, data)
        assert clean_counts(first) == expected, (name, clean_counts(first), expected)
        assert report_tuple(first) == report_tuple(second), name
        assert first.function_count > 0
        assert first.symbol_count >= first.function_count
        observed[name] = expected
    return observed


def assert_failure_routing(library):
    lex = run_frontend(library, b"#")
    assert (lex.stage, lex.status, lex.error_start, lex.error_end) == (
        FRONTEND_LEX,
        LEX_UNKNOWN_BYTE,
        0,
        1,
    )
    assert (lex.token_count, lex.node_count, lex.type_count, lex.symbol_count) == (
        0,
        0,
        0,
        0,
    )

    parse_error = run_frontend(library, b"fn")
    assert (
        parse_error.stage,
        parse_error.status,
        parse_error.error_start,
        parse_error.error_end,
    ) == (FRONTEND_PARSE, PARSE_EXPECTED_NAME, 2, 2)
    assert parse_error.token_count == 2
    assert parse_error.node_count > 0

    duplicate = run_frontend(
        library,
        b"fn same () -> own unit pure { return unit; }\n"
        b"fn same () -> own unit pure { return unit; }\n",
    )
    assert (duplicate.stage, duplicate.status) == (
        FRONTEND_GLOBALS,
        GLOBALS_DUPLICATE,
    )
    assert duplicate.node != AST_NONE
    assert duplicate.related != AST_NONE
    assert duplicate.error_end > duplicate.error_start

    prelude_type = run_frontend(
        library,
        b"struct Bool { value: u64; }\n",
    )
    assert (prelude_type.stage, prelude_type.status) == (
        FRONTEND_GLOBALS,
        GLOBALS_DUPLICATE,
    )
    assert prelude_type.node != AST_NONE
    assert prelude_type.related == AST_NONE
    assert prelude_type.error_end > prelude_type.error_start

    duplicate_variant = run_frontend(
        library,
        b"enum First { Shared(); }\n"
        b"enum Second { Shared(); }\n",
    )
    assert (duplicate_variant.stage, duplicate_variant.status) == (
        FRONTEND_TYPE_MEMBERS,
        TYPE_MEMBERS_DUPLICATE,
    )
    assert duplicate_variant.node != AST_NONE
    assert duplicate_variant.related != AST_NONE
    assert duplicate_variant.error_end > duplicate_variant.error_start

    prelude_constructor = run_frontend(
        library,
        b"enum Flag { True(); }\n",
    )
    assert (prelude_constructor.stage, prelude_constructor.status) == (
        FRONTEND_TYPE_MEMBERS,
        TYPE_MEMBERS_DUPLICATE,
    )
    assert prelude_constructor.node != AST_NONE
    assert prelude_constructor.related == AST_NONE
    assert prelude_constructor.error_end > prelude_constructor.error_start

    unknown = run_frontend(library, b"struct Holder { missing: Missing; }\n")
    assert (unknown.stage, unknown.status) == (
        FRONTEND_TYPES,
        TYPE_UNKNOWN_NAMED,
    )
    assert unknown.node != AST_NONE
    assert unknown.related != AST_NONE
    assert unknown.error_end > unknown.error_start
    assert unknown.type_count == 0


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        observed = assert_clean_cases(library)
        assert_failure_routing(library)
        mixed = observed["mixed"]
        compiler = observed["compiler"]
        print(
            "frontend ABI: "
            f"mixed tokens/nodes/types/symbols/functions={mixed}; "
            f"compiler={compiler}; deterministic with lex/parse/semantic routing"
        )


if __name__ == "__main__":
    main()
