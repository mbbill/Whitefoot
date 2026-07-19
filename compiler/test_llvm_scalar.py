#!/usr/bin/env python3
"""Lower the first typed scalar function from semantic facts to executable LLVM."""

import ctypes
import subprocess
import sys
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape
from test_parser import AST_NONE, AstTape, children_of, parse
from test_ast_validate import AstValidationReport, validate
from test_semantic_body import configure as configure_semantic_body, index_symbols
from test_semantic_facts import (
    NODE_COLUMNS,
    TYPE_COLUMNS,
    NodeFacts,
    TypeTape,
    assert_guards,
    make_tape,
)
from test_symbols import SymbolTape
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

SEMANTIC_TYPE_U64 = 3
SEMANTIC_MODE_NONE = 0
SEMANTIC_OPERATION_IADD_TRAP = 1
SEMANTIC_OPERATION_ILE = 7
SEMANTIC_FACTS_CLEAN = 0
SEMANTIC_FACTS_INVALID_SHAPE = 1
SEMANTIC_BODY_CLEAN = 0
SCRATCH_U64_GUARD = 0xB4B4B4B4B4B4B4B4
SCRATCH_MODE_GUARD = 0x4B4B4B4B


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


SOURCE = b"""fn lexer_is_lower (c: own u8) -> own Bool pure {
  let ge: own Bool = ige<u8>(c, 97_u8);
  let le: own Bool = ile<u8>(c, 122_u8);
  return band<Bool>(ge, le);
}
"""

EXPECTED = b"""define i1 @lexer_is_lower(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 97
  %v1 = icmp ule i8 %p0, 122
  %v2 = and i1 %v0, %v1
  ret i1 %v2
}
"""

ALTERNATE_SOURCE = b"""fn lexer_is_upper (c: own u8) -> own Bool pure {
  let ge: own Bool = ige<u8>(c, 65_u8);
  let le: own Bool = ile<u8>(c, 90_u8);
  return band<Bool>(ge, le);
}
"""

ALTERNATE_EXPECTED = b"""define i1 @lexer_is_upper(i8 %p0) {
entry:
  %v0 = icmp uge i8 %p0, 65
  %v1 = icmp ule i8 %p0, 90
  %v2 = and i1 %v0, %v1
  ret i1 %v2
}
"""


def compiler_source_isolated():
    names = [
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    ]
    excluded = {
        "src/frontend.wf",
        "src/semantic_body.wf",
        "src/llvm_scalar.wf",
    }
    paths = [HERE / name for name in names if name not in excluded]
    paths.append(HERE / "src" / "semantic_body.wf")
    paths.append(HERE / "src" / "llvm_scalar.wf")
    return "\n\n".join(path.read_text().rstrip("\n") for path in paths) + "\n"


def build_scalar_library(directory):
    source = compiler_source_isolated()
    ir = democ.compile_program(source, alias=False)
    forbidden_facts = [
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
    assert not forbidden_facts, forbidden_facts
    ll = directory / "llvm_scalar_stage0.ll"
    library_path = directory / (
        "llvm_scalar_stage0.dylib" if sys.platform == "darwin" else "llvm_scalar_stage0.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected the isolated scalar emitter:\n{result.stderr}")
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
    library.llvm_scalar_emit_function.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(ByteTape),
    ]
    library.llvm_scalar_emit_function.restype = None


def scalar_nodes(ast_storage, ast):
    assert ast.status == 0
    assert ast.count == 31
    root_children = children_of(ast_storage, ast.root)
    assert len(root_children) == 1
    function = root_children[0]
    name, parameter, return_mode, return_type, pure, block = children_of(
        ast_storage, function
    )
    parameter_mode, parameter_type = children_of(ast_storage, parameter)
    first_let, second_let, return_node = children_of(ast_storage, block)
    first_name, first_mode, first_type, first_call = children_of(ast_storage, first_let)
    second_name, second_mode, second_type, second_call = children_of(
        ast_storage, second_let
    )
    first_type_argument, first_place, first_numeric = children_of(
        ast_storage, first_call
    )
    second_type_argument, second_place, second_numeric = children_of(
        ast_storage, second_call
    )
    (return_call,) = children_of(ast_storage, return_node)
    return_type_argument, return_left, return_right = children_of(
        ast_storage, return_call
    )
    return {
        name: "name",
        parameter: "parameter",
        return_mode: "return_mode",
        return_type: "return_type",
        pure: "pure",
        block: "block",
        parameter_mode: "parameter_mode",
        parameter_type: "parameter_type",
        first_let: "first_let",
        second_let: "second_let",
        return_node: "return",
        first_name: "first_name",
        first_mode: "first_mode",
        first_type: "first_type",
        first_call: "first_call",
        second_name: "second_name",
        second_mode: "second_mode",
        second_type: "second_type",
        second_call: "second_call",
        first_type_argument: "first_type_argument",
        first_place: "first_place",
        first_numeric: "first_numeric",
        second_type_argument: "second_type_argument",
        second_place: "second_place",
        second_numeric: "second_numeric",
        return_call: "return_call",
        return_type_argument: "return_type_argument",
        return_left: "return_left",
        return_right: "return_right",
        function: "function",
    }


def invert_nodes(nodes):
    return {name: node for node, name in nodes.items()}


def analyze_semantic_body(
    library, source_storage, tokens, ast_storage, ast, nodes
):
    validation = validate(library, len(source_storage), tokens.count, ast)
    assert (validation.status, validation.node, validation.related) == (
        0,
        AST_NONE,
        AST_NONE,
    )
    source = source_buffer(source_storage)
    symbol_storage, symbol_physical, symbols = index_symbols(
        library, source, tokens, ast_storage, ast
    )

    type_capacities = (2,) * len(TYPE_COLUMNS)
    type_storage, types = make_tape(TypeTape, TYPE_COLUMNS, type_capacities)
    types.count = 0
    types.status = SEMANTIC_FACTS_CLEAN
    types.node = U64_MAX
    types.related = U64_MAX

    fact_capacities = (ast.count,) * len(NODE_COLUMNS)
    fact_storage, facts = make_tape(NodeFacts, NODE_COLUMNS, fact_capacities)
    facts.count = 0
    facts.status = SEMANTIC_FACTS_CLEAN
    facts.node = U64_MAX
    facts.related = U64_MAX

    scratch_capacity = 3
    name_tokens = (ctypes.c_uint64 * (scratch_capacity + 1))()
    declarations = (ctypes.c_uint64 * (scratch_capacity + 1))()
    type_ids = (ctypes.c_uint64 * (scratch_capacity + 1))()
    modes = (ctypes.c_int32 * (scratch_capacity + 1))()
    loop_labels = (ctypes.c_uint64 * (scratch_capacity + 1))()
    for column in (name_tokens, declarations, type_ids, loop_labels):
        column[scratch_capacity] = SCRATCH_U64_GUARD
    modes[scratch_capacity] = SCRATCH_MODE_GUARD
    scratch = SemanticBodyScratch(
        Buffer(ctypes.cast(name_tokens, ctypes.c_void_p), scratch_capacity),
        Buffer(ctypes.cast(declarations, ctypes.c_void_p), scratch_capacity),
        Buffer(ctypes.cast(type_ids, ctypes.c_void_p), scratch_capacity),
        Buffer(ctypes.cast(modes, ctypes.c_void_p), scratch_capacity),
        0,
        Buffer(ctypes.cast(loop_labels, ctypes.c_void_p), scratch_capacity),
        0,
    )
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    library.semantic_body_run(
        source,
        ctypes.byref(tokens),
        ctypes.byref(ast),
        nodes["function"],
        ctypes.byref(validation),
        ctypes.byref(symbols),
        ctypes.byref(types),
        ctypes.byref(facts),
        ctypes.byref(scratch),
        ctypes.byref(report),
    )
    assert (report.status, report.node, report.related) == (
        SEMANTIC_BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert (types.count, types.status, facts.count, facts.status) == (
        2,
        SEMANTIC_FACTS_CLEAN,
        ast.count,
        SEMANTIC_FACTS_CLEAN,
    )
    assert scratch.count == scratch_capacity
    assert name_tokens[scratch_capacity] == SCRATCH_U64_GUARD
    assert declarations[scratch_capacity] == SCRATCH_U64_GUARD
    assert type_ids[scratch_capacity] == SCRATCH_U64_GUARD
    assert modes[scratch_capacity] == SCRATCH_MODE_GUARD
    assert symbol_storage and symbol_physical >= symbols.count
    assert_guards(type_storage, TYPE_COLUMNS, type_capacities)
    assert_guards(fact_storage, NODE_COLUMNS, fact_capacities)
    return (
        type_storage,
        type_capacities,
        types,
        fact_storage,
        fact_capacities,
        facts,
    )


def source_buffer(source_storage):
    return Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(source_storage))


def emit(library, source_storage, tokens, ast, function, types, facts, out):
    library.llvm_scalar_emit_function(
        source_buffer(source_storage),
        ctypes.byref(tokens),
        ctypes.byref(ast),
        function,
        ctypes.byref(types),
        ctypes.byref(facts),
        ctypes.byref(out),
    )


def assert_output_modes(
    library, source_storage, tokens, ast, function, types, facts
):
    measured_storage, measured = make_output(0)
    emit(library, source_storage, tokens, ast, function, types, facts, measured)
    assert (measured.status, measured.count) == (BYTE_NEED_CAPACITY, len(EXPECTED))
    assert measured_storage[0] == GUARD

    exact_storage, exact = make_output(len(EXPECTED))
    emit(library, source_storage, tokens, ast, function, types, facts, exact)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED))
    assert bytes(exact_storage[: exact.count]) == EXPECTED
    assert exact_storage[len(EXPECTED)] == GUARD

    short_capacity = len(EXPECTED) - 13
    short_storage, short = make_output(short_capacity)
    emit(library, source_storage, tokens, ast, function, types, facts, short)
    assert (short.status, short.count) == (BYTE_NEED_CAPACITY, len(EXPECTED))
    assert bytes(short_storage[:short_capacity]) == EXPECTED[:short_capacity]
    assert short_storage[short_capacity] == GUARD

    for ordinal in range(len(EXPECTED)):
        exact_storage[ordinal] = POISON
    exact.count = U64_MAX
    exact.status = BYTE_INVALID_STATE
    emit(library, source_storage, tokens, ast, function, types, facts, exact)
    assert (exact.status, exact.count) == (BYTE_CLEAN, len(EXPECTED))
    assert bytes(exact_storage[: exact.count]) == EXPECTED
    assert exact_storage[len(EXPECTED)] == GUARD
    return bytes(exact_storage[: exact.count])


def assert_invalid(
    library, source_storage, tokens, ast, function, types, facts
):
    storage, out = make_output(len(EXPECTED))
    emit(library, source_storage, tokens, ast, function, types, facts, out)
    assert (out.status, out.count) == (BYTE_INVALID_STATE, 0)
    assert all(byte == POISON for byte in storage[: len(EXPECTED)])
    assert storage[len(EXPECTED)] == GUARD


def mutate_restore(target, index, replacement, action):
    prior = target[index]
    target[index] = replacement
    try:
        action()
    finally:
        target[index] = prior


def assert_malformed_inputs(
    library,
    source_storage,
    token_storage,
    tokens,
    ast_storage,
    ast,
    nodes,
    type_storage,
    types,
    fact_storage,
    facts,
):
    function = nodes["function"]

    def reject():
        assert_invalid(library, source_storage, tokens, ast, function, types, facts)

    for replacement in (U64_MAX, ast.count + 1):
        mutate_restore(ast_storage[4], function, replacement, reject)
        mutate_restore(ast_storage[6], nodes["name"], replacement, reject)
        mutate_restore(ast_storage[4], nodes["first_call"], replacement, reject)

    mutate_restore(ast_storage[0], nodes["first_call"], 15, reject)
    mutate_restore(ast_storage[1], nodes["name"], tokens.count, reject)

    name_token = ast_storage[1][nodes["name"]]
    mutate_restore(token_storage[2], name_token, len(SOURCE) + 1, reject)
    name_start = token_storage[1][name_token]
    mutate_restore(source_storage, name_start, ord("@"), reject)
    prior_token_status = tokens.status
    tokens.status = 1
    try:
        reject()
    finally:
        tokens.status = prior_token_status

    mutate_restore(
        fact_storage[3], nodes["first_call"], SEMANTIC_OPERATION_IADD_TRAP, reject
    )
    mutate_restore(fact_storage[4], nodes["first_numeric"], 256, reject)
    mutate_restore(fact_storage[1], nodes["first_place"], U64_MAX, reject)
    mutate_restore(fact_storage[2], nodes["second_let"], 0, reject)
    mutate_restore(fact_storage[6], nodes["return_call"], SEMANTIC_MODE_NONE, reject)
    mutate_restore(fact_storage[0], nodes["return_right"], U64_MAX, reject)
    mutate_restore(type_storage[0], 0, SEMANTIC_TYPE_U64, reject)

    prior_facts_status = facts.status
    facts.status = SEMANTIC_FACTS_INVALID_SHAPE
    try:
        reject()
    finally:
        facts.status = prior_facts_status

    prior_fact_count = facts.count
    facts.count = ast.count - 1
    try:
        reject()
    finally:
        facts.count = prior_fact_count

    prior_type_ids_length = facts.type_ids.length
    facts.type_ids.length = ast.count - 1
    try:
        reject()
    finally:
        facts.type_ids.length = prior_type_ids_length

    prior_kinds_length = types.kinds.length
    types.kinds.length = 1
    try:
        reject()
    finally:
        types.kinds.length = prior_kinds_length


def assert_fact_driven_lowering(
    library, source_storage, tokens, ast, nodes, types, fact_storage, facts
):
    first_call = nodes["first_call"]
    first_numeric = nodes["first_numeric"]
    prior_operation = fact_storage[3][first_call]
    prior_constant = fact_storage[4][first_numeric]
    fact_storage[3][first_call] = SEMANTIC_OPERATION_ILE
    fact_storage[4][first_numeric] = 42
    try:
        storage, out = make_output(len(EXPECTED))
        emit(
            library,
            source_storage,
            tokens,
            ast,
            nodes["function"],
            types,
            facts,
            out,
        )
        observed = bytes(storage[: out.count])
        assert out.status == BYTE_CLEAN
        assert b"%v0 = icmp ule i8 %p0, 42\n" in observed
        assert b"%v0 = icmp uge" not in observed
    finally:
        fact_storage[3][first_call] = prior_operation
        fact_storage[4][first_numeric] = prior_constant

    replacements = ((b"ige", b"zzz"), (b"ile", b"qqq"), (b"band", b"xxxx"))
    changed = []
    for old, new in replacements:
        start = SOURCE.index(old)
        changed.append((start, bytes(source_storage[start : start + len(old)])))
        for offset, byte in enumerate(new):
            source_storage[start + offset] = byte
    try:
        storage, out = make_output(len(EXPECTED))
        emit(
            library,
            source_storage,
            tokens,
            ast,
            nodes["function"],
            types,
            facts,
            out,
        )
        assert (out.status, out.count) == (BYTE_CLEAN, len(EXPECTED))
        assert bytes(storage[: out.count]) == EXPECTED
    finally:
        for start, original in changed:
            for offset, byte in enumerate(original):
                source_storage[start + offset] = byte


def assert_alternate_same_shape(library):
    source_storage, _, tokens, ast_storage, ast = parse(library, ALTERNATE_SOURCE)
    nodes = invert_nodes(scalar_nodes(ast_storage, ast))
    _, _, types, fact_storage, _, facts = analyze_semantic_body(
        library, source_storage, tokens, ast_storage, ast, nodes
    )
    storage, out = make_output(len(ALTERNATE_EXPECTED))
    emit(
        library,
        source_storage,
        tokens,
        ast,
        nodes["function"],
        types,
        facts,
        out,
    )
    assert (out.status, out.count) == (BYTE_CLEAN, len(ALTERNATE_EXPECTED))
    assert bytes(storage[: out.count]) == ALTERNATE_EXPECTED
    assert storage[len(ALTERNATE_EXPECTED)] == GUARD


def assert_executable_ir(directory, ir):
    ll = directory / "lexer_is_lower.ll"
    library_path = directory / (
        "lexer_is_lower.dylib" if sys.platform == "darwin" else "lexer_is_lower.so"
    )
    ll.write_bytes(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O3"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected generated lexer_is_lower IR:\n{result.stderr}")
    generated = ctypes.CDLL(str(library_path))
    function = generated.lexer_is_lower
    function.argtypes = [ctypes.c_uint8]
    function.restype = ctypes.c_bool
    for value in range(256):
        assert function(value) is (97 <= value <= 122), value


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        directory = Path(raw_directory)
        library = build_scalar_library(directory)
        configure(library)

        source_storage, token_storage, tokens, ast_storage, ast = parse(library, SOURCE)
        by_node = scalar_nodes(ast_storage, ast)
        nodes = invert_nodes(by_node)
        (
            type_storage,
            type_capacities,
            types,
            fact_storage,
            fact_capacities,
            facts,
        ) = analyze_semantic_body(
            library, source_storage, tokens, ast_storage, ast, nodes
        )

        generated_ir = assert_output_modes(
            library,
            source_storage,
            tokens,
            ast,
            nodes["function"],
            types,
            facts,
        )
        assert_fact_driven_lowering(
            library,
            source_storage,
            tokens,
            ast,
            nodes,
            types,
            fact_storage,
            facts,
        )
        assert_alternate_same_shape(library)
        assert_malformed_inputs(
            library,
            source_storage,
            token_storage,
            tokens,
            ast_storage,
            ast,
            nodes,
            type_storage,
            types,
            fact_storage,
            facts,
        )
        assert_guards(type_storage, TYPE_COLUMNS, type_capacities)
        assert_guards(fact_storage, NODE_COLUMNS, fact_capacities)
        assert_executable_ir(directory, generated_ir)

        print(
            "typed scalar LLVM: exact/measure/short/repeat, fact-driven lowering, "
            "hostile tapes, clang acceptance, and all 256 u8 inputs pass"
        )


if __name__ == "__main__":
    main()
