#!/usr/bin/env python3
"""Lower all real fixed-width checked buffer matchers to composable LLVM."""

import ctypes
import random
import signal
import subprocess
import sys
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library
from test_parser import AstTape, children_of
from test_semantic_body import (
    AST,
    SemanticBodyReport,
    configure as configure_semantic_body,
    find_function_by_text,
    make_outputs,
    parsed,
)
from test_semantic_facts import NODE_COLUMNS, TYPE_COLUMNS, NodeFacts, TypeTape
from test_llvm_text import (
    BYTE_CLEAN,
    BYTE_INVALID_STATE,
    BYTE_NEED_CAPACITY,
    GUARD,
    POISON,
    ByteTape,
    make_output,
)


HERE = Path(__file__).resolve().parent
WIDTHS = (3, 4, 6, 7)
U64_MAX = (1 << 64) - 1
BODY_CLEAN = 0
PREFIX = b"checked-prefix:"

EXPECTED_PRELUDE = b"""declare { i64, i1 } @llvm.uadd.with.overflow.i64(i64, i64)
declare void @llvm.trap()

"""

MATCH3_BYTE_REGRESSION = b"""define i1 @lexer_match3({ ptr, i64 } %p0, i64 %p1, i8 %p2, i8 %p3, i8 %p4) {
entry:
  %v0 = extractvalue { ptr, i64 } %p0, 0
  %v1 = extractvalue { ptr, i64 } %p0, 1
  %v2 = call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %p1, i64 1)
  %v3 = extractvalue { i64, i1 } %v2, 0
  %v4 = extractvalue { i64, i1 } %v2, 1
  br i1 %v4, label %bb3, label %bb0
bb0:
  %v5 = call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %p1, i64 2)
  %v6 = extractvalue { i64, i1 } %v5, 0
  %v7 = extractvalue { i64, i1 } %v5, 1
  br i1 %v7, label %bb3, label %bb1
bb1:
  %v8 = icmp ult i64 %v6, %v1
  br i1 %v8, label %bb2, label %bb3
bb2:
  %v9 = getelementptr i8, ptr %v0, i64 %p1
  %v10 = load i8, ptr %v9
  %v11 = getelementptr i8, ptr %v0, i64 %v3
  %v12 = load i8, ptr %v11
  %v13 = getelementptr i8, ptr %v0, i64 %v6
  %v14 = load i8, ptr %v13
  %v15 = icmp eq i8 %v10, %p2
  %v16 = icmp eq i8 %v12, %p3
  %v17 = icmp eq i8 %v14, %p4
  %v18 = and i1 %v15, %v16
  %v19 = and i1 %v18, %v17
  ret i1 %v19
bb3:
  call void @llvm.trap()
  unreachable
}
"""

BAND_PLANS = {
    3: (
        ("first", "e0", "e1"),
        ("return", "first", "e2"),
    ),
    4: (
        ("first", "e0", "e1"),
        ("second", "e2", "e3"),
        ("return", "first", "second"),
    ),
    6: (
        ("pair0", "e0", "e1"),
        ("pair1", "e2", "e3"),
        ("pair2", "e4", "e5"),
        ("first_four", "pair0", "pair1"),
        ("return", "first_four", "pair2"),
    ),
    7: (
        ("pair0", "e0", "e1"),
        ("pair1", "e2", "e3"),
        ("pair2", "e4", "e5"),
        ("first_four", "pair0", "pair1"),
        ("first_six", "first_four", "pair2"),
        ("return", "first_six", "e6"),
    ),
}


def real_matcher_source(width):
    lexer = (HERE / "src" / "lexer.wf").read_bytes()
    marker = f"fn lexer_match{width} ".encode()
    start = lexer.index(marker)
    end = lexer.index(b"\nfn ", start)
    return lexer[start:end].rstrip() + b"\n"


def renamed_match4_source():
    exact = real_matcher_source(4)
    renamed = exact.replace(b"'s", b"'input_region")
    renamed = renamed.replace(b"source", b"input")
    renamed = renamed.replace(b"start", b"offset")
    renamed = renamed.replace(b"d: own u8", b"needle: own u8")
    renamed = renamed.replace(b"b3, d)", b"b3, needle)")
    renamed = renamed.replace(b"p3", b"position3")
    assert renamed != exact
    return renamed


def expected_function(width):
    parameters = ["{ ptr, i64 } %p0", "i64 %p1"]
    parameters.extend(f"i8 %p{ordinal + 2}" for ordinal in range(width))
    lines = [
        f"define i1 @lexer_match{width}({', '.join(parameters)}) {{",
        "entry:",
        "  %v0 = extractvalue { ptr, i64 } %p0, 0",
        "  %v1 = extractvalue { ptr, i64 } %p0, 1",
    ]
    for addend in range(1, width):
        pair = 2 + 3 * (addend - 1)
        total = pair + 1
        overflow = pair + 2
        continuation = addend - 1
        lines.extend(
            (
                f"  %v{pair} = call {{ i64, i1 }} "
                f"@llvm.uadd.with.overflow.i64(i64 %p1, i64 {addend})",
                f"  %v{total} = extractvalue {{ i64, i1 }} %v{pair}, 0",
                f"  %v{overflow} = extractvalue {{ i64, i1 }} %v{pair}, 1",
                f"  br i1 %v{overflow}, label %bb{width}, "
                f"label %bb{continuation}",
            )
        )
        if addend != width - 1:
            lines.append(f"bb{continuation}:")
    final_add_block = width - 2
    bounds = 3 * width - 1
    maximum = 3 * width - 3
    pass_block = width - 1
    lines.extend(
        (
            f"bb{final_add_block}:",
            f"  %v{bounds} = icmp ult i64 %v{maximum}, %v1",
            f"  br i1 %v{bounds}, label %bb{pass_block}, label %bb{width}",
            f"bb{pass_block}:",
        )
    )
    for byte in range(width):
        pointer = 3 * width + 2 * byte
        loaded = pointer + 1
        index = "%p1" if byte == 0 else f"%v{3 * byte}"
        lines.extend(
            (
                f"  %v{pointer} = getelementptr i8, ptr %v0, i64 {index}",
                f"  %v{loaded} = load i8, ptr %v{pointer}",
            )
        )
    registers = {}
    for byte in range(width):
        loaded = 3 * width + 2 * byte + 1
        comparison = 5 * width + byte
        registers[f"e{byte}"] = comparison
        lines.append(
            f"  %v{comparison} = icmp eq i8 %v{loaded}, %p{byte + 2}"
        )
    final_result = None
    for ordinal, (name, left, right) in enumerate(BAND_PLANS[width]):
        result = 6 * width + ordinal
        lines.append(
            f"  %v{result} = and i1 %v{registers[left]}, %v{registers[right]}"
        )
        registers[name] = result
        final_result = result
    lines.extend(
        (
            f"  ret i1 %v{final_result}",
            f"bb{width}:",
            "  call void @llvm.trap()",
            "  unreachable",
            "}",
        )
    )
    return ("\n".join(lines) + "\n").encode()


EXPECTED_FUNCTIONS = {width: expected_function(width) for width in WIDTHS}
EXPECTED_MODULE = EXPECTED_PRELUDE + b"".join(
    EXPECTED_FUNCTIONS[width] for width in WIDTHS
)


def configure(library):
    configure_semantic_body(library)
    semantic_arguments = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_fixed_width_profile.argtypes = semantic_arguments
    library.semantic_buffer_fixed_width_profile.restype = ctypes.c_uint64
    library.semantic_buffer_fixed_width_body_valid.argtypes = semantic_arguments
    library.semantic_buffer_fixed_width_body_valid.restype = ctypes.c_bool
    library.semantic_buffer_fixed_width_run.argtypes = semantic_arguments + [
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.c_void_p,
        ctypes.POINTER(SemanticBodyReport),
    ]
    library.semantic_buffer_fixed_width_run.restype = None

    backend_arguments = semantic_arguments + [
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
    ]
    library.llvm_buffer_fixed_width_facts_valid.argtypes = backend_arguments
    library.llvm_buffer_fixed_width_facts_valid.restype = ctypes.c_bool
    library.llvm_buffer_match3_facts_valid.argtypes = backend_arguments
    library.llvm_buffer_match3_facts_valid.restype = ctypes.c_bool
    library.llvm_buffer_emit_prelude.argtypes = [ctypes.POINTER(ByteTape)]
    library.llvm_buffer_emit_prelude.restype = None
    for name in (
        "llvm_buffer_append_fixed_width",
        "llvm_buffer_emit_fixed_width",
        "llvm_buffer_append_match3",
        "llvm_buffer_emit_match3",
    ):
        function = getattr(library, name)
        function.argtypes = backend_arguments + [ctypes.POINTER(ByteTape)]
        function.restype = None


def analyze(library, data, width):
    case = parsed(library, data)
    name = f"lexer_match{width}".encode()
    function = find_function_by_text(data, case[4], case[5], name)
    outputs = make_outputs(
        library,
        case[5].count,
        type_caps=(4,) * len(TYPE_COLUMNS),
        fact_caps=(case[5].count,) * len(NODE_COLUMNS),
        scratch_caps=(5 * width - 1,) * 5,
    )
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    library.semantic_buffer_fixed_width_run(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        function,
        ctypes.byref(case[6]),
        ctypes.byref(case[9]),
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
        ctypes.byref(outputs[6]),
        ctypes.byref(report),
    )
    assert (report.status, report.node, report.related) == (
        BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    profile = library.semantic_buffer_fixed_width_profile(
        case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
    )
    assert profile == width
    assert library.semantic_buffer_fixed_width_body_valid(
        case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
    )
    assert library.llvm_buffer_fixed_width_facts_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        function,
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
    )
    return {
        "case": case,
        "data": data,
        "width": width,
        "function": function,
        "outputs": outputs,
    }


def call_backend(library, analyzed, out, *, append, match3=False):
    stem = "append" if append else "emit"
    suffix = "match3" if match3 else "fixed_width"
    function = getattr(library, f"llvm_buffer_{stem}_{suffix}")
    case = analyzed["case"]
    outputs = analyzed["outputs"]
    function(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        analyzed["function"],
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
        ctypes.byref(out),
    )


def output_bytes(storage, out):
    return bytes(storage[: min(out.count, len(storage) - 1)])


def assert_output_modes(library, analyzed):
    function_expected = EXPECTED_FUNCTIONS[analyzed["width"]]
    full_expected = EXPECTED_PRELUDE + function_expected

    measured_storage, measured = make_output(0)
    call_backend(library, analyzed, measured, append=False)
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(full_expected),
    )
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(full_expected))
    call_backend(library, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(full_expected))
    assert output_bytes(exact_storage, exact) == full_expected
    assert exact_storage[len(full_expected)] == GUARD

    short_storage, short = make_output(len(full_expected) - 1)
    call_backend(library, analyzed, short, append=False)
    assert (short.status, short.count) == (
        BYTE_NEED_CAPACITY,
        len(full_expected),
    )
    assert output_bytes(short_storage, short) == full_expected[:-1]
    assert short_storage[len(full_expected) - 1] == GUARD

    for index in range(len(full_expected)):
        exact_storage[index] = POISON
    exact.count = U64_MAX
    exact.status = BYTE_INVALID_STATE
    call_backend(library, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(full_expected))
    assert output_bytes(exact_storage, exact) == full_expected

    append_storage, append = make_output(len(function_expected))
    call_backend(library, analyzed, append, append=True)
    assert (append.status, append.count) == (
        BYTE_CLEAN,
        len(function_expected),
    )
    assert output_bytes(append_storage, append) == function_expected
    assert b"declare " not in output_bytes(append_storage, append)
    assert append_storage[len(function_expected)] == GUARD

    if analyzed["width"] == 3:
        wrapper_storage, wrapper = make_output(len(full_expected))
        call_backend(library, analyzed, wrapper, append=False, match3=True)
        assert (wrapper.status, wrapper.count) == (BYTE_CLEAN, len(full_expected))
        assert output_bytes(wrapper_storage, wrapper) == full_expected
        case = analyzed["case"]
        outputs = analyzed["outputs"]
        assert library.llvm_buffer_match3_facts_valid(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            analyzed["function"],
            ctypes.byref(outputs[1]),
            ctypes.byref(outputs[3]),
        )
        append_wrapper_storage, append_wrapper = make_output(
            len(function_expected)
        )
        call_backend(
            library,
            analyzed,
            append_wrapper,
            append=True,
            match3=True,
        )
        assert (append_wrapper.status, append_wrapper.count) == (
            BYTE_CLEAN,
            len(function_expected),
        )
        assert (
            output_bytes(append_wrapper_storage, append_wrapper)
            == function_expected
        )
        assert append_wrapper_storage[len(function_expected)] == GUARD


def assert_ir_structure():
    assert EXPECTED_FUNCTIONS[3] == MATCH3_BYTE_REGRESSION
    for width in WIDTHS:
        function = EXPECTED_FUNCTIONS[width]
        assert function.count(
            b"call { i64, i1 } @llvm.uadd.with.overflow.i64"
        ) == width - 1
        assert function.count(b" = icmp ult i64 ") == 1
        assert function.count(b" = getelementptr i8, ptr ") == width
        assert b"getelementptr inbounds" not in function
        assert function.count(b" = load i8, ptr ") == width
        assert function.count(b" = icmp eq i8 ") == width
        assert function.count(b" = and i1 ") == width - 1
        assert function.count(f"label %bb{width}".encode()) == width
        assert function.count(f"\nbb{width}:\n".encode()) == 1
        assert b" alloca " not in function
        assert b" store " not in function
        final_register = 7 * width - 2
        assert f"  ret i1 %v{final_register}\n".encode() in function


def assert_composition(library, analyzed_cases):
    storage, out = make_output(len(EXPECTED_MODULE))
    library.llvm_buffer_emit_prelude(ctypes.byref(out))
    assert output_bytes(storage, out) == EXPECTED_PRELUDE
    for analyzed in analyzed_cases:
        call_backend(library, analyzed, out, append=True)
    assert (out.status, out.count) == (BYTE_CLEAN, len(EXPECTED_MODULE))
    observed = output_bytes(storage, out)
    assert observed == EXPECTED_MODULE
    assert observed.count(b"declare void @llvm.trap()") == 1
    assert observed.count(b"declare { i64, i1 }") == 1
    assert storage[len(EXPECTED_MODULE)] == GUARD
    return observed


def assert_atomic_failure(
    library, analyzed, label, *, append=True, match3=False
):
    storage, out = make_output(len(PREFIX))
    for index, value in enumerate(PREFIX):
        storage[index] = value
    out.count = len(PREFIX)
    call_backend(library, analyzed, out, append=append, match3=match3)
    expected_count = len(PREFIX) if append else 0
    assert (out.status, out.count) == (
        BYTE_INVALID_STATE,
        expected_count,
    ), label
    assert bytes(storage[: len(PREFIX)]) == PREFIX, label
    assert storage[len(PREFIX)] == GUARD, label


def alternate(value):
    return 1 if int(value) == 0 else 0


def assert_hostile_facts(library, analyzed_cases):
    for analyzed in analyzed_cases:
        case = analyzed["case"]
        fact_storage = analyzed["outputs"][2]
        for column, storage in enumerate(fact_storage):
            for node in range(case[5].count):
                previous = storage[node]
                storage[node] = alternate(previous)
                assert_atomic_failure(
                    library,
                    analyzed,
                    f"width {analyzed['width']} fact {column}:{node}",
                )
                storage[node] = previous

    first = analyzed_cases[0]
    for column, storage in enumerate(first["outputs"][0]):
        for row in range(4):
            previous = storage[row]
            storage[row] = alternate(previous)
            assert_atomic_failure(library, first, f"type {column}:{row}")
            storage[row] = previous

    assert_atomic_failure(
        library,
        analyzed_cases[1],
        "match3 wrapper rejects width4",
        match3=True,
    )
    assert_atomic_failure(
        library,
        analyzed_cases[1],
        "match3 emit wrapper rejects width4",
        append=False,
        match3=True,
    )
    rejected = analyzed_cases[1]
    rejected_case = rejected["case"]
    rejected_outputs = rejected["outputs"]
    assert not library.llvm_buffer_match3_facts_valid(
        rejected_case[1],
        ctypes.byref(rejected_case[3]),
        ctypes.byref(rejected_case[5]),
        rejected["function"],
        ctypes.byref(rejected_outputs[1]),
        ctypes.byref(rejected_outputs[3]),
    )

    emit_case = analyzed_cases[1]
    emit_type_ids = emit_case["outputs"][2][0]
    emit_node = emit_case["function"]
    previous_emit_type = emit_type_ids[emit_node]
    emit_type_ids[emit_node] = alternate(previous_emit_type)
    assert_atomic_failure(
        library,
        emit_case,
        "generic emit rejects hostile facts",
        append=False,
    )
    emit_type_ids[emit_node] = previous_emit_type

    for analyzed in analyzed_cases:
        source_storage = analyzed["case"][0]
        marker = f"lexer_match{analyzed['width']}".encode()
        offset = analyzed["data"].index(marker) + len(marker) - 1
        previous = source_storage[offset]
        source_storage[offset] = ord("5")
        assert_atomic_failure(
            library, analyzed, f"width {analyzed['width']} hostile profile"
        )
        source_storage[offset] = previous


def assert_tape_metadata_and_shape_rejected(library, analyzed):
    outputs = analyzed["outputs"]
    types = outputs[1]
    facts = outputs[3]

    def reject_attribute(target, attribute, value, label):
        previous = getattr(target, attribute)
        try:
            setattr(target, attribute, value)
            assert_atomic_failure(library, analyzed, label)
        finally:
            setattr(target, attribute, previous)

    for attribute, value in (
        ("count", 3),
        ("status", 1),
        ("node", 0),
        ("related", 0),
    ):
        reject_attribute(types, attribute, value, f"types {attribute}")

    for attribute, value in (
        ("count", facts.count - 1),
        ("status", 1),
        ("node", 0),
        ("related", 0),
    ):
        reject_attribute(facts, attribute, value, f"facts {attribute}")

    for tape_name, tape, columns in (
        ("types", types, TYPE_COLUMNS),
        ("facts", facts, NODE_COLUMNS),
    ):
        for column_name, *_ in columns:
            column = getattr(tape, column_name)
            previous_length = column.length
            try:
                column.length = previous_length - 1
                assert_atomic_failure(
                    library,
                    analyzed,
                    f"{tape_name} {column_name} length",
                )
            finally:
                column.length = previous_length

    case = analyzed["case"]
    assert library.llvm_buffer_fixed_width_facts_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        analyzed["function"],
        ctypes.byref(types),
        ctypes.byref(facts),
    )


def assert_identity_forgery_rejected(library, analyzed):
    case = analyzed["case"]
    columns = case[4]
    direct = children_of(columns, analyzed["function"])
    a_parameter = direct[4]
    b_parameter = direct[5]
    a_type = children_of(columns, a_parameter)[1]
    b_mode, orphan_type = children_of(columns, b_parameter)
    block = direct[11]
    statements = children_of(columns, block)
    e0_let = statements[5]
    e0_name = children_of(columns, e0_let)[0]
    first_let = statements[8]
    first_call = children_of(columns, first_let)[3]
    first_left = children_of(columns, first_call)[1]

    previous_b_last = columns[5][b_parameter]
    previous_b_mode_next = columns[6][b_mode]
    previous_orphan_kind = columns[0][orphan_type]
    previous_orphan_first = columns[4][orphan_type]
    previous_orphan_last = columns[5][orphan_type]
    columns[5][b_parameter] = a_type
    columns[6][b_mode] = a_type
    columns[0][orphan_type] = AST["AstLet"]
    columns[4][orphan_type] = e0_name
    columns[5][orphan_type] = e0_name

    facts = analyzed["outputs"][2]
    previous_orphan_facts = tuple(column[orphan_type] for column in facts)
    previous_resolution = facts[1][first_left]
    forged = (1, U64_MAX, 6, 0, U64_MAX, U64_MAX, 1, 0)
    for column, value in zip(facts, forged):
        column[orphan_type] = value
    facts[1][first_left] = orphan_type

    assert library.semantic_buffer_fixed_width_body_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        analyzed["function"],
    )
    outputs = analyzed["outputs"]
    assert not library.llvm_buffer_fixed_width_facts_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        analyzed["function"],
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
    )
    assert_atomic_failure(library, analyzed, "same-typed declaration forgery")

    facts[1][first_left] = previous_resolution
    for column, value in zip(facts, previous_orphan_facts):
        column[orphan_type] = value
    columns[5][b_parameter] = previous_b_last
    columns[6][b_mode] = previous_b_mode_next
    columns[0][orphan_type] = previous_orphan_kind
    columns[4][orphan_type] = previous_orphan_first
    columns[5][orphan_type] = previous_orphan_last


def assert_multi_function_profile(library):
    data = b"\n".join(real_matcher_source(width) for width in WIDTHS)
    analyzed = analyze(library, data, 7)
    assert analyzed["case"][5].count != 36 * 7
    expected = EXPECTED_PRELUDE + EXPECTED_FUNCTIONS[7]
    storage, out = make_output(len(expected))
    call_backend(library, analyzed, out, append=False)
    assert (out.status, out.count) == (BYTE_CLEAN, len(expected))
    assert output_bytes(storage, out) == expected

    foreign = find_function_by_text(
        data, analyzed["case"][4], analyzed["case"][5], b"lexer_match3"
    )
    fact_types = analyzed["outputs"][2][0]
    previous = fact_types[foreign]
    fact_types[foreign] = 0
    assert_atomic_failure(library, analyzed, "foreign profile fact")
    fact_types[foreign] = previous


def compile_module(directory, module):
    ll = directory / "fixed_width_matchers.ll"
    library_path = directory / (
        "fixed_width_matchers.dylib"
        if sys.platform == "darwin"
        else "fixed_width_matchers.so"
    )
    ll.write_bytes(module)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O3"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected composed matcher IR:\n{result.stderr}")
    return library_path


def native_function(library, width):
    function = getattr(library, f"lexer_match{width}")
    function.argtypes = [Buffer, ctypes.c_uint64] + [ctypes.c_uint8] * width
    function.restype = ctypes.c_bool
    return function


def invoke_native(function, payload, start, expected):
    storage = (ctypes.c_uint8 * len(payload))(*payload)
    source = Buffer(ctypes.cast(storage, ctypes.c_void_p), len(payload))
    return bool(function(source, start, *expected))


def assert_runtime_differential(library_path):
    library = ctypes.CDLL(str(library_path))
    rng = random.Random(0x584C414E47)
    calls = 0
    for width in WIDTHS:
        function = native_function(library, width)
        for size in range(width, 49):
            payload = bytes(
                ((index * 73 + size * 29 + width * 11) ^ (index >> 1)) & 0xFF
                for index in range(size)
            )
            for start in range(size - width + 1):
                exact = tuple(payload[start : start + width])
                first_wrong = (exact[0] ^ 1,) + exact[1:]
                last_wrong = exact[:-1] + (exact[-1] ^ 1,)
                random_candidate = tuple(rng.randrange(256) for _ in range(width))
                for candidate in (exact, first_wrong, last_wrong, random_candidate):
                    observed = invoke_native(function, payload, start, candidate)
                    expected = payload[start : start + width] == bytes(candidate)
                    assert observed == expected, (width, size, start, candidate)
                    calls += 1
    assert calls > 12000


TRAP_SCRIPT = r"""
import ctypes
import sys

class Buffer(ctypes.Structure):
    _fields_ = [("data", ctypes.c_void_p), ("length", ctypes.c_uint64)]

library = ctypes.CDLL(sys.argv[1])
width = int(sys.argv[2])
size = int(sys.argv[3])
start = int(sys.argv[4])
function = getattr(library, f"lexer_match{width}")
function.argtypes = [Buffer, ctypes.c_uint64] + [ctypes.c_uint8] * width
function.restype = ctypes.c_bool
storage = (ctypes.c_uint8 * max(size, 1))()
source = Buffer(ctypes.cast(storage, ctypes.c_void_p), size)
function(source, start, *([0] * width))
"""


def trap_process(library_path, width, size, start):
    return subprocess.run(
        [
            sys.executable,
            "-c",
            TRAP_SCRIPT,
            str(library_path),
            str(width),
            str(size),
            str(start),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=5,
    )


def assert_runtime_traps(library_path):
    trap_signal = signal.SIGTRAP if sys.platform == "darwin" else signal.SIGILL
    expected_returncode = -int(trap_signal)
    for width in WIDTHS:
        assert trap_process(library_path, width, width, 0).returncode == 0
        cases = [(width - 1, 0), (width, 1), (0, 0)]
        cases.extend((width, U64_MAX - offset) for offset in range(width - 1))
        for size, start in cases:
            result = trap_process(library_path, width, size, start)
            assert result.returncode == expected_returncode, (
                width,
                size,
                start,
                result.returncode,
                expected_returncode,
            )


def main():
    assert_ir_structure()
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_library(directory)
        configure(library)
        analyzed_cases = [
            analyze(library, real_matcher_source(width), width) for width in WIDTHS
        ]
        for analyzed in analyzed_cases:
            assert_output_modes(library, analyzed)
        renamed = analyze(library, renamed_match4_source(), 4)
        assert_output_modes(library, renamed)
        module = assert_composition(library, analyzed_cases)
        assert_identity_forgery_rejected(library, analyzed_cases[0])
        assert_hostile_facts(library, analyzed_cases)
        assert_tape_metadata_and_shape_rejected(library, analyzed_cases[1])
        assert_multi_function_profile(library)
        native_path = compile_module(directory, module)
        assert_runtime_differential(native_path)
        assert_runtime_traps(native_path)
    print(
        "llvm buffer: 3/4/6/7 formula SSA, exact match3 regression, one-prelude "
        "composition, wrappers, renamed bindings, exhaustive facts/metadata, "
        "runtime differential, and traps pass"
    )


if __name__ == "__main__":
    main()
