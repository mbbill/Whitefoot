#!/usr/bin/env python3
"""Small fail-closed lexical views for Rust workspace policy checks."""

from __future__ import annotations

from dataclasses import dataclass


class RustSourcePolicyError(ValueError):
    """Rust source cannot be safely inspected by the policy lexer."""


@dataclass(frozen=True)
class LexicalViews:
    """Source with comments removed and with comments plus literals removed."""

    comments_removed: str
    code_only: str


def _masked(text: str) -> str:
    return "".join(character if character in "\r\n" else " " for character in text)


def _raw_string_end(text: str, start: int) -> int | None:
    for prefix in ("br", "cr", "r"):
        if not text.startswith(prefix, start):
            continue
        cursor = start + len(prefix)
        hashes = 0
        while cursor < len(text) and text[cursor] == "#":
            hashes += 1
            cursor += 1
        if cursor >= len(text) or text[cursor] != '"':
            continue
        delimiter = '"' + ("#" * hashes)
        end = text.find(delimiter, cursor + 1)
        if end < 0:
            raise RustSourcePolicyError("unterminated raw string literal")
        return end + len(delimiter)
    return None


def _quoted_string_end(text: str, start: int) -> int | None:
    quote = start
    if text.startswith(('b"', 'c"'), start):
        quote += 1
    elif not text.startswith('"', start):
        return None
    cursor = quote + 1
    while cursor < len(text):
        if text[cursor] == "\\":
            cursor += 2
        elif text[cursor] == '"':
            return cursor + 1
        else:
            cursor += 1
    raise RustSourcePolicyError("unterminated string literal")


def _character_end(text: str, start: int) -> int | None:
    quote = start
    if text.startswith("b'", start):
        quote += 1
    elif not text.startswith("'", start):
        return None
    cursor = quote + 1
    if cursor >= len(text) or text[cursor] in "\r\n'":
        return None
    if text[cursor] != "\\":
        end = cursor + 1
        return end + 1 if end < len(text) and text[end] == "'" else None

    cursor += 1
    if cursor >= len(text):
        raise RustSourcePolicyError("unterminated character escape")
    if text[cursor] == "x":
        cursor += 3
    elif text[cursor] == "u" and cursor + 1 < len(text) and text[cursor + 1] == "{":
        close = text.find("}", cursor + 2)
        if close < 0:
            raise RustSourcePolicyError("unterminated Unicode character escape")
        cursor = close + 1
    else:
        cursor += 1
    if cursor < len(text) and text[cursor] == "'":
        return cursor + 1
    raise RustSourcePolicyError("malformed character literal")


def lexical_views(text: str) -> LexicalViews:
    """Remove Rust comments and literals without confusing their delimiters."""
    comments_removed: list[str] = []
    code_only: list[str] = []
    cursor = 0
    while cursor < len(text):
        if text.startswith("//", cursor):
            end = text.find("\n", cursor + 2)
            end = len(text) if end < 0 else end
            masked = _masked(text[cursor:end])
            comments_removed.append(masked)
            code_only.append(masked)
            cursor = end
            continue
        if text.startswith("/*", cursor):
            depth = 1
            end = cursor + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            if depth:
                raise RustSourcePolicyError("unterminated block comment")
            masked = _masked(text[cursor:end])
            comments_removed.append(masked)
            code_only.append(masked)
            cursor = end
            continue

        end = _raw_string_end(text, cursor)
        if end is None:
            end = _quoted_string_end(text, cursor)
        if end is None:
            end = _character_end(text, cursor)
        if end is not None:
            literal = text[cursor:end]
            comments_removed.append(literal)
            code_only.append(_masked(literal))
            cursor = end
            continue

        comments_removed.append(text[cursor])
        code_only.append(text[cursor])
        cursor += 1
    return LexicalViews("".join(comments_removed), "".join(code_only))
