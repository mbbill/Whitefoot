#!/usr/bin/env python3
"""Independent byte-oriented lexical reference model for Whitefoot v0.9.

The model partitions ordered raw sources into shape-only token and trivia
pieces.  It deliberately does not parse, check canonical layout, interpret
numeric values, or produce normative diagnostics.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Literal, Sequence, Union


SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
STATIC_CATALOG_SHA256 = (
    "3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae"
)

U32_MAX = (1 << 32) - 1
U64_MAX = (1 << 64) - 1

OUTCOME_KINDS = frozenset(("complete", "source_issue", "resource_failure"))

TOKEN_KINDS = frozenset(
    (
        "lower_word_form",
        "upper_word_form",
        "region_form",
        "label_form",
        "operation_name_form",
        "number_form",
        "string_form",
        "left_paren",
        "right_paren",
        "left_brace",
        "right_brace",
        "left_bracket",
        "right_bracket",
        "left_angle",
        "right_angle",
        "comma",
        "colon",
        "semicolon",
        "dot",
        "equal",
        "thin_arrow",
        "fat_arrow",
        "ampersand",
    )
)
TRIVIA_KINDS = frozenset(("spaces", "line_feed"))
PIECE_KINDS = TOKEN_KINDS | TRIVIA_KINDS

SOURCE_ISSUE_KINDS = frozenset(
    (
        "invalid_utf8",
        "unexpected_byte",
        "missing_region_name",
        "missing_label_name",
        "unterminated_string",
        "invalid_string_byte",
        "invalid_string_escape",
    )
)

RESOURCE_LIMIT_KINDS = frozenset(
    (
        "sources",
        "source_bytes",
        "total_source_bytes",
        "token_bytes",
        "tokens",
        "lexemes",
    )
)

_OPERATION_SUFFIXES = (b"checked", b"strict", b"wrap", b"trap", b"sat")

_FIXED_TOKENS = {
    b"(": "left_paren",
    b")": "right_paren",
    b"{": "left_brace",
    b"}": "right_brace",
    b"[": "left_bracket",
    b"]": "right_bracket",
    b"<": "left_angle",
    b">": "right_angle",
    b",": "comma",
    b":": "colon",
    b";": "semicolon",
    b".": "dot",
    b"=": "equal",
    b"->": "thin_arrow",
    b"=>": "fat_arrow",
    b"&": "ampersand",
}


@dataclass(frozen=True)
class LexLimits:
    """Caller-selected inclusive ceilings for one model run."""

    max_sources: int
    max_source_bytes: int
    max_total_source_bytes: int
    max_token_bytes: int
    max_tokens: int
    max_lexemes: int

    def __post_init__(self) -> None:
        for name, value in vars(self).items():
            maximum = U32_MAX if name == "max_sources" else U64_MAX
            if type(value) is not int or not 0 <= value <= maximum:
                raise ValueError(
                    f"{name} must be an integer from 0 through {maximum}"
                )


@dataclass(frozen=True)
class Piece:
    """One exact nonempty member of a source-local lexical partition."""

    source_ordinal: int
    start: int
    end: int
    kind: str
    exact: bytes

    def __post_init__(self) -> None:
        if self.kind not in PIECE_KINDS:
            raise ValueError(f"unknown lexical piece kind: {self.kind}")
        if self.source_ordinal < 0 or self.start < 0 or self.end <= self.start:
            raise ValueError("lexical piece coordinates are not a nonempty span")
        if self.end - self.start != len(self.exact):
            raise ValueError("lexical piece bytes do not match its span")

    @property
    def is_token(self) -> bool:
        """Whether this piece is token-shaped rather than retained trivia."""

        return self.kind in TOKEN_KINDS


@dataclass(frozen=True)
class Complete:
    """Every input byte has exactly one source-local partition owner."""

    pieces: tuple[Piece, ...]
    source_ranges: tuple[tuple[int, int], ...]
    token_count: int
    outcome: Literal["complete"] = field(default="complete", init=False)
    spec_sha256: str = field(default=SPEC_SHA256, init=False)
    static_catalog_sha256: str = field(default=STATIC_CATALOG_SHA256, init=False)

    def source_pieces(self, source_ordinal: int) -> tuple[Piece, ...] | None:
        """Return one source partition, including a distinct empty partition."""

        if not 0 <= source_ordinal < len(self.source_ranges):
            return None
        start, end = self.source_ranges[source_ordinal]
        return self.pieces[start:end]


@dataclass(frozen=True)
class SourceIssue:
    """A source-local byte shape prevented a complete partition."""

    source_ordinal: int
    start: int
    end: int
    kind: str
    exact: bytes
    outcome: Literal["source_issue"] = field(default="source_issue", init=False)
    spec_sha256: str = field(default=SPEC_SHA256, init=False)
    static_catalog_sha256: str = field(default=STATIC_CATALOG_SHA256, init=False)

    def __post_init__(self) -> None:
        if self.kind not in SOURCE_ISSUE_KINDS:
            raise ValueError(f"unknown source issue kind: {self.kind}")
        if self.source_ordinal < 0 or self.start < 0 or self.end <= self.start:
            raise ValueError("source issue coordinates are not a nonempty span")
        if self.end - self.start != len(self.exact):
            raise ValueError("source issue bytes do not match its span")


@dataclass(frozen=True)
class ResourceFailure:
    """An explicit model ceiling was exceeded before publication."""

    limit: str
    maximum: int
    actual: int
    outcome: Literal["resource_failure"] = field(
        default="resource_failure", init=False
    )
    spec_sha256: str = field(default=SPEC_SHA256, init=False)
    static_catalog_sha256: str = field(default=STATIC_CATALOG_SHA256, init=False)

    def __post_init__(self) -> None:
        if self.limit not in RESOURCE_LIMIT_KINDS:
            raise ValueError(f"unknown resource limit kind: {self.limit}")
        if self.maximum < 0 or self.actual <= self.maximum:
            raise ValueError("resource failure must identify the first excess value")


LexOutcome = Union[Complete, SourceIssue, ResourceFailure]


@dataclass(frozen=True)
class _RawPiece:
    start: int
    end: int
    kind: str


@dataclass(frozen=True)
class _RawIssue:
    start: int
    end: int
    kind: str


def lex_v0_9(sources: Sequence[bytes], limits: LexLimits) -> LexOutcome:
    """Partition ordered immutable source bytes under explicit ceilings.

    Publication is failure-atomic: only ``Complete`` contains partition pieces.
    Source boundaries are lexical boundaries, including for empty sources.
    """

    source_count = len(sources)
    if source_count > limits.max_sources:
        return ResourceFailure("sources", limits.max_sources, source_count)
    frozen_sources = tuple(_require_bytes(source) for source in sources)

    total_source_bytes = sum(len(source) for source in frozen_sources)
    if total_source_bytes > limits.max_total_source_bytes:
        return ResourceFailure(
            "total_source_bytes",
            limits.max_total_source_bytes,
            total_source_bytes,
        )

    pieces: list[Piece] = []
    source_ranges: list[tuple[int, int]] = []
    token_count = 0

    for source_ordinal, source in enumerate(frozen_sources):
        if len(source) > limits.max_source_bytes:
            return ResourceFailure(
                "source_bytes", limits.max_source_bytes, len(source)
            )
        source_start = len(pieces)
        cursor = 0
        while cursor < len(source):
            scanned = _scan_one(source, cursor)
            if isinstance(scanned, _RawIssue):
                return SourceIssue(
                    source_ordinal,
                    scanned.start,
                    scanned.end,
                    scanned.kind,
                    source[scanned.start : scanned.end],
                )

            lexeme_actual = len(pieces) + 1
            if lexeme_actual > limits.max_lexemes:
                return ResourceFailure(
                    "lexemes", limits.max_lexemes, lexeme_actual
                )

            if scanned.kind in TOKEN_KINDS:
                token_bytes = scanned.end - scanned.start
                if token_bytes > limits.max_token_bytes:
                    return ResourceFailure(
                        "token_bytes", limits.max_token_bytes, token_bytes
                    )
                token_actual = token_count + 1
                if token_actual > limits.max_tokens:
                    return ResourceFailure(
                        "tokens", limits.max_tokens, token_actual
                    )
                token_count = token_actual

            pieces.append(
                Piece(
                    source_ordinal,
                    scanned.start,
                    scanned.end,
                    scanned.kind,
                    source[scanned.start : scanned.end],
                )
            )
            cursor = scanned.end
        source_ranges.append((source_start, len(pieces)))

    return Complete(tuple(pieces), tuple(source_ranges), token_count)


def _require_bytes(source: bytes) -> bytes:
    if type(source) is not bytes:
        raise TypeError("each source must be exact immutable bytes")
    return source


def _scan_one(source: bytes, start: int) -> _RawPiece | _RawIssue:
    byte = source[start]

    if byte == 0x20:
        return _RawPiece(start, _take_while(source, start + 1, _is_space), "spaces")
    if byte == 0x0A:
        return _RawPiece(start, start + 1, "line_feed")
    if _is_ascii_lower(byte):
        return _lower_word(source, start)
    if _is_ascii_upper(byte):
        return _RawPiece(
            start,
            _take_while(source, start + 1, _is_ascii_alphanumeric),
            "upper_word_form",
        )
    if byte == ord("'"):
        return _prefixed_name(source, start, "missing_region_name", "region_form")
    if byte == ord("@"):
        return _prefixed_name(source, start, "missing_label_name", "label_form")
    if _is_ascii_digit(byte):
        return _RawPiece(start, _number_end(source, start), "number_form")
    if byte == ord("-"):
        if source[start : start + 2] == b"->":
            return _RawPiece(start, start + 2, "thin_arrow")
        if start + 1 < len(source) and _is_ascii_digit(source[start + 1]):
            return _RawPiece(start, _number_end(source, start), "number_form")
    if source[start : start + 2] == b"=>":
        return _RawPiece(start, start + 2, "fat_arrow")
    if byte == ord('"'):
        return _string(source, start)

    fixed = _FIXED_TOKENS.get(source[start : start + 1])
    if fixed is not None:
        return _RawPiece(start, start + 1, fixed)

    if byte >= 0x80:
        scalar_length = _utf8_scalar_length(source, start)
        if scalar_length is not None:
            return _RawIssue(start, start + scalar_length, "unexpected_byte")
        return _RawIssue(start, start + 1, "invalid_utf8")
    return _RawIssue(start, start + 1, "unexpected_byte")


def _lower_word(source: bytes, start: int) -> _RawPiece:
    base_end = _take_while(source, start + 1, _is_lower_continuation)
    operation_end = _operation_name_end(source, base_end)
    if operation_end is not None:
        return _RawPiece(start, operation_end, "operation_name_form")
    return _RawPiece(start, base_end, "lower_word_form")


def _operation_name_end(source: bytes, base_end: int) -> int | None:
    if base_end >= len(source) or source[base_end] != ord("."):
        return None
    suffix_start = base_end + 1
    for suffix in _OPERATION_SUFFIXES:
        end = suffix_start + len(suffix)
        if source[suffix_start:end] != suffix:
            continue
        if end < len(source) and _is_suffix_continuation(source[end]):
            continue
        return end
    return None


def _prefixed_name(
    source: bytes, start: int, issue_kind: str, piece_kind: str
) -> _RawPiece | _RawIssue:
    if start + 1 >= len(source) or not _is_ascii_lower(source[start + 1]):
        return _RawIssue(start, start + 1, issue_kind)
    end = _take_while(source, start + 2, _is_lower_continuation)
    return _RawPiece(start, end, piece_kind)


def _number_end(source: bytes, start: int) -> int:
    """Retain an opaque maximal numeric candidate without interpreting it."""

    cursor = start + 2 if source[start] == ord("-") else start + 1
    while cursor < len(source):
        byte = source[cursor]
        if _is_ascii_alphanumeric(byte) or byte in (ord("_"), ord(".")):
            cursor += 1
            continue
        if byte in (ord("+"), ord("-")) and source[cursor - 1] in (
            ord("e"),
            ord("E"),
        ):
            cursor += 1
            continue
        break
    return cursor


def _string(source: bytes, start: int) -> _RawPiece | _RawIssue:
    cursor = start + 1
    while cursor < len(source):
        byte = source[cursor]
        if byte == ord('"'):
            return _RawPiece(start, cursor + 1, "string_form")
        if byte == ord("\\"):
            if cursor + 1 >= len(source):
                return _RawIssue(cursor, cursor + 1, "invalid_string_escape")
            if source[cursor + 1] not in (ord("\\"), ord('"'), ord("n")):
                return _RawIssue(cursor, cursor + 2, "invalid_string_escape")
            cursor += 2
            continue
        if 0x20 <= byte <= 0x7E:
            cursor += 1
            continue
        if byte >= 0x80:
            scalar_length = _utf8_scalar_length(source, cursor)
            if scalar_length is not None:
                return _RawIssue(
                    cursor, cursor + scalar_length, "invalid_string_byte"
                )
            return _RawIssue(cursor, cursor + 1, "invalid_utf8")
        return _RawIssue(cursor, cursor + 1, "invalid_string_byte")
    return _RawIssue(start, len(source), "unterminated_string")


def _utf8_scalar_length(source: bytes, start: int) -> int | None:
    for length in range(2, min(4, len(source) - start) + 1):
        candidate = source[start : start + length]
        try:
            text = candidate.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            continue
        if len(text) == 1:
            return length
    return None


def _take_while(
    source: bytes, cursor: int, predicate: Callable[[int], bool]
) -> int:
    while cursor < len(source) and predicate(source[cursor]):
        cursor += 1
    return cursor


def _is_space(byte: int) -> bool:
    return byte == 0x20


def _is_ascii_digit(byte: int) -> bool:
    return ord("0") <= byte <= ord("9")


def _is_ascii_lower(byte: int) -> bool:
    return ord("a") <= byte <= ord("z")


def _is_ascii_upper(byte: int) -> bool:
    return ord("A") <= byte <= ord("Z")


def _is_ascii_alphanumeric(byte: int) -> bool:
    return _is_ascii_digit(byte) or _is_ascii_lower(byte) or _is_ascii_upper(byte)


def _is_lower_continuation(byte: int) -> bool:
    return _is_ascii_lower(byte) or _is_ascii_digit(byte) or byte == ord("_")


def _is_suffix_continuation(byte: int) -> bool:
    return _is_ascii_alphanumeric(byte) or byte == ord("_")
