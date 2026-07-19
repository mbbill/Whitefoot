#!/usr/bin/env python3
"""Lower real scan-while lexer loops to proof-elided checked SSA."""

import ctypes
import random
import signal
import subprocess
import sys
import tempfile
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_lexer import Buffer, TokenTape, build_library
from test_parser import AstTape, children_of
from test_semantic_body import (
    SemanticBodyReport,
    SemanticBodyScratch,
    configure as configure_semantic_body,
    find_function_by_text,
    make_outputs,
    parsed,
)
from test_semantic_facts import NODE_COLUMNS, TYPE_COLUMNS, NodeFacts, TypeTape
from test_symbols import SymbolTape
from test_llvm_linear import EXPECTED as LINEAR_EXPECTED
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
U64_MAX = (1 << 64) - 1
BODY_CLEAN = 0
SCANNERS = (
    (b"lexer_scan_ident", b"lexer_is_ident_tail"),
    (b"lexer_scan_typeid", b"lexer_is_type_tail"),
    (b"lexer_scan_number", b"lexer_is_number_tail"),
)
PREFIX = b"scanner-prefix:"
TRAP_PRELUDE = b"declare void @llvm.trap()\n\n"


def expected_function(name, predicate):
    return b"""define i64 @%b({ ptr, i64 } %%p0, i64 %%p1, i64 %%p2) {
entry:
  %%v0 = extractvalue { ptr, i64 } %%p0, 0
  %%v1 = extractvalue { ptr, i64 } %%p0, 1
  br label %%bb0
bb0:
  %%v2 = phi i64 [ %%p1, %%entry ], [ %%v8, %%bb3 ]
  %%v3 = icmp uge i64 %%v2, %%p2
  br i1 %%v3, label %%bb4, label %%bb1
bb1:
  %%v4 = icmp ult i64 %%v2, %%v1
  br i1 %%v4, label %%bb2, label %%bb5
bb2:
  %%v5 = getelementptr i8, ptr %%v0, i64 %%v2
  %%v6 = load i8, ptr %%v5
  %%v7 = call i1 @%b(i8 %%v6)
  br i1 %%v7, label %%bb3, label %%bb4
bb3:
  %%v8 = add nuw i64 %%v2, 1
  br label %%bb0
bb4:
  ret i64 %%v2
bb5:
  call void @llvm.trap()
  unreachable
}
""" % (name, predicate)


EXPECTED_FUNCTIONS = {
    name: expected_function(name, predicate) for name, predicate in SCANNERS
}
EXPECTED_STANDALONE = {
    name: (
        b"declare void @llvm.trap()\n"
        + b"declare i1 @"
        + predicate
        + b"(i8)\n\n"
        + EXPECTED_FUNCTIONS[name]
    )
    for name, predicate in SCANNERS
}
CUSTOM_SCANNER = b"scan_custom"
CUSTOM_PREDICATE = b"custom_predicate"
CUSTOM_FUNCTION = expected_function(CUSTOM_SCANNER, CUSTOM_PREDICATE)
CUSTOM_STANDALONE = (
    b"declare void @llvm.trap()\n"
    + b"declare i1 @"
    + CUSTOM_PREDICATE
    + b"(i8)\n\n"
    + CUSTOM_FUNCTION
)
CUSTOM_PREDICATE_IR = b"""define i1 @custom_predicate(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 97
  %v1 = icmp ule i8 %p0, 122
  %v2 = and i1 %v0, %v1
  %v3 = icmp uge i8 %p0, 48
  %v4 = icmp ule i8 %p0, 57
  %v5 = and i1 %v3, %v4
  %v6 = icmp eq i8 %p0, 95
  %v7 = or i1 %v2, %v5
  %v8 = or i1 %v7, %v6
  ret i1 %v8
}
"""


def configure(library):
    configure_semantic_body(library)
    semantic_arguments = [
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
    library.semantic_scanner_run.argtypes = semantic_arguments
    library.semantic_scanner_run.restype = None
    backend_arguments = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
    ]
    library.llvm_scanner_facts_valid.argtypes = backend_arguments
    library.llvm_scanner_facts_valid.restype = ctypes.c_bool
    for name in ("llvm_scanner_append_function", "llvm_scanner_emit"):
        function = getattr(library, name)
        function.argtypes = backend_arguments + [ctypes.POINTER(ByteTape)]
        function.restype = None


def analyze(library, data, name):
    case = parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], name)
    outputs = make_outputs(
        library,
        case[5].count,
        type_caps=(4,) * len(TYPE_COLUMNS),
        fact_caps=(case[5].count,) * len(NODE_COLUMNS),
        scratch_caps=(6,) * 5,
    )
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    library.semantic_scanner_run(
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
    assert library.llvm_scanner_facts_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        function,
        ctypes.byref(case[9]),
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
    )
    return {
        "case": case,
        "data": data,
        "name": name,
        "function": function,
        "outputs": outputs,
    }


def call_backend(library, analyzed, out, *, append):
    function = (
        library.llvm_scanner_append_function
        if append
        else library.llvm_scanner_emit
    )
    case = analyzed["case"]
    outputs = analyzed["outputs"]
    function(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        analyzed["function"],
        ctypes.byref(case[9]),
        ctypes.byref(outputs[1]),
        ctypes.byref(outputs[3]),
        ctypes.byref(out),
    )


def output_bytes(storage, out):
    return bytes(storage[: min(out.count, len(storage) - 1)])


def assert_output_modes(library, analyzed):
    standalone = EXPECTED_STANDALONE[analyzed["name"]]
    function_expected = EXPECTED_FUNCTIONS[analyzed["name"]]

    measured_storage, measured = make_output(0)
    call_backend(library, analyzed, measured, append=False)
    assert (measured.status, measured.count) == (
        BYTE_NEED_CAPACITY,
        len(standalone),
    )
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(standalone))
    call_backend(library, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(standalone))
    assert output_bytes(exact_storage, exact) == standalone
    assert exact_storage[len(standalone)] == GUARD

    short_storage, short = make_output(len(standalone) - 1)
    call_backend(library, analyzed, short, append=False)
    assert (short.status, short.count) == (
        BYTE_NEED_CAPACITY,
        len(standalone),
    )
    assert output_bytes(short_storage, short) == standalone[:-1]
    assert short_storage[len(standalone) - 1] == GUARD

    for index in range(len(standalone)):
        exact_storage[index] = POISON
    exact.status = BYTE_INVALID_STATE
    exact.count = U64_MAX
    call_backend(library, analyzed, exact, append=False)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(standalone))
    assert output_bytes(exact_storage, exact) == standalone

    append_storage, append = make_output(len(function_expected))
    call_backend(library, analyzed, append, append=True)
    assert (append.status, append.count) == (
        BYTE_CLEAN,
        len(function_expected),
    )
    assert output_bytes(append_storage, append) == function_expected
    assert b"declare " not in output_bytes(append_storage, append)
    assert append_storage[len(function_expected)] == GUARD


def assert_ir_shape():
    for name, predicate in SCANNERS:
        function = EXPECTED_FUNCTIONS[name]
        assert function.count(b" = phi i64 ") == 1
        assert function.count(b" = add nuw i64 ") == 1
        assert function.count(b" = icmp uge i64 ") == 1
        assert function.count(b" = icmp ult i64 ") == 1
        assert function.count(b" = getelementptr i8, ptr ") == 1
        assert function.count(b" = load i8, ptr ") == 1
        assert function.count(b"call void @llvm.trap()") == 1
        assert b"llvm.uadd.with.overflow" not in function
        assert b" alloca " not in function
        assert b" store " not in function
        assert b"getelementptr inbounds" not in function
        assert b"call i1 @" + predicate + b"(i8 %v6)" in function


def predicate_module_prefix():
    order = (
        b"lexer_is_lower",
        b"lexer_is_upper",
        b"lexer_is_digit",
        b"lexer_is_ident_tail",
        b"lexer_is_type_tail",
        b"lexer_is_number_tail",
    )
    return TRAP_PRELUDE + b"".join(LINEAR_EXPECTED[name] for name in order)


def assert_composition(library, analyzed_cases):
    expected = predicate_module_prefix() + b"".join(
        EXPECTED_FUNCTIONS[analyzed["name"]] for analyzed in analyzed_cases
    )
    storage, out = make_output(len(expected))
    prefix = predicate_module_prefix()
    for index, value in enumerate(prefix):
        storage[index] = value
    out.count = len(prefix)
    for analyzed in analyzed_cases:
        call_backend(library, analyzed, out, append=True)
    assert (out.status, out.count) == (BYTE_CLEAN, len(expected))
    assert output_bytes(storage, out) == expected
    assert output_bytes(storage, out).count(b"declare void @llvm.trap()") == 1
    assert storage[len(expected)] == GUARD
    return expected


def assert_atomic_failure(library, analyzed, label):
    storage, out = make_output(len(PREFIX))
    for index, value in enumerate(PREFIX):
        storage[index] = value
    out.count = len(PREFIX)
    call_backend(library, analyzed, out, append=True)
    assert (out.status, out.count) == (
        BYTE_INVALID_STATE,
        len(PREFIX),
    ), label
    assert bytes(storage[: len(PREFIX)]) == PREFIX, label
    assert storage[len(PREFIX)] == GUARD, label


def assert_atomic_emit_failure(library, analyzed, label):
    expected = EXPECTED_STANDALONE[analyzed["name"]]
    storage, out = make_output(len(expected))
    call_backend(library, analyzed, out, append=False)
    assert (out.status, out.count) == (BYTE_INVALID_STATE, 0), label
    assert bytes(storage[: len(expected)]) == bytes([POISON]) * len(expected), label
    assert storage[len(expected)] == GUARD, label


def alternate(value):
    return 1 if int(value) == 0 else 0


def subtree_nodes(columns, root):
    result = []

    def visit(node):
        result.append(node)
        for child in children_of(columns, node):
            visit(child)

    visit(root)
    return result


def scanner_nodes(analyzed):
    columns = analyzed["case"][4]
    direct = children_of(columns, analyzed["function"])
    block = direct[9]
    cursor_let, loop_node, return_node = children_of(columns, block)
    _, loop_block = children_of(columns, loop_node)
    guard_match, byte_let, keep_let, keep_match = children_of(columns, loop_block)
    guard_call = children_of(columns, guard_match)[0]
    predicate_call = children_of(columns, keep_let)[3]
    predicate_named = children_of(columns, predicate_call)[0]
    keep_true = children_of(columns, keep_match)[1]
    keep_true_block = children_of(columns, keep_true)[0]
    set_node = children_of(columns, keep_true_block)[0]
    set_target, increment = children_of(columns, set_node)
    increment_left = children_of(columns, increment)[1]
    return_value = children_of(columns, return_node)[0]
    guard_left = children_of(columns, guard_call)[1]
    return {
        "cursor_let": cursor_let,
        "guard_call": guard_call,
        "guard_left": guard_left,
        "byte_let": byte_let,
        "predicate_call": predicate_call,
        "predicate_named": predicate_named,
        "keep_match": keep_match,
        "increment": increment,
        "increment_left": increment_left,
        "set_target": set_target,
        "return_value": return_value,
    }


def assert_hostile_facts(library, analyzed_cases):
    for analyzed in analyzed_cases:
        outputs = analyzed["outputs"]
        facts = outputs[2]
        nodes = subtree_nodes(analyzed["case"][4], analyzed["function"])
        assert len(nodes) == 72
        for column_index, column in enumerate(facts):
            for node in nodes:
                previous = column[node]
                column[node] = alternate(previous)
                assert_atomic_failure(
                    library,
                    analyzed,
                    f"{analyzed['name']} fact {column_index}:{node}",
                )
                column[node] = previous

    first = analyzed_cases[0]
    type_storage = first["outputs"][0]
    for column_index, column in enumerate(type_storage):
        for row in range(4):
            previous = column[row]
            column[row] = alternate(previous)
            assert_atomic_failure(library, first, f"type {column_index}:{row}")
            column[row] = previous

    facts_object = first["outputs"][3]
    types_object = first["outputs"][1]
    mutations = (
        (facts_object, "count", facts_object.count - 1),
        (facts_object, "status", 1),
        (facts_object, "node", 0),
        (facts_object, "related", 0),
        (types_object, "count", 3),
        (types_object, "status", 1),
        (types_object, "node", 0),
        (types_object, "related", 0),
    )
    for owner, field, replacement in mutations:
        previous = getattr(owner, field)
        setattr(owner, field, replacement)
        assert_atomic_failure(library, first, f"{type(owner).__name__}.{field}")
        setattr(owner, field, previous)

    foreign = first["case"][5].root
    fact_types = first["outputs"][2][0]
    previous_foreign = fact_types[foreign]
    fact_types[foreign] = 0
    assert_atomic_failure(library, first, "foreign default fact")
    fact_types[foreign] = previous_foreign


def assert_coordinated_forgeries(library, analyzed_cases):
    first = analyzed_cases[0]
    second = analyzed_cases[1]
    facts = first["outputs"][2]
    nodes = scanner_nodes(first)

    wrong_callee = second["outputs"][2][1][scanner_nodes(second)["predicate_call"]]
    wrong_formal = children_of(first["case"][4], wrong_callee)[1]
    previous_callee = facts[1][nodes["predicate_call"]]
    previous_formal = facts[1][nodes["predicate_named"]]
    facts[1][nodes["predicate_call"]] = wrong_callee
    facts[1][nodes["predicate_named"]] = wrong_formal
    assert_atomic_failure(library, first, "coordinated callee identity")
    facts[1][nodes["predicate_call"]] = previous_callee
    facts[1][nodes["predicate_named"]] = previous_formal

    previous_proof = facts[5][nodes["increment"]]
    previous_call_target = facts[5][nodes["predicate_call"]]
    facts[5][nodes["increment"]] = nodes["predicate_call"]
    facts[5][nodes["predicate_call"]] = nodes["keep_match"]
    assert_atomic_emit_failure(library, first, "coordinated proof reference")
    facts[5][nodes["increment"]] = previous_proof
    facts[5][nodes["predicate_call"]] = previous_call_target

    start_parameter = children_of(first["case"][4], first["function"])[3]
    resolution_nodes = (
        nodes["guard_left"],
        nodes["increment_left"],
        nodes["set_target"],
        nodes["return_value"],
    )
    previous_resolutions = tuple(facts[1][node] for node in resolution_nodes)
    for node in resolution_nodes:
        facts[1][node] = start_parameter
    assert_atomic_failure(library, first, "coordinated cursor declaration")
    for node, value in zip(resolution_nodes, previous_resolutions):
        facts[1][node] = value


def renamed_scanner_source(data, name):
    start = data.index(b"fn " + name + b" ")
    end = data.index(b"\nfn ", start)
    body = data[start:end]
    label = name.removeprefix(b"lexer_scan_")
    body = body.replace(b"cursor", b"position")
    body = body.replace(b"byte", b"octet")
    body = body.replace(b"keep", b"continue_scan")
    body = body.replace(b"@" + label, b"@again")
    return data[:start] + body + data[end:]


def function_source(data, name):
    start = data.index(b"fn " + name + b" ")
    end = data.find(b"\nfn ", start)
    if end < 0:
        end = len(data)
    return data[start:end].rstrip() + b"\n"


def custom_scanner_source(data):
    predicate = function_source(data, b"lexer_is_ident_tail")
    scanner = function_source(data, b"lexer_scan_ident")
    predicate = predicate.replace(b"lexer_is_ident_tail", CUSTOM_PREDICATE)
    predicate = predicate.replace(b"(c: own u8)", b"(value: own u8)")
    predicate = predicate.replace(b"(c: c)", b"(c: value)")
    predicate = predicate.replace(b"(c, 95_u8)", b"(value, 95_u8)")
    replacements = (
        (b"lexer_scan_ident", CUSTOM_SCANNER),
        (b"'s", b"'scan_region"),
        (b"source", b"input"),
        (b"start", b"begin"),
        (b"size", b"limit"),
        (b"cursor", b"position"),
        (b"byte", b"element"),
        (b"keep", b"continue_flag"),
        (b"@ident", b"@scan"),
        (b"lexer_is_ident_tail", CUSTOM_PREDICATE),
        (b"c: element", b"value: element"),
    )
    for before, after in replacements:
        scanner = scanner.replace(before, after)
    return predicate + b"\n" + scanner


def assert_renamed_bindings(library, data):
    for name, _ in SCANNERS:
        renamed = analyze(library, renamed_scanner_source(data, name), name)
        expected = EXPECTED_STANDALONE[name]
        storage, out = make_output(len(expected))
        call_backend(library, renamed, out, append=False)
        assert (out.status, out.count) == (BYTE_CLEAN, len(expected))
        assert output_bytes(storage, out) == expected


def assert_custom_scanner_pipeline(library, data, directory):
    analyzed = analyze(library, custom_scanner_source(data), CUSTOM_SCANNER)
    storage, out = make_output(len(CUSTOM_STANDALONE))
    call_backend(library, analyzed, out, append=False)
    assert (out.status, out.count) == (BYTE_CLEAN, len(CUSTOM_STANDALONE))
    assert output_bytes(storage, out) == CUSTOM_STANDALONE
    assert b"lexer_scan_ident" not in output_bytes(storage, out)
    assert b"lexer_is_ident_tail" not in output_bytes(storage, out)
    compile_ir(
        directory,
        "scanner_custom_standalone",
        CUSTOM_STANDALONE,
        shared=False,
    )

    function_storage, function_out = make_output(len(CUSTOM_FUNCTION))
    call_backend(library, analyzed, function_out, append=True)
    assert (function_out.status, function_out.count) == (
        BYTE_CLEAN,
        len(CUSTOM_FUNCTION),
    )
    assert output_bytes(function_storage, function_out) == CUSTOM_FUNCTION
    module = TRAP_PRELUDE + CUSTOM_PREDICATE_IR + CUSTOM_FUNCTION
    native_path = compile_ir(
        directory, "scanner_custom_module", module, shared=True
    )
    native = ctypes.CDLL(str(native_path))
    function = native_function(native, CUSTOM_SCANNER)
    for byte in range(256):
        payload = bytes((byte,))
        assert invoke_native(function, payload, 0, 1) == reference(
            payload, 0, 1, keep_ident
        )
    cases = (
        (b"abc_91!tail", 0, 11),
        (b"!abc", 1, 4),
        (b"abc", 2, 2),
        (b"", U64_MAX, 0),
    )
    for payload, start, size in cases:
        assert invoke_native(function, payload, start, size) == reference(
            payload, start, size, keep_ident
        )


def compile_ir(directory, name, ir, *, shared):
    ll = directory / f"{name}.ll"
    output = directory / (
        f"{name}.dylib" if shared and sys.platform == "darwin" else f"{name}.so"
    )
    if not shared:
        output = directory / f"{name}.o"
    ll.write_bytes(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O3"]
    if shared:
        command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    else:
        command += ["-c"]
    command += [str(ll), "-o", str(output)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected scanner IR:\n{result.stderr}")
    return output


def native_function(library, name):
    function = getattr(library, name.decode())
    function.argtypes = [Buffer, ctypes.c_uint64, ctypes.c_uint64]
    function.restype = ctypes.c_uint64
    return function


def keep_ident(byte):
    return ord("a") <= byte <= ord("z") or ord("0") <= byte <= ord("9") or byte == ord("_")


def keep_typeid(byte):
    return (
        ord("a") <= byte <= ord("z")
        or ord("A") <= byte <= ord("Z")
        or ord("0") <= byte <= ord("9")
    )


def keep_number(byte):
    return keep_typeid(byte) or byte in (ord("_"), ord("."), ord("-"))


KEEP = {
    b"lexer_scan_ident": keep_ident,
    b"lexer_scan_typeid": keep_typeid,
    b"lexer_scan_number": keep_number,
}


def invoke_native(function, payload, start, size):
    physical = max(len(payload), 1)
    storage = (ctypes.c_uint8 * physical)()
    for index, value in enumerate(payload):
        storage[index] = value
    source = Buffer(ctypes.cast(storage, ctypes.c_void_p), len(payload))
    return int(function(source, start, size))


def reference(payload, start, size, keep):
    cursor = start
    while cursor < size:
        if cursor >= len(payload):
            raise IndexError
        if not keep(payload[cursor]):
            break
        cursor += 1
    return cursor


def assert_runtime_differential(library_path):
    library = ctypes.CDLL(str(library_path))
    rng = random.Random(0x5343414E)
    calls = 0
    for name, _ in SCANNERS:
        function = native_function(library, name)
        keep = KEEP[name]
        for byte in range(256):
            payload = bytes((byte,))
            assert invoke_native(function, payload, 0, 1) == reference(
                payload, 0, 1, keep
            )
            calls += 1
        for length in range(0, 49):
            payload = bytes(rng.randrange(256) for _ in range(length))
            for _ in range(12):
                size = rng.randrange(length + 1)
                start = rng.randrange(length + 3)
                observed = invoke_native(function, payload, start, size)
                expected = reference(payload, start, size, keep)
                assert observed == expected, (name, payload, start, size)
                calls += 1
        assert invoke_native(function, b"!", 0, 9) == 0
        assert invoke_native(function, b"", U64_MAX, 0) == U64_MAX
    assert calls > 2400


TRAP_SCRIPT = r"""
import ctypes
import sys

class Buffer(ctypes.Structure):
    _fields_ = [("data", ctypes.c_void_p), ("length", ctypes.c_uint64)]

library = ctypes.CDLL(sys.argv[1])
name = sys.argv[2]
payload = bytes.fromhex(sys.argv[3])
size = int(sys.argv[4])
storage = (ctypes.c_uint8 * max(len(payload), 1))()
for index, value in enumerate(payload):
    storage[index] = value
source = Buffer(ctypes.cast(storage, ctypes.c_void_p), len(payload))
function = getattr(library, name)
function.argtypes = [Buffer, ctypes.c_uint64, ctypes.c_uint64]
function.restype = ctypes.c_uint64
function(source, 0, size)
"""


def assert_runtime_traps(library_path):
    trap_signal = signal.SIGTRAP if sys.platform == "darwin" else signal.SIGILL
    expected_returncode = -int(trap_signal)
    payloads = {
        b"lexer_scan_ident": b"abc",
        b"lexer_scan_typeid": b"Ab3",
        b"lexer_scan_number": b"a.-",
    }
    for name, _ in SCANNERS:
        payload = payloads[name]
        result = subprocess.run(
            [
                sys.executable,
                "-c",
                TRAP_SCRIPT,
                str(library_path),
                name.decode(),
                payload.hex(),
                str(len(payload) + 1),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5,
        )
        assert result.returncode == expected_returncode, (
            name,
            result.returncode,
            expected_returncode,
        )


def main():
    assert_ir_shape()
    data = (HERE / "src" / "lexer.wf").read_bytes()
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_library(directory)
        configure(library)
        analyzed_cases = [analyze(library, data, name) for name, _ in SCANNERS]
        assert_renamed_bindings(library, data)
        assert_custom_scanner_pipeline(library, data, directory)
        assert_coordinated_forgeries(library, analyzed_cases)
        assert_hostile_facts(library, analyzed_cases)
        for analyzed in analyzed_cases:
            assert_output_modes(library, analyzed)
            compile_ir(
                directory,
                analyzed["name"].decode() + "_standalone",
                EXPECTED_STANDALONE[analyzed["name"]],
                shared=False,
            )
        module = assert_composition(library, analyzed_cases)
        native_path = compile_ir(directory, "scanner_module", module, shared=True)
        assert_runtime_differential(native_path)
        assert_runtime_traps(native_path)
    print(
        "llvm scanner: ident/typeid/number exact phi+nuw SSA, standalone/append "
        "composition, clang, differential runtime, OOB traps, binding renames, "
        "dynamic scanner/predicate names, and exhaustive hostile-fact rejection pass"
    )


if __name__ == "__main__":
    main()
