#!/usr/bin/env python3
"""Lower real linear scalar functions from retained semantic facts to LLVM."""

import ctypes
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_lexer import Buffer, TokenTape
from test_parser import AstTape, children_of
from test_semantic_body import (
    AST,
    BODY_CLEAN,
    configure as configure_semantic_body,
    find_function_by_text,
    invoke,
    make_outputs,
    parsed,
    symbol_body_nodes,
)
from test_semantic_facts import NodeFacts, TypeTape
from test_llvm_text import (
    BYTE_CLEAN,
    BYTE_INVALID_STATE,
    BYTE_NEED_CAPACITY,
    GUARD,
    POISON,
    ByteTape,
    make_output,
)

import democ


ROOT = Path(__file__).resolve().parents[1]
HERE = ROOT / "compiler"
U64_MAX = (1 << 64) - 1
OP_NONE = 0
OP_IEQ = 5
MODE_NONE = 0
PRELUDE_UNKNOWN = 0
PRELUDE_TRUE = 1
PRELUDE_FALSE = 2


def real_profile_source():
    data = (HERE / "src" / "lexer.wf").read_bytes()
    return data[: data.index(b"\nfn lexer_scan_ident")]


SOURCE = real_profile_source()
ORDER = (
    b"lexer_is_lower",
    b"lexer_is_upper",
    b"lexer_is_digit",
    b"lexer_is_space",
    b"lexer_is_ident_tail",
    b"lexer_is_type_tail",
    b"lexer_is_number_tail",
    b"lexer_is_symbol",
)

SYMBOL_GUARD_BYTES = (40, 41, 123, 125, 60, 62, 58, 59, 44, 61, 91, 93)


def expected_symbol_ir():
    lines = [
        b"define i1 @lexer_is_symbol(i8 %p0) {",
        b"entry:",
        b"  %v0 = icmp eq i8 %p0, 40",
        b"  br i1 %v0, label %bb0, label %bb1",
        b"bb0:",
        b"  ret i1 true",
    ]
    for ordinal, literal in enumerate(SYMBOL_GUARD_BYTES[1:], start=1):
        lines.extend(
            (
                f"bb{ordinal}:".encode(),
                f"  %v{ordinal} = icmp eq i8 %p0, {literal}".encode(),
                (
                    f"  br i1 %v{ordinal}, label %bb0, "
                    f"label %bb{ordinal + 1}"
                ).encode(),
            )
        )
    lines.extend(
        (
            b"bb12:",
            b"  %v12 = icmp eq i8 %p0, 46",
            b"  ret i1 %v12",
            b"}",
        )
    )
    return b"\n".join(lines) + b"\n"

EXPECTED = {
    b"lexer_is_lower": b"""define i1 @lexer_is_lower(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 97
  %v1 = icmp ule i8 %p0, 122
  %v2 = and i1 %v0, %v1
  ret i1 %v2
}
""",
    b"lexer_is_upper": b"""define i1 @lexer_is_upper(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 65
  %v1 = icmp ule i8 %p0, 90
  %v2 = and i1 %v0, %v1
  ret i1 %v2
}
""",
    b"lexer_is_digit": b"""define i1 @lexer_is_digit(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 48
  %v1 = icmp ule i8 %p0, 57
  %v2 = and i1 %v0, %v1
  ret i1 %v2
}
""",
    b"lexer_is_space": b"""define i1 @lexer_is_space(i8 %p0) {
entry:
  %v0 = icmp eq i8 %p0, 32
  %v1 = icmp uge i8 %p0, 9
  %v2 = icmp ule i8 %p0, 13
  %v3 = and i1 %v1, %v2
  %v4 = or i1 %v0, %v3
  ret i1 %v4
}
""",
    b"lexer_is_ident_tail": b"""define i1 @lexer_is_ident_tail(i8 %p0) {
entry:
  %v0 = call i1 @lexer_is_lower(i8 %p0)
  %v1 = call i1 @lexer_is_digit(i8 %p0)
  %v2 = icmp eq i8 %p0, 95
  %v3 = or i1 %v0, %v1
  %v4 = or i1 %v3, %v2
  ret i1 %v4
}
""",
    b"lexer_is_type_tail": b"""define i1 @lexer_is_type_tail(i8 %p0) {
entry:
  %v0 = call i1 @lexer_is_lower(i8 %p0)
  %v1 = call i1 @lexer_is_upper(i8 %p0)
  %v2 = call i1 @lexer_is_digit(i8 %p0)
  %v3 = or i1 %v0, %v1
  %v4 = or i1 %v3, %v2
  ret i1 %v4
}
""",
    b"lexer_is_number_tail": b"""define i1 @lexer_is_number_tail(i8 %p0) {
entry:
  %v0 = call i1 @lexer_is_lower(i8 %p0)
  %v1 = call i1 @lexer_is_upper(i8 %p0)
  %v2 = call i1 @lexer_is_digit(i8 %p0)
  %v3 = icmp eq i8 %p0, 95
  %v4 = icmp eq i8 %p0, 46
  %v5 = icmp eq i8 %p0, 45
  %v6 = or i1 %v0, %v1
  %v7 = or i1 %v6, %v2
  %v8 = or i1 %v7, %v3
  %v9 = or i1 %v8, %v4
  %v10 = or i1 %v9, %v5
  ret i1 %v10
}
""",
    b"lexer_is_symbol": expected_symbol_ir(),
}


def compiler_source_isolated():
    names = [
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    ]
    excluded = {"src/frontend.wf", "src/llvm_scalar.wf", "src/llvm_linear.wf"}
    paths = [HERE / name for name in names if name not in excluded]
    paths.append(HERE / "src" / "llvm_linear.wf")
    return "\n\n".join(path.read_text().rstrip("\n") for path in paths) + "\n"


def build_library(directory):
    source = compiler_source_isolated()
    ir = democ.compile_program(source, alias=False)
    forbidden = [
        marker
        for marker in (
            " noalias",
            " readonly",
            " dereferenceable(",
            " willreturn",
            "!alias.scope",
            "!noalias",
            " memory(",
        )
        if marker in ir
    ]
    assert not forbidden, forbidden
    ll = directory / "llvm_linear_stage0.ll"
    library_path = directory / (
        "llvm_linear_stage0.dylib" if sys.platform == "darwin" else "llvm_linear_stage0.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected the linear emitter:\n{result.stderr}")
    return ctypes.CDLL(str(library_path))


def configure(library):
    configure_semantic_body(library)
    library.lexer_run.argtypes = [Buffer, ctypes.POINTER(TokenTape)]
    library.lexer_run.restype = None
    library.parser_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
    ]
    library.parser_run.restype = None
    library.ast_validate.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
    ]
    library.ast_validate.restype = None
    signature = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(ByteTape),
    ]
    library.llvm_linear_append_function.argtypes = signature
    library.llvm_linear_append_function.restype = None
    library.llvm_linear_emit_function.argtypes = signature
    library.llvm_linear_emit_function.restype = None


def analyze(library, parsed_case, name):
    ast = parsed_case[5]
    function = find_function_by_text(SOURCE, parsed_case[4], ast, name)
    direct = children_of(parsed_case[4], function)
    statements = children_of(parsed_case[4], direct[-1])
    scratch_capacity = 1 + sum(
        parsed_case[4][0][statement] == AST["AstLet"]
        for statement in statements
    )
    outputs = make_outputs(
        library,
        ast.count,
        scratch_caps=(scratch_capacity,) * 5,
    )
    report = invoke(library, parsed_case, function, outputs)
    assert report.status == BODY_CLEAN, (name, report.status, report.node, report.related)
    return function, outputs


def facts_from(outputs):
    return outputs[1], outputs[3]


def call(library, parsed_case, analyzed, out, *, append):
    function, outputs = analyzed
    types, facts = facts_from(outputs)
    emitter = (
        library.llvm_linear_append_function
        if append
        else library.llvm_linear_emit_function
    )
    emitter(
        parsed_case[1],
        ctypes.byref(parsed_case[3]),
        ctypes.byref(parsed_case[5]),
        function,
        ctypes.byref(types),
        ctypes.byref(facts),
        ctypes.byref(out),
    )


def output_bytes(storage, out):
    return bytes(storage[: min(out.count, len(storage) - 1)])


def assert_single_modes(library, parsed_case, analyzed, expected):
    measured_storage, measured = make_output(0)
    call(library, parsed_case, analyzed, measured, append=False)
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(expected),
    ), (measured.status, measured.count, len(expected), expected.splitlines()[0])
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(expected))
    call(library, parsed_case, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(expected))
    assert output_bytes(exact_storage, exact) == expected
    assert exact_storage[len(expected)] == GUARD

    short_capacity = len(expected) - 1
    short_storage, short = make_output(short_capacity)
    call(library, parsed_case, analyzed, short, append=False)
    assert (short.status, short.count) == (BYTE_NEED_CAPACITY, len(expected))
    assert output_bytes(short_storage, short) == expected[:short_capacity]
    assert short_storage[short_capacity] == GUARD

    exact.count = U64_MAX
    exact.status = BYTE_INVALID_STATE
    for index in range(len(expected)):
        exact_storage[index] = POISON
    call(library, parsed_case, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(expected))
    assert output_bytes(exact_storage, exact) == expected
    assert exact_storage[len(expected)] == GUARD


def assert_every_short(library, parsed_case, analyzed, expected):
    for capacity in range(len(expected)):
        storage, out = make_output(capacity)
        call(library, parsed_case, analyzed, out, append=False)
        assert (out.status, out.count) == (
            BYTE_NEED_CAPACITY,
            len(expected),
        ), (capacity, out.status, out.count)
        assert output_bytes(storage, out) == expected[:capacity]
        assert storage[capacity] == GUARD


def assert_symbol_ir_shape(ir):
    value_ids = [int(raw) for raw in re.findall(rb"%v([0-9]+) =", ir)]
    block_ids = [int(raw) for raw in re.findall(rb"(?m)^bb([0-9]+):$", ir)]
    assert value_ids == list(range(13)), value_ids
    assert block_ids == list(range(13)), block_ids
    assert ir.count(b"label %bb0") == 12
    assert ir.count(b"bb0:\n  ret i1 true\n") == 1
    assert b"\n  br label " not in ir
    for forbidden in (b" phi ", b" alloca ", b" load ", b" store "):
        assert forbidden not in ir


def assert_append_matrix(library, parsed_case, analyzed_by_name):
    combined = b"".join(EXPECTED[name] for name in ORDER)
    analyzed = [analyzed_by_name[name] for name in ORDER]
    dependencies = {
        b"lexer_is_ident_tail": (
            b"lexer_is_lower",
            b"lexer_is_digit",
        ),
        b"lexer_is_type_tail": (
            b"lexer_is_lower",
            b"lexer_is_upper",
            b"lexer_is_digit",
        ),
        b"lexer_is_number_tail": (
            b"lexer_is_lower",
            b"lexer_is_upper",
            b"lexer_is_digit",
        ),
    }
    for caller, callees in dependencies.items():
        caller_offset = combined.index(b"define i1 @" + caller)
        for callee in callees:
            assert combined.index(b"define i1 @" + callee) < caller_offset

    exact_storage, exact = make_output(len(combined))
    observed = 0
    for name, item in zip(ORDER, analyzed):
        call(library, parsed_case, item, exact, append=True)
        observed += len(EXPECTED[name])
        assert (exact.status, exact.count) == (BYTE_CLEAN, observed)
    assert output_bytes(exact_storage, exact) == combined
    assert exact_storage[len(combined)] == GUARD

    measured_storage, measured = make_output(0)
    observed = 0
    for name, item in zip(ORDER, analyzed):
        call(library, parsed_case, item, measured, append=True)
        observed += len(EXPECTED[name])
        assert (measured.status, measured.count) == (BYTE_NEED_CAPACITY, observed)
    assert measured_storage[0] == GUARD

    for capacity in range(len(combined)):
        storage, out = make_output(capacity)
        for item in analyzed:
            call(library, parsed_case, item, out, append=True)
        assert (out.status, out.count) == (BYTE_NEED_CAPACITY, len(combined))
        assert output_bytes(storage, out) == combined[:capacity]
        assert storage[capacity] == GUARD
    return combined


def first_call(parsed_case, function):
    columns = parsed_case[4]
    block = children_of(columns, function)[5]
    first_let = children_of(columns, block)[0]
    return children_of(columns, first_let)[3]


def direct_user_calls(parsed_case, function):
    columns = parsed_case[4]
    direct = children_of(columns, function)
    statements = children_of(columns, direct[-1])
    pairs = []
    for statement in statements[:-1]:
        initializer = children_of(columns, statement)[3]
        if columns[0][initializer] == AST["AstUserCall"]:
            pairs.append((statement, initializer))
    return pairs


def user_call_parts(parsed_case, function, ordinal=0):
    statement, call_node = direct_user_calls(parsed_case, function)[ordinal]
    named = children_of(parsed_case[4], call_node)[0]
    actual = children_of(parsed_case[4], named)[0]
    return statement, call_node, named, actual


def assert_fact_driven(library, parsed_case, analyzed):
    function, outputs = analyzed
    fact_storage = outputs[2]
    call_node = first_call(parsed_case, function)
    literal = children_of(parsed_case[4], call_node)[2]
    prior_operation = fact_storage[3][call_node]
    prior_constant = fact_storage[4][literal]
    fact_storage[3][call_node] = OP_IEQ
    fact_storage[4][literal] = 42
    try:
        storage, out = make_output(len(EXPECTED[b"lexer_is_lower"]) + 8)
        call(library, parsed_case, analyzed, out, append=False)
        observed = output_bytes(storage, out)
        assert out.status == BYTE_CLEAN
        assert b"%v0 = icmp eq i8 %p0, 42\n" in observed
    finally:
        fact_storage[3][call_node] = prior_operation
        fact_storage[4][literal] = prior_constant

    source_storage = parsed_case[0]
    offset = SOURCE.index(b"ige<u8>")
    original = bytes(source_storage[offset : offset + 3])
    for index, byte in enumerate(b"zzz"):
        source_storage[offset + index] = byte
    try:
        storage, out = make_output(len(EXPECTED[b"lexer_is_lower"]))
        call(library, parsed_case, analyzed, out, append=False)
        assert (out.status, out.count) == (
            BYTE_CLEAN,
            len(EXPECTED[b"lexer_is_lower"]),
        )
        assert output_bytes(storage, out) == EXPECTED[b"lexer_is_lower"]
    finally:
        for index, byte in enumerate(original):
            source_storage[offset + index] = byte


def assert_resolved_callee_fact_driven(library, parsed_case, analyzed):
    function, outputs = analyzed
    _, call_node, named, _ = user_call_parts(parsed_case, function)
    resolutions = outputs[2][1]
    callee = resolutions[call_node]
    assert callee != U64_MAX
    formal = children_of(parsed_case[4], callee)[1]
    assert resolutions[named] == formal

    call_head = parsed_case[4][1][call_node]
    start = parsed_case[2][1][call_head]
    end = parsed_case[2][2][call_head]
    source_storage = parsed_case[0]
    original = bytes(source_storage[start:end])
    assert original == b"lexer_is_lower"
    replacement = b"x" * len(original)
    for offset, byte in enumerate(replacement):
        source_storage[start + offset] = byte
    try:
        expected = EXPECTED[b"lexer_is_ident_tail"]
        storage, out = make_output(len(expected))
        call(library, parsed_case, analyzed, out, append=False)
        assert (out.status, out.count) == (BYTE_CLEAN, len(expected))
        assert output_bytes(storage, out) == expected
        assert b"call i1 @lexer_is_lower(i8 %p0)" in output_bytes(storage, out)
        assert replacement not in output_bytes(storage, out)
        assert storage[len(expected)] == GUARD
    finally:
        for offset, byte in enumerate(original):
            source_storage[start + offset] = byte


def assert_append_atomicity(library, parsed_case, lower, upper):
    lower_expected = EXPECTED[b"lexer_is_lower"]
    capacity = len(lower_expected) + len(EXPECTED[b"lexer_is_upper"])
    storage, out = make_output(capacity)
    call(library, parsed_case, lower, out, append=True)
    assert (out.status, out.count) == (BYTE_CLEAN, len(lower_expected))
    before = bytes(storage[:capacity])

    upper_function, upper_outputs = upper
    call_node = first_call(parsed_case, upper_function)
    operations = upper_outputs[2][3]
    prior = operations[call_node]
    operations[call_node] = OP_NONE
    try:
        call(library, parsed_case, upper, out, append=True)
    finally:
        operations[call_node] = prior
    assert (out.status, out.count) == (BYTE_INVALID_STATE, len(lower_expected))
    assert bytes(storage[:capacity]) == before
    assert storage[capacity] == GUARD

    count = out.count
    snapshot = bytes(storage[:capacity])
    call(library, parsed_case, upper, out, append=True)
    assert (out.status, out.count) == (BYTE_INVALID_STATE, count)
    assert bytes(storage[:capacity]) == snapshot

    for status, count in (
        (99, 0),
        (BYTE_CLEAN, capacity + 1),
        (BYTE_NEED_CAPACITY, 0),
        (BYTE_NEED_CAPACITY, U64_MAX - 1),
    ):
        hostile_storage, hostile = make_output(capacity)
        hostile.status = status
        hostile.count = count
        call(library, parsed_case, lower, hostile, append=True)
        assert (hostile.status, hostile.count) == (BYTE_INVALID_STATE, count)
        assert all(byte == POISON for byte in hostile_storage[:capacity])
        assert hostile_storage[capacity] == GUARD


def assert_direct_hostile_facts_and_atomicity(
    library, parsed_case, analyzed_by_name
):
    ident = analyzed_by_name[b"lexer_is_ident_tail"]
    lower = analyzed_by_name[b"lexer_is_lower"]
    upper = analyzed_by_name[b"lexer_is_upper"]
    function, outputs = ident
    statement, call_node, named, actual = user_call_parts(
        parsed_case, function
    )
    fact_storage = outputs[2]
    resolved = fact_storage[1]
    ordinals = fact_storage[2]
    caller_parameter = children_of(parsed_case[4], function)[1]
    upper_function = upper[0]
    upper_formal = children_of(parsed_case[4], upper_function)[1]
    expected = EXPECTED[b"lexer_is_ident_tail"]
    prefix = EXPECTED[b"lexer_is_lower"]

    mutations = (
        ("callee-none", resolved, call_node, U64_MAX),
        ("callee-out-of-range", resolved, call_node, parsed_case[5].count),
        ("callee-nonfunction", resolved, call_node, call_node),
        ("named-none", resolved, named, U64_MAX),
        ("named-caller-parameter", resolved, named, caller_parameter),
        ("named-other-formal", resolved, named, upper_formal),
        ("named-ordinal", ordinals, named, 1),
        ("call-ordinal", ordinals, call_node, ordinals[call_node] + 1),
        ("let-ordinal", ordinals, statement, ordinals[statement] + 1),
        ("actual-resolution", resolved, actual, upper_formal),
    )
    for label, column, node, replacement in mutations:
        prior = column[node]
        column[node] = replacement
        try:
            storage, out = make_output(len(expected))
            call(library, parsed_case, ident, out, append=False)
            assert (out.status, out.count) == (
                BYTE_INVALID_STATE,
                0,
            ), (label, out.status, out.count)
            assert all(byte == POISON for byte in storage[:-1]), label
            assert storage[-1] == GUARD

            capacity = len(prefix) + len(expected)
            append_storage, appended = make_output(capacity)
            call(library, parsed_case, lower, appended, append=True)
            assert (appended.status, appended.count) == (
                BYTE_CLEAN,
                len(prefix),
            )
            before = bytes(append_storage[:capacity])
            call(library, parsed_case, ident, appended, append=True)
            assert (appended.status, appended.count) == (
                BYTE_INVALID_STATE,
                len(prefix),
            ), (label, appended.status, appended.count)
            assert bytes(append_storage[:capacity]) == before, label
            assert append_storage[capacity] == GUARD
        finally:
            column[node] = prior


def assert_symbol_hostile_facts_and_atomicity(
    library, parsed_case, analyzed_by_name
):
    symbol = analyzed_by_name[b"lexer_is_symbol"]
    prefix_item = analyzed_by_name[b"lexer_is_lower"]
    function, outputs = symbol
    nodes = symbol_body_nodes(parsed_case[4], function)
    guard = nodes["guards"][0]
    last_guard = nodes["guards"][-1]
    fact_storage = outputs[2]
    type_ids = fact_storage[0]
    resolved = fact_storage[1]
    ordinals = fact_storage[2]
    operations = fact_storage[3]
    constants = fact_storage[4]
    targets = fact_storage[5]
    modes = fact_storage[6]
    constructors = fact_storage[7]
    expected = EXPECTED[b"lexer_is_symbol"]
    prefix = EXPECTED[b"lexer_is_lower"]

    mutations = (
        ("match-target", targets, guard["match"], guard["match"]),
        ("match-target-oob", targets, guard["match"], parsed_case[5].count),
        (
            "last-match-target",
            targets,
            last_guard["match"],
            guard["continuation"],
        ),
        (
            "scrutinee-target",
            targets,
            guard["scrutinee"],
            guard["continuation"],
        ),
        (
            "true-arm-target",
            targets,
            guard["true_arm"],
            guard["continuation"],
        ),
        (
            "false-arm-target",
            targets,
            guard["false_arm"],
            guard["early_return"],
        ),
        (
            "last-false-arm-target",
            targets,
            last_guard["false_arm"],
            guard["continuation"],
        ),
        (
            "true-block-target",
            targets,
            guard["true_block"],
            guard["continuation"],
        ),
        (
            "false-block-target",
            targets,
            guard["false_block"],
            guard["early_return"],
        ),
        (
            "last-false-block-target",
            targets,
            last_guard["false_block"],
            guard["continuation"],
        ),
        (
            "early-return-target",
            targets,
            guard["early_return"],
            guard["continuation"],
        ),
        (
            "constructor-target",
            targets,
            guard["constructor"],
            guard["continuation"],
        ),
        (
            "final-place-target",
            targets,
            nodes["final_place"],
            function,
        ),
        (
            "final-return-target",
            targets,
            nodes["final_return"],
            nodes["final_return"],
        ),
        ("true-arm-ordinal", ordinals, guard["true_arm"], 1),
        ("false-arm-ordinal", ordinals, guard["false_arm"], 0),
        ("true-arm-tag", constructors, guard["true_arm"], PRELUDE_FALSE),
        ("false-arm-tag", constructors, guard["false_arm"], PRELUDE_TRUE),
        ("true-arm-constant", constants, guard["true_arm"], 0),
        ("false-arm-constant", constants, guard["false_arm"], 1),
        ("constructor-type", type_ids, guard["constructor"], 0),
        (
            "constructor-resolution",
            resolved,
            guard["constructor"],
            nodes["parameter"],
        ),
        ("constructor-ordinal", ordinals, guard["constructor"], 0),
        ("constructor-operation", operations, guard["constructor"], OP_IEQ),
        ("constructor-constant", constants, guard["constructor"], 0),
        ("constructor-mode", modes, guard["constructor"], MODE_NONE),
        (
            "constructor-tag",
            constructors,
            guard["constructor"],
            PRELUDE_FALSE,
        ),
        (
            "scrutinee-resolution",
            resolved,
            guard["scrutinee"],
            nodes["parameter"],
        ),
        (
            "final-resolution",
            resolved,
            nodes["final_place"],
            nodes["parameter"],
        ),
        ("final-return-ordinal", ordinals, nodes["final_return"], 11),
    )

    for label, column, node, replacement in mutations:
        prior = column[node]
        column[node] = replacement
        try:
            storage, out = make_output(len(expected))
            call(library, parsed_case, symbol, out, append=False)
            assert (out.status, out.count) == (
                BYTE_INVALID_STATE,
                0,
            ), (label, out.status, out.count)
            assert all(byte == POISON for byte in storage[:-1]), label
            assert storage[-1] == GUARD

            capacity = len(prefix) + len(expected)
            append_storage, appended = make_output(capacity)
            call(library, parsed_case, prefix_item, appended, append=True)
            assert (appended.status, appended.count) == (
                BYTE_CLEAN,
                len(prefix),
            )
            before = bytes(append_storage[:capacity])
            call(library, parsed_case, symbol, appended, append=True)
            assert (appended.status, appended.count) == (
                BYTE_INVALID_STATE,
                len(prefix),
            ), (label, appended.status, appended.count)
            assert bytes(append_storage[:capacity]) == before, label
            assert append_storage[capacity] == GUARD
        finally:
            column[node] = prior


def assert_profile_boundaries(library, parsed_case, space):
    direct_source = b"""fn direct (c: own u8) -> own Bool pure {
  return ieq<u8>(c, 0_u8);
}
"""
    direct_case = parsed(library, direct_source)
    direct_function = find_function_by_text(
        direct_source, direct_case[4], direct_case[5], b"direct"
    )
    direct_outputs = make_outputs(
        library, direct_case[5].count, scratch_caps=(1, 1, 1, 1, 1)
    )
    direct_report = invoke(
        library, direct_case, direct_function, direct_outputs
    )
    assert direct_report.status == BODY_CLEAN
    direct_storage, direct_out = make_output(256)
    call(
        library,
        direct_case,
        (direct_function, direct_outputs),
        direct_out,
        append=False,
    )
    assert (direct_out.status, direct_out.count) == (BYTE_INVALID_STATE, 0)
    assert all(byte == POISON for byte in direct_storage[:-1])
    assert direct_storage[-1] == GUARD

    function, outputs = space
    statements = children_of(
        parsed_case[4], children_of(parsed_case[4], function)[5]
    )
    control_let = statements[3]
    control_call = children_of(parsed_case[4], control_let)[3]
    control_ge_use = children_of(parsed_case[4], control_call)[1]
    resolutions = outputs[2][1]
    prior = resolutions[control_ge_use]
    resolutions[control_ge_use] = control_let
    try:
        storage, out = make_output(len(EXPECTED[b"lexer_is_space"]))
        call(library, parsed_case, space, out, append=False)
        assert (out.status, out.count) == (BYTE_INVALID_STATE, 0)
        assert all(byte == POISON for byte in storage[:-1])
        assert storage[-1] == GUARD
    finally:
        resolutions[control_ge_use] = prior


def assert_selected_hostile_facts(library, parsed_case, analyzed):
    function, outputs = analyzed
    fact_storage = outputs[2]
    call_node = first_call(parsed_case, function)
    literal = children_of(parsed_case[4], call_node)[2]

    def rejected():
        storage, out = make_output(len(EXPECTED[b"lexer_is_lower"]))
        call(library, parsed_case, analyzed, out, append=False)
        assert (out.status, out.count) == (BYTE_INVALID_STATE, 0)
        assert all(byte == POISON for byte in storage[:-1])
        assert storage[-1] == GUARD

    mutations = (
        (fact_storage[3], call_node, OP_NONE),
        (fact_storage[4], literal, 256),
        (fact_storage[2], call_node, 7),
    )
    for column, index, replacement in mutations:
        prior = column[index]
        column[index] = replacement
        try:
            rejected()
        finally:
            column[index] = prior

    facts = outputs[3]
    prior_count = facts.count
    facts.count -= 1
    try:
        rejected()
    finally:
        facts.count = prior_count


def assert_executable_ir(directory, ir):
    ll = directory / "linear_profile.ll"
    library_path = directory / (
        "linear_profile.dylib" if sys.platform == "darwin" else "linear_profile.so"
    )
    ll.write_bytes(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O3"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected generated linear IR:\n{result.stderr}")
    generated = ctypes.CDLL(str(library_path))
    predicates = {
        b"lexer_is_lower": lambda value: 97 <= value <= 122,
        b"lexer_is_upper": lambda value: 65 <= value <= 90,
        b"lexer_is_digit": lambda value: 48 <= value <= 57,
        b"lexer_is_space": lambda value: value == 32 or 9 <= value <= 13,
        b"lexer_is_ident_tail": lambda value: (
            97 <= value <= 122 or 48 <= value <= 57 or value == 95
        ),
        b"lexer_is_type_tail": lambda value: (
            97 <= value <= 122
            or 65 <= value <= 90
            or 48 <= value <= 57
        ),
        b"lexer_is_number_tail": lambda value: (
            97 <= value <= 122
            or 65 <= value <= 90
            or 48 <= value <= 57
            or value in (95, 46, 45)
        ),
        b"lexer_is_symbol": lambda value: (
            value in SYMBOL_GUARD_BYTES or value == 46
        ),
    }
    for name, predicate in predicates.items():
        function = getattr(generated, name.decode())
        function.argtypes = [ctypes.c_uint8]
        function.restype = ctypes.c_bool
        for value in range(256):
            assert function(value) is predicate(value), (name, value)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_library(directory)
        configure(library)
        parsed_case = parsed(library, SOURCE)
        analyzed = {
            name: analyze(library, parsed_case, name)
            for name in ORDER
        }

        for name in ORDER:
            assert_single_modes(library, parsed_case, analyzed[name], EXPECTED[name])
        assert_every_short(
            library,
            parsed_case,
            analyzed[b"lexer_is_symbol"],
            EXPECTED[b"lexer_is_symbol"],
        )
        assert_symbol_ir_shape(EXPECTED[b"lexer_is_symbol"])
        assert_fact_driven(library, parsed_case, analyzed[b"lexer_is_lower"])
        assert_resolved_callee_fact_driven(
            library,
            parsed_case,
            analyzed[b"lexer_is_ident_tail"],
        )
        assert_append_atomicity(
            library,
            parsed_case,
            analyzed[b"lexer_is_lower"],
            analyzed[b"lexer_is_upper"],
        )
        assert_selected_hostile_facts(
            library, parsed_case, analyzed[b"lexer_is_lower"]
        )
        assert_direct_hostile_facts_and_atomicity(
            library, parsed_case, analyzed
        )
        assert_symbol_hostile_facts_and_atomicity(
            library, parsed_case, analyzed
        )
        assert_profile_boundaries(
            library, parsed_case, analyzed[b"lexer_is_space"]
        )
        combined = assert_append_matrix(library, parsed_case, analyzed)
        assert_executable_ir(directory, combined)

        print(
            "linear LLVM: 8 real predicates in dependency order, "
            "exact/measure/all-short/repeat, fact-driven calls, atomic append, "
            "hostile facts/profile boundaries, clang, and all 2048 u8 cases pass"
        )


if __name__ == "__main__":
    main()
