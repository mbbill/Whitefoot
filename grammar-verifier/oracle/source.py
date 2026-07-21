"""Independent byte-level source census and ownership discovery."""

from __future__ import annotations

from dataclasses import dataclass
import re

from core import LogicalBudget, fail


_RULE = re.compile(rb"^\[([A-Z][A-Z0-9]*-[0-9]+)\](?: |$)")
_DEFINITION_HEAD = re.compile(rb"^[a-z][a-z0-9_]*[ ]*:=[ ]*")
_ALTERNATE_HEAD = re.compile(
    rb"^[a-z][a-z0-9_]*[ \t]*(=)(?!=|>)[ \t]*[^ \t\n][^\n]*$"
)
_CLASS_CUE = re.compile(rb"(?:Lexical|Token) class(?:es)?(?=$|[^A-Za-z0-9_])")
_LITERAL_CUE = re.compile(rb"Literals, exhaustively:")
_TABLE_CUE = re.compile(rb"[A-Z][A-Z0-9_]* is a closed table:")


@dataclass(frozen=True)
class Line:
    number: int
    start: int
    content_end: int
    end: int
    content: bytes


@dataclass(frozen=True)
class Rule:
    owner: str
    start: int
    end: int


@dataclass(frozen=True)
class Surface:
    kind: str
    start: int
    end: int
    owner: str


@dataclass(frozen=True)
class GrammarRegion:
    owner: str
    kind: str
    start: int
    end: int
    lines: tuple[Line, ...]


@dataclass(frozen=True)
class LexicalCue:
    owner: str
    dialect: str
    start: int
    end: int
    line_index: int


@dataclass(frozen=True)
class SourceScan:
    lines: tuple[Line, ...]
    rules: tuple[Rule, ...]
    regions: tuple[GrammarRegion, ...]
    lexical_cues: tuple[LexicalCue, ...]
    surfaces: tuple[Surface, ...]
    assignment_offsets: tuple[int, ...]


def _split_lines(document: bytes, budget: LogicalBudget) -> tuple[Line, ...]:
    if not document:
        fail("input", "empty_document")
    try:
        document.decode("utf-8", "strict")
    except UnicodeDecodeError:
        fail("input", "document_utf8")
    if b"\r" in document:
        fail("input", "document_line_endings")
    result: list[Line] = []
    start = 0
    number = 1
    while start < len(document):
        newline = document.find(b"\n", start)
        terminated = newline >= 0
        content_end = newline if terminated else len(document)
        budget.add("max_lines")
        budget.maximum("max_line_bytes", content_end - start)
        content = document[start:content_end]
        if content.endswith((b" ", b"\t")):
            fail("extraction", "trailing_whitespace")
        end = content_end + int(terminated)
        result.append(Line(number, start, content_end, end, content))
        start = end
        number += 1
    if start != len(document):
        fail("internal", "line_coverage")
    return tuple(result)


def _discover_rules(
    lines: tuple[Line, ...],
    document_length: int,
    budget: LogicalBudget,
) -> tuple[tuple[Rule, ...], dict[int, str | None]]:
    starts: list[tuple[str, Line]] = []
    seen: set[str] = set()
    owner_by_line: dict[int, str | None] = {}
    owner: str | None = None
    masked = False
    for index, line in enumerate(lines):
        if masked:
            owner_by_line[index] = owner
            if line.content == b"```":
                masked = False
            continue
        if line.content.lstrip(b" \t").startswith(b"```"):
            owner_by_line[index] = owner
            masked = True
            continue
        match = _RULE.match(line.content)
        if match is not None:
            owner = match.group(1).decode("ascii")
            if owner in seen:
                fail("extraction", "duplicate_rule")
            budget.add("max_rules")
            seen.add(owner)
            starts.append((owner, line))
        owner_by_line[index] = owner
    rules = tuple(
        Rule(
            item_owner,
            line.start,
            starts[index + 1][1].start if index + 1 < len(starts) else document_length,
        )
        for index, (item_owner, line) in enumerate(starts)
    )
    if not rules:
        fail("extraction", "rules_missing")
    return rules, owner_by_line


def _fences_and_regions(
    lines: tuple[Line, ...],
    owner_by_line: dict[int, str | None],
) -> tuple[list[GrammarRegion], list[Surface], set[int]]:
    regions: list[GrammarRegion] = []
    surfaces: list[Surface] = []
    fenced: set[int] = set()
    index = 0
    while index < len(lines):
        line = lines[index]
        if not line.content.lstrip(b" \t").startswith(b"```"):
            index += 1
            continue
        if line.content != b"```":
            fail("extraction", "malformed_fence")
        close = index + 1
        while close < len(lines) and lines[close].content != b"```":
            if lines[close].content.lstrip(b" \t").startswith(b"```"):
                fail("extraction", "malformed_fence")
            close += 1
        if close >= len(lines):
            fail("extraction", "unterminated_fence")
        owner = owner_by_line[index]
        body_lines = lines[index + 1 : close]
        grammar_shaped = any(
            b":=" in body.content
            or _DEFINITION_HEAD.match(body.content) is not None
            or _ALTERNATE_HEAD.match(body.content) is not None
            for body in body_lines
        )
        grammar_owned = owner is not None and owner.startswith("GRAM-")
        if grammar_owned or grammar_shaped:
            if owner is None or not grammar_owned:
                fail("extraction", "unowned_grammar_fence")
            surfaces.append(Surface("grammar-fence", line.start, lines[close].end, owner))
            body_start = body_lines[0].start if body_lines else lines[close].start
            regions.append(
                GrammarRegion(owner, "fence", body_start, lines[close].start, tuple(body_lines))
            )
        fenced.update(range(index, close + 1))
        index = close + 1
    return regions, surfaces, fenced


def _inline_regions(
    lines: tuple[Line, ...],
    fenced: set[int],
    owner_by_line: dict[int, str | None],
) -> tuple[list[GrammarRegion], list[Surface], dict[int, tuple[tuple[int, int], ...]]]:
    regions: list[GrammarRegion] = []
    surfaces: list[Surface] = []
    ranges_by_line: dict[int, tuple[tuple[int, int], ...]] = {}
    for index, line in enumerate(lines):
        if index in fenced:
            continue
        ranges: list[tuple[int, int]] = []
        cursor = 0
        while cursor < len(line.content):
            opening = line.content.find(b"`", cursor)
            if opening < 0:
                break
            closing = line.content.find(b"`", opening + 1)
            if closing < 0:
                fail("extraction", "unterminated_inline_code")
            ranges.append((opening, closing + 1))
            body = line.content[opening + 1 : closing]
            alternate = _ALTERNATE_HEAD.fullmatch(body)
            if alternate is not None:
                fail("extraction", "single_equals_grammar")
            if b":=" in body:
                owner = owner_by_line[index]
                if owner is None or _DEFINITION_HEAD.match(body) is None:
                    fail("extraction", "unowned_inline_grammar")
                start = line.start + opening + 1
                end = line.start + closing
                synthetic = Line(line.number, start, end, end, body)
                regions.append(GrammarRegion(owner, "inline", start, end, (synthetic,)))
                surfaces.append(Surface("grammar-inline", start, end, owner))
            cursor = closing + 1
        ranges_by_line[index] = tuple(ranges)
    return regions, surfaces, ranges_by_line


def _reject_raw_single_equals(
    lines: tuple[Line, ...],
    fenced: set[int],
    inline_ranges: dict[int, tuple[tuple[int, int], ...]],
) -> None:
    for index, line in enumerate(lines):
        if index in fenced:
            continue
        candidate = _ALTERNATE_HEAD.fullmatch(line.content)
        if candidate is None:
            continue
        equals = candidate.start(1)
        if any(start <= equals < end for start, end in inline_ranges.get(index, ())):
            continue
        fail("extraction", "single_equals_grammar")


def _at_cue_boundary(line: Line, start: int) -> bool:
    marker = _RULE.match(line.content)
    at_payload = marker is not None and start == marker.end()
    at_sentence = start >= 2 and line.content[start - 2 : start] == b". "
    return at_payload or at_sentence


def _lexical_cues(
    lines: tuple[Line, ...],
    fenced: set[int],
    inline_ranges: dict[int, tuple[tuple[int, int], ...]],
    owner_by_line: dict[int, str | None],
) -> list[LexicalCue]:
    result: list[LexicalCue] = []
    patterns = (
        ("class", _CLASS_CUE),
        ("literal", _LITERAL_CUE),
        ("table", _TABLE_CUE),
    )
    for index, line in enumerate(lines):
        owner = owner_by_line[index]
        if index in fenced or owner is None:
            continue
        for dialect, pattern in patterns:
            for match in pattern.finditer(line.content):
                if any(start <= match.start() < end for start, end in inline_ranges.get(index, ())):
                    continue
                if not _at_cue_boundary(line, match.start()):
                    continue
                result.append(
                    LexicalCue(
                        owner,
                        dialect,
                        line.start + match.start(),
                        line.start + match.end(),
                        index,
                    )
                )
    result.sort(key=lambda cue: (cue.start, cue.end, cue.dialect))
    return result


def scan_source(document: bytes, budget: LogicalBudget) -> SourceScan:
    budget.limits.require("max_document_bytes", len(document))
    lines = _split_lines(document, budget)
    rules, owner_by_line = _discover_rules(lines, len(document), budget)
    regions, surfaces, fenced = _fences_and_regions(lines, owner_by_line)
    inline, inline_surfaces, inline_ranges = _inline_regions(lines, fenced, owner_by_line)
    _reject_raw_single_equals(lines, fenced, inline_ranges)
    regions.extend(inline)
    surfaces.extend(inline_surfaces)
    regions.sort(key=lambda region: region.start)
    if not regions:
        fail("extraction", "grammar_missing")
    assignments: list[int] = []
    cursor = 0
    while True:
        found = document.find(b":=", cursor)
        if found < 0:
            break
        budget.limits.require("max_definitions", len(assignments) + 1)
        assignments.append(found)
        cursor = found + 2
    for offset in assignments:
        line_index = next(
            (index for index, line in enumerate(lines) if line.start <= offset < line.content_end),
            None,
        )
        if line_index is None or owner_by_line[line_index] is None:
            fail("extraction", "unowned_assignment")
        surfaces.append(
            Surface("assignment", offset, offset + 2, owner_by_line[line_index] or "")
        )
    cues = _lexical_cues(lines, fenced, inline_ranges, owner_by_line)
    return SourceScan(
        lines,
        rules,
        tuple(regions),
        tuple(cues),
        tuple(surfaces),
        tuple(assignments),
    )
