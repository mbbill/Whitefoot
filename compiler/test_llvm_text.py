#!/usr/bin/env python3
"""Exercise the deterministic LLVM text vocabulary through its C ABI."""

import ctypes
import subprocess
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library


BYTE_CLEAN = 0
BYTE_NEED_CAPACITY = 1
BYTE_INVALID_STATE = 2
LEX_CLEAN = 0
TOK_WORD = 1
U64_MAX = 18446744073709551615
POISON = 0xA5
GUARD = 0xD3


class ByteTape(ctypes.Structure):
    _fields_ = [
        ("bytes", Buffer),
        ("count", ctypes.c_uint64),
        ("status", ctypes.c_int32),
    ]


NO_ARG_EMITTERS = (
    "llvm_text_emit_i1",
    "llvm_text_emit_i8",
    "llvm_text_emit_i16",
    "llvm_text_emit_i32",
    "llvm_text_emit_i64",
    "llvm_text_emit_ptr",
    "llvm_text_emit_buffer_type",
    "llvm_text_emit_overflow_pair_type",
    "llvm_text_emit_space",
    "llvm_text_emit_newline",
    "llvm_text_emit_at",
    "llvm_text_emit_uadd_overflow_i64",
    "llvm_text_emit_llvm_trap",
    "llvm_text_emit_left_paren",
    "llvm_text_emit_right_paren",
    "llvm_text_emit_comma",
    "llvm_text_emit_equal",
    "llvm_text_emit_colon",
    "llvm_text_emit_left_brace",
    "llvm_text_emit_right_brace",
    "llvm_text_emit_define",
    "llvm_text_emit_declare",
    "llvm_text_emit_void",
    "llvm_text_emit_entry",
    "llvm_text_emit_alloca",
    "llvm_text_emit_store",
    "llvm_text_emit_load",
    "llvm_text_emit_extractvalue",
    "llvm_text_emit_getelementptr",
    "llvm_text_emit_br",
    "llvm_text_emit_label",
    "llvm_text_emit_true",
    "llvm_text_emit_icmp_uge",
    "llvm_text_emit_icmp_ule",
    "llvm_text_emit_icmp_eq",
    "llvm_text_emit_icmp_ult",
    "llvm_text_emit_and",
    "llvm_text_emit_or",
    "llvm_text_emit_ret",
    "llvm_text_emit_unreachable",
)

ID_EMITTERS = (
    "llvm_text_emit_value",
    "llvm_text_emit_place",
    "llvm_text_emit_block",
    "llvm_text_emit_block_ref",
)


EXPECTED = b"""define i64 @kernel({ ptr, i64 } %p0) {
entry:
  %v0 = alloca i1
  %v1 = alloca i8
  %v2 = alloca i16
  %v3 = alloca i32
  %v4 = alloca i64
  %v5 = alloca ptr
  store i64 0, ptr %v4
  %v6 = load i64, ptr %v4
  %v7 = icmp uge i64 %v6, 0
  %v8 = icmp ule i64 %v6, 9
  %v9 = and i1 %v7, %v8
  ret i64 %v6
}
"""

BRANCH_EXPECTED = b"""define i1 @branch(i1 %p0) {
entry:
  br i1 %p0, label %bb0, label %bb1
bb0:
  ret i1 true
bb1:
  ret i1 %p0
}
"""

CHECKED_EXPECTED = b"""declare { i64, i1 } @llvm.uadd.with.overflow.i64(i64, i64)
declare void @llvm.trap()

define i8 @checked({ ptr, i64 } %p0, i64 %p1) {
entry:
  %v0 = call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %p1, i64 1)
  %v1 = extractvalue { i64, i1 } %v0, 0
  %v2 = extractvalue { i64, i1 } %v0, 1
  br i1 %v2, label %bb2, label %bb0
bb0:
  %v3 = extractvalue { ptr, i64 } %p0, 0
  %v4 = extractvalue { ptr, i64 } %p0, 1
  %v5 = icmp ult i64 %v1, %v4
  br i1 %v5, label %bb1, label %bb2
bb1:
  %v6 = getelementptr i8, ptr %v3, i64 %v1
  %v7 = load i8, ptr %v6
  ret i8 %v7
bb2:
  call void @llvm.trap()
  unreachable
}
"""


def make_source(data):
    storage = (ctypes.c_uint8 * max(1, len(data)))()
    for ordinal, byte in enumerate(data):
        storage[ordinal] = byte
    return storage, Buffer(ctypes.cast(storage, ctypes.c_void_p), len(data))


def make_tokens(spans, *, count=None, starts_size=None, ends_size=None):
    capacity = max(1, len(spans))
    kinds = (ctypes.c_int32 * capacity)()
    starts = (ctypes.c_uint64 * capacity)()
    ends = (ctypes.c_uint64 * capacity)()
    for ordinal, (start, end) in enumerate(spans):
        kinds[ordinal] = TOK_WORD
        starts[ordinal] = start
        ends[ordinal] = end
    if count is None:
        count = len(spans)
    if starts_size is None:
        starts_size = len(spans)
    if ends_size is None:
        ends_size = len(spans)
    tape = TokenTape(
        Buffer(ctypes.cast(kinds, ctypes.c_void_p), len(spans)),
        Buffer(ctypes.cast(starts, ctypes.c_void_p), starts_size),
        Buffer(ctypes.cast(ends, ctypes.c_void_p), ends_size),
        count,
        LEX_CLEAN,
        0,
        0,
    )
    return (kinds, starts, ends), tape


def make_output(capacity):
    storage = (ctypes.c_uint8 * (capacity + 1))()
    for ordinal in range(capacity):
        storage[ordinal] = POISON
    storage[capacity] = GUARD
    tape = ByteTape(
        Buffer(ctypes.cast(storage, ctypes.c_void_p), capacity),
        0,
        BYTE_CLEAN,
    )
    return storage, tape


def configure(library):
    out_pointer = ctypes.POINTER(ByteTape)
    for name in NO_ARG_EMITTERS:
        emitter = getattr(library, name)
        emitter.argtypes = [out_pointer]
        emitter.restype = None
    for name in ID_EMITTERS:
        emitter = getattr(library, name)
        emitter.argtypes = [out_pointer, ctypes.c_uint64]
        emitter.restype = None
    library.llvm_text_emit_token.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.c_uint64,
        out_pointer,
    ]
    library.llvm_text_emit_token.restype = None
    library.byte_tape_emit_u64.argtypes = [out_pointer, ctypes.c_uint64]
    library.byte_tape_emit_u64.restype = None
    library.byte_tape_reset.argtypes = [out_pointer]
    library.byte_tape_reset.restype = None


def emit_skeleton(library, source, tokens, out):
    out_pointer = ctypes.byref(out)

    def atom(name):
        getattr(library, name)(out_pointer)

    def identifier(name, value):
        getattr(library, name)(out_pointer, value)

    def space():
        atom("llvm_text_emit_space")

    def indent():
        space()
        space()

    def number(value):
        library.byte_tape_emit_u64(out_pointer, value)

    def alloca_line(value_id, type_name):
        indent()
        identifier("llvm_text_emit_value", value_id)
        space()
        atom("llvm_text_emit_equal")
        space()
        atom("llvm_text_emit_alloca")
        space()
        atom(type_name)
        atom("llvm_text_emit_newline")

    atom("llvm_text_emit_define")
    space()
    atom("llvm_text_emit_i64")
    space()
    atom("llvm_text_emit_at")
    library.llvm_text_emit_token(source, ctypes.byref(tokens), 0, out_pointer)
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_buffer_type")
    space()
    identifier("llvm_text_emit_place", 0)
    atom("llvm_text_emit_right_paren")
    space()
    atom("llvm_text_emit_left_brace")
    atom("llvm_text_emit_newline")

    atom("llvm_text_emit_entry")
    atom("llvm_text_emit_newline")
    for value_id, type_name in enumerate((
        "llvm_text_emit_i1",
        "llvm_text_emit_i8",
        "llvm_text_emit_i16",
        "llvm_text_emit_i32",
        "llvm_text_emit_i64",
        "llvm_text_emit_ptr",
    )):
        alloca_line(value_id, type_name)

    indent()
    atom("llvm_text_emit_store")
    space()
    atom("llvm_text_emit_i64")
    space()
    number(0)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_ptr")
    space()
    identifier("llvm_text_emit_value", 4)
    atom("llvm_text_emit_newline")

    indent()
    identifier("llvm_text_emit_value", 6)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_load")
    space()
    atom("llvm_text_emit_i64")
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_ptr")
    space()
    identifier("llvm_text_emit_value", 4)
    atom("llvm_text_emit_newline")

    for result_id, predicate_name, bound in (
        (7, "llvm_text_emit_icmp_uge", 0),
        (8, "llvm_text_emit_icmp_ule", 9),
    ):
        indent()
        identifier("llvm_text_emit_value", result_id)
        space()
        atom("llvm_text_emit_equal")
        space()
        atom(predicate_name)
        space()
        atom("llvm_text_emit_i64")
        space()
        identifier("llvm_text_emit_value", 6)
        atom("llvm_text_emit_comma")
        space()
        number(bound)
        atom("llvm_text_emit_newline")

    indent()
    identifier("llvm_text_emit_value", 9)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_and")
    space()
    atom("llvm_text_emit_i1")
    space()
    identifier("llvm_text_emit_value", 7)
    atom("llvm_text_emit_comma")
    space()
    identifier("llvm_text_emit_value", 8)
    atom("llvm_text_emit_newline")

    indent()
    atom("llvm_text_emit_ret")
    space()
    atom("llvm_text_emit_i64")
    space()
    identifier("llvm_text_emit_value", 6)
    atom("llvm_text_emit_newline")
    atom("llvm_text_emit_right_brace")
    atom("llvm_text_emit_newline")


def emit_branch_skeleton(library, source, tokens, out):
    out_pointer = ctypes.byref(out)

    def atom(name):
        getattr(library, name)(out_pointer)

    def identifier(name, value):
        getattr(library, name)(out_pointer, value)

    def space():
        atom("llvm_text_emit_space")

    def indent():
        space()
        space()

    atom("llvm_text_emit_define")
    space()
    atom("llvm_text_emit_i1")
    space()
    atom("llvm_text_emit_at")
    library.llvm_text_emit_token(source, ctypes.byref(tokens), 0, out_pointer)
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_i1")
    space()
    identifier("llvm_text_emit_place", 0)
    atom("llvm_text_emit_right_paren")
    space()
    atom("llvm_text_emit_left_brace")
    atom("llvm_text_emit_newline")

    atom("llvm_text_emit_entry")
    atom("llvm_text_emit_newline")
    indent()
    atom("llvm_text_emit_br")
    space()
    atom("llvm_text_emit_i1")
    space()
    identifier("llvm_text_emit_place", 0)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_label")
    space()
    identifier("llvm_text_emit_block_ref", 0)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_label")
    space()
    identifier("llvm_text_emit_block_ref", 1)
    atom("llvm_text_emit_newline")

    identifier("llvm_text_emit_block", 0)
    atom("llvm_text_emit_colon")
    atom("llvm_text_emit_newline")
    indent()
    atom("llvm_text_emit_ret")
    space()
    atom("llvm_text_emit_i1")
    space()
    atom("llvm_text_emit_true")
    atom("llvm_text_emit_newline")

    identifier("llvm_text_emit_block", 1)
    atom("llvm_text_emit_colon")
    atom("llvm_text_emit_newline")
    indent()
    atom("llvm_text_emit_ret")
    space()
    atom("llvm_text_emit_i1")
    space()
    identifier("llvm_text_emit_place", 0)
    atom("llvm_text_emit_newline")
    atom("llvm_text_emit_right_brace")
    atom("llvm_text_emit_newline")


def emit_checked_skeleton(library, source, tokens, out):
    out_pointer = ctypes.byref(out)

    def atom(name):
        getattr(library, name)(out_pointer)

    def identifier(name, value):
        getattr(library, name)(out_pointer, value)

    def space():
        atom("llvm_text_emit_space")

    def indent():
        space()
        space()

    def number(value):
        library.byte_tape_emit_u64(out_pointer, value)

    def value(value_id):
        identifier("llvm_text_emit_value", value_id)

    def place(place_id):
        identifier("llvm_text_emit_place", place_id)

    def block_label(block_id):
        identifier("llvm_text_emit_block", block_id)
        atom("llvm_text_emit_colon")
        atom("llvm_text_emit_newline")

    def branch(condition, true_block, false_block):
        indent()
        atom("llvm_text_emit_br")
        space()
        atom("llvm_text_emit_i1")
        space()
        value(condition)
        atom("llvm_text_emit_comma")
        space()
        atom("llvm_text_emit_label")
        space()
        identifier("llvm_text_emit_block_ref", true_block)
        atom("llvm_text_emit_comma")
        space()
        atom("llvm_text_emit_label")
        space()
        identifier("llvm_text_emit_block_ref", false_block)
        atom("llvm_text_emit_newline")

    atom("llvm_text_emit_declare")
    space()
    atom("llvm_text_emit_overflow_pair_type")
    space()
    atom("llvm_text_emit_uadd_overflow_i64")
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_i64")
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_i64")
    atom("llvm_text_emit_right_paren")
    atom("llvm_text_emit_newline")

    atom("llvm_text_emit_declare")
    space()
    atom("llvm_text_emit_void")
    space()
    atom("llvm_text_emit_llvm_trap")
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_right_paren")
    atom("llvm_text_emit_newline")
    atom("llvm_text_emit_newline")

    atom("llvm_text_emit_define")
    space()
    atom("llvm_text_emit_i8")
    space()
    atom("llvm_text_emit_at")
    library.llvm_text_emit_token(source, ctypes.byref(tokens), 0, out_pointer)
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_buffer_type")
    space()
    place(0)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_i64")
    space()
    place(1)
    atom("llvm_text_emit_right_paren")
    space()
    atom("llvm_text_emit_left_brace")
    atom("llvm_text_emit_newline")
    atom("llvm_text_emit_entry")
    atom("llvm_text_emit_newline")

    indent()
    value(0)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_call")
    space()
    atom("llvm_text_emit_overflow_pair_type")
    space()
    atom("llvm_text_emit_uadd_overflow_i64")
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_i64")
    space()
    place(1)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_i64")
    space()
    number(1)
    atom("llvm_text_emit_right_paren")
    atom("llvm_text_emit_newline")

    for result, field in ((1, 0), (2, 1)):
        indent()
        value(result)
        space()
        atom("llvm_text_emit_equal")
        space()
        atom("llvm_text_emit_extractvalue")
        space()
        atom("llvm_text_emit_overflow_pair_type")
        space()
        value(0)
        atom("llvm_text_emit_comma")
        space()
        number(field)
        atom("llvm_text_emit_newline")
    branch(2, 2, 0)

    block_label(0)
    for result, field in ((3, 0), (4, 1)):
        indent()
        value(result)
        space()
        atom("llvm_text_emit_equal")
        space()
        atom("llvm_text_emit_extractvalue")
        space()
        atom("llvm_text_emit_buffer_type")
        space()
        place(0)
        atom("llvm_text_emit_comma")
        space()
        number(field)
        atom("llvm_text_emit_newline")

    indent()
    value(5)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_icmp_ult")
    space()
    atom("llvm_text_emit_i64")
    space()
    value(1)
    atom("llvm_text_emit_comma")
    space()
    value(4)
    atom("llvm_text_emit_newline")
    branch(5, 1, 2)

    block_label(1)
    indent()
    value(6)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_getelementptr")
    space()
    atom("llvm_text_emit_i8")
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_ptr")
    space()
    value(3)
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_i64")
    space()
    value(1)
    atom("llvm_text_emit_newline")

    indent()
    value(7)
    space()
    atom("llvm_text_emit_equal")
    space()
    atom("llvm_text_emit_load")
    space()
    atom("llvm_text_emit_i8")
    atom("llvm_text_emit_comma")
    space()
    atom("llvm_text_emit_ptr")
    space()
    value(6)
    atom("llvm_text_emit_newline")
    indent()
    atom("llvm_text_emit_ret")
    space()
    atom("llvm_text_emit_i8")
    space()
    value(7)
    atom("llvm_text_emit_newline")

    block_label(2)
    indent()
    atom("llvm_text_emit_call")
    space()
    atom("llvm_text_emit_void")
    space()
    atom("llvm_text_emit_llvm_trap")
    atom("llvm_text_emit_left_paren")
    atom("llvm_text_emit_right_paren")
    atom("llvm_text_emit_newline")
    indent()
    atom("llvm_text_emit_unreachable")
    atom("llvm_text_emit_newline")
    atom("llvm_text_emit_right_brace")
    atom("llvm_text_emit_newline")


def assert_id_vocabulary(library):
    cases = (
        ("llvm_text_emit_value", U64_MAX, b"%v18446744073709551615"),
        ("llvm_text_emit_place", 42, b"%p42"),
        ("llvm_text_emit_block", 7, b"bb7"),
        ("llvm_text_emit_block_ref", U64_MAX, b"%bb18446744073709551615"),
    )
    for name, value, expected in cases:
        storage, tape = make_output(len(expected))
        getattr(library, name)(ctypes.byref(tape), value)
        assert (tape.status, tape.count) == (BYTE_CLEAN, len(expected))
        assert bytes(storage[:tape.count]) == expected
        assert storage[len(expected)] == GUARD


def assert_operation_vocabulary(library):
    cases = (
        ("llvm_text_emit_icmp_eq", b"icmp eq"),
        ("llvm_text_emit_or", b"or"),
        ("llvm_text_emit_br", b"br"),
        ("llvm_text_emit_label", b"label"),
        ("llvm_text_emit_colon", b":"),
        ("llvm_text_emit_true", b"true"),
        ("llvm_text_emit_extractvalue", b"extractvalue"),
        ("llvm_text_emit_getelementptr", b"getelementptr"),
        ("llvm_text_emit_icmp_ult", b"icmp ult"),
        ("llvm_text_emit_declare", b"declare"),
        ("llvm_text_emit_void", b"void"),
        ("llvm_text_emit_unreachable", b"unreachable"),
        ("llvm_text_emit_overflow_pair_type", b"{ i64, i1 }"),
        (
            "llvm_text_emit_uadd_overflow_i64",
            b"@llvm.uadd.with.overflow.i64",
        ),
        ("llvm_text_emit_llvm_trap", b"@llvm.trap"),
    )
    for name, expected in cases:
        storage, tape = make_output(len(expected))
        getattr(library, name)(ctypes.byref(tape))
        assert (tape.status, tape.count) == (BYTE_CLEAN, len(expected))
        assert bytes(storage[:tape.count]) == expected
        assert storage[len(expected)] == GUARD


def assert_invalid_tokens(library, source):
    cases = (
        ([(0, 6)], 1, {}, 1),
        ([(5, 4)], 0, {}, 1),
        ([(0, 7)], 0, {}, 1),
        ([(0, 6)], 0, {"starts_size": 0}, 1),
        ([(0, 6)], 0, {"ends_size": 0}, 1),
        ([(0, 6)], 0, {"count": 0}, 1),
    )
    for spans, ordinal, options, expected_count in cases:
        token_storage, tokens = make_tokens(spans, **options)
        storage, out = make_output(8)
        storage[0] = ord("x")
        out.count = expected_count
        before = bytes(storage)
        library.llvm_text_emit_token(
            source, ctypes.byref(tokens), ordinal, ctypes.byref(out)
        )
        assert (out.status, out.count) == (BYTE_INVALID_STATE, expected_count)
        assert bytes(storage) == before
        assert token_storage


def assert_assembled_ir(directory, expected, stem="llvm_text_skeleton"):
    source_path = directory / f"{stem}.ll"
    object_path = directory / f"{stem}.o"
    source_path.write_bytes(expected)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    result = subprocess.run(
        [cc, "-x", "ir", "-c", str(source_path), "-o", str(object_path)],
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise AssertionError(f"LLVM rejected the composed skeleton:\n{result.stderr}")


def assert_branch_skeleton(library, directory):
    source_storage, source = make_source(b"branch")
    token_storage, tokens = make_tokens([(0, 6)])

    measured_storage, measured = make_output(0)
    emit_branch_skeleton(library, source, tokens, measured)
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(BRANCH_EXPECTED),
    )
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(BRANCH_EXPECTED))
    emit_branch_skeleton(library, source, tokens, exact)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(BRANCH_EXPECTED))
    assert bytes(exact_storage[:exact.count]) == BRANCH_EXPECTED
    assert exact_storage[len(BRANCH_EXPECTED)] == GUARD

    short_capacity = len(BRANCH_EXPECTED) - 7
    short_storage, short = make_output(short_capacity)
    emit_branch_skeleton(library, source, tokens, short)
    assert (short.status, short.count) == (
        BYTE_NEED_CAPACITY,
        len(BRANCH_EXPECTED),
    )
    assert bytes(short_storage[:short_capacity]) == BRANCH_EXPECTED[:short_capacity]
    assert short_storage[short_capacity] == GUARD

    assert_assembled_ir(directory, BRANCH_EXPECTED, "llvm_text_branch")
    assert source_storage and token_storage


def assert_checked_skeleton(library, directory):
    source_storage, source = make_source(b"checked")
    token_storage, tokens = make_tokens([(0, 7)])

    measured_storage, measured = make_output(0)
    emit_checked_skeleton(library, source, tokens, measured)
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(CHECKED_EXPECTED),
    )
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(CHECKED_EXPECTED))
    emit_checked_skeleton(library, source, tokens, exact)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(CHECKED_EXPECTED))
    assert bytes(exact_storage[:exact.count]) == CHECKED_EXPECTED
    assert exact_storage[len(CHECKED_EXPECTED)] == GUARD

    short_capacity = len(CHECKED_EXPECTED) - 13
    short_storage, short = make_output(short_capacity)
    emit_checked_skeleton(library, source, tokens, short)
    assert (short.status, short.count) == (
        BYTE_NEED_CAPACITY,
        len(CHECKED_EXPECTED),
    )
    assert bytes(short_storage[:short_capacity]) == CHECKED_EXPECTED[:short_capacity]
    assert short_storage[short_capacity] == GUARD

    assert_assembled_ir(directory, CHECKED_EXPECTED, "llvm_text_checked")
    assert source_storage and token_storage


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_library(directory)
        configure(library)

        source_storage, source = make_source(b"kernel")
        token_storage, tokens = make_tokens([(0, 6)])

        measured_storage, measured = make_output(0)
        emit_skeleton(library, source, tokens, measured)
        assert (measured.status, measured.count) == (
            BYTE_NEED_CAPACITY,
            len(EXPECTED),
        )
        assert measured_storage[0] == GUARD

        exact_storage, exact = make_output(len(EXPECTED))
        emit_skeleton(library, source, tokens, exact)
        assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED))
        assert bytes(exact_storage[:exact.count]) == EXPECTED
        assert exact_storage[len(EXPECTED)] == GUARD

        short_capacity = len(EXPECTED) - 11
        short_storage, short = make_output(short_capacity)
        emit_skeleton(library, source, tokens, short)
        assert (short.status, short.count) == (
            BYTE_NEED_CAPACITY,
            len(EXPECTED),
        )
        assert bytes(short_storage[:short_capacity]) == EXPECTED[:short_capacity]
        assert short_storage[short_capacity] == GUARD

        first = bytes(exact_storage[:exact.count])
        for ordinal in range(len(EXPECTED)):
            exact_storage[ordinal] = POISON
        exact.count = 99
        exact.status = BYTE_INVALID_STATE
        library.byte_tape_reset(ctypes.byref(exact))
        emit_skeleton(library, source, tokens, exact)
        second = bytes(exact_storage[:exact.count])
        assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED))
        assert second == first == EXPECTED
        assert exact_storage[len(EXPECTED)] == GUARD

        assert_id_vocabulary(library)
        assert_operation_vocabulary(library)
        assert_branch_skeleton(library, directory)
        assert_checked_skeleton(library, directory)
        assert_invalid_tokens(library, source)
        assert_assembled_ir(directory, EXPECTED)

        assert source_storage and token_storage
        print(
            "self-hosted LLVM text: compositional vocabulary, exact straight-line/branch "
            "skeletons, checked add/load/trap, measure/exact/short capacity, "
            "invalid tokens/spans, repeat, "
            "and LLVM parse pass"
        )


if __name__ == "__main__":
    main()
