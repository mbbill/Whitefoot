"""Complete independent extraction and common-ledger model."""

from __future__ import annotations

from dataclasses import dataclass
import re

from core import Limits, LogicalBudget, fail
from ebnf import Node, Production, parse_regions, walk_nodes
from lexical import LexicalDefinition, extract_lexical
from source import Rule, SourceScan, Surface, scan_source


_COMPOUND = re.compile(r"([^A-Za-z0-9_]+)([A-Za-z_][A-Za-z0-9_]*)\Z")
_LOWERWORD = re.compile(r"[a-z][a-z0-9_]*\Z")
_IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\Z")


@dataclass(frozen=True)
class ExpansionAtom:
    kind: str
    spelling: bytes


@dataclass(frozen=True)
class FixedOccurrence:
    lhs: str
    path: str
    start: int
    end: int
    spelling: bytes
    expansion: tuple[ExpansionAtom, ...]

    @property
    def descriptor(self) -> bytes:
        return b",".join(
            atom.kind.encode("ascii") + b":" + atom.spelling.hex().encode("ascii")
            for atom in self.expansion
        )


@dataclass(frozen=True)
class ReferenceOccurrence:
    lhs: str
    path: str
    start: int
    end: int
    name: str


@dataclass(frozen=True)
class Coverage:
    assignments: int
    fences: int
    inline: int
    lexical_cues: int
    unclassified: int = 0


@dataclass(frozen=True)
class GrammarDocument:
    name: str
    source: bytes
    rules: tuple[Rule, ...]
    surfaces: tuple[Surface, ...]
    productions: tuple[Production, ...]
    lexical: tuple[LexicalDefinition, ...]
    fixed: tuple[FixedOccurrence, ...]
    references: tuple[ReferenceOccurrence, ...]
    coverage: Coverage
    production_by_name: dict[str, Production]
    lexical_by_name: dict[str, LexicalDefinition]
    source_lowerwords: tuple[bytes, ...]
    expanded_lowerwords: tuple[bytes, ...]
    fixed_token_spellings: tuple[bytes, ...]
    has_number_pattern: bool


def _shape_kind(spelling: str) -> str:
    if _LOWERWORD.fullmatch(spelling):
        return "lowerword"
    if _IDENTIFIER.fullmatch(spelling):
        return "identifier"
    return {
        "&": "ampersand",
        "->": "thin-arrow",
        "=>": "fat-arrow",
    }.get(spelling, "punctuation")


def _expand_fixed(spelling: str) -> tuple[ExpansionAtom, ...]:
    compound = _COMPOUND.fullmatch(spelling)
    parts = compound.groups() if compound is not None else (spelling,)
    return tuple(
        ExpansionAtom(_shape_kind(part), part.encode("ascii"))
        for part in parts
    )


def _node_nullable(node: Node, nullable: dict[str, bool], defined: set[str]) -> bool:
    if node.kind in {"fixed", "pattern"}:
        return False
    if node.kind == "ref":
        if node.value is None:
            fail("internal", "reference_value")
        return nullable.get(node.value, False) if node.value in defined else False
    if node.kind in {"group", "repeat1"}:
        return _node_nullable(node.children[0], nullable, defined)
    if node.kind in {"optional", "repeat0"}:
        return True
    if node.kind == "sequence":
        return all(_node_nullable(child, nullable, defined) for child in node.children)
    if node.kind == "choice":
        return any(_node_nullable(child, nullable, defined) for child in node.children)
    fail("internal", "nullable_node_kind")


def _validate_nullable_repetitions(productions: tuple[Production, ...]) -> None:
    defined = {production.lhs for production in productions}
    nullable = {name: False for name in defined}
    changed = True
    while changed:
        changed = False
        for production in productions:
            value = _node_nullable(production.root, nullable, defined)
            if value and not nullable[production.lhs]:
                nullable[production.lhs] = True
                changed = True
    for production in productions:
        for _path, node in walk_nodes(production.root):
            if node.kind in {"repeat0", "repeat1"} and _node_nullable(
                node.children[0], nullable, defined
            ):
                fail("extraction", "nullable_repetition")


def _validate_production_reachability(
    production_by_name: dict[str, Production],
) -> None:
    if "program" not in production_by_name:
        fail("extraction", "program_start_missing")
    reachable = {"program"}
    pending = ["program"]
    while pending:
        name = pending.pop()
        for _path, node in walk_nodes(production_by_name[name].root):
            if (
                node.kind == "ref"
                and node.value in production_by_name
                and node.value not in reachable
            ):
                reachable.add(node.value)
                pending.append(node.value)
    if len(reachable) != len(production_by_name):
        fail("extraction", "unreachable_production")


def extract_document(name: str, source: bytes, limits: Limits) -> GrammarDocument:
    budget = LogicalBudget(limits)
    scan: SourceScan = scan_source(source, budget)
    productions = parse_regions(scan.regions, budget)
    parsed_assignments = tuple(sorted(item.assignment_start for item in productions))
    if parsed_assignments != scan.assignment_offsets:
        fail("extraction", "assignment_coverage")
    lexical, lexical_surfaces = extract_lexical(scan, budget)
    production_by_name = {item.lhs: item for item in productions}
    lexical_by_name = {item.name: item for item in lexical}
    if set(production_by_name) & set(lexical_by_name):
        fail("extraction", "symbol_kind_collision")

    fixed: list[FixedOccurrence] = []
    references: list[ReferenceOccurrence] = []
    has_number_pattern = False
    for production in productions:
        for path, node in walk_nodes(production.root):
            if node.kind == "fixed":
                if node.value is None:
                    fail("internal", "fixed_value")
                fixed.append(
                    FixedOccurrence(
                        production.lhs,
                        path,
                        node.start,
                        node.end,
                        node.value.encode("ascii"),
                        _expand_fixed(node.value),
                    )
                )
            elif node.kind == "pattern":
                has_number_pattern = True
            elif node.kind == "ref":
                if node.value is None:
                    fail("internal", "reference_value")
                references.append(
                    ReferenceOccurrence(
                        production.lhs,
                        path,
                        node.start,
                        node.end,
                        node.value,
                    )
                )
    undefined = {
        item.name
        for item in references
        if item.name not in production_by_name and item.name not in lexical_by_name
    }
    if undefined:
        fail("extraction", "undefined_reference")
    _validate_production_reachability(production_by_name)
    _validate_nullable_repetitions(productions)

    source_lowerwords = tuple(
        sorted(
            {
                item.spelling
                for item in fixed
                if _LOWERWORD.fullmatch(item.spelling.decode("ascii"))
            }
        )
    )
    expanded_lowerwords = tuple(
        sorted(
            {
                atom.spelling
                for item in fixed
                for atom in item.expansion
                if atom.kind == "lowerword"
            }
        )
    )
    fixed_token_spellings = tuple(
        sorted({atom.spelling for item in fixed for atom in item.expansion})
    )
    surfaces = tuple(sorted(scan.surfaces + lexical_surfaces, key=lambda item: (item.start, item.kind)))
    coverage = Coverage(
        len(scan.assignment_offsets),
        sum(item.kind == "grammar-fence" for item in surfaces),
        sum(item.kind == "grammar-inline" for item in surfaces),
        sum(item.kind == "lexical-cue" for item in surfaces),
    )
    return GrammarDocument(
        name,
        source,
        scan.rules,
        surfaces,
        productions,
        lexical,
        tuple(fixed),
        tuple(references),
        coverage,
        production_by_name,
        lexical_by_name,
        source_lowerwords,
        expanded_lowerwords,
        fixed_token_spellings,
        has_number_pattern,
    )
