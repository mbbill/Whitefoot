"""Source-preserving EBNF extraction for the independent Oracle."""

from __future__ import annotations

from dataclasses import dataclass
import re

from core import LogicalBudget, fail
from source import GrammarRegion, Line


_HEAD = re.compile(rb"^([a-z][a-z0-9_]*)([ ]*):=([ ]*)(.*)$")
_ALTERNATE = re.compile(rb"^[a-z][a-z0-9_]*[ \t]*(=)(?!=|>)[ \t]+[^ \t\n][^\n]*$")
_NAME = re.compile(rb"[A-Za-z_][A-Za-z0-9_]*")
_UNREVIEWED_PATTERN = re.compile(rb"\[[^\]\n]+\][*+?]?")


@dataclass(frozen=True)
class Segment:
    data: bytes
    start: int


@dataclass(frozen=True)
class Token:
    kind: str
    start: int
    end: int
    value: str | None = None


@dataclass(frozen=True)
class Node:
    kind: str
    start: int
    end: int
    value: str | None = None
    children: tuple["Node", ...] = ()


@dataclass(frozen=True)
class Production:
    owner: str
    lhs: str
    definition_start: int
    definition_end: int
    rhs_start: int
    rhs_end: int
    assignment_start: int
    root: Node


def _node(
    budget: LogicalBudget,
    kind: str,
    start: int,
    end: int,
    *,
    value: str | None = None,
    children: tuple[Node, ...] = (),
) -> Node:
    budget.add("max_grammar_nodes")
    depth = 1 + max((_depth(child) for child in children), default=0)
    budget.maximum("max_ebnf_depth", depth)
    return Node(kind, start, end, value, children)


def _depth(node: Node) -> int:
    return 1 + max((_depth(child) for child in node.children), default=0)


def _semantic_segment(line: Line, start: int = 0) -> Segment:
    data = line.content[start:]
    if b"\t" in data:
        fail("extraction", "grammar_tab")
    marker = -1
    quoted = False
    for index, byte in enumerate(data):
        if byte == 0x22:
            quoted = not quoted
        elif byte == 0x23 and not quoted:
            marker = index
            break
    if marker >= 0:
        if marker == 0 or data[marker - 1] != 0x20 or not data.startswith(b"# ", marker):
            fail("extraction", "grammar_comment")
        if b"#" in data[marker + 1 :]:
            fail("extraction", "grammar_comment")
        data = data[:marker].rstrip(b" ")
    return Segment(data, line.start + start)


def _tokenize(segments: tuple[Segment, ...], budget: LogicalBudget) -> tuple[Token, ...]:
    result: list[Token] = []
    punctuation = {
        ord("("): "left",
        ord(")"): "right",
        ord("|"): "choice",
        ord("?"): "optional",
        ord("*"): "star",
        ord("+"): "plus",
    }
    for segment in segments:
        cursor = 0
        while cursor < len(segment.data):
            byte = segment.data[cursor]
            if byte == 0x20:
                cursor += 1
                continue
            start = segment.start + cursor
            if byte == ord('"'):
                close = segment.data.find(b'"', cursor + 1)
                if close < 0:
                    fail("extraction", "quoted_atom_unterminated")
                raw = segment.data[cursor + 1 : close]
                if not raw:
                    fail("extraction", "quoted_atom_empty")
                if any(value < 0x20 or value > 0x7E or value in (0x22, 0x5C) for value in raw):
                    fail("extraction", "quoted_atom_byte")
                if raw != b"[0-9]+" and _UNREVIEWED_PATTERN.fullmatch(raw) is not None:
                    fail("extraction", "quoted_pattern")
                budget.add("max_terminal_occurrences")
                result.append(Token("quoted", start, segment.start + close + 1, raw.decode("ascii")))
                cursor = close + 1
                continue
            match = _NAME.match(segment.data, cursor)
            if match is not None:
                raw = match.group()
                budget.maximum("max_symbol_bytes", len(raw))
                result.append(
                    Token("ref", start, segment.start + match.end(), raw.decode("ascii"))
                )
                cursor = match.end()
                continue
            kind = punctuation.get(byte)
            if kind is None:
                fail("extraction", "ebnf_character")
            result.append(Token(kind, start, start + 1))
            cursor += 1
    if not result:
        fail("extraction", "empty_rhs")
    return tuple(result)


class _Parser:
    def __init__(self, tokens: tuple[Token, ...], budget: LogicalBudget) -> None:
        self.tokens = tokens
        self.budget = budget
        self.cursor = 0

    def parse(self) -> Node:
        root = self._choice(1)
        if self.cursor != len(self.tokens):
            fail("extraction", "ebnf_trailing")
        return root

    def _check_depth(self, depth: int) -> None:
        self.budget.maximum("max_ebnf_depth", depth)

    def _choice(self, depth: int) -> Node:
        self._check_depth(depth)
        branches = [self._sequence(depth + 1)]
        while self._peek("choice"):
            self.cursor += 1
            branches.append(self._sequence(depth + 1))
        if len(branches) == 1:
            return branches[0]
        return _node(
            self.budget,
            "choice",
            branches[0].start,
            branches[-1].end,
            children=tuple(branches),
        )

    def _sequence(self, depth: int) -> Node:
        self._check_depth(depth)
        members: list[Node] = []
        while self.cursor < len(self.tokens) and not self._peek("choice") and not self._peek("right"):
            members.append(self._postfix(depth + 1))
        if not members:
            fail("extraction", "ebnf_empty_branch")
        if len(members) == 1:
            return members[0]
        return _node(
            self.budget,
            "sequence",
            members[0].start,
            members[-1].end,
            children=tuple(members),
        )

    def _postfix(self, depth: int) -> Node:
        self._check_depth(depth)
        child = self._primary(depth + 1)
        if self.cursor >= len(self.tokens):
            return child
        token = self.tokens[self.cursor]
        kinds = {"optional": "optional", "star": "repeat0", "plus": "repeat1"}
        kind = kinds.get(token.kind)
        if kind is None:
            return child
        self.cursor += 1
        result = _node(
            self.budget,
            kind,
            child.start,
            token.end,
            children=(child,),
        )
        if self.cursor < len(self.tokens) and self.tokens[self.cursor].kind in kinds:
            fail("extraction", "stacked_postfix")
        return result

    def _primary(self, depth: int) -> Node:
        self._check_depth(depth)
        if self.cursor >= len(self.tokens):
            fail("extraction", "ebnf_primary")
        token = self.tokens[self.cursor]
        self.cursor += 1
        if token.kind == "ref":
            return _node(self.budget, "ref", token.start, token.end, value=token.value)
        if token.kind == "quoted":
            kind = "pattern" if token.value == "[0-9]+" else "fixed"
            return _node(self.budget, kind, token.start, token.end, value=token.value)
        if token.kind != "left":
            fail("extraction", "ebnf_primary")
        child = self._choice(depth + 1)
        if self.cursor >= len(self.tokens) or self.tokens[self.cursor].kind != "right":
            fail("extraction", "ebnf_group_close")
        close = self.tokens[self.cursor]
        self.cursor += 1
        return _node(
            self.budget,
            "group",
            token.start,
            close.end,
            children=(child,),
        )

    def _peek(self, kind: str) -> bool:
        return self.cursor < len(self.tokens) and self.tokens[self.cursor].kind == kind


def _build_production(
    owner: str,
    lines: list[Line],
    budget: LogicalBudget,
) -> Production:
    first = lines[0]
    match = _HEAD.fullmatch(first.content)
    if match is None:
        fail("internal", "definition_head")
    lhs_raw = match.group(1)
    budget.maximum("max_symbol_bytes", len(lhs_raw))
    raw_segments = [_semantic_segment(first, match.start(4))]
    for continuation in lines[1:]:
        if not continuation.content.startswith(b" "):
            fail("extraction", "grammar_continuation")
        raw_segments.append(_semantic_segment(continuation))
    tokens = _tokenize(tuple(raw_segments), budget)
    root = _Parser(tokens, budget).parse()
    assignment_start = first.start + match.start(2) + len(match.group(2))
    return Production(
        owner,
        lhs_raw.decode("ascii"),
        first.start,
        tokens[-1].end,
        first.start + match.start(4),
        tokens[-1].end,
        assignment_start,
        root,
    )


def parse_regions(
    regions: tuple[GrammarRegion, ...],
    budget: LogicalBudget,
) -> tuple[Production, ...]:
    productions: list[Production] = []
    seen: dict[str, int] = {}
    for region in regions:
        pending: list[Line] = []
        for line in region.lines:
            if not line.content:
                continue
            if _ALTERNATE.fullmatch(line.content) is not None:
                fail("extraction", "single_equals_grammar")
            if _HEAD.fullmatch(line.content) is not None:
                if pending:
                    budget.add("max_definitions")
                    productions.append(_build_production(region.owner, pending, budget))
                pending = [line]
                continue
            if not pending or not line.content.startswith(b" "):
                fail("extraction", "grammar_continuation")
            pending.append(line)
        if not pending:
            fail("extraction", "empty_grammar_region")
        budget.add("max_definitions")
        productions.append(_build_production(region.owner, pending, budget))
    productions.sort(key=lambda production: production.definition_start)
    for production in productions:
        if production.lhs in seen:
            fail("extraction", "duplicate_definition")
        seen[production.lhs] = production.definition_start
    return tuple(productions)


def walk_nodes(node: Node, path: str = "0") -> tuple[tuple[str, Node], ...]:
    result = [(path, node)]
    for index, child in enumerate(node.children):
        result.extend(walk_nodes(child, f"{path}.{index}"))
    return tuple(result)
