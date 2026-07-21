"""Closed lexical declarations and independent membership predicates."""

from __future__ import annotations

from dataclasses import dataclass
import re

from core import LogicalBudget, fail
from source import LexicalCue, SourceScan, Surface


_CLASS_NAME = re.compile(rb"[A-Z][A-Z0-9_]*")
_TABLE_SIGNATURE = re.compile(rb"[a-z][a-z0-9_]*\([a-z](?:, [a-z])*\)")
_MODIFIER = (
    b" excluding every lowercase token spelling produced by exact fixed grammar atoms in the "
    b"complete grammar"
)
_REGION_ANNOTATION = b" (apostrophe-prefixed, the only region spelling)"
_OPNAME_ANNOTATION = (
        b" (single token; the base is an IDENT and the mode suffix is a closed word set, "
        b"so an OPNAME can never maximal-munch a field-access place `p.field` [GRAM-5]; "
        b"e.g. `iadd.checked`)"
)
_CLASS_SPECS = (
    (b"IDENT", b"[a-z][a-z0-9_]*", None),
    (b"TYPEID", b"[A-Z][A-Za-z0-9]*", None),
    (b"REGIONID", b"'[a-z][a-z0-9_]*", _REGION_ANNOTATION),
    (b"LABEL", b"@[a-z][a-z0-9_]*", None),
    (
        b"OPNAME",
        b"[a-z][a-z0-9_]*\\.(wrap|trap|checked|sat|strict)",
        _OPNAME_ANNOTATION,
    ),
)
_INTEGER_TEMPLATE = b"-?[0-9]+_TYPE"
_FLOAT_TEMPLATE = b"-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE"
_BETWEEN_PATTERNS = (
    b" (decimal only, mandatory suffix; a leading `-` is legal for signed TYPE, "
    b"and the signed value must lie in TYPE's range [FORM-7]; e.g. `42_i32`, "
    b"`-2147483648_i32`); floats `"
)
_AFTER_FLOAT = (
    b"` (a leading `-` is legal for the value; the canonical spelling is the unique "
    b"shortest decimal digit string that round-trips under round-to-nearest-even, "
    b"with at least one integer and one fraction digit, lowercase `e`, and no leading "
    b"zeros; `-0.0` is distinct from `0.0`; e.g. `1.5_f64`, `6.022e23_f64`); "
    b'`unit`; STRING `"..."` whose interior is a sequence of items, each one raw '
    b'ASCII-printable byte in U+0020..U+007E other than `"` and `\\`, or one of '
    b'exactly three escapes `\\\\ \\" \\n`; no other byte is legal, and each character '
    b"has exactly one spelling (the escape where one is defined, the raw byte otherwise). "
    b"STRING appears only in `doc` and `check` messages; non-ASCII diagnostic text is "
    b"DEFERRED. There are no boolean literals: `Bool` is a prelude enum (\xc2\xa715). "
    b"Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound "
    b"by a numeric contract (`Int` or `Float`, \xc2\xa715), denoting T's additive and "
    b"multiplicative identity; a concrete type uses `0_i32` and the like, so there is "
    b"no dual spelling. NaN and the infinities are not literals; they are the nullary "
    b"ops `fnan` and `finf` [OP-1]."
)
_FORM5_PAYLOAD = (
    b"Literals, exhaustively: integers `"
    + _INTEGER_TEMPLATE
    + b"`"
    + _BETWEEN_PATTERNS
    + _FLOAT_TEMPLATE
    + _AFTER_FLOAT
)
_STRING_START = b'STRING `"..."`'
_STRING_END = b"non-ASCII diagnostic text is DEFERRED."

_LITERAL_PREDICATE = (
    b"integer=-?[0-9]+_TYPE;float=-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE;"
    b"unit=unit;generic=0_T,1_T"
)
_STRING_PREDICATE = (
    b"range=32-126;exclude=34,92;escapes=backslash,quote,n;contexts=doc,check"
)
_INTEGER = re.compile(rb"-?[0-9]+_(?:i8|i16|i32|i64|u8|u16|u32|u64)")
_FLOAT = re.compile(rb"-?[0-9]+\.[0-9]+(?:e-?[0-9]+)?_(?:f32|f64)")
_GENERIC = re.compile(rb"(?:0|1)_T")


@dataclass(frozen=True)
class LexicalDefinition:
    owner: str
    name: str
    kind: str
    start: int
    end: int
    predicate: bytes
    pattern: re.Pattern[bytes] | None = None
    exclude_fixed_lowerwords: bool = False
    spellings: tuple[bytes, ...] = ()

    def match_prefix(
        self,
        source: bytes,
        start: int,
        fixed_lowerwords: frozenset[bytes],
    ) -> int | None:
        if self.kind == "regex":
            if self.pattern is None:
                fail("internal", "regex_missing")
            match = self.pattern.match(source, start)
            if match is None or match.end() == start:
                return None
            spelling = source[start : match.end()]
            if self.exclude_fixed_lowerwords and spelling in fixed_lowerwords:
                return None
            return match.end()
        if self.kind == "literal-union":
            ends = [
                match.end()
                for matcher in (_FLOAT, _INTEGER, _GENERIC)
                if (match := matcher.match(source, start)) is not None
            ]
            if source.startswith(b"unit", start):
                ends.append(start + 4)
            return max(ends) if ends else None
        if self.kind == "byte-string":
            return _match_string(source, start)
        if self.kind == "closed-table":
            ends = [start + len(value) for value in self.spellings if source.startswith(value, start)]
            return max(ends) if ends else None
        fail("internal", "lexical_kind")


class _RegexParser:
    def __init__(self, raw: bytes, budget: LogicalBudget) -> None:
        self.raw = raw
        self.cursor = 0
        self.budget = budget

    def validate(self) -> re.Pattern[bytes]:
        self._expression(0)
        if self.cursor != len(self.raw):
            fail("extraction", "lexical_pattern")
        try:
            compiled = re.compile(self.raw)
        except re.error:
            fail("internal", "regex_compile")
        if compiled.fullmatch(b"") is not None:
            fail("extraction", "nullable_lexical_pattern")
        return compiled

    def _expression(self, depth: int) -> None:
        self._sequence(depth)
        while self._take(ord("|")):
            self._sequence(depth)

    def _sequence(self, depth: int) -> None:
        count = 0
        while self.cursor < len(self.raw) and self.raw[self.cursor] not in (ord("|"), ord(")")):
            self._repetition(depth)
            count += 1
        if count == 0:
            fail("extraction", "lexical_pattern")

    def _repetition(self, depth: int) -> None:
        self._atom(depth)
        if self.cursor < len(self.raw) and self.raw[self.cursor] in b"*+?":
            self.cursor += 1
            if self.cursor < len(self.raw) and self.raw[self.cursor] in b"*+?":
                fail("extraction", "lexical_pattern")

    def _atom(self, depth: int) -> None:
        if self.cursor >= len(self.raw):
            fail("extraction", "lexical_pattern")
        byte = self.raw[self.cursor]
        if byte in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_'@":
            self.cursor += 1
            return
        if byte == ord("\\"):
            self.cursor += 1
            if self.cursor >= len(self.raw) or self.raw[self.cursor] not in b".\\|()[]*+?-":
                fail("extraction", "lexical_pattern")
            self.cursor += 1
            return
        if byte == ord("["):
            self._character_class()
            return
        if byte == ord("("):
            next_depth = depth + 1
            self.budget.maximum("max_ebnf_depth", next_depth)
            self.cursor += 1
            self._expression(next_depth)
            if not self._take(ord(")")):
                fail("extraction", "lexical_pattern")
            return
        fail("extraction", "lexical_pattern")

    def _character_class(self) -> None:
        self.cursor += 1
        count = 0
        while self.cursor < len(self.raw) and self.raw[self.cursor] != ord("]"):
            first = self._class_byte()
            if self.cursor < len(self.raw) and self.raw[self.cursor] == ord("-"):
                self.cursor += 1
                last = self._class_byte()
                if first > last:
                    fail("extraction", "lexical_pattern")
            count += 1
        if count == 0 or not self._take(ord("]")):
            fail("extraction", "lexical_pattern")

    def _class_byte(self) -> int:
        if self.cursor >= len(self.raw):
            fail("extraction", "lexical_pattern")
        byte = self.raw[self.cursor]
        if not 0x20 <= byte <= 0x7E or byte in b"[]\\`-":
            fail("extraction", "lexical_pattern")
        self.cursor += 1
        return byte

    def _take(self, expected: int) -> bool:
        if self.cursor < len(self.raw) and self.raw[self.cursor] == expected:
            self.cursor += 1
            return True
        return False


def _match_string(source: bytes, start: int) -> int | None:
    if start >= len(source) or source[start] != ord('"'):
        return None
    cursor = start + 1
    while cursor < len(source):
        byte = source[cursor]
        if byte == ord('"'):
            return cursor + 1
        if byte == ord("\\"):
            if cursor + 1 >= len(source) or source[cursor + 1] not in (ord("\\"), ord('"'), ord("n")):
                return None
            cursor += 2
            continue
        if not 0x20 <= byte <= 0x7E:
            return None
        cursor += 1
    return None


def _classes(
    cue: LexicalCue,
    line_start: int,
    content: bytes,
    budget: LogicalBudget,
) -> tuple[list[LexicalDefinition], Surface]:
    local_start = cue.start - line_start
    prefix = b"Lexical classes: "
    if not content.startswith(prefix, local_start):
        fail("extraction", "lexical_classes_shape")
    cursor = local_start + len(prefix)
    result: list[LexicalDefinition] = []
    for index, (expected_name, expected_pattern, expected_annotation) in enumerate(
        _CLASS_SPECS
    ):
        name_match = _CLASS_NAME.match(content, cursor)
        if name_match is None or name_match.group() != expected_name:
            fail("extraction", "lexical_classes_shape")
        name = name_match.group().decode("ascii")
        budget.maximum("max_symbol_bytes", len(name_match.group()))
        name_start = name_match.start()
        cursor = name_match.end()
        if not content.startswith(b" `", cursor):
            fail("extraction", "lexical_classes_shape")
        pattern_start = cursor + 2
        close = content.find(b"`", pattern_start)
        if close < 0 or close == pattern_start:
            fail("extraction", "lexical_pattern")
        raw_pattern = content[pattern_start:close]
        if raw_pattern != expected_pattern:
            fail("extraction", "lexical_classes_shape")
        compiled = _RegexParser(raw_pattern, budget).validate()
        cursor = close + 1
        semantic_end = cursor
        excluded = False
        if content.startswith(_MODIFIER, cursor):
            if expected_name != b"IDENT":
                fail("extraction", "lexical_classes_shape")
            cursor += len(_MODIFIER)
            semantic_end = cursor
            excluded = True
        if expected_annotation is not None:
            if not content.startswith(expected_annotation, cursor):
                fail("extraction", "lexical_classes_shape")
            cursor += len(expected_annotation)
        if content.startswith(b" (", cursor):
            fail("extraction", "lexical_annotation")
        budget.add("max_lexical_definitions")
        predicate = (
            b"pattern="
            + raw_pattern
            + (b";exclude=fixed-lowerwords" if excluded else b";exclude=none")
        )
        result.append(
            LexicalDefinition(
                cue.owner,
                name,
                "regex",
                line_start + name_start,
                line_start + semantic_end,
                predicate,
                compiled,
                excluded,
            )
        )
        if index + 1 == len(_CLASS_SPECS):
            if content[cursor:] != b".":
                fail("extraction", "lexical_classes_shape")
            cursor += 1
        else:
            if not content.startswith(b"; ", cursor):
                fail("extraction", "lexical_classes_shape")
            cursor += 2
    if cursor != len(content):
        fail("extraction", "lexical_classes_shape")
    cue_end = cue.start + len(b"Lexical classes:")
    return result, Surface("lexical-cue", cue.start, cue_end, cue.owner)


def _literals(
    cue: LexicalCue,
    line_start: int,
    content: bytes,
    budget: LogicalBudget,
) -> tuple[list[LexicalDefinition], Surface]:
    local_start = cue.start - line_start
    payload = content[local_start:]
    if payload != _FORM5_PAYLOAD:
        fail("extraction", "literal_shape")
    string_start = payload.index(_STRING_START)
    string_end = payload.index(_STRING_END, string_start) + len(_STRING_END)
    budget.add("max_lexical_definitions", 2)
    return [
        LexicalDefinition(
            cue.owner,
            "literal",
            "literal-union",
            cue.start,
            line_start + len(content),
            _LITERAL_PREDICATE,
        ),
        LexicalDefinition(
            cue.owner,
            "STRING",
            "byte-string",
            cue.start + string_start,
            cue.start + string_end,
            _STRING_PREDICATE,
        ),
    ], Surface("lexical-cue", cue.start, cue.end, cue.owner)


def _table(
    cue: LexicalCue,
    line_start: int,
    content: bytes,
    budget: LogicalBudget,
) -> tuple[LexicalDefinition, Surface]:
    local_start = cue.start - line_start
    header = content[local_start : cue.end - line_start]
    suffix = b" is a closed table:"
    if not header.endswith(suffix):
        fail("extraction", "closed_table_shape")
    name_raw = header[: -len(suffix)]
    if _CLASS_NAME.fullmatch(name_raw) is None:
        fail("extraction", "closed_table_shape")
    cursor = local_start + len(header)
    if not content.startswith(b" `", cursor):
        fail("extraction", "closed_table_shape")
    cursor += 1
    signatures: list[bytes] = []
    spellings: list[bytes] = []
    while True:
        if cursor >= len(content) or content[cursor] != ord("`"):
            fail("extraction", "closed_table_shape")
        close = content.find(b"`", cursor + 1)
        if close < 0:
            fail("extraction", "closed_table_shape")
        signature = content[cursor + 1 : close]
        if _TABLE_SIGNATURE.fullmatch(signature) is None:
            fail("extraction", "closed_table_signature")
        spelling = signature.split(b"(", 1)[0]
        if spelling in spellings:
            fail("extraction", "closed_table_duplicate")
        signatures.append(signature.replace(b", ", b","))
        spellings.append(spelling)
        cursor = close + 1
        if content[cursor:] == b".":
            cursor += 1
            break
        if not content.startswith(b", ", cursor):
            fail("extraction", "closed_table_shape")
        cursor += 2
    if cursor != len(content):
        fail("extraction", "closed_table_shape")
    budget.add("max_lexical_definitions")
    budget.maximum("max_symbol_bytes", len(name_raw))
    definition = LexicalDefinition(
        cue.owner,
        name_raw.decode("ascii"),
        "closed-table",
        cue.start,
        line_start + len(content),
        b",".join(signatures),
        spellings=tuple(spellings),
    )
    return definition, Surface("lexical-cue", cue.start, cue.end, cue.owner)


def extract_lexical(
    scan: SourceScan,
    budget: LogicalBudget,
) -> tuple[tuple[LexicalDefinition, ...], tuple[Surface, ...]]:
    definitions: list[LexicalDefinition] = []
    surfaces: list[Surface] = []
    for cue in scan.lexical_cues:
        line = scan.lines[cue.line_index]
        spelling = line.content[cue.start - line.start : cue.end - line.start]
        if cue.dialect == "class" and spelling == b"Lexical classes":
            added, surface = _classes(cue, line.start, line.content, budget)
            definitions.extend(added)
            surfaces.append(surface)
        elif cue.dialect == "literal" and spelling == b"Literals, exhaustively:":
            added, surface = _literals(cue, line.start, line.content, budget)
            definitions.extend(added)
            surfaces.append(surface)
        elif cue.dialect == "table":
            added, surface = _table(cue, line.start, line.content, budget)
            definitions.append(added)
            surfaces.append(surface)
        else:
            fail("extraction", "unclassified_lexical_cue")
    definitions.sort(key=lambda item: (item.start, item.name))
    first: dict[str, int] = {}
    for definition in definitions:
        if definition.name in first:
            fail("extraction", "duplicate_lexical_definition")
        first[definition.name] = definition.start
    if not definitions:
        fail("extraction", "lexical_definitions_missing")
    return tuple(definitions), tuple(surfaces)
