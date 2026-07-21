from __future__ import annotations

from dataclasses import dataclass
from bisect import bisect_right
from collections import Counter
from typing import Any


HORIZONTAL = b" \t\r"
PUNCTUATION = frozenset(b"()<>[]{}.,;:=&")


@dataclass(frozen=True)
class Token:
    start: int
    end: int
    line: int
    column: int
    kind: str
    raw: bytes

    @property
    def role(self) -> str:
        if len(self.raw) == 1 and self.raw[0] in PUNCTUATION:
            return self.raw.decode("ascii")
        return self.kind


@dataclass(frozen=True)
class LexicalObservation:
    tokens: tuple[Token, ...]
    issues: tuple[dict[str, Any], ...]


@dataclass(frozen=True)
class SpacingPolicy:
    name: str
    compact_square_interiors: bool


CURRENT_POLICY = SpacingPolicy("exact-v0.8", False)
PROPOSED_POLICY = SpacingPolicy("proposed-uniform-attached-punctuation", True)


def _line_starts(raw: bytes) -> tuple[int, ...]:
    starts = [0]
    starts.extend(index + 1 for index, byte in enumerate(raw) if byte == 0x0A)
    return tuple(starts)


def position(starts: tuple[int, ...], offset: int) -> tuple[int, int]:
    line_index = bisect_right(starts, offset) - 1
    return line_index + 1, offset - starts[line_index] + 1


def _is_word_start(byte: int) -> bool:
    return 0x41 <= byte <= 0x5A or 0x61 <= byte <= 0x7A or byte == 0x5F


def _is_word_continue(byte: int) -> bool:
    return _is_word_start(byte) or 0x30 <= byte <= 0x39


def _scan_string(raw: bytes, start: int) -> tuple[int, str | None]:
    cursor = start + 1
    while cursor < len(raw):
        byte = raw[cursor]
        if byte == 0x22:
            return cursor + 1, None
        if byte == 0x5C:
            cursor += 2
            continue
        if byte == 0x0A:
            return cursor, "raw-line-feed-in-string-candidate"
        cursor += 1
    return cursor, "unterminated-string-candidate"


def _scan_word(raw: bytes, start: int) -> int:
    cursor = start + 1
    while cursor < len(raw) and _is_word_continue(raw[cursor]):
        cursor += 1
    if (
        0x61 <= raw[start] <= 0x7A
        and cursor + 1 < len(raw)
        and raw[cursor] == 0x2E
        and _is_word_start(raw[cursor + 1])
    ):
        suffix_start = cursor + 1
        suffix_end = suffix_start + 1
        while suffix_end < len(raw) and _is_word_continue(raw[suffix_end]):
            suffix_end += 1
        if raw[suffix_start:suffix_end] in {
            b"checked",
            b"sat",
            b"strict",
            b"trap",
            b"wrap",
        }:
            cursor = suffix_end
    return cursor


def _scan_prefixed_name(raw: bytes, start: int) -> int:
    cursor = start + 1
    while cursor < len(raw) and _is_word_continue(raw[cursor]):
        cursor += 1
    return cursor


def _scan_number(raw: bytes, start: int) -> int:
    cursor = start
    if raw[cursor] == 0x2D:
        cursor += 1
    while cursor < len(raw):
        byte = raw[cursor]
        if _is_word_continue(byte) or byte in b".+-":
            cursor += 1
            continue
        break
    return cursor


def tokenize(raw: bytes) -> LexicalObservation:
    starts = _line_starts(raw)
    tokens: list[Token] = []
    issues: list[dict[str, Any]] = []
    cursor = 0
    while cursor < len(raw):
        byte = raw[cursor]
        if byte in b" \t\r\n":
            cursor += 1
            continue
        start = cursor
        issue: str | None = None
        if byte == 0x22:
            cursor, issue = _scan_string(raw, cursor)
            kind = "string"
        elif byte in (0x27, 0x40):
            cursor = _scan_prefixed_name(raw, cursor)
            kind = "region" if byte == 0x27 else "label"
        elif _is_word_start(byte):
            cursor = _scan_word(raw, cursor)
            first = raw[start]
            kind = "type-word" if 0x41 <= first <= 0x5A else "word"
        elif 0x30 <= byte <= 0x39 or (
            byte == 0x2D and cursor + 1 < len(raw) and 0x30 <= raw[cursor + 1] <= 0x39
        ):
            cursor = _scan_number(raw, cursor)
            kind = "number"
        elif byte == 0x2D and cursor + 1 < len(raw) and raw[cursor + 1] == 0x3E:
            cursor += 2
            kind = "arrow"
        elif byte == 0x3D and cursor + 1 < len(raw) and raw[cursor + 1] == 0x3E:
            cursor += 2
            kind = "fat-arrow"
        else:
            cursor += 1
            kind = "punctuation" if byte in PUNCTUATION else "unknown-byte"
            if kind == "unknown-byte":
                issue = "unclassified-source-byte"
        line, column = position(starts, start)
        token = Token(start, cursor, line, column, kind, raw[start:cursor])
        tokens.append(token)
        if issue is not None:
            issues.append(
                {
                    "byte_end": cursor,
                    "byte_start": start,
                    "column": column,
                    "issue": issue,
                    "line": line,
                    "raw_hex": raw[start:cursor].hex(),
                }
            )
    return LexicalObservation(tuple(tokens), tuple(issues))


def _requirements(
    left: Token,
    right: Token,
    policy: SpacingPolicy,
) -> tuple[tuple[str, bytes], ...]:
    requirements: list[tuple[str, bytes]] = []
    no_after = {b"(", b"<", b"&"}
    no_before = {b")", b">", b",", b";", b".", b":", b"(", b"<"}
    if policy.compact_square_interiors:
        no_after.add(b"[")
        no_before.add(b"]")
    if left.raw in no_after:
        requirements.append((f"no-space-after-{left.raw.decode('ascii')}", b""))
    if right.raw in no_before:
        requirements.append((f"no-space-before-{right.raw.decode('ascii')}", b""))
    if left.raw == b",":
        requirements.append(("one-space-after-comma", b" "))
    if left.raw == b":":
        requirements.append(("one-space-after-colon", b" "))
    if left.raw == b".":
        requirements.append(("no-space-after-place-dot", b""))
    if not requirements:
        requirements.append(("default-one-space-between-adjacent-tokens", b" "))
    return tuple(requirements)


def boundary_observations(
    raw: bytes,
    tokens: tuple[Token, ...],
    policy: SpacingPolicy,
) -> tuple[tuple[dict[str, Any], ...], tuple[dict[str, Any], ...], dict[str, int]]:
    contexts: Counter[tuple[str, str, str, str]] = Counter()
    violations: list[dict[str, Any]] = []
    checked = 0
    conflicts = 0
    indeterminate = 0
    line_boundary_indeterminate = 0
    for left, right in zip(tokens, tokens[1:]):
        gap = raw[left.end : right.start]
        if b"\n" in gap:
            line_boundary_indeterminate += 1
            context = (
                left.role,
                right.role,
                "indeterminate-line-boundary-adjacency",
                gap.hex(),
            )
            contexts[context] += 1
            continue
        if left.kind == "unknown-byte" or right.kind == "unknown-byte":
            indeterminate += 1
            context = (
                left.role,
                right.role,
                "indeterminate-non-token-byte",
                gap.hex(),
            )
            contexts[context] += 1
            continue
        checked += 1
        requirements = _requirements(left, right, policy)
        expected_values = {expected for _, expected in requirements}
        requirement_names = [name for name, _ in requirements]
        context = (left.role, right.role, "+".join(requirement_names), gap.hex())
        contexts[context] += 1
        if len(expected_values) != 1:
            conflicts += 1
            violations.append(
                {
                    "actual_hex": gap.hex(),
                    "byte_end": right.start,
                    "byte_start": left.end,
                    "clause": "token-spacing-policy-conflict",
                    "column": left.column + len(left.raw),
                    "left_hex": left.raw.hex(),
                    "line": left.line,
                    "requirements": requirement_names,
                    "right_hex": right.raw.hex(),
                }
            )
            continue
        expected = next(iter(expected_values))
        if gap != expected:
            violations.append(
                {
                    "actual_hex": gap.hex(),
                    "byte_end": right.start,
                    "byte_start": left.end,
                    "clause": "token-spacing",
                    "column": left.column + len(left.raw),
                    "expected_hex": expected.hex(),
                    "left_hex": left.raw.hex(),
                    "line": left.line,
                    "requirements": requirement_names,
                    "right_hex": right.raw.hex(),
                }
            )
    context_rows = tuple(
        {
            "actual_hex": actual,
            "count": count,
            "left_role": left,
            "requirements": requirements.split("+"),
            "right_role": right,
        }
        for (left, right, requirements, actual), count in sorted(
            contexts.items(), key=lambda item: item[0]
        )
    )
    return context_rows, tuple(violations), {
        "checked_boundary_count": checked,
        "indeterminate_non_token_boundary_count": indeterminate,
        "indeterminate_line_boundary_count": line_boundary_indeterminate,
        "policy_conflict_count": conflicts,
        "violation_count": len(violations),
    }


def token_lexeme_stream(tokens: tuple[Token, ...]) -> bytes:
    output = bytearray()
    for token in tokens:
        output.extend(len(token.raw).to_bytes(8, "big"))
        output.extend(token.raw)
    return bytes(output)
