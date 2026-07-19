#!/usr/bin/env python3
"""Type and resolve the first scalar, direct-call self-hosting profile."""

import ctypes
import tempfile
from pathlib import Path

from test_ast_validate import AstValidationReport, validate
from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_NONE, AstTape, children_of, parse
from test_parser_expressions import enum_ordinals
from test_semantic_facts import (
    NODE_COLUMNS,
    TYPE_COLUMNS,
    NodeFacts,
    TypeTape,
    assert_guards,
    make_tape,
    snapshot,
)
from test_semantic_globals import SemanticGlobalsReport
from test_semantic_function_symbols import SemanticFunctionSymbolsReport
from test_symbols import (
    SYMBOL_CLEAN,
    SYMBOL_NONE,
    SYMBOL_VALUE,
    SymbolTape,
    assert_guards as assert_symbol_guards,
    find,
    make_symbols,
    reset,
)


AST = enum_ordinals("AstKind")
U64_MAX = (1 << 64) - 1
U64_POISON = 0x7777777777777777
U64_GUARD = 0x8888888888888888
ENUM_POISON = 0x55555555
ENUM_GUARD = 0x66666666

BODY_CLEAN = 0
BODY_INVALID_TOKEN_TAPE = 1
BODY_INVALID_AST_TAPE = 2
BODY_INVALID_VALIDATION = 3
BODY_INVALID_FUNCTION = 4
BODY_INVALID_TYPE_TAPE = 5
BODY_INVALID_NODE_FACTS = 6
BODY_INVALID_SCRATCH = 7
BODY_CAPACITY = 8
BODY_MALFORMED = 9
BODY_UNSUPPORTED = 10
BODY_UNKNOWN_NAME = 11
BODY_TYPE_MISMATCH = 12
BODY_INVALID_LITERAL = 13
BODY_INVALID_SYMBOL_TAPE = 14
BODY_EFFECT_MISMATCH = 15

RULE_NONE = 0
RULE_FORM7 = 1
RULE_GRAM11 = 2
RULE_TYPE5 = 3
RULE_TYPE6 = 4
RULE_TYPE7 = 5
RULE_EFFECT2 = 6
RULE_FORM5 = 7

FIX_NONE = 0
FIX_CANONICAL_LITERAL = 1
FIX_NAME_ARGUMENTS = 2
FIX_MATCH_TYPE = 3
FIX_DECLARE_BEFORE_USE = 4
FIX_RENAME_BINDING = 5
FIX_DEREF_BORROW = 6
FIX_DECLARE_EFFECT = 7
FIX_ADD_LITERAL_TYPE_SUFFIX = 8

U8_LITERAL_READY = 0
U8_LITERAL_WRONG_TYPE = 1
U8_LITERAL_MISSING_SUFFIX = 2
U8_LITERAL_NONCANONICAL = 3
U8_LITERAL_INVALID_VALUE = 4
U8_LITERAL_MALFORMED = 5

FACTS_CLEAN = 0
FACTS_INVALID_SHAPE = 1
FACTS_CAPACITY = 2
TYPE_U8 = 2
TYPE_BOOL = 4
MODE_NONE = 0
MODE_OWN = 1
OP_NONE = 0
OP_IEQ = 5
OP_ILE = 7
OP_IGE = 9
OP_BAND = 10
OP_BOR = 11
PRELUDE_TYPE_UNKNOWN = 0
PRELUDE_TYPE_BOOL = 1
PRELUDE_CONSTRUCTOR_UNKNOWN = 0
PRELUDE_CONSTRUCTOR_TRUE = 1
PRELUDE_CONSTRUCTOR_FALSE = 2


class SemanticBodyScratch(ctypes.Structure):
    _fields_ = [
        ("name_tokens", Buffer),
        ("declarations", Buffer),
        ("type_ids", Buffer),
        ("modes", Buffer),
        ("count", ctypes.c_uint64),
        ("loop_labels", Buffer),
        ("loop_count", ctypes.c_uint64),
    ]


class SemanticBodyReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("node", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("rule", ctypes.c_int32),
        ("fix", ctypes.c_int32),
        ("related_node", ctypes.c_uint64),
    ]


class SemanticU8LiteralResult(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("value", ctypes.c_uint64),
    ]


SCRATCH_COLUMNS = (
    (ctypes.c_uint64, U64_POISON, U64_GUARD),
    (ctypes.c_uint64, U64_POISON, U64_GUARD),
    (ctypes.c_uint64, U64_POISON, U64_GUARD),
    (ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
    (ctypes.c_uint64, U64_POISON, U64_GUARD),
)


def fixture(
    parameter=b"c",
    first_operand=b"c",
    first_literal=b"97_u8",
    first_binding_type=b"Bool",
    return_op=b"band",
    first_binding=b"ge",
    second_binding=b"le",
):
    return (
        b"fn lexer_is_lower ("
        + parameter
        + b": own u8) -> own Bool pure {\n"
        b"  let "
        + first_binding
        + b": own "
        + first_binding_type
        + b" = ige<u8>("
        + first_operand
        + b", "
        + first_literal
        + b");\n"
        b"  let "
        + second_binding
        + b": own Bool = ile<u8>("
        + parameter
        + b", 122_u8);\n"
        b"  return "
        + return_op
        + b"<Bool>("
        + first_binding
        + b", "
        + second_binding
        + b");\n"
        b"}\n"
    )


def space_fixture(
    horizontal_op=b"ieq",
    horizontal_type=b"u8",
    horizontal_operand=b"c",
    horizontal_literal=b"32_u8",
    return_op=b"bor",
    return_type=b"Bool",
    control_first=False,
):
    horizontal = (
        b"  let horizontal: own Bool = "
        + horizontal_op
        + b"<"
        + horizontal_type
        + b">("
        + horizontal_operand
        + b", "
        + horizontal_literal
        + b");\n"
    )
    control_ge = b"  let control_ge: own Bool = ige<u8>(c, 9_u8);\n"
    control_le = b"  let control_le: own Bool = ile<u8>(c, 13_u8);\n"
    control = b"  let control: own Bool = band<Bool>(control_ge, control_le);\n"
    if control_first:
        statements = horizontal + control + control_ge + control_le
    else:
        statements = horizontal + control_ge + control_le + control
    return (
        b"fn lexer_is_space (c: own u8) -> own Bool pure {\n"
        + statements
        + b"  return "
        + return_op
        + b"<"
        + return_type
        + b">(horizontal, control);\n"
        + b"}\n"
    )


def user_call_fixture(
    call_name=b"classify",
    arguments=b"x: c",
    callee_parameters=b"x: own u8",
    callee_return=b"own Bool",
    callee_effect=b"pure",
    callee_body=(
        b"  let ge: own Bool = ige<u8>(x, 1_u8);\n"
        b"  let le: own Bool = ile<u8>(x, 2_u8);\n"
        b"  return band<Bool>(ge, le);\n"
    ),
    caller_prefix=b"",
):
    return (
        b"fn classify ("
        + callee_parameters
        + b") -> "
        + callee_return
        + b" "
        + callee_effect
        + b" {\n"
        + callee_body
        + b"}\n"
        b"fn caller (c: own u8) -> own Bool pure {\n"
        + caller_prefix
        + b"  let answer: own Bool = "
        + call_name
        + b"("
        + arguments
        + b");\n"
        b"  return bor<Bool>(answer, answer);\n"
        b"}\n"
    )


def earlier_return_fixture():
    return (
        b"fn earlier (c: own u8) -> own Bool pure {\n"
        b"  let first: own Bool = ige<u8>(c, 1_u8);\n"
        b"  let second: own Bool = ile<u8>(c, 2_u8);\n"
        b"  return first;\n"
        b"}\n"
    )


def configure(library):
    library.semantic_body_parse_u8_literal.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.c_uint64,
    ]
    library.semantic_body_parse_u8_literal.restype = SemanticU8LiteralResult
    library.semantic_body_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(SemanticBodyScratch),
        ctypes.POINTER(SemanticBodyReport),
    ]
    library.semantic_body_run.restype = None
    library.semantic_type_tape_reset.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_reset.restype = None
    library.semantic_node_facts_reset.argtypes = [ctypes.POINTER(NodeFacts)]
    library.semantic_node_facts_reset.restype = None
    library.semantic_index_globals.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(SemanticGlobalsReport),
    ]
    library.semantic_index_globals.restype = None
    library.semantic_index_function_symbols.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(SemanticFunctionSymbolsReport),
    ]
    library.semantic_index_function_symbols.restype = None
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


def make_scratch(capacities):
    storage = []
    buffers = []
    for (ctype, poison, guard), capacity in zip(SCRATCH_COLUMNS, capacities):
        column = (ctype * (capacity + 1))()
        for slot in range(capacity):
            column[slot] = poison
        column[capacity] = guard
        storage.append(column)
        buffers.append(Buffer(ctypes.cast(column, ctypes.c_void_p), capacity))
    return (
        tuple(storage),
        tuple(capacities),
        SemanticBodyScratch(*buffers[:4], 0, buffers[4], 0),
    )


def assert_scratch_guards(storage, capacities):
    for column, (_, _, guard), capacity in zip(
        storage, SCRATCH_COLUMNS, capacities
    ):
        assert column[capacity] == guard


def source_buffer(storage, length):
    return Buffer(ctypes.cast(storage, ctypes.c_void_p), length)


def index_symbols(library, source, tokens, columns, ast):
    symbol_storage, symbol_physical, symbols = make_symbols(
        (ast.count,) * 4
    )
    reset(library, symbols)
    globals_report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(globals_report),
    )
    assert (globals_report.status, globals_report.declaration, globals_report.related) == (
        0,
        AST_NONE,
        SYMBOL_NONE,
    )
    for declaration in children_of(columns, ast.root):
        if columns[0][declaration] != AST["AstFunction"]:
            continue
        report = SemanticFunctionSymbolsReport(99, 99, 99)
        library.semantic_index_function_symbols(
            source,
            ctypes.byref(tokens),
            ctypes.byref(ast),
            declaration,
            ctypes.byref(symbols),
            ctypes.byref(report),
        )
        assert (report.status, report.declaration, report.related) == (
            0,
            AST_NONE,
            SYMBOL_NONE,
        ), (declaration, report.status, report.declaration, report.related)
    assert_symbol_guards(symbol_storage, symbol_physical)
    return symbol_storage, symbol_physical, symbols


def parsed(library, data):
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    validation = validate(library, len(data), tokens.count, ast)
    assert validation.status == 0
    source = source_buffer(source_storage, len(data))
    symbol_storage, symbol_physical, symbols = index_symbols(
        library, source, tokens, columns, ast
    )
    return (
        source_storage,
        source,
        token_storage,
        tokens,
        columns,
        ast,
        validation,
        symbol_storage,
        symbol_physical,
        symbols,
    )


def find_function_by_text(data, columns, ast, text):
    kinds, _, starts, ends, _, _, _ = columns
    matches = [
        node
        for node in children_of(columns, ast.root)
        if kinds[node] == AST["AstFunction"]
        and any(
            kinds[child] == AST["AstFunctionName"]
            and data[starts[child] : ends[child]] == text
            for child in children_of(columns, node)
        )
    ]
    assert len(matches) == 1, (text, matches)
    return matches[0]


def make_outputs(library, ast_count, type_caps=None, fact_caps=None, scratch_caps=None):
    if type_caps is None:
        type_caps = (2,) * len(TYPE_COLUMNS)
    if fact_caps is None:
        fact_caps = (ast_count,) * len(NODE_COLUMNS)
    if scratch_caps is None:
        scratch_caps = (3,) * len(SCRATCH_COLUMNS)
    type_storage, types = make_tape(TypeTape, TYPE_COLUMNS, type_caps)
    fact_storage, facts = make_tape(NodeFacts, NODE_COLUMNS, fact_caps)
    library.semantic_type_tape_reset(ctypes.byref(types))
    library.semantic_node_facts_reset(ctypes.byref(facts))
    scratch_storage, scratch_physical, scratch = make_scratch(scratch_caps)
    return (
        type_storage,
        types,
        fact_storage,
        facts,
        scratch_storage,
        scratch_physical,
        scratch,
        type_caps,
        fact_caps,
    )


def invoke(library, parsed_case, function, outputs, symbols=None):
    source = parsed_case[1]
    tokens = parsed_case[3]
    ast = parsed_case[5]
    validation = parsed_case[6]
    if symbols is None:
        symbols = parsed_case[9]
    (
        _,
        types,
        _,
        facts,
        _,
        _,
        scratch,
        _,
        _,
    ) = outputs
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    library.semantic_body_run(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        function,
        ctypes.byref(validation),
        ctypes.byref(symbols),
        ctypes.byref(types),
        ctypes.byref(facts),
        ctypes.byref(scratch),
        ctypes.byref(report),
    )
    return report


def assert_output_guards(outputs):
    (
        type_storage,
        _,
        fact_storage,
        _,
        scratch_storage,
        scratch_physical,
        _,
        type_caps,
        fact_caps,
    ) = outputs
    assert_guards(type_storage, TYPE_COLUMNS, type_caps)
    assert_guards(fact_storage, NODE_COLUMNS, fact_caps)
    assert_scratch_guards(scratch_storage, scratch_physical)


def output_snapshot(outputs):
    (
        type_storage,
        types,
        fact_storage,
        facts,
        scratch_storage,
        scratch_physical,
        scratch,
        type_caps,
        fact_caps,
    ) = outputs
    return (
        snapshot(type_storage, type_caps),
        (types.count, types.status, types.node, types.related),
        snapshot(fact_storage, fact_caps),
        (facts.count, facts.status, facts.node, facts.related),
        tuple(
            tuple(column[:capacity])
            for column, capacity in zip(scratch_storage, scratch_physical)
        ),
        scratch.count,
    )


def report_tuple(report):
    return (report.status, report.node, report.related)


def diagnostic_tuple(report):
    return (report.rule, report.fix, report.related_node)


def body_nodes(columns, function):
    direct = children_of(columns, function)
    assert len(direct) == 6
    parameter = direct[1]
    block = direct[5]
    statements = children_of(columns, block)
    assert len(statements) == 3
    first_let, second_let, return_node = statements
    first_call = children_of(columns, first_let)[3]
    second_call = children_of(columns, second_let)[3]
    return_call = children_of(columns, return_node)[0]
    first_call_children = children_of(columns, first_call)
    second_call_children = children_of(columns, second_call)
    return_call_children = children_of(columns, return_call)
    return {
        "parameter": parameter,
        "parameter_mode": children_of(columns, parameter)[0],
        "parameter_type": children_of(columns, parameter)[1],
        "return_mode": direct[2],
        "return_type": direct[3],
        "first_let": first_let,
        "first_binding_type": children_of(columns, first_let)[2],
        "first_binding_value": children_of(columns, first_let)[3],
        "second_let": second_let,
        "return": return_node,
        "first_call": first_call,
        "second_call": second_call,
        "return_call": return_call,
        "first_type_arg": first_call_children[0],
        "second_type_arg": second_call_children[0],
        "return_type_arg": return_call_children[0],
        "first_c": first_call_children[1],
        "first_literal": first_call_children[2],
        "second_c": second_call_children[1],
        "second_literal": second_call_children[2],
        "ge_use": return_call_children[1],
        "le_use": return_call_children[2],
    }


def space_body_nodes(columns, function):
    direct = children_of(columns, function)
    assert len(direct) == 6
    parameter = direct[1]
    statements = children_of(columns, direct[5])
    assert len(statements) == 5
    horizontal_let, control_ge_let, control_le_let, control_let, return_node = (
        statements
    )
    horizontal_call = children_of(columns, horizontal_let)[3]
    control_ge_call = children_of(columns, control_ge_let)[3]
    control_le_call = children_of(columns, control_le_let)[3]
    control_call = children_of(columns, control_let)[3]
    return_call = children_of(columns, return_node)[0]
    horizontal_children = children_of(columns, horizontal_call)
    control_ge_children = children_of(columns, control_ge_call)
    control_le_children = children_of(columns, control_le_call)
    control_children = children_of(columns, control_call)
    return_children = children_of(columns, return_call)
    return {
        "parameter": parameter,
        "horizontal_let": horizontal_let,
        "control_ge_let": control_ge_let,
        "control_le_let": control_le_let,
        "control_let": control_let,
        "return": return_node,
        "horizontal_call": horizontal_call,
        "control_ge_call": control_ge_call,
        "control_le_call": control_le_call,
        "control_call": control_call,
        "return_call": return_call,
        "horizontal_type_arg": horizontal_children[0],
        "horizontal_c": horizontal_children[1],
        "horizontal_literal": horizontal_children[2],
        "control_ge_type_arg": control_ge_children[0],
        "control_ge_c": control_ge_children[1],
        "control_ge_literal": control_ge_children[2],
        "control_le_type_arg": control_le_children[0],
        "control_le_c": control_le_children[1],
        "control_le_literal": control_le_children[2],
        "control_type_arg": control_children[0],
        "control_ge_use": control_children[1],
        "control_le_use": control_children[2],
        "return_type_arg": return_children[0],
        "horizontal_use": return_children[1],
        "control_use": return_children[2],
    }


def linear_body_nodes(columns, function):
    direct = children_of(columns, function)
    block = direct[-1]
    statements = children_of(columns, block)
    assert statements and columns[0][statements[-1]] == AST["AstReturn"]
    lets = statements[:-1]
    assert all(columns[0][node] == AST["AstLet"] for node in lets)
    initializers = [children_of(columns, node)[3] for node in lets]
    return_node = statements[-1]
    return_root = children_of(columns, return_node)[0]
    return {
        "parameter": next(
            node for node in direct if columns[0][node] == AST["AstParameter"]
        ),
        "lets": lets,
        "initializers": initializers,
        "return": return_node,
        "return_root": return_root,
    }


def symbol_body_nodes(columns, function):
    direct = children_of(columns, function)
    assert len(direct) == 6
    parameter = direct[1]
    block = direct[5]
    statements = children_of(columns, block)
    assert len(statements) == 26

    guards = []
    for ordinal in range(12):
        let_node = statements[ordinal * 2]
        match_node = statements[ordinal * 2 + 1]
        continuation = statements[ordinal * 2 + 2]
        assert columns[0][let_node] == AST["AstLet"]
        assert columns[0][match_node] == AST["AstMatch"]
        let_children = children_of(columns, let_node)
        assert len(let_children) == 4
        call = let_children[3]
        assert columns[0][call] == AST["AstTableCall"]
        call_children = children_of(columns, call)
        assert len(call_children) == 3

        match_children = children_of(columns, match_node)
        assert len(match_children) == 3
        scrutinee, true_arm, false_arm = match_children
        assert columns[0][scrutinee] == AST["AstPlaceUse"]
        assert columns[0][true_arm] == AST["AstMatchArm"]
        assert columns[0][false_arm] == AST["AstMatchArm"]
        (true_block,) = children_of(columns, true_arm)
        (false_block,) = children_of(columns, false_arm)
        assert columns[0][true_block] == AST["AstBlock"]
        assert columns[0][false_block] == AST["AstBlock"]
        (early_return,) = children_of(columns, true_block)
        assert children_of(columns, false_block) == []
        assert columns[0][early_return] == AST["AstReturn"]
        (constructor,) = children_of(columns, early_return)
        assert columns[0][constructor] == AST["AstConstructor"]
        assert children_of(columns, constructor) == []
        guards.append(
            {
                "ordinal": ordinal,
                "let": let_node,
                "call": call,
                "type_argument": call_children[0],
                "parameter_use": call_children[1],
                "literal": call_children[2],
                "match": match_node,
                "scrutinee": scrutinee,
                "true_arm": true_arm,
                "true_block": true_block,
                "early_return": early_return,
                "constructor": constructor,
                "false_arm": false_arm,
                "false_block": false_block,
                "continuation": continuation,
            }
        )

    dot_let = statements[24]
    final_return = statements[25]
    assert columns[0][dot_let] == AST["AstLet"]
    assert columns[0][final_return] == AST["AstReturn"]
    dot_children = children_of(columns, dot_let)
    assert len(dot_children) == 4
    dot_call = dot_children[3]
    dot_call_children = children_of(columns, dot_call)
    assert columns[0][dot_call] == AST["AstTableCall"]
    assert len(dot_call_children) == 3
    (final_place,) = children_of(columns, final_return)
    assert columns[0][final_place] == AST["AstPlaceUse"]
    return {
        "parameter": parameter,
        "block": block,
        "statements": statements,
        "guards": guards,
        "dot_let": dot_let,
        "dot_call": dot_call,
        "dot_type_argument": dot_call_children[0],
        "dot_parameter_use": dot_call_children[1],
        "dot_literal": dot_call_children[2],
        "final_return": final_return,
        "final_place": final_place,
    }


def user_call_nodes(columns, function):
    nodes = linear_body_nodes(columns, function)
    user_pairs = [
        (let_node, initializer)
        for let_node, initializer in zip(nodes["lets"], nodes["initializers"])
        if columns[0][initializer] == AST["AstUserCall"]
    ]
    return nodes, user_pairs


def assert_clean_facts(library):
    data = fixture()
    case = parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], b"lexer_is_lower")
    first = make_outputs(library, case[5].count)
    first_report = invoke(library, case, function, first)
    assert (first_report.status, first_report.node, first_report.related) == (
        BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert_output_guards(first)

    type_storage, types, fact_storage, facts, _, _, scratch, _, _ = first
    assert (types.count, types.status, types.node, types.related) == (
        2,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert list(type_storage[0][:2]) == [TYPE_U8, TYPE_BOOL]
    assert list(type_storage[1][:2]) == [U64_MAX, U64_MAX]
    assert list(type_storage[2][:2]) == [U64_MAX, U64_MAX]
    assert list(type_storage[3][:2]) == [U64_MAX, U64_MAX]
    assert list(type_storage[4][:2]) == [U64_MAX, U64_MAX]
    assert list(type_storage[5][:2]) == [PRELUDE_TYPE_UNKNOWN, PRELUDE_TYPE_BOOL]
    assert (facts.count, facts.status, facts.node, facts.related) == (
        case[5].count,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert scratch.count == 3

    nodes = body_nodes(case[4], function)
    type_ids = fact_storage[0]
    resolved = fact_storage[1]
    ordinals = fact_storage[2]
    operations = fact_storage[3]
    constants = fact_storage[4]
    modes = fact_storage[6]
    assert (type_ids[nodes["parameter"]], ordinals[nodes["parameter"]], modes[nodes["parameter"]]) == (0, 0, MODE_OWN)
    assert (type_ids[nodes["first_let"]], ordinals[nodes["first_let"]], modes[nodes["first_let"]]) == (1, 0, MODE_OWN)
    assert (type_ids[nodes["second_let"]], ordinals[nodes["second_let"]], modes[nodes["second_let"]]) == (1, 1, MODE_OWN)
    assert operations[nodes["first_call"]] == OP_IGE
    assert operations[nodes["second_call"]] == OP_ILE
    assert operations[nodes["return_call"]] == OP_BAND
    assert ordinals[nodes["first_call"]] == 0
    assert ordinals[nodes["second_call"]] == 1
    assert ordinals[nodes["return_call"]] == 2
    assert ordinals[nodes["return"]] == 2
    assert constants[nodes["first_literal"]] == 97
    assert constants[nodes["second_literal"]] == 122
    assert resolved[nodes["first_c"]] == nodes["parameter"]
    assert resolved[nodes["second_c"]] == nodes["parameter"]
    assert resolved[nodes["ge_use"]] == nodes["first_let"]
    assert resolved[nodes["le_use"]] == nodes["second_let"]
    for name in ("first_call", "second_call", "return_call", "first_let", "second_let", "return"):
        assert type_ids[nodes[name]] == 1
        assert modes[nodes[name]] == MODE_OWN
    for name in ("first_literal", "second_literal", "first_c", "second_c"):
        assert type_ids[nodes[name]] == 0
        assert modes[nodes[name]] == MODE_OWN
    for name in ("ge_use", "le_use"):
        assert type_ids[nodes[name]] == 1
        assert modes[nodes[name]] == MODE_OWN
    for name in ("parameter_type", "first_type_arg", "second_type_arg"):
        assert type_ids[nodes[name]] == 0
    for name in ("return_type", "return_type_arg"):
        assert type_ids[nodes[name]] == 1

    second = make_outputs(library, case[5].count)
    second_report = invoke(library, case, function, second)
    assert report_tuple(first_report) == report_tuple(second_report)
    assert output_snapshot(first) == output_snapshot(second)
    assert_output_guards(second)
    assert case[0]
    return case[5].count


def assert_space_clean_facts(library):
    data = space_fixture()
    case = parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], b"lexer_is_space")
    scratch_caps = (5,) * len(SCRATCH_COLUMNS)
    first = make_outputs(
        library, case[5].count, scratch_caps=scratch_caps
    )
    first_report = invoke(library, case, function, first)
    assert report_tuple(first_report) == (BODY_CLEAN, U64_MAX, U64_MAX)
    assert_output_guards(first)

    _, types, fact_storage, facts, _, _, scratch, _, _ = first
    assert (types.count, types.status, types.node, types.related) == (
        2,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert (facts.count, facts.status, facts.node, facts.related) == (
        case[5].count,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert scratch.count == 5

    nodes = space_body_nodes(case[4], function)
    type_ids = fact_storage[0]
    resolved = fact_storage[1]
    ordinals = fact_storage[2]
    operations = fact_storage[3]
    constants = fact_storage[4]
    modes = fact_storage[6]
    expected_operations = (
        ("horizontal_call", OP_IEQ),
        ("control_ge_call", OP_IGE),
        ("control_le_call", OP_ILE),
        ("control_call", OP_BAND),
        ("return_call", OP_BOR),
    )
    for ordinal, (name, operation) in enumerate(expected_operations):
        assert operations[nodes[name]] == operation
        assert type_ids[nodes[name]] == 1
        assert modes[nodes[name]] == MODE_OWN
        assert ordinals[nodes[name]] == ordinal
    expected_lets = (
        ("horizontal_let", 0),
        ("control_ge_let", 1),
        ("control_le_let", 2),
        ("control_let", 3),
    )
    for name, ordinal in expected_lets:
        assert (
            type_ids[nodes[name]],
            ordinals[nodes[name]],
            modes[nodes[name]],
            operations[nodes[name]],
        ) == (1, ordinal, MODE_OWN, OP_NONE)
    assert (
        type_ids[nodes["parameter"]],
        ordinals[nodes["parameter"]],
        modes[nodes["parameter"]],
    ) == (0, 0, MODE_OWN)
    assert type_ids[nodes["return"]] == 1
    assert modes[nodes["return"]] == MODE_OWN
    assert operations[nodes["return"]] == OP_NONE
    assert ordinals[nodes["return"]] == 4

    literal_values = (
        ("horizontal_literal", 32),
        ("control_ge_literal", 9),
        ("control_le_literal", 13),
    )
    for name, value in literal_values:
        assert constants[nodes[name]] == value
        assert type_ids[nodes[name]] == 0
        assert modes[nodes[name]] == MODE_OWN
        assert operations[nodes[name]] == OP_NONE
    for name in ("horizontal_c", "control_ge_c", "control_le_c"):
        assert resolved[nodes[name]] == nodes["parameter"]
        assert type_ids[nodes[name]] == 0
        assert modes[nodes[name]] == MODE_OWN
    expected_resolutions = (
        ("control_ge_use", "control_ge_let"),
        ("control_le_use", "control_le_let"),
        ("horizontal_use", "horizontal_let"),
        ("control_use", "control_let"),
    )
    for use, declaration in expected_resolutions:
        assert resolved[nodes[use]] == nodes[declaration]
        assert type_ids[nodes[use]] == 1
        assert modes[nodes[use]] == MODE_OWN
    for name in (
        "horizontal_type_arg",
        "control_ge_type_arg",
        "control_le_type_arg",
    ):
        assert type_ids[nodes[name]] == 0
    for name in ("control_type_arg", "return_type_arg"):
        assert type_ids[nodes[name]] == 1

    second = make_outputs(
        library, case[5].count, scratch_caps=scratch_caps
    )
    second_report = invoke(library, case, function, second)
    assert report_tuple(first_report) == report_tuple(second_report)
    assert output_snapshot(first) == output_snapshot(second)
    assert_output_guards(second)
    return case[5].count


def assert_space_failures(library):
    cases = (
        (space_fixture(horizontal_type=b"Bool"), BODY_TYPE_MISMATCH),
        (space_fixture(return_type=b"u8"), BODY_TYPE_MISMATCH),
        (space_fixture(horizontal_op=b"ilt"), BODY_UNSUPPORTED),
        (space_fixture(return_op=b"bxor"), BODY_UNSUPPORTED),
        (space_fixture(control_first=True), BODY_UNKNOWN_NAME),
    )
    for data, expected in cases:
        parsed_case = parsed(library, data)
        function = find_function_by_text(
            data, parsed_case[4], parsed_case[5], b"lexer_is_space"
        )
        outputs = make_outputs(
            library,
            parsed_case[5].count,
            scratch_caps=(5,) * len(SCRATCH_COLUMNS),
        )
        report = invoke(library, parsed_case, function, outputs)
        assert report.status == expected, (data, report_tuple(report), expected)
        assert report.node != U64_MAX
        assert outputs[1].status == FACTS_INVALID_SHAPE
        assert outputs[3].status == FACTS_INVALID_SHAPE
        assert_output_guards(outputs)
        assert_symbol_guards(parsed_case[7], parsed_case[8])

    data = space_fixture()
    parsed_case = parsed(library, data)
    function = find_function_by_text(
        data, parsed_case[4], parsed_case[5], b"lexer_is_space"
    )
    for short in range(len(SCRATCH_COLUMNS)):
        capacities = [5] * len(SCRATCH_COLUMNS)
        capacities[short] = 4
        outputs = make_outputs(
            library,
            parsed_case[5].count,
            scratch_caps=tuple(capacities),
        )
        report = invoke(library, parsed_case, function, outputs)
        assert report.status == BODY_CAPACITY, (short, report_tuple(report))
        assert outputs[1].count == 0
        assert outputs[3].count == 0
        assert outputs[1].status == FACTS_CAPACITY
        assert outputs[3].status == FACTS_CAPACITY
        assert_output_guards(outputs)


def assert_symbol_clean_facts(library):
    data = (Path(__file__).resolve().parent / "src" / "lexer.wf").read_bytes()
    case = parsed(library, data)
    function = find_function_by_text(
        data, case[4], case[5], b"lexer_is_symbol"
    )
    nodes = symbol_body_nodes(case[4], function)
    outputs = make_outputs(
        library,
        case[5].count,
        scratch_caps=(14,) * len(SCRATCH_COLUMNS),
    )
    report = invoke(library, case, function, outputs)
    assert report_tuple(report) == (BODY_CLEAN, U64_MAX, U64_MAX)
    assert diagnostic_tuple(report) == (RULE_NONE, FIX_NONE, U64_MAX)
    assert_output_guards(outputs)

    _, types, fact_storage, facts, _, _, scratch, _, _ = outputs
    assert (types.count, types.status, types.node, types.related) == (
        2,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert (facts.count, facts.status, facts.node, facts.related) == (
        case[5].count,
        FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert scratch.count == 14

    (
        type_ids,
        resolved,
        ordinals,
        operations,
        constants,
        targets,
        modes,
        constructors,
    ) = fact_storage
    expected_literals = (40, 41, 123, 125, 60, 62, 58, 59, 44, 61, 91, 93)
    for guard, literal_value in zip(nodes["guards"], expected_literals):
        ordinal = guard["ordinal"]
        for node in (guard["let"], guard["call"]):
            assert (
                type_ids[node],
                ordinals[node],
                modes[node],
            ) == (1, ordinal, MODE_OWN)
            assert targets[node] == U64_MAX
            assert constructors[node] == PRELUDE_CONSTRUCTOR_UNKNOWN
        assert operations[guard["let"]] == OP_NONE
        assert operations[guard["call"]] == OP_IEQ
        assert constants[guard["literal"]] == literal_value
        assert resolved[guard["parameter_use"]] == nodes["parameter"]

        assert (
            type_ids[guard["scrutinee"]],
            resolved[guard["scrutinee"]],
            ordinals[guard["scrutinee"]],
            operations[guard["scrutinee"]],
            constants[guard["scrutinee"]],
            targets[guard["scrutinee"]],
            modes[guard["scrutinee"]],
            constructors[guard["scrutinee"]],
        ) == (
            1,
            guard["let"],
            U64_MAX,
            OP_NONE,
            U64_MAX,
            guard["match"],
            MODE_OWN,
            PRELUDE_CONSTRUCTOR_UNKNOWN,
        )
        assert (
            type_ids[guard["match"]],
            ordinals[guard["match"]],
            constants[guard["match"]],
            targets[guard["match"]],
            modes[guard["match"]],
            constructors[guard["match"]],
        ) == (
            U64_MAX,
            U64_MAX,
            U64_MAX,
            guard["continuation"],
            MODE_NONE,
            PRELUDE_CONSTRUCTOR_UNKNOWN,
        )

        expected_arms = (
            (
                guard["true_arm"],
                guard["true_block"],
                0,
                1,
                PRELUDE_CONSTRUCTOR_TRUE,
                guard["early_return"],
            ),
            (
                guard["false_arm"],
                guard["false_block"],
                1,
                0,
                PRELUDE_CONSTRUCTOR_FALSE,
                guard["continuation"],
            ),
        )
        for arm, block, arm_ordinal, constant, constructor, target in expected_arms:
            assert (
                type_ids[arm],
                resolved[arm],
                ordinals[arm],
                operations[arm],
                constants[arm],
                targets[arm],
                modes[arm],
                constructors[arm],
            ) == (
                1,
                U64_MAX,
                arm_ordinal,
                OP_NONE,
                constant,
                target,
                MODE_NONE,
                constructor,
            )
            assert (
                type_ids[block],
                resolved[block],
                ordinals[block],
                operations[block],
                constants[block],
                targets[block],
                modes[block],
                constructors[block],
            ) == (
                U64_MAX,
                U64_MAX,
                U64_MAX,
                OP_NONE,
                U64_MAX,
                target,
                MODE_NONE,
                PRELUDE_CONSTRUCTOR_UNKNOWN,
            )

        assert (
            type_ids[guard["early_return"]],
            resolved[guard["early_return"]],
            ordinals[guard["early_return"]],
            operations[guard["early_return"]],
            constants[guard["early_return"]],
            targets[guard["early_return"]],
            modes[guard["early_return"]],
            constructors[guard["early_return"]],
        ) == (
            1,
            U64_MAX,
            U64_MAX,
            OP_NONE,
            U64_MAX,
            function,
            MODE_OWN,
            PRELUDE_CONSTRUCTOR_UNKNOWN,
        )
        assert (
            type_ids[guard["constructor"]],
            resolved[guard["constructor"]],
            ordinals[guard["constructor"]],
            operations[guard["constructor"]],
            constants[guard["constructor"]],
            targets[guard["constructor"]],
            modes[guard["constructor"]],
            constructors[guard["constructor"]],
        ) == (
            1,
            U64_MAX,
            U64_MAX,
            OP_NONE,
            1,
            guard["early_return"],
            MODE_OWN,
            PRELUDE_CONSTRUCTOR_TRUE,
        )

    for node in (nodes["dot_let"], nodes["dot_call"]):
        assert (type_ids[node], ordinals[node], modes[node]) == (
            1,
            12,
            MODE_OWN,
        )
    assert operations[nodes["dot_let"]] == OP_NONE
    assert operations[nodes["dot_call"]] == OP_IEQ
    assert constants[nodes["dot_literal"]] == 46
    assert resolved[nodes["dot_parameter_use"]] == nodes["parameter"]
    assert (
        type_ids[nodes["final_place"]],
        resolved[nodes["final_place"]],
        ordinals[nodes["final_place"]],
        operations[nodes["final_place"]],
        constants[nodes["final_place"]],
        targets[nodes["final_place"]],
        modes[nodes["final_place"]],
        constructors[nodes["final_place"]],
    ) == (
        1,
        nodes["dot_let"],
        U64_MAX,
        OP_NONE,
        U64_MAX,
        nodes["final_return"],
        MODE_OWN,
        PRELUDE_CONSTRUCTOR_UNKNOWN,
    )
    assert (
        type_ids[nodes["final_return"]],
        resolved[nodes["final_return"]],
        ordinals[nodes["final_return"]],
        operations[nodes["final_return"]],
        constants[nodes["final_return"]],
        targets[nodes["final_return"]],
        modes[nodes["final_return"]],
        constructors[nodes["final_return"]],
    ) == (
        1,
        U64_MAX,
        12,
        OP_NONE,
        U64_MAX,
        function,
        MODE_OWN,
        PRELUDE_CONSTRUCTOR_UNKNOWN,
    )

    second = make_outputs(
        library,
        case[5].count,
        scratch_caps=(14,) * len(SCRATCH_COLUMNS),
    )
    second_report = invoke(library, case, function, second)
    assert report_tuple(second_report) == report_tuple(report)
    assert output_snapshot(second) == output_snapshot(outputs)
    assert_output_guards(second)
    return case[5].count


def assert_symbol_failures(library):
    data = (Path(__file__).resolve().parent / "src" / "lexer.wf").read_bytes()

    def rejected(mutate):
        case = parsed(library, data)
        function = find_function_by_text(
            data, case[4], case[5], b"lexer_is_symbol"
        )
        nodes = symbol_body_nodes(case[4], function)
        mutate(case, nodes)
        outputs = make_outputs(
            library,
            case[5].count,
            scratch_caps=(14,) * len(SCRATCH_COLUMNS),
        )
        report = invoke(library, case, function, outputs)
        assert report.status != BODY_CLEAN, report_tuple(report)
        assert report.node != U64_MAX
        assert outputs[1].status == FACTS_INVALID_SHAPE
        assert outputs[3].status == FACTS_INVALID_SHAPE
        assert_output_guards(outputs)
        assert_symbol_guards(case[7], case[8])

    def swap_arm_tags(case, nodes):
        first = nodes["guards"][0]
        heads = case[4][1]
        heads[first["true_arm"]], heads[first["false_arm"]] = (
            heads[first["false_arm"]],
            heads[first["true_arm"]],
        )

    def wrong_target(case, nodes):
        first = nodes["guards"][0]
        case[4][6][first["match"]] = first["match"]

    def wrong_scrutinee_resolution(case, nodes):
        first = nodes["guards"][0]
        case[4][1][first["scrutinee"]] = case[4][1][nodes["parameter"]]

    def wrong_final_resolution(case, nodes):
        case[4][1][nodes["final_place"]] = case[4][1][nodes["parameter"]]

    for mutate in (
        swap_arm_tags,
        wrong_target,
        wrong_scrutinee_resolution,
        wrong_final_resolution,
    ):
        rejected(mutate)


def assert_user_call_clean_facts(library):
    data = user_call_fixture()
    case = parsed(library, data)
    callee = find_function_by_text(data, case[4], case[5], b"classify")
    caller = find_function_by_text(data, case[4], case[5], b"caller")
    nodes, user_pairs = user_call_nodes(case[4], caller)
    assert len(user_pairs) == 1
    let_node, call = user_pairs[0]
    named = children_of(case[4], call)[0]
    actual = children_of(case[4], named)[0]
    formal = children_of(case[4], callee)[1]
    outputs = make_outputs(
        library,
        case[5].count,
        scratch_caps=(len(nodes["lets"]) + 1,) * len(SCRATCH_COLUMNS),
    )
    report = invoke(library, case, caller, outputs)
    assert report_tuple(report) == (BODY_CLEAN, U64_MAX, U64_MAX)
    assert diagnostic_tuple(report) == (RULE_NONE, FIX_NONE, U64_MAX)
    facts = outputs[2]
    type_ids, resolved, ordinals, operations, _, _, modes, _ = facts
    assert (
        type_ids[call],
        resolved[call],
        ordinals[call],
        operations[call],
        modes[call],
    ) == (1, callee, 0, OP_NONE, MODE_OWN)
    assert (
        type_ids[named],
        resolved[named],
        ordinals[named],
        operations[named],
        modes[named],
    ) == (0, formal, 0, OP_NONE, MODE_OWN)
    assert resolved[actual] == nodes["parameter"]
    assert type_ids[actual] == 0
    assert modes[actual] == MODE_OWN
    assert ordinals[let_node] == ordinals[call] == 0
    assert ordinals[nodes["return_root"]] == 1
    assert ordinals[nodes["return"]] == 1
    assert_output_guards(outputs)

    second = make_outputs(
        library,
        case[5].count,
        scratch_caps=(len(nodes["lets"]) + 1,) * len(SCRATCH_COLUMNS),
    )
    second_report = invoke(library, case, caller, second)
    assert report_tuple(second_report) == report_tuple(report)
    assert output_snapshot(second) == output_snapshot(outputs)
    assert_output_guards(second)
    return case[5].count


def assert_user_call_failures(library):
    wrong_actual_prefix = b"  let flag: own Bool = ieq<u8>(c, 0_u8);\n"
    cases = (
        (
            user_call_fixture(call_name=b"missing"),
            BODY_UNKNOWN_NAME,
            RULE_TYPE5,
            FIX_DECLARE_BEFORE_USE,
        ),
        (
            user_call_fixture(arguments=b"missing: c"),
            BODY_UNKNOWN_NAME,
            RULE_GRAM11,
            FIX_NAME_ARGUMENTS,
        ),
        (
            user_call_fixture(arguments=b"x: c, x: c"),
            BODY_MALFORMED,
            RULE_GRAM11,
            FIX_NAME_ARGUMENTS,
        ),
        (
            user_call_fixture(arguments=b""),
            BODY_TYPE_MISMATCH,
            RULE_GRAM11,
            FIX_NAME_ARGUMENTS,
        ),
        (
            user_call_fixture(arguments=b"x: c, extra: c"),
            BODY_UNKNOWN_NAME,
            RULE_GRAM11,
            FIX_NAME_ARGUMENTS,
        ),
        (
            user_call_fixture(
                callee_parameters=b"x: own Bool",
                callee_body=b"  return bor<Bool>(x, x);\n",
            ),
            BODY_TYPE_MISMATCH,
            RULE_TYPE5,
            FIX_MATCH_TYPE,
        ),
        (
            user_call_fixture(callee_parameters=b"x: &'r u8"),
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        ),
        (
            user_call_fixture(
                callee_return=b"own u8",
                callee_body=b"  return x;\n",
            ),
            BODY_TYPE_MISMATCH,
            RULE_TYPE5,
            FIX_MATCH_TYPE,
        ),
        (
            user_call_fixture(
                callee_effect=b"traps",
                callee_body=(
                    b"  let next: own u8 = iadd.trap<u8>(x, 1_u8);\n"
                    b"  return ieq<u8>(next, next);\n"
                ),
            ),
            BODY_EFFECT_MISMATCH,
            RULE_EFFECT2,
            FIX_DECLARE_EFFECT,
        ),
        (
            user_call_fixture(
                callee_parameters=b"x: own u8, y: own u8",
                callee_body=b"  return ieq<u8>(x, y);\n",
            ),
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        ),
        (
            user_call_fixture(
                arguments=b"x: flag", caller_prefix=wrong_actual_prefix
            ),
            BODY_TYPE_MISMATCH,
            RULE_TYPE5,
            FIX_MATCH_TYPE,
        ),
    )
    for data, expected, expected_rule, expected_fix in cases:
        case = parsed(library, data)
        caller = find_function_by_text(data, case[4], case[5], b"caller")
        nodes = linear_body_nodes(case[4], caller)
        outputs = make_outputs(
            library,
            case[5].count,
            scratch_caps=(len(nodes["lets"]) + 1,) * len(SCRATCH_COLUMNS),
        )
        report = invoke(library, case, caller, outputs)
        assert report.status == expected, (data, report_tuple(report), expected)
        assert (report.rule, report.fix) == (expected_rule, expected_fix), (
            data,
            diagnostic_tuple(report),
        )
        assert report.related_node == U64_MAX or report.related_node < case[5].count
        assert report.node != U64_MAX
        assert outputs[1].status == FACTS_INVALID_SHAPE
        assert outputs[3].status == FACTS_INVALID_SHAPE
        assert_output_guards(outputs)
        assert_symbol_guards(case[7], case[8])

    nonfunction = (
        b"const classify: u64 = 7_u64;\n"
        b"fn caller (c: own u8) -> own Bool pure {\n"
        b"  let answer: own Bool = classify(x: c);\n"
        b"  return bor<Bool>(answer, answer);\n"
        b"}\n"
    )
    case = parsed(library, nonfunction)
    caller = find_function_by_text(nonfunction, case[4], case[5], b"caller")
    outputs = make_outputs(library, case[5].count)
    report = invoke(library, case, caller, outputs)
    assert report.status == BODY_TYPE_MISMATCH, report_tuple(report)
    assert (report.rule, report.fix, report.related_node) == (
        RULE_TYPE5,
        FIX_MATCH_TYPE,
        children_of(case[4], case[5].root)[0],
    )
    assert_output_guards(outputs)


def assert_earlier_return_is_accepted(library):
    data = earlier_return_fixture()
    case = parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], b"earlier")
    outputs = make_outputs(
        library,
        case[5].count,
        scratch_caps=(3,) * len(SCRATCH_COLUMNS),
    )
    report = invoke(library, case, function, outputs)
    assert report_tuple(report) == (BODY_CLEAN, U64_MAX, U64_MAX)
    assert diagnostic_tuple(report) == (RULE_NONE, FIX_NONE, U64_MAX)
    assert_output_guards(outputs)


def assert_hostile_symbol_tapes(library):
    data = user_call_fixture()

    def run(case):
        caller = find_function_by_text(data, case[4], case[5], b"caller")
        outputs = make_outputs(library, case[5].count)
        report = invoke(library, case, caller, outputs, symbols=case[9])
        assert report.status == BODY_INVALID_SYMBOL_TAPE, report_tuple(report)
        assert outputs[1].status == FACTS_INVALID_SHAPE
        assert outputs[3].status == FACTS_INVALID_SHAPE
        assert_output_guards(outputs)
        assert_symbol_guards(case[7], case[8])

    case = parsed(library, data)
    case[9].status = 1
    run(case)

    case = parsed(library, data)
    original_namespaces = case[9].namespaces
    case[9].namespaces = Buffer(
        original_namespaces.data, case[9].count - 1
    )
    run(case)

    case = parsed(library, data)
    case[7][2][0] = case[3].count
    run(case)

    case = parsed(library, data)
    callee = find_function_by_text(data, case[4], case[5], b"classify")
    global_slot = next(
        slot
        for slot in range(case[9].count)
        if case[7][0][slot] == SYMBOL_VALUE
        and case[7][1][slot] == SYMBOL_NONE
        and case[7][3][slot] == callee
    )
    case[7][3][global_slot] = case[5].count
    run(case)

    case = parsed(library, data)
    callee = find_function_by_text(data, case[4], case[5], b"classify")
    formal = children_of(case[4], callee)[1]
    formal_slot = next(
        slot
        for slot in range(case[9].count)
        if case[7][3][slot] == formal
    )
    case[7][1][formal_slot] = SYMBOL_NONE
    run(case)


def assert_semantic_failures(library):
    cases = (
        (
            fixture(first_operand=b"missing"),
            BODY_UNKNOWN_NAME,
            RULE_TYPE5,
            FIX_DECLARE_BEFORE_USE,
        ),
        (
            fixture(first_binding_type=b"u8"),
            BODY_TYPE_MISMATCH,
            RULE_TYPE5,
            FIX_MATCH_TYPE,
        ),
        (fixture(return_op=b"bxor"), BODY_UNSUPPORTED, RULE_NONE, FIX_NONE),
        (
            fixture(first_literal=b"097_u8"),
            BODY_INVALID_LITERAL,
            RULE_FORM7,
            FIX_CANONICAL_LITERAL,
        ),
        (
            fixture(first_literal=b"1_u16"),
            BODY_TYPE_MISMATCH,
            RULE_TYPE5,
            FIX_MATCH_TYPE,
        ),
        (
            fixture(first_literal=b"256_u8"),
            BODY_INVALID_LITERAL,
            RULE_FORM7,
            FIX_NONE,
        ),
        (
            fixture(first_literal=b"1"),
            BODY_INVALID_LITERAL,
            RULE_FORM5,
            FIX_ADD_LITERAL_TYPE_SUFFIX,
        ),
        (
            fixture(first_binding=b"c"),
            BODY_MALFORMED,
            RULE_TYPE6,
            FIX_RENAME_BINDING,
        ),
        (
            fixture(second_binding=b"ge"),
            BODY_MALFORMED,
            RULE_TYPE6,
            FIX_RENAME_BINDING,
        ),
    )
    for data, expected, expected_rule, expected_fix in cases:
        parsed_case = parsed(library, data)
        function = find_function_by_text(
            data, parsed_case[4], parsed_case[5], b"lexer_is_lower"
        )
        outputs = make_outputs(library, parsed_case[5].count)
        report = invoke(library, parsed_case, function, outputs)
        assert report.status == expected, (data, report.status, expected)
        assert (report.rule, report.fix) == (expected_rule, expected_fix)
        assert report.related_node == U64_MAX or report.related_node < parsed_case[5].count
        if data == fixture(first_binding_type=b"u8"):
            nodes = body_nodes(parsed_case[4], function)
            assert report.node == nodes["first_binding_value"]
            assert report.related_node == nodes["first_binding_type"]
        assert report.node != U64_MAX
        assert outputs[1].status == FACTS_INVALID_SHAPE
        assert outputs[3].status == FACTS_INVALID_SHAPE
        assert_output_guards(outputs)

    unsupported_u8_local = (
        b"fn unsupported_local (c: own u8) -> own Bool pure {\n"
        b"  let x: own u8 = iadd.wrap<u8>(c, 1_u8);\n"
        b"  return ieq<u8>(x, x);\n"
        b"}\n"
    )
    unsupported_case = parsed(library, unsupported_u8_local)
    unsupported_function = find_function_by_text(
        unsupported_u8_local,
        unsupported_case[4],
        unsupported_case[5],
        b"unsupported_local",
    )
    unsupported_outputs = make_outputs(
        library,
        unsupported_case[5].count,
        scratch_caps=(2,) * len(SCRATCH_COLUMNS),
    )
    unsupported_report = invoke(
        library,
        unsupported_case,
        unsupported_function,
        unsupported_outputs,
    )
    assert unsupported_report.status == BODY_UNSUPPORTED
    assert diagnostic_tuple(unsupported_report) == (RULE_NONE, FIX_NONE, U64_MAX)
    assert_output_guards(unsupported_outputs)

    poison_data = fixture(first_operand=b"missing")
    poison_case = parsed(library, poison_data)
    poison_function = find_function_by_text(
        poison_data, poison_case[4], poison_case[5], b"lexer_is_lower"
    )
    poison_results = []
    for poison in (0x1234, 0xABCDEF):
        poison_outputs = make_outputs(library, poison_case[5].count)
        for tape in (poison_outputs[1], poison_outputs[3]):
            tape.status = 99
            tape.node = poison
            tape.related = poison + 1
        poison_report = invoke(
            library, poison_case, poison_function, poison_outputs
        )
        poison_results.append(
            (
                report_tuple(poison_report),
                (
                    poison_outputs[1].status,
                    poison_outputs[1].node,
                    poison_outputs[1].related,
                ),
                (
                    poison_outputs[3].status,
                    poison_outputs[3].node,
                    poison_outputs[3].related,
                ),
            )
        )
        assert_output_guards(poison_outputs)
    assert poison_results[0] == poison_results[1]


def assert_u8_literal_classification(library):
    cases = (
        (b"0_u8", U8_LITERAL_READY, 0),
        (b"97_u8", U8_LITERAL_READY, 97),
        (b"255_u8", U8_LITERAL_READY, 255),
        (b"1_u16", U8_LITERAL_WRONG_TYPE, U64_MAX),
        (b"1", U8_LITERAL_MISSING_SUFFIX, U64_MAX),
        (b"097_u8", U8_LITERAL_NONCANONICAL, U64_MAX),
        (b"256_u8", U8_LITERAL_INVALID_VALUE, U64_MAX),
    )
    for spelling, expected_status, expected_value in cases:
        data = fixture(first_literal=spelling)
        parsed_case = parsed(library, data)
        function = find_function_by_text(
            data, parsed_case[4], parsed_case[5], b"lexer_is_lower"
        )
        literal = body_nodes(parsed_case[4], function)["first_literal"]
        token = parsed_case[4][1][literal]
        result = library.semantic_body_parse_u8_literal(
            parsed_case[1], ctypes.byref(parsed_case[3]), token
        )
        assert (result.status, result.value) == (
            expected_status,
            expected_value,
        ), (spelling, result.status, result.value)

    data = fixture()
    parsed_case = parsed(library, data)
    malformed_ordinal = library.semantic_body_parse_u8_literal(
        parsed_case[1],
        ctypes.byref(parsed_case[3]),
        parsed_case[3].count,
    )
    assert (malformed_ordinal.status, malformed_ordinal.value) == (
        U8_LITERAL_MALFORMED,
        U64_MAX,
    )

    function = find_function_by_text(
        data, parsed_case[4], parsed_case[5], b"lexer_is_lower"
    )
    literal = body_nodes(parsed_case[4], function)["first_literal"]
    token = parsed_case[4][1][literal]
    original_kind = parsed_case[2][0][token]
    parsed_case[2][0][token] = 1
    malformed_kind = library.semantic_body_parse_u8_literal(
        parsed_case[1], ctypes.byref(parsed_case[3]), token
    )
    assert (malformed_kind.status, malformed_kind.value) == (
        U8_LITERAL_MALFORMED,
        U64_MAX,
    )
    malformed_outputs = make_outputs(library, parsed_case[5].count)
    malformed_report = invoke(
        library, parsed_case, function, malformed_outputs
    )
    assert report_tuple(malformed_report) == (
        BODY_MALFORMED,
        literal,
        token,
    )
    assert diagnostic_tuple(malformed_report) == (
        RULE_NONE,
        FIX_NONE,
        U64_MAX,
    )
    assert_output_guards(malformed_outputs)
    parsed_case[2][0][token] = original_kind


def assert_capacity_and_input_guards(library):
    data = fixture()
    case = parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], b"lexer_is_lower")

    for short in range(len(TYPE_COLUMNS)):
        capacities = [2] * len(TYPE_COLUMNS)
        capacities[short] = 1
        outputs = make_outputs(library, case[5].count, type_caps=tuple(capacities))
        report = invoke(library, case, function, outputs)
        assert report.status == BODY_CAPACITY, ("type", short, report.status)
        assert outputs[1].count == 0
        assert outputs[1].status == FACTS_CAPACITY
        assert_output_guards(outputs)

    for short in range(len(NODE_COLUMNS)):
        capacities = [case[5].count] * len(NODE_COLUMNS)
        capacities[short] -= 1
        outputs = make_outputs(library, case[5].count, fact_caps=tuple(capacities))
        report = invoke(library, case, function, outputs)
        assert report.status == BODY_CAPACITY, ("fact", short, report.status)
        assert outputs[3].count == 0
        assert outputs[3].status == FACTS_CAPACITY
        assert_output_guards(outputs)

    for short in range(len(SCRATCH_COLUMNS)):
        capacities = [3] * len(SCRATCH_COLUMNS)
        capacities[short] = 2
        outputs = make_outputs(library, case[5].count, scratch_caps=tuple(capacities))
        report = invoke(library, case, function, outputs)
        assert report.status == BODY_CAPACITY, ("scratch", short, report.status)
        assert outputs[1].count == 0
        assert outputs[3].count == 0
        assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    report = invoke(library, case, case[5].count, outputs)
    assert report.status == BODY_INVALID_FUNCTION
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    case[6].status = 1
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_VALIDATION
    case[6].status = 0
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    case[3].kinds.length = case[3].count - 1
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_TOKEN_TAPE
    case[3].kinds.length += 1
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    outputs[1].count = 1
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_TYPE_TAPE
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    outputs[3].count = 1
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_NODE_FACTS
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    outputs[6].count = 4
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_SCRATCH
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    original_first = case[4][4][function]
    case[4][4][function] = case[5].count
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_INVALID_FUNCTION
    case[4][4][function] = original_first
    assert_output_guards(outputs)

    outputs = make_outputs(library, case[5].count)
    return_node = body_nodes(case[4], function)["return"]
    original_return_next = case[4][6][return_node]
    case[4][6][return_node] = return_node
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_MALFORMED
    assert report.node == return_node
    case[4][6][return_node] = original_return_next
    assert_output_guards(outputs)


def global_lookup_function(library, data, parsed_case, name):
    source = parsed_case[1]
    token_storage = parsed_case[2]
    tokens = parsed_case[3]
    ast = parsed_case[5]
    top_level_count = len(children_of(parsed_case[4], ast.root))
    symbol_storage, symbol_physical, symbols = make_symbols(
        (top_level_count,) * 4
    )
    reset(library, symbols)
    globals_report = SemanticGlobalsReport(99, 99, 99)
    library.semantic_index_globals(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(symbols),
        ctypes.byref(globals_report),
    )
    assert globals_report.status == 0
    _, starts, ends = token_storage
    candidates = [
        token
        for token in range(tokens.count)
        if data[starts[token] : ends[token]] == name
    ]
    assert candidates
    found = None
    for token in candidates:
        status, slot = find(
            library,
            source,
            tokens,
            symbols,
            SYMBOL_VALUE,
            SYMBOL_NONE,
            token,
        )
        if status == SYMBOL_CLEAN:
            found = symbol_storage[3][slot]
            break
    assert found is not None
    assert symbol_storage[3][slot] == found
    assert symbol_storage[0][symbol_physical] != 0
    return found


def assert_current_compiler(library):
    data = compiler_source().encode("ascii")
    case = parsed(library, data)
    lower_function = global_lookup_function(
        library, data, case, b"lexer_is_lower"
    )
    assert case[4][0][lower_function] == AST["AstFunction"]
    outputs = make_outputs(library, case[5].count)
    report = invoke(library, case, lower_function, outputs)
    assert (report.status, report.node, report.related) == (
        BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    nodes = body_nodes(case[4], lower_function)
    assert outputs[2][3][nodes["first_call"]] == OP_IGE
    assert outputs[2][3][nodes["second_call"]] == OP_ILE
    assert outputs[2][3][nodes["return_call"]] == OP_BAND
    assert outputs[2][4][nodes["first_literal"]] == 97
    assert outputs[2][4][nodes["second_literal"]] == 122
    assert_output_guards(outputs)

    space_function = global_lookup_function(
        library, data, case, b"lexer_is_space"
    )
    assert case[4][0][space_function] == AST["AstFunction"]
    space_outputs = make_outputs(
        library,
        case[5].count,
        scratch_caps=(5,) * len(SCRATCH_COLUMNS),
    )
    space_report = invoke(library, case, space_function, space_outputs)
    assert report_tuple(space_report) == (BODY_CLEAN, U64_MAX, U64_MAX)
    space_nodes = space_body_nodes(case[4], space_function)
    space_operations = space_outputs[2][3]
    assert space_operations[space_nodes["horizontal_call"]] == OP_IEQ
    assert space_operations[space_nodes["control_ge_call"]] == OP_IGE
    assert space_operations[space_nodes["control_le_call"]] == OP_ILE
    assert space_operations[space_nodes["control_call"]] == OP_BAND
    assert space_operations[space_nodes["return_call"]] == OP_BOR
    assert space_outputs[2][4][space_nodes["horizontal_literal"]] == 32
    assert space_outputs[2][4][space_nodes["control_ge_literal"]] == 9
    assert space_outputs[2][4][space_nodes["control_le_literal"]] == 13
    assert_output_guards(space_outputs)

    tail_summaries = []
    for name, expected_user_calls in (
        (b"lexer_is_ident_tail", 2),
        (b"lexer_is_type_tail", 3),
        (b"lexer_is_number_tail", 3),
    ):
        function = find_function_by_text(data, case[4], case[5], name)
        nodes, user_pairs = user_call_nodes(case[4], function)
        assert len(user_pairs) == expected_user_calls
        scratch_capacity = len(nodes["lets"]) + 1
        outputs = make_outputs(
            library,
            case[5].count,
            scratch_caps=(scratch_capacity,) * len(SCRATCH_COLUMNS),
        )
        report = invoke(library, case, function, outputs)
        assert report_tuple(report) == (BODY_CLEAN, U64_MAX, U64_MAX), (
            name,
            report_tuple(report),
        )
        assert diagnostic_tuple(report) == (RULE_NONE, FIX_NONE, U64_MAX)
        facts = outputs[2]
        type_ids, resolved, ordinals, operations, _, _, modes, _ = facts
        for ordinal, (let_node, initializer) in enumerate(
            zip(nodes["lets"], nodes["initializers"])
        ):
            assert ordinals[initializer] == ordinal, (name, initializer)
            assert ordinals[let_node] == ordinal, (name, let_node)
            assert type_ids[initializer] == 1
            assert type_ids[let_node] == 1
            assert modes[initializer] == MODE_OWN
            assert modes[let_node] == MODE_OWN
        return_ordinal = len(nodes["lets"])
        assert ordinals[nodes["return_root"]] == return_ordinal
        assert ordinals[nodes["return"]] == return_ordinal
        for _, call in user_pairs:
            head = case[4][1][call]
            callee_name = data[case[2][1][head] : case[2][2][head]]
            callee = find_function_by_text(
                data, case[4], case[5], callee_name
            )
            formal = children_of(case[4], callee)[1]
            arguments = children_of(case[4], call)
            assert len(arguments) == 1
            named = arguments[0]
            actual = children_of(case[4], named)[0]
            assert resolved[call] == callee
            assert operations[call] == OP_NONE
            assert resolved[named] == formal
            assert ordinals[named] == 0
            assert type_ids[named] == 0
            assert modes[named] == MODE_OWN
            assert type_ids[actual] == 0
            assert modes[actual] == MODE_OWN
        assert_output_guards(outputs)
        tail_summaries.append((name, len(nodes["lets"]), expected_user_calls))
    return (case[3].count, case[5].count)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        focused_nodes = assert_clean_facts(library)
        focused_space_nodes = assert_space_clean_facts(library)
        focused_user_nodes = assert_user_call_clean_facts(library)
        symbol_nodes = assert_symbol_clean_facts(library)
        assert_semantic_failures(library)
        assert_u8_literal_classification(library)
        assert_space_failures(library)
        assert_user_call_failures(library)
        assert_earlier_return_is_accepted(library)
        assert_symbol_failures(library)
        assert_hostile_symbol_tapes(library)
        assert_capacity_and_input_guards(library)
        compiler_counts = assert_current_compiler(library)
        print(
            "semantic body: lexer_is_lower and lexer_is_space "
            "typed/resolved deterministically; "
            f"focused nodes={focused_nodes}/{focused_space_nodes}; "
            f"user-call nodes={focused_user_nodes}; "
            f"symbol nodes={symbol_nodes}; "
            f"compiler tokens/nodes={compiler_counts}; "
            "current ident/type/number/symbol predicates resolved; "
            "hostile names/types/ops/order/literals/symbols/capacities fail closed"
        )


if __name__ == "__main__":
    main()
