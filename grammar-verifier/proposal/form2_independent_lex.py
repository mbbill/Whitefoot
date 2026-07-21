from __future__ import annotations

from dataclasses import dataclass
import re


MAX_TOKENS = 1_000_000

RESERVED = frozenset(
    {
        "allocates",
        "arena",
        "array",
        "box",
        "break",
        "buffer",
        "check",
        "conform",
        "const",
        "contract",
        "deref",
        "doc",
        "else",
        "f32",
        "f64",
        "fn",
        "give",
        "heap",
        "i16",
        "i32",
        "i64",
        "i8",
        "index",
        "law",
        "loop",
        "match",
        "move",
        "own",
        "pure",
        "reads",
        "region",
        "requires",
        "return",
        "set",
        "slice",
        "struct",
        "trap",
        "traps",
        "try",
        "u16",
        "u32",
        "u64",
        "u8",
        "uniq",
        "unit",
        "writes",
    }
)
OP_SUFFIXES = frozenset({"wrap", "trap", "checked", "sat", "strict"})
PUNCTUATION = frozenset(b"()<>[]{}.,;:=&")
INTEGER_LITERAL = re.compile(rb"-?[0-9]+_(?:i8|i16|i32|i64|u8|u16|u32|u64)\Z")
FLOAT_LITERAL = re.compile(
    rb"-?(?:0|[1-9][0-9]*)\.[0-9]+(?:e-?(?:0|[1-9][0-9]*))?_(?:f32|f64)\Z"
)
GENERIC_LITERAL = re.compile(rb"[01]_[A-Z][A-Za-z0-9]*\Z")


class IndependentLexError(RuntimeError):
    def __init__(self, offset: int, reason: str):
        super().__init__(f"byte {offset}: {reason}")
        self.offset = offset
        self.reason = reason


@dataclass(frozen=True)
class IndependentToken:
    kind: str
    raw: bytes
    start: int
    end: int

    @property
    def text(self) -> str:
        return self.raw.decode("ascii")


def satisfies_literal(token: IndependentToken) -> bool:
    if token.kind == "STRING" or token.raw == b"unit":
        return True
    if token.kind != "NUMBER":
        return False
    return any(
        pattern.fullmatch(token.raw) is not None
        for pattern in (INTEGER_LITERAL, FLOAT_LITERAL, GENERIC_LITERAL)
    )


def _letter(byte: int) -> bool:
    return 0x41 <= byte <= 0x5A or 0x61 <= byte <= 0x7A


def _lower(byte: int) -> bool:
    return 0x61 <= byte <= 0x7A


def _lower_tail(byte: int) -> bool:
    return _lower(byte) or 0x30 <= byte <= 0x39 or byte == 0x5F


def _type_tail(byte: int) -> bool:
    return _letter(byte) or 0x30 <= byte <= 0x39


def _number_tail(byte: int) -> bool:
    return _letter(byte) or 0x30 <= byte <= 0x39 or byte == 0x5F


def _string(raw: bytes, start: int) -> int:
    cursor = start + 1
    while cursor < len(raw):
        byte = raw[cursor]
        if byte == 0x22:
            return cursor + 1
        if byte == 0x5C:
            if cursor + 1 >= len(raw) or raw[cursor + 1] not in b'\\"n':
                raise IndependentLexError(cursor, "invalid or incomplete string escape")
            cursor += 2
            continue
        if byte < 0x20 or byte > 0x7E or byte in (0x22, 0x5C):
            raise IndependentLexError(cursor, "invalid raw string byte")
        cursor += 1
    raise IndependentLexError(start, "unterminated string")


def _lower_name_end(raw: bytes, start: int) -> int:
    cursor = start + 1
    while cursor < len(raw) and _lower_tail(raw[cursor]):
        cursor += 1
    return cursor


def _type_name_end(raw: bytes, start: int) -> int:
    cursor = start + 1
    while cursor < len(raw) and _type_tail(raw[cursor]):
        cursor += 1
    return cursor


def _number_end(raw: bytes, start: int) -> int:
    cursor = start + int(raw[start] == 0x2D)
    while cursor < len(raw):
        byte = raw[cursor]
        if _number_tail(byte) or byte in b".+-":
            cursor += 1
        else:
            break
    return cursor


def lex_independently(raw: bytes) -> tuple[IndependentToken, ...]:
    try:
        raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise IndependentLexError(error.start, "source is not UTF-8") from error

    result: list[IndependentToken] = []
    cursor = 0
    while cursor < len(raw):
        byte = raw[cursor]
        if byte in b" \n":
            cursor += 1
            continue
        if byte in b"\t\r":
            raise IndependentLexError(cursor, "forbidden source-format byte")
        start = cursor
        if byte == 0x22:
            cursor = _string(raw, start)
            kind = "STRING"
        elif byte in (0x27, 0x40):
            if cursor + 1 >= len(raw) or not _lower(raw[cursor + 1]):
                raise IndependentLexError(start, "malformed prefixed name")
            cursor = _lower_name_end(raw, start + 1)
            kind = "REGIONID" if byte == 0x27 else "LABEL"
        elif _letter(byte):
            cursor = (
                _lower_name_end(raw, start)
                if _lower(byte)
                else _type_name_end(raw, start)
            )
            word = raw[start:cursor].decode("ascii")
            if _lower(byte) and cursor < len(raw) and raw[cursor] == 0x2E:
                suffix_start = cursor + 1
                suffix_end = suffix_start
                while suffix_end < len(raw) and _lower_tail(raw[suffix_end]):
                    suffix_end += 1
                suffix = raw[suffix_start:suffix_end].decode("ascii")
                if suffix in OP_SUFFIXES:
                    cursor = suffix_end
                    word = raw[start:cursor].decode("ascii")
                    kind = "OPNAME"
                else:
                    kind = "IDENT"
            elif 0x41 <= byte <= 0x5A:
                kind = "TYPEID"
            elif word in RESERVED:
                kind = "FIXED"
            else:
                kind = "IDENT"
        elif 0x30 <= byte <= 0x39 or (
            byte == 0x2D
            and cursor + 1 < len(raw)
            and 0x30 <= raw[cursor + 1] <= 0x39
        ):
            cursor = _number_end(raw, start)
            kind = "NUMBER"
        elif raw.startswith(b"->", cursor):
            cursor += 2
            kind = "ARROW"
        elif raw.startswith(b"=>", cursor):
            cursor += 2
            kind = "FAT_ARROW"
        elif byte in PUNCTUATION:
            cursor += 1
            kind = "PUNCT"
        else:
            raise IndependentLexError(start, f"unclassified source byte 0x{byte:02x}")
        result.append(IndependentToken(kind, raw[start:cursor], start, cursor))
        if len(result) > MAX_TOKENS:
            raise IndependentLexError(start, "token limit exceeded")
    return tuple(result)


def terminal_projection(tokens: tuple[IndependentToken, ...]) -> bytes:
    output = bytearray()
    for token in tokens:
        output.extend(len(token.raw).to_bytes(8, "big"))
        output.extend(token.raw)
    return bytes(output)
