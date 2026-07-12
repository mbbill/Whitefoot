#!/usr/bin/env python3
"""Exercise the first fixed buffer type rows and lexer_match3 signature gate."""

import ctypes
import subprocess
import sys
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_NONE, AstTape, children_of, parse
from test_parser_expressions import enum_ordinals
from test_semantic_facts import (
    TYPE_COLUMNS,
    TypeTape,
    assert_guards,
    make_tape,
)


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT / "prototype" / "democ"))

import democ


SEMANTIC_FACTS_CLEAN = 0
SEMANTIC_TYPE_U8 = 2
SEMANTIC_TYPE_U64 = 3
SEMANTIC_TYPE_BOOL = 4
SEMANTIC_TYPE_BUFFER = 7
PRELUDE_TYPE_UNKNOWN = 0
PRELUDE_TYPE_BOOL = 1
U64_MAX = (1 << 64) - 1
AST = enum_ordinals("AstKind")


def module_is_wired():
    entries = {
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    }
    return "src/semantic_buffer.xl" in entries


def build_focused_library(directory):
    if module_is_wired():
        return build_library(directory)
    source = compiler_source()
    source += "\n" + (HERE / "src" / "semantic_buffer.xl").read_text()
    ir = democ.compile_program(source, alias=False)
    ll = directory / "semantic_buffer.ll"
    library_path = directory / (
        "semantic_buffer.dylib" if sys.platform == "darwin" else "semantic_buffer.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected semantic buffer IR:\n{result.stderr}")
    return ctypes.CDLL(str(library_path))


def configure(library):
    library.lexer_run.argtypes = [Buffer, ctypes.POINTER(TokenTape)]
    library.lexer_run.restype = None
    library.parser_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
    ]
    library.parser_run.restype = None
    library.semantic_type_tape_reset.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_reset.restype = None
    library.semantic_buffer_type_capacity_valid.argtypes = [
        ctypes.POINTER(TypeTape)
    ]
    library.semantic_buffer_type_capacity_valid.restype = ctypes.c_bool
    library.semantic_buffer_initialize_types.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_buffer_initialize_types.restype = None
    library.semantic_buffer_match3_signature_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_match3_signature_valid.restype = ctypes.c_bool


def assert_type_initializer(library):
    capacities = (4,) * len(TYPE_COLUMNS)
    storage, types = make_tape(TypeTape, TYPE_COLUMNS, capacities)
    library.semantic_type_tape_reset(ctypes.byref(types))
    assert library.semantic_buffer_type_capacity_valid(ctypes.byref(types))

    library.semantic_buffer_initialize_types(ctypes.byref(types))

    assert list(storage[0][:4]) == [
        SEMANTIC_TYPE_U8,
        SEMANTIC_TYPE_BOOL,
        SEMANTIC_TYPE_U64,
        SEMANTIC_TYPE_BUFFER,
    ]
    assert list(storage[1][:4]) == [U64_MAX] * 4
    assert list(storage[2][:4]) == [U64_MAX, U64_MAX, U64_MAX, 0]
    assert list(storage[3][:4]) == [U64_MAX] * 4
    assert list(storage[4][:4]) == [U64_MAX] * 4
    assert list(storage[5][:4]) == [
        PRELUDE_TYPE_UNKNOWN,
        PRELUDE_TYPE_BOOL,
        PRELUDE_TYPE_UNKNOWN,
        PRELUDE_TYPE_UNKNOWN,
    ]
    assert (types.count, types.status, types.node, types.related) == (
        4,
        SEMANTIC_FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert_guards(storage, TYPE_COLUMNS, capacities)

    for short_column in range(len(TYPE_COLUMNS)):
        short_capacities = [4] * len(TYPE_COLUMNS)
        short_capacities[short_column] = 3
        short_storage, short_types = make_tape(
            TypeTape, TYPE_COLUMNS, tuple(short_capacities)
        )
        library.semantic_type_tape_reset(ctypes.byref(short_types))
        assert not library.semantic_buffer_type_capacity_valid(
            ctypes.byref(short_types)
        )
        assert_guards(short_storage, TYPE_COLUMNS, tuple(short_capacities))

    library.semantic_type_tape_reset(ctypes.byref(types))
    types.count = 1
    assert not library.semantic_buffer_type_capacity_valid(ctypes.byref(types))
    assert_guards(storage, TYPE_COLUMNS, capacities)

    types.count = U64_MAX
    assert not library.semantic_buffer_type_capacity_valid(ctypes.byref(types))
    assert_guards(storage, TYPE_COLUMNS, capacities)


def real_match3_source():
    lexer = (HERE / "src" / "lexer.xl").read_bytes()
    start = lexer.index(b"fn lexer_match3 ")
    end = lexer.index(b"\nfn lexer_match4 ", start)
    return lexer[start:end].rstrip() + b"\n"


def parsed_match3(library):
    data = real_match3_source()
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    functions = children_of(columns, ast.root)
    assert len(functions) == 1
    function = functions[0]
    direct = children_of(columns, function)
    assert len(direct) == 12
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
    return source_storage, source, token_storage, tokens, columns, ast, function


def signature_valid(library, source, tokens, ast, function):
    return bool(
        library.semantic_buffer_match3_signature_valid(
            source,
            ctypes.byref(tokens),
            ctypes.byref(ast),
            function,
        )
    )


def assert_real_signature(library):
    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    assert signature_valid(library, source, tokens, ast, function)


def signature_nodes(columns, function):
    direct = children_of(columns, function)
    regions = direct[1]
    source_parameter = direct[2]
    source_mode = children_of(columns, source_parameter)[0]
    reads_effect = direct[9]
    return {
        "regions": regions,
        "source mode": source_mode,
        "reads effect": reads_effect,
    }


def assert_hostile_ast_fails_closed(library):
    # These are the three containers whose absent child used to feed AST_NONE
    # into a later head lookup. Each corruption must be rejected before that lookup.
    for label in ("regions", "source mode", "reads effect"):
        fixture = parsed_match3(library)
        _, source, _, tokens, columns, ast, function = fixture
        node = signature_nodes(columns, function)[label]
        columns[4][node] = AST_NONE
        columns[5][node] = AST_NONE
        assert not signature_valid(library, source, tokens, ast, function), label

    # A non-terminating direct-child chain is malformed but remains in-bounds.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    direct = children_of(columns, function)
    columns[6][direct[0]] = direct[0]
    assert not signature_valid(library, source, tokens, ast, function)

    # Invalid public ordinals and truncated advertised columns fail before reads.
    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, _ = fixture
    assert not signature_valid(library, source, tokens, ast, ast.count)
    assert not signature_valid(library, source, tokens, ast, U64_MAX)

    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    tokens.ends.length = tokens.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    ast.heads.length = ast.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    # Out-of-range child links must be rejected without indexing that child.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    regions = children_of(columns, function)[1]
    columns[4][regions] = ast.count
    columns[5][regions] = ast.count
    assert not signature_valid(library, source, tokens, ast, function)

    # A one-byte parameter head aimed at the zero-width EOF token used to reach
    # parser_word_byte eagerly. It must now fail at the width gate.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    a_parameter = children_of(columns, function)[4]
    columns[1][a_parameter] = tokens.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    # Keep the twelve-child function shape while corrupting gated node kinds.
    for child_ordinal, wrong_kind in (
        (10, AST["AstPureEffect"]),
        (11, AST["AstReturn"]),
    ):
        fixture = parsed_match3(library)
        _, source, _, tokens, columns, ast, function = fixture
        child = children_of(columns, function)[child_ordinal]
        columns[0][child] = wrong_kind
        assert not signature_valid(library, source, tokens, ast, function)


def assert_wrong_signature_fails(library):
    variants = (
        (b"lexer_match3", b"lexer_matchx"),
        (b"['s]", b"['r]"),
        (b"source: &'s buffer<u8>", b"source: own buffer<u8>"),
        (b"source: &'s buffer<u8>", b"source: &'s buffer<u64>"),
        (b"source: &'s buffer<u8>", b"input: &'s buffer<u8>"),
        (b"start: own u64", b"offset: own u64"),
        (b"start: own u64", b"start: own u8"),
        (b"a: own u8", b"x: own u8"),
        (b"b: own u8", b"b: own u64"),
        (b"c: own u8", b"c: own Bool"),
        (b"-> own Bool", b"-> &'s Bool"),
        (b"-> own Bool", b"-> own u8"),
        (b"reads('s), traps", b"reads('other), traps"),
        (b"reads('s), traps", b"reads('s), writes('s)"),
        (b"reads('s), traps", b"reads('s)"),
    )
    exact = real_match3_source()
    for before, after in variants:
        data = exact.replace(before, after, 1)
        assert data != exact
        source_storage, token_storage, tokens, columns, ast = parse(library, data)
        assert ast.status == 0
        function = children_of(columns, ast.root)[0]
        source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
        assert not signature_valid(library, source, tokens, ast, function)
        assert token_storage and columns


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_focused_library(Path(raw_directory))
        configure(library)
        assert_type_initializer(library)
        assert_real_signature(library)
        assert_hostile_ast_fails_closed(library)
        assert_wrong_signature_fails(library)
    print(
        "semantic buffer: canonical 4-row types, exact real lexer_match3 "
        "signature, and hostile malformed ASTs pass"
    )


if __name__ == "__main__":
    main()
