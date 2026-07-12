#!/usr/bin/env python3
"""Recognize PRE-1 types and constructors by exact source bytes."""

import ctypes
import subprocess
import sys
import tempfile
from contextlib import contextmanager
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT / "prototype" / "democ"))

import democ


TOK_END = 0
TOK_WORD = 1
TOK_TYPE_ID = 3
TOK_NUMBER = 6
LEX_CLEAN = 0

PRELUDE_TYPE_UNKNOWN = 0
PRELUDE_TYPE_BOOL = 1
PRELUDE_TYPE_OPTION = 2
PRELUDE_TYPE_RESULT = 3
PRELUDE_TYPE_OVERFLOW = 4
PRELUDE_TYPE_DIV_ERROR = 5
PRELUDE_TYPE_NARROW_ERROR = 6

PRELUDE_CONSTRUCTOR_UNKNOWN = 0
PRELUDE_CONSTRUCTOR_TRUE = 1
PRELUDE_CONSTRUCTOR_FALSE = 2
PRELUDE_CONSTRUCTOR_NONE = 3
PRELUDE_CONSTRUCTOR_SOME = 4
PRELUDE_CONSTRUCTOR_OK = 5
PRELUDE_CONSTRUCTOR_ERR = 6
PRELUDE_CONSTRUCTOR_OVERFLOW = 7
PRELUDE_CONSTRUCTOR_DIVIDE_BY_ZERO = 8
PRELUDE_CONSTRUCTOR_DIV_OVERFLOW = 9
PRELUDE_CONSTRUCTOR_NARROW_ERROR = 10


def module_is_wired():
    entries = {
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    }
    return "src/semantic_prelude.xl" in entries


def build_focused_library(directory):
    if module_is_wired():
        return build_library(directory)
    source = compiler_source()
    source += "\n" + (HERE / "src" / "semantic_prelude.xl").read_text()
    ir = democ.compile_program(source, alias=False)
    ll = directory / "semantic_prelude.ll"
    library_path = directory / (
        "semantic_prelude.dylib" if sys.platform == "darwin" else "semantic_prelude.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected semantic prelude IR:\n{result.stderr}")
    return ctypes.CDLL(str(library_path))


def configure(library):
    library.lexer_run.argtypes = [Buffer, ctypes.POINTER(TokenTape)]
    library.lexer_run.restype = None
    arguments = [Buffer, ctypes.POINTER(TokenTape), ctypes.c_uint64]
    library.semantic_prelude_type_code.argtypes = arguments
    library.semantic_prelude_type_code.restype = ctypes.c_int32
    library.semantic_prelude_constructor_code.argtypes = arguments
    library.semantic_prelude_constructor_code.restype = ctypes.c_int32


def lex(library, data):
    source_storage = (ctypes.c_uint8 * max(1, len(data)))(*data)
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
    capacity = len(data) + 1
    kinds = (ctypes.c_int32 * capacity)()
    starts = (ctypes.c_uint64 * capacity)()
    ends = (ctypes.c_uint64 * capacity)()
    tokens = TokenTape(
        Buffer(ctypes.cast(kinds, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(starts, ctypes.c_void_p), capacity),
        Buffer(ctypes.cast(ends, ctypes.c_void_p), capacity),
        0,
        LEX_CLEAN,
        0,
        0,
    )
    library.lexer_run(source, ctypes.byref(tokens))
    assert tokens.status == LEX_CLEAN
    return source_storage, source, (kinds, starts, ends), tokens


def recognize(library, source, tokens, token):
    return (
        library.semantic_prelude_type_code(
            source, ctypes.byref(tokens), token
        ),
        library.semantic_prelude_constructor_code(
            source, ctypes.byref(tokens), token
        ),
    )


def token_text(data, storage, token):
    return data[storage[1][token] : storage[2][token]].decode("ascii")


def exact_fixture():
    return (
        b"Bool Option Result Overflow DivError NarrowError "
        b"True False None Some Ok Err DivideByZero DivOverflow "
        b"BOOL Boo Boolx XBool Option2 ResultX Overflo OverflowX "
        b"DivErro DivErrorX NarrowErro NarrowErrorX "
        b"DivideByZer DivideByZeroX DivOverflo DivOverflowX"
    )


def assert_exact_sets(library):
    data = exact_fixture()
    source_storage, source, token_storage, tokens = lex(library, data)
    type_codes = {
        "Bool": PRELUDE_TYPE_BOOL,
        "Option": PRELUDE_TYPE_OPTION,
        "Result": PRELUDE_TYPE_RESULT,
        "Overflow": PRELUDE_TYPE_OVERFLOW,
        "DivError": PRELUDE_TYPE_DIV_ERROR,
        "NarrowError": PRELUDE_TYPE_NARROW_ERROR,
    }
    constructor_codes = {
        "True": PRELUDE_CONSTRUCTOR_TRUE,
        "False": PRELUDE_CONSTRUCTOR_FALSE,
        "None": PRELUDE_CONSTRUCTOR_NONE,
        "Some": PRELUDE_CONSTRUCTOR_SOME,
        "Ok": PRELUDE_CONSTRUCTOR_OK,
        "Err": PRELUDE_CONSTRUCTOR_ERR,
        "Overflow": PRELUDE_CONSTRUCTOR_OVERFLOW,
        "DivideByZero": PRELUDE_CONSTRUCTOR_DIVIDE_BY_ZERO,
        "DivOverflow": PRELUDE_CONSTRUCTOR_DIV_OVERFLOW,
        "NarrowError": PRELUDE_CONSTRUCTOR_NARROW_ERROR,
    }
    exact_tokens = {}
    for token in range(tokens.count - 1):
        text = token_text(data, token_storage, token)
        observed = recognize(library, source, tokens, token)
        expected = (
            type_codes.get(text, PRELUDE_TYPE_UNKNOWN),
            constructor_codes.get(text, PRELUDE_CONSTRUCTOR_UNKNOWN),
        )
        assert observed == expected, (text, observed, expected)
        if text in type_codes or text in constructor_codes:
            exact_tokens[text] = token

    assert recognize(library, source, tokens, exact_tokens["Overflow"]) == (
        PRELUDE_TYPE_OVERFLOW,
        PRELUDE_CONSTRUCTOR_OVERFLOW,
    )
    assert recognize(library, source, tokens, exact_tokens["NarrowError"]) == (
        PRELUDE_TYPE_NARROW_ERROR,
        PRELUDE_CONSTRUCTOR_NARROW_ERROR,
    )
    assert recognize(library, source, tokens, tokens.count - 1) == (
        PRELUDE_TYPE_UNKNOWN,
        PRELUDE_CONSTRUCTOR_UNKNOWN,
    )
    assert source_storage and token_storage
    return data, source, token_storage, tokens, exact_tokens


@contextmanager
def changed(target, field, value):
    before = getattr(target, field)
    setattr(target, field, value)
    try:
        yield
    finally:
        setattr(target, field, before)


@contextmanager
def changed_index(column, index, value):
    before = column[index]
    column[index] = value
    try:
        yield
    finally:
        column[index] = before


def assert_wrong_classes(library, source, token_storage, tokens, token):
    kinds = token_storage[0]
    for wrong_kind in (TOK_WORD, TOK_NUMBER, TOK_END):
        with changed_index(kinds, token, wrong_kind):
            assert recognize(library, source, tokens, token) == (
                PRELUDE_TYPE_UNKNOWN,
                PRELUDE_CONSTRUCTOR_UNKNOWN,
            )


def assert_hostile_spans(library, data, source, token_storage, tokens, token):
    starts, ends = token_storage[1], token_storage[2]
    hostile = (
        (starts, token, ends[token] + 1),
        (ends, token, starts[token] - 1),
        (ends, token, len(data) + 1),
        (starts, token, (1 << 64) - 1),
    )
    for column, index, value in hostile:
        with changed_index(column, index, value):
            assert recognize(library, source, tokens, token) == (
                PRELUDE_TYPE_UNKNOWN,
                PRELUDE_CONSTRUCTOR_UNKNOWN,
            )

    for bad_token in (tokens.count, tokens.count + 9, (1 << 64) - 1):
        assert recognize(library, source, tokens, bad_token) == (
            PRELUDE_TYPE_UNKNOWN,
            PRELUDE_CONSTRUCTOR_UNKNOWN,
        )

    for field in ("kinds", "starts", "ends"):
        column = getattr(tokens, field)
        with changed(column, "length", 0):
            assert recognize(library, source, tokens, token) == (
                PRELUDE_TYPE_UNKNOWN,
                PRELUDE_CONSTRUCTOR_UNKNOWN,
            )

    with changed(tokens, "count", 0):
        assert recognize(library, source, tokens, token) == (
            PRELUDE_TYPE_UNKNOWN,
            PRELUDE_CONSTRUCTOR_UNKNOWN,
        )


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_focused_library(Path(raw_directory))
        configure(library)
        data, source, token_storage, tokens, exact_tokens = assert_exact_sets(library)
        bool_token = exact_tokens["Bool"]
        assert_wrong_classes(library, source, token_storage, tokens, bool_token)
        assert_hostile_spans(
            library, data, source, token_storage, tokens, bool_token
        )
        print(
            "semantic prelude: 6 types and 10 constructors recognized exactly; "
            "shared spellings keep distinct namespaces; hostile spans fail closed"
        )


if __name__ == "__main__":
    main()
