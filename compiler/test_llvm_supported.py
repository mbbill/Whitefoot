#!/usr/bin/env python3
"""Compose every currently supported lexer function into one LLVM module."""

import ctypes
import subprocess
import sys
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library
from test_parser import AstTape
from test_ast_validate import AstValidationReport
from test_semantic_body import (
    SemanticBodyScratch,
    configure as configure_semantic_body,
    find_function_by_text,
    make_outputs,
    parsed,
)
from test_semantic_facts import NodeFacts, TypeTape
from test_symbols import SymbolTape
from test_llvm_text import (
    BYTE_CLEAN,
    BYTE_INVALID_STATE,
    BYTE_NEED_CAPACITY,
    GUARD,
    ByteTape,
    make_output,
)
from test_llvm_linear import EXPECTED as LINEAR_EXPECTED, ORDER as LINEAR_ORDER
from test_llvm_buffer import (
    EXPECTED_FUNCTIONS as BUFFER_EXPECTED,
    EXPECTED_PRELUDE,
    WIDTHS,
)
from test_llvm_scanner import (
    EXPECTED_FUNCTIONS as SCANNER_EXPECTED,
    KEEP as SCANNER_KEEP,
    SCANNERS,
)


HERE = Path(__file__).resolve().parent
U64_MAX = (1 << 64) - 1
STATUS_CLEAN = 0
STATUS_INVALID_WORKLIST = 1
STATUS_DUPLICATE = 2
STATUS_UNSUPPORTED = 3
STATUS_SEMANTIC = 4
FUNCTION_NAMES = (
    LINEAR_ORDER
    + tuple(f"lexer_match{width}".encode() for width in WIDTHS)
    + tuple(name for name, _ in SCANNERS)
)
EXPECTED_MODULE = (
    EXPECTED_PRELUDE
    + b"".join(LINEAR_EXPECTED[name] for name in LINEAR_ORDER)
    + b"".join(BUFFER_EXPECTED[width] for width in WIDTHS)
    + b"".join(SCANNER_EXPECTED[name] for name, _ in SCANNERS)
)


class LlvmSupportedReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("function", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("emitted", ctypes.c_uint64),
    ]


def configure(library):
    configure_semantic_body(library)
    library.llvm_supported_emit_module.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SymbolTape),
        Buffer,
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(SemanticBodyScratch),
        ctypes.POINTER(ByteTape),
        ctypes.POINTER(LlvmSupportedReport),
    ]
    library.llvm_supported_emit_module.restype = None


def make_worklist(nodes):
    physical = max(len(nodes), 1)
    storage = (ctypes.c_uint64 * physical)()
    for index, node in enumerate(nodes):
        storage[index] = node
    tape = Buffer(ctypes.cast(storage, ctypes.c_void_p), len(nodes))
    return storage, tape


def make_work(library, ast_count, *, type_capacity=4, fact_capacity=None, scratch_capacity=34):
    if fact_capacity is None:
        fact_capacity = ast_count
    return make_outputs(
        library,
        ast_count,
        type_caps=(type_capacity,) * 6,
        fact_caps=(fact_capacity,) * 8,
        scratch_caps=(scratch_capacity,) * 5,
    )


def invoke(library, case, worklist, work, out):
    report = LlvmSupportedReport(99, 123, 456, 789)
    library.llvm_supported_emit_module(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        ctypes.byref(case[6]),
        ctypes.byref(case[9]),
        worklist,
        ctypes.byref(work[1]),
        ctypes.byref(work[3]),
        ctypes.byref(work[6]),
        ctypes.byref(out),
        ctypes.byref(report),
    )
    return report


def output_bytes(storage, out):
    return bytes(storage[: min(out.count, len(storage) - 1)])


def analyze_case(library):
    data = (HERE / "src" / "lexer.wf").read_bytes()
    case = parsed(library, data)
    nodes = tuple(
        find_function_by_text(data, case[4], case[5], name)
        for name in FUNCTION_NAMES
    )
    worklist_storage, worklist = make_worklist(nodes)
    work = make_work(library, case[5].count)
    return data, case, nodes, worklist_storage, worklist, work


def assert_output_modes(library, case, worklist, work):
    measured_storage, measured = make_output(0)
    report = invoke(library, case, worklist, work, measured)
    assert (report.status, report.function, report.related, report.emitted) == (
        STATUS_CLEAN,
        U64_MAX,
        U64_MAX,
        15,
    )
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(EXPECTED_MODULE),
    )
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(measured.count)
    report = invoke(library, case, worklist, work, exact)
    assert (report.status, report.emitted) == (STATUS_CLEAN, 15)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED_MODULE))
    assert output_bytes(exact_storage, exact) == EXPECTED_MODULE
    assert exact_storage[len(EXPECTED_MODULE)] == GUARD
    observed = output_bytes(exact_storage, exact)
    assert observed.count(b"declare { i64, i1 } @llvm.uadd.with.overflow.i64") == 1
    assert observed.count(b"declare void @llvm.trap()") == 1
    assert observed.count(b"\ndefine ") + observed.startswith(b"define ") == 15
    assert b"declare i1 @lexer_is_" not in observed

    short_storage, short = make_output(len(EXPECTED_MODULE) - 1)
    report = invoke(library, case, worklist, work, short)
    assert (report.status, report.emitted) == (STATUS_CLEAN, 15)
    assert (short.status, short.count) == (
        BYTE_NEED_CAPACITY,
        len(EXPECTED_MODULE),
    )
    assert output_bytes(short_storage, short) == EXPECTED_MODULE[:-1]
    assert short_storage[len(EXPECTED_MODULE) - 1] == GUARD

    for index in range(len(EXPECTED_MODULE)):
        exact_storage[index] = 0xA5
    exact.status = BYTE_INVALID_STATE
    exact.count = U64_MAX
    report = invoke(library, case, worklist, work, exact)
    assert (report.status, report.emitted) == (STATUS_CLEAN, 15)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED_MODULE))
    assert output_bytes(exact_storage, exact) == EXPECTED_MODULE
    assert (work[1].count, work[3].count, work[6].count) == (
        4,
        case[5].count,
        6,
    )
    return EXPECTED_MODULE


def compile_shared(directory, module):
    ll = directory / "supported_module.ll"
    library_path = directory / (
        "supported_module.dylib"
        if sys.platform == "darwin"
        else "supported_module.so"
    )
    ll.write_bytes(module)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O3"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected supported module:\n{result.stderr}")
    return library_path


def native_buffer(payload):
    physical = max(len(payload), 1)
    storage = (ctypes.c_uint8 * physical)()
    for index, value in enumerate(payload):
        storage[index] = value
    return storage, Buffer(ctypes.cast(storage, ctypes.c_void_p), len(payload))


def assert_native_behavior(library_path):
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
            value in (40, 41, 123, 125, 60, 62, 58, 59, 44, 61, 91, 93, 46)
        ),
    }
    for name, predicate in predicates.items():
        function = getattr(generated, name.decode())
        function.argtypes = [ctypes.c_uint8]
        function.restype = ctypes.c_bool
        for value in range(256):
            assert function(value) is predicate(value), (name, value)

    payload = bytes(range(32, 96))
    payload_storage, source = native_buffer(payload)
    assert payload_storage[0] == payload[0]
    for width in WIDTHS:
        function = getattr(generated, f"lexer_match{width}")
        function.argtypes = [Buffer, ctypes.c_uint64] + [ctypes.c_uint8] * width
        function.restype = ctypes.c_bool
        start = width + 1
        exact = tuple(payload[start : start + width])
        wrong = exact[:-1] + (exact[-1] ^ 1,)
        assert function(source, start, *exact) is True
        assert function(source, start, *wrong) is False

    scanner_payloads = {
        b"lexer_scan_ident": b"abc_19!tail",
        b"lexer_scan_typeid": b"AbZ19_tail",
        b"lexer_scan_number": b"Ab9_.-!tail",
    }
    for name, _ in SCANNERS:
        function = getattr(generated, name.decode())
        function.argtypes = [Buffer, ctypes.c_uint64, ctypes.c_uint64]
        function.restype = ctypes.c_uint64
        payload = scanner_payloads[name]
        payload_storage, source = native_buffer(payload)
        expected = 0
        while expected < len(payload) and SCANNER_KEEP[name](payload[expected]):
            expected += 1
        assert function(source, 0, len(payload)) == expected
        assert function(source, expected, expected) == expected
        assert payload_storage[0] == payload[0]


def assert_failure_atomic(library, case, nodes, expected_status, label, *, work=None):
    if work is None:
        work = make_work(library, case[5].count)
    worklist_storage, worklist = make_worklist(nodes)
    assert len(worklist_storage) >= max(len(nodes), 1)
    storage, out = make_output(96)
    prefix = b"supported-prefix:"
    for index, value in enumerate(prefix):
        storage[index] = value
    out.count = len(prefix)
    before_bytes = bytes(storage)
    before_state = (out.status, out.count)
    report = invoke(library, case, worklist, work, out)
    assert report.status == expected_status, (label, report.status, report.function, report.related)
    assert report.emitted == 0, label
    assert (out.status, out.count) == before_state, label
    assert bytes(storage) == before_bytes, label


def assert_worklist_failures(library, data, case, nodes):
    by_name = dict(zip(FUNCTION_NAMES, nodes))
    unsupported = find_function_by_text(
        data, case[4], case[5], b"lexer_scan_op_suffix"
    )
    cases = (
        ((), STATUS_INVALID_WORKLIST, "empty"),
        ((case[5].root,), STATUS_INVALID_WORKLIST, "non-function"),
        ((U64_MAX,), STATUS_INVALID_WORKLIST, "out-of-range function"),
        (nodes + (nodes[-1],), STATUS_DUPLICATE, "duplicate"),
        ((unsupported,), STATUS_UNSUPPORTED, "unsupported"),
        ((by_name[b"lexer_is_ident_tail"],), STATUS_INVALID_WORKLIST, "missing callee"),
        (
            (
                by_name[b"lexer_is_ident_tail"],
                by_name[b"lexer_is_lower"],
                by_name[b"lexer_is_digit"],
            ),
            STATUS_INVALID_WORKLIST,
            "out-of-order callee",
        ),
    )
    for candidate, status, label in cases:
        assert_failure_atomic(library, case, candidate, status, label)


def assert_work_capacity_failures(library, case, nodes):
    cases = (
        (
            make_work(library, case[5].count, type_capacity=3),
            "type capacity",
        ),
        (
            make_work(
                library,
                case[5].count,
                fact_capacity=case[5].count - 1,
            ),
            "fact capacity",
        ),
        (
            make_work(library, case[5].count, scratch_capacity=33),
            "scratch capacity",
        ),
    )
    for work, label in cases:
        assert_failure_atomic(
            library,
            case,
            nodes,
            STATUS_SEMANTIC,
            label,
            work=work,
        )


def assert_stale_validation_failure_atomic(library, case, nodes):
    function = nodes[0]
    case[4][2][function] = case[1].length + 1
    worklist_storage, worklist = make_worklist(nodes)
    assert len(worklist_storage) == len(nodes)
    work = make_work(library, case[5].count)
    storage, out = make_output(96)
    prefix = b"validated-prefix:"
    for index, value in enumerate(prefix):
        storage[index] = value
    out.count = len(prefix)
    before_bytes = bytes(storage)
    before_state = (out.status, out.count)
    report = invoke(library, case, worklist, work, out)
    assert (report.status, report.emitted, case[6].status) == (
        STATUS_SEMANTIC,
        0,
        8,
    )
    assert (out.status, out.count) == before_state
    assert bytes(storage) == before_bytes


def main():
    assert len(FUNCTION_NAMES) == 15
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_library(directory)
        configure(library)
        data, case, nodes, _, worklist, work = analyze_case(library)
        module = assert_output_modes(library, case, worklist, work)
        assert_worklist_failures(library, data, case, nodes)
        assert_work_capacity_failures(library, case, nodes)
        assert_stale_validation_failure_atomic(library, case, nodes)
        native_path = compile_shared(directory, module)
        assert_native_behavior(native_path)
    print(
        "llvm supported: exact 15-function module, one shared prelude, "
        "measure/exact/short/retry, clang/native families, atomic worklists, "
        "fresh validation, and work capacities pass"
    )


if __name__ == "__main__":
    main()
