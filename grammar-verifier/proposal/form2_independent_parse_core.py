from __future__ import annotations

from form2_independent_lex import IndependentToken
from form2_independent_syntax import IndependentNode, IndependentParseError


MAX_PARSE_DEPTH = 512


class IndependentParserCore:
    def __init__(self, tokens: tuple[IndependentToken, ...], source_length: int):
        self.tokens = tokens
        self.source_length = source_length
        self.cursor = 0
        self.depth = 0

    def _offset(self) -> int:
        if self.cursor < len(self.tokens):
            return self.tokens[self.cursor].start
        return self.source_length

    def _error(self, reason: str) -> IndependentParseError:
        return IndependentParseError(self.cursor, self._offset(), reason)

    def _enter(self) -> None:
        if self.depth >= MAX_PARSE_DEPTH:
            raise self._error("parse nesting limit exceeded")
        self.depth += 1

    def _leave(self) -> None:
        self.depth -= 1

    def _peek(self, distance: int = 0) -> IndependentToken | None:
        index = self.cursor + distance
        return self.tokens[index] if index < len(self.tokens) else None

    def _raw(self, distance: int = 0) -> bytes | None:
        token = self._peek(distance)
        return None if token is None else token.raw

    def _kind(self, distance: int = 0) -> str | None:
        token = self._peek(distance)
        return None if token is None else token.kind

    def _take_raw(self, expected: bytes) -> IndependentToken:
        token = self._peek()
        if token is None or token.raw != expected:
            actual = "end of source" if token is None else repr(token.raw)
            raise self._error(f"expected {expected!r}, found {actual}")
        self.cursor += 1
        return token

    def _take_kind(self, expected: str) -> IndependentToken:
        token = self._peek()
        if token is None or token.kind != expected:
            actual = "end of source" if token is None else f"{token.kind} {token.raw!r}"
            raise self._error(f"expected {expected}, found {actual}")
        self.cursor += 1
        return token

    def _node(
        self, kind: str, start: int, children: list[IndependentNode]
    ) -> IndependentNode:
        if self.cursor <= start:
            raise self._error(f"production {kind} consumed no terminal")
        return IndependentNode(kind, start, self.cursor, tuple(children))
