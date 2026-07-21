from __future__ import annotations

from collections import Counter, defaultdict
from dataclasses import dataclass
from typing import Any

from form2_lex import (
    CURRENT_POLICY,
    PROPOSED_POLICY,
    LexicalObservation,
    SpacingPolicy,
    Token,
    boundary_observations,
    tokenize,
)


STATEMENT_TERMINATORS = {
    b"break": frozenset((b";",)),
    b"check": frozenset((b";",)),
    b"give": frozenset((b";",)),
    b"let": frozenset((b";", b"{")),
    b"loop": frozenset((b"{",)),
    b"match": frozenset((b"{",)),
    b"region": frozenset((b"{",)),
    b"return": frozenset((b";",)),
    b"set": frozenset((b";",)),
}
TOP_LEVEL_DECLARATIONS = frozenset(
    (b"fn", b"struct", b"enum", b"contract", b"conform", b"const")
)


POLICY_GAPS = (
    {
        "id": "adjacent-token-line-boundary-domain",
        "impact": "The rule does not define whether lexical tokens separated by LF remain adjacent for the one-space rule; treating them as adjacent would forbid every multiline declaration, while treating adjacency as same-line only is unstated.",
    },
    {
        "id": "closing-brace-indentation-depth",
        "impact": "The rule does not say whether a closing brace is indented at the nesting level before or after that brace closes its block.",
    },
    {
        "id": "declaration-separator-scope",
        "impact": "The word declaration is not scoped to top-level GRAM-2 items or extended to fields, variants, contract members, and conform bindings.",
    },
    {
        "id": "non-statement-line-placement",
        "impact": "Only statements are forbidden from wrapping; declaration headers, match-arm headers, members, and block delimiters therefore lack one required line placement.",
    },
    {
        "id": "overlapping-punctuation-obligations",
        "impact": "A boundary after comma or colon and before a no-space-before punctuation receives both one-space and zero-space requirements; no precedence or exclusion removes the contradiction.",
        "conflicting_boundary_classes": [
            {"left": left, "right": right}
            for left in (",", ":")
            for right in (")", ">", ",", ";", ".", ":", "(", "<")
        ],
    },
)


@dataclass(frozen=True)
class SourceAudit:
    report: dict[str, Any]
    lexical: LexicalObservation


def _physical_lines(raw: bytes) -> tuple[tuple[int, bytes], ...]:
    lines: list[tuple[int, bytes]] = []
    start = 0
    for index, byte in enumerate(raw):
        if byte == 0x0A:
            lines.append((start, raw[start:index]))
            start = index + 1
    if start < len(raw) or not raw:
        lines.append((start, raw[start:]))
    return tuple(lines)


def _tokens_by_line(tokens: tuple[Token, ...]) -> dict[int, list[Token]]:
    grouped: dict[int, list[Token]] = defaultdict(list)
    for token in tokens:
        grouped[token.line].append(token)
    return grouped


def _utf8_violations(raw: bytes) -> tuple[dict[str, Any], ...]:
    remaining = raw
    base = 0
    violations: list[dict[str, Any]] = []
    while remaining:
        try:
            remaining.decode("utf-8")
            break
        except UnicodeDecodeError as error:
            start = base + error.start
            end = base + max(error.end, error.start + 1)
            violations.append(
                {
                    "byte_end": end,
                    "byte_start": start,
                    "clause": "utf8",
                    "raw_hex": raw[start:end].hex(),
                    "reason": error.reason,
                }
            )
            base = end
            remaining = raw[end:]
    return tuple(violations)


def _line_ending_violations(raw: bytes) -> tuple[dict[str, Any], ...]:
    return tuple(
        {
            "byte_end": index + 1,
            "byte_start": index,
            "clause": "lf-line-endings",
            "raw_hex": "0d",
        }
        for index, byte in enumerate(raw)
        if byte == 0x0D
    )


def _eof_violations(raw: bytes) -> tuple[dict[str, Any], ...]:
    if raw.endswith(b"\n") and not raw.endswith(b"\n\n"):
        return ()
    trailing_lf = len(raw) - len(raw.rstrip(b"\n"))
    return (
        {
            "byte_end": len(raw),
            "byte_start": len(raw) - trailing_lf,
            "clause": "exactly-one-final-lf",
            "trailing_lf_count": trailing_lf,
        },
    )


def _trailing_whitespace_violations(
    raw: bytes,
    lines: tuple[tuple[int, bytes], ...],
) -> tuple[dict[str, Any], ...]:
    violations: list[dict[str, Any]] = []
    for line_number, (offset, line) in enumerate(lines, 1):
        trimmed = line.rstrip(b" \t\r")
        if len(trimmed) == len(line):
            continue
        start = offset + len(trimmed)
        end = offset + len(line)
        violations.append(
            {
                "actual_hex": raw[start:end].hex(),
                "byte_end": end,
                "byte_start": start,
                "clause": "no-trailing-whitespace",
                "column": len(trimmed) + 1,
                "line": line_number,
            }
        )
    return tuple(violations)


def _indentation_observations(
    raw: bytes,
    lines: tuple[tuple[int, bytes], ...],
    by_line: dict[int, list[Token]],
) -> tuple[tuple[dict[str, Any], ...], dict[str, int]]:
    violations: list[dict[str, Any]] = []
    depth = 0
    checked = 0
    closing_brace_lines = 0
    match_arm_lines = 0
    for line_number, (offset, line) in enumerate(lines, 1):
        tokens = by_line.get(line_number, [])
        if not tokens:
            continue
        first = tokens[0]
        leading = raw[offset:first.start]
        begins_with_close = first.raw == b"}"
        if begins_with_close:
            closing_brace_lines += 1
        expected_depth = max(depth - 1, 0) if begins_with_close else depth
        expected = b"  " * expected_depth
        checked += 1
        if leading != expected:
            violations.append(
                {
                    "actual_hex": leading.hex(),
                    "byte_end": first.start,
                    "byte_start": offset,
                    "clause": "indentation-two-spaces-per-brace-level",
                    "column": 1,
                    "convention": "closing brace is observed at the post-close depth; FORM-2 does not state this convention",
                    "expected_hex": expected.hex(),
                    "line": line_number,
                }
            )
        if any(
            left.raw == b"=" and right.raw == b">"
            for left, right in zip(tokens, tokens[1:])
        ):
            match_arm_lines += 1
        for token in tokens:
            if token.raw == b"{":
                depth += 1
            elif token.raw == b"}":
                depth = max(depth - 1, 0)
    return tuple(violations), {
        "checked_nonempty_line_count": checked,
        "closing_brace_line_count": closing_brace_lines,
        "final_observed_brace_depth": depth,
        "match_arm_line_count": match_arm_lines,
        "violation_count": len(violations),
    }


def _statement_observations(
    by_line: dict[int, list[Token]],
) -> tuple[tuple[dict[str, Any], ...], dict[str, int]]:
    violations: list[dict[str, Any]] = []
    kinds: Counter[str] = Counter()
    for line_number in sorted(by_line):
        tokens = by_line[line_number]
        if not tokens:
            continue
        first = tokens[0]
        required = STATEMENT_TERMINATORS.get(first.raw)
        if required is None:
            continue
        kinds[first.raw.decode("ascii")] += 1
        if not any(token.raw in required for token in tokens):
            violations.append(
                {
                    "byte_end": tokens[-1].end,
                    "byte_start": first.start,
                    "clause": "statement-is-one-line",
                    "column": first.column,
                    "line": line_number,
                    "statement_kind": first.raw.decode("ascii"),
                }
            )
    return tuple(violations), {
        "checked_statement_header_count": sum(kinds.values()),
        "statement_kind_counts": dict(sorted(kinds.items())),
        "violation_count": len(violations),
    }


def _top_level_declaration_starts(
    by_line: dict[int, list[Token]],
) -> tuple[tuple[int, Token], ...]:
    starts: list[tuple[int, Token]] = []
    depth = 0
    for line_number in sorted(by_line):
        tokens = by_line[line_number]
        if not tokens:
            continue
        first = tokens[0]
        observed_depth = max(depth - 1, 0) if first.raw == b"}" else depth
        if observed_depth == 0 and first.raw in TOP_LEVEL_DECLARATIONS:
            starts.append((line_number, first))
        for token in tokens:
            if token.raw == b"{":
                depth += 1
            elif token.raw == b"}":
                depth = max(depth - 1, 0)
    return tuple(starts)


def _declaration_separator_observations(
    lines: tuple[tuple[int, bytes], ...],
    by_line: dict[int, list[Token]],
) -> tuple[tuple[dict[str, Any], ...], dict[str, int]]:
    starts = _top_level_declaration_starts(by_line)
    violations: list[dict[str, Any]] = []
    for line_number, token in starts[1:]:
        previous = line_number - 1
        blank_count = 0
        while previous >= 1 and lines[previous - 1][1] == b"":
            blank_count += 1
            previous -= 1
        if blank_count != 1:
            violations.append(
                {
                    "blank_line_count": blank_count,
                    "byte_end": token.start,
                    "byte_start": lines[max(previous, 1) - 1][0],
                    "clause": "one-blank-line-between-top-level-declarations",
                    "column": token.column,
                    "interpretation": "declaration means a top-level GRAM-2 item; FORM-2 does not state this scope",
                    "line": line_number,
                }
            )
    return tuple(violations), {
        "checked_separator_count": max(len(starts) - 1, 0),
        "top_level_declaration_count": len(starts),
        "violation_count": len(violations),
    }


def audit_source(raw: bytes, policy: SpacingPolicy = CURRENT_POLICY) -> SourceAudit:
    lexical = tokenize(raw)
    lines = _physical_lines(raw)
    by_line = _tokens_by_line(lexical.tokens)
    contexts, spacing, spacing_summary = boundary_observations(
        raw, lexical.tokens, policy
    )
    utf8 = _utf8_violations(raw)
    line_endings = _line_ending_violations(raw)
    eof = _eof_violations(raw)
    trailing = _trailing_whitespace_violations(raw, lines)
    indentation, indentation_summary = _indentation_observations(
        raw, lines, by_line
    )
    statements, statement_summary = _statement_observations(by_line)
    declarations, declaration_summary = _declaration_separator_observations(
        lines, by_line
    )
    violations = sorted(
        (
            *utf8,
            *line_endings,
            *eof,
            *trailing,
            *indentation,
            *statements,
            *declarations,
            *spacing,
        ),
        key=lambda item: (
            item.get("byte_start", -1),
            item["clause"],
            item.get("byte_end", -1),
        ),
    )
    clause_counts = Counter(item["clause"] for item in violations)
    report = {
        "boundary_contexts": contexts,
        "clauses": {
            "declaration_separation": declaration_summary,
            "exactly_one_final_lf": {
                "violation_count": len(eof),
            },
            "indentation": indentation_summary,
            "lf_line_endings": {
                "violation_count": len(line_endings),
            },
            "no_trailing_whitespace": {
                "checked_physical_line_count": len(lines),
                "violation_count": len(trailing),
            },
            "statement_line_wrapping": statement_summary,
            "token_spacing": spacing_summary,
            "utf8": {
                "violation_count": len(utf8),
            },
        },
        "definite_and_interpretation_bound_violation_count": len(violations),
        "lexical_observation_issues": lexical.issues,
        "policy": policy.name,
        "token_count": len(lexical.tokens),
        "violation_counts_by_clause": dict(sorted(clause_counts.items())),
        "violations": violations,
    }
    return SourceAudit(report, lexical)


def audit_current(raw: bytes) -> SourceAudit:
    return audit_source(raw, CURRENT_POLICY)


def audit_proposed(raw: bytes) -> SourceAudit:
    return audit_source(raw, PROPOSED_POLICY)
