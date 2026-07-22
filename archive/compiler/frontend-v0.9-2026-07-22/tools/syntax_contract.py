#!/usr/bin/env python3
"""Extract and analyze the exact v0.9 source grammar for production data."""

from __future__ import annotations

from dataclasses import dataclass, replace
import hashlib
import re
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT.parent / "spec" / "kernel-spec-v0.9.md"
TERMINALS = ROOT / "crates" / "whitefoot-language-data" / "src" / "terminal.rs"
GENERATED = ROOT / "crates" / "whitefoot-syntax-data" / "src" / "generated_v0_9.rs"
SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
SELECT2_PROJECTION_SHA256 = (
    "986dc32d51f584488fd1cedab402a9987b7bbdef60f68dabb5e4b5bacb74b4bc"
)

PRODUCTION_NAMES = (
    "program", "item", "struct_decl", "field", "enum_decl", "variant",
    "vfield_list", "vfield", "fn_decl", "requires_block", "requires_entry",
    "contract_decl", "fn_sig", "law", "law_arg", "conform_decl", "const_decl",
    "fn_bind", "doc", "generics", "gparam", "region_params", "param_list",
    "param", "type", "rtype", "mode", "targs", "targ", "stmt", "let_stmt",
    "ordinary_let_rhs", "try_let_rhs", "set_stmt", "expr_stmt", "return_stmt",
    "loop_stmt", "break_stmt", "region_stmt", "check_stmt", "give_stmt",
    "match_stmt", "value_match", "arm", "fieldbind_list", "fieldbind", "expr",
    "atom", "call", "callee", "construct", "fieldinit_list", "fieldinit",
    "borrow_expr", "atom_list", "place", "pbase", "psuffix", "const", "cvalue",
    "effects", "effect",
)

RULE_OWNER = {
    **{name: "Gram2" for name in PRODUCTION_NAMES[:24]},
    **{name: "Gram3" for name in PRODUCTION_NAMES[24:29]},
    **{name: "Gram4" for name in PRODUCTION_NAMES[29:46]},
    **{name: "Gram5" for name in PRODUCTION_NAMES[46:58]},
    "const": "Const1",
    "cvalue": "Const2",
    "effects": "Eff1",
    "effect": "Eff1",
}

EXTERNAL = {
    "IDENT": "Identifier",
    "TYPEID": "TypeIdentifier",
    "REGIONID": "RegionIdentifier",
    "LABEL": "Label",
    "OPNAME": "OperationName",
    "literal": "Literal",
    "STRING": "String",
}
NAME_PREDICATES = frozenset(
    {"Identifier", "TypeIdentifier", "RegionIdentifier", "Label", "OperationName"}
)


class ContractError(ValueError):
    """A controlled source-grammar or generated-data mismatch."""


@dataclass
class Node:
    """One source-EBNF node before stable preorder numbering."""

    kind: str
    value: str | None = None
    children: tuple["Node", ...] = ()
    production: int = -1
    path: str = ""
    index: int = -1


@dataclass(frozen=True)
class Production:
    """One production in exact specification-definition order."""

    name: str
    owner: str
    root: Node


@dataclass(frozen=True, order=True)
class Atom:
    """One lookahead predicate with source provenance."""

    predicate: str
    node: int | None
    transparent_name: str | None
    atom_only: bool


Word = tuple[Atom, ...]
Words = frozenset[Word]


@dataclass(frozen=True, order=True)
class SelectAtom:
    """One decision-row position and its diagnostic metadata."""

    predicate: str
    node: int | None
    inside_arm: bool
    transparent_name: str | None
    atom_only: bool


@dataclass(frozen=True, order=True)
class SelectRow:
    """One source arm's complete two-position SELECT row."""

    arm: int
    first: SelectAtom
    second: SelectAtom


@dataclass(frozen=True)
class Decision:
    """One source choice, optional, or repetition-continuation decision."""

    node: int
    production: int
    kind: str
    context: str
    arm_count: int
    rows: tuple[SelectRow, ...]


@dataclass(frozen=True)
class Grammar:
    """Complete exact-v0.9 grammar and predictive relation."""

    productions: tuple[Production, ...]
    nodes: tuple[Node, ...]
    children: tuple[int, ...]
    terminals: tuple[str, ...]
    fixed_variants: dict[bytes, str]
    decisions: tuple[Decision, ...]
    diagnostic_order: tuple[str, ...]


TOKEN = re.compile(r'\s+|"[^"\n]+"|[A-Za-z_][A-Za-z0-9_]*|[()|?*+]')
DEFINITION = re.compile(r"(?m)^([a-z][a-z0-9_]*)\s*:=")
INLINE = re.compile(rb"`([a-z][a-z0-9_]*\s*:=\s*[^`]+)`")
SPELLING_ARM = re.compile(r'Self::([A-Za-z][A-Za-z0-9_]*)\s*=>\s*b"([^"\\]*)"')


def camel(name: str) -> str:
    """Return the closed Rust variant spelling for one grammar identifier."""
    return "".join(part[:1].upper() + part[1:] for part in name.split("_"))


def extract_definition_groups(specification: bytes) -> tuple[tuple[int, str], ...]:
    """Recover grammar-bearing fences and inline production quotations."""
    groups: list[tuple[int, str]] = []
    fenced_ranges: list[tuple[int, int]] = []
    for match in re.finditer(rb"```[^\n]*\n(.*?)```", specification, re.DOTALL):
        fenced_ranges.append((match.start(), match.end()))
        if b":=" in match.group(1):
            groups.append((match.start(1), match.group(1).decode("ascii")))
    for match in INLINE.finditer(specification):
        if any(start <= match.start() < end for start, end in fenced_ranges):
            continue
        name = match.group(1).split(b":=", 1)[0].strip().decode("ascii")
        if name in {"const", "cvalue", "effects", "effect"}:
            groups.append((match.start(1), match.group(1).decode("ascii")))
    groups.sort()
    return tuple(groups)


def production_texts(specification: bytes) -> tuple[tuple[str, str], ...]:
    """Return all and only approved productions in definition order."""
    output: list[tuple[str, str]] = []
    for _, group in extract_definition_groups(specification):
        matches = list(DEFINITION.finditer(group))
        for index, match in enumerate(matches):
            end = matches[index + 1].start() if index + 1 < len(matches) else len(group)
            rhs = group[match.end() : end].strip()
            output.append((match.group(1), rhs))
    names = tuple(name for name, _ in output)
    if names != PRODUCTION_NAMES:
        raise ContractError(f"production order differs from v0.9: {names!r}")
    return tuple(output)


class EbnfParser:
    """Small closed parser for the approved source-EBNF notation."""

    def __init__(self, source: str) -> None:
        self.tokens: list[str] = []
        cursor = 0
        while cursor < len(source):
            match = TOKEN.match(source, cursor)
            if match is None:
                raise ContractError(f"unknown EBNF text at {source[cursor:cursor + 20]!r}")
            token = match.group(0)
            cursor = match.end()
            if not token.isspace():
                self.tokens.append(token)
        self.cursor = 0

    def peek(self) -> str | None:
        return self.tokens[self.cursor] if self.cursor < len(self.tokens) else None

    def take(self) -> str:
        token = self.peek()
        if token is None:
            raise ContractError("unexpected end of EBNF")
        self.cursor += 1
        return token

    def parse(self) -> Node:
        node = self.choice()
        if self.peek() is not None:
            raise ContractError(f"trailing EBNF token {self.peek()!r}")
        return node

    def choice(self) -> Node:
        children = [self.sequence()]
        while self.peek() == "|":
            self.take()
            children.append(self.sequence())
        return children[0] if len(children) == 1 else Node("choice", children=tuple(children))

    def sequence(self) -> Node:
        children: list[Node] = []
        while self.peek() not in (None, "|", ")"):
            children.append(self.postfix())
        if not children:
            raise ContractError("empty EBNF sequence")
        return children[0] if len(children) == 1 else Node("sequence", children=tuple(children))

    def postfix(self) -> Node:
        node = self.primary()
        token = self.peek()
        if token in ("?", "*", "+"):
            self.take()
            node = Node({"?": "optional", "*": "repeat0", "+": "repeat1"}[token], children=(node,))
        return node

    def primary(self) -> Node:
        token = self.take()
        if token == "(":
            child = self.choice()
            if self.take() != ")":
                raise ContractError("unclosed EBNF group")
            return Node("group", children=(child,))
        if token.startswith('"'):
            return Node("fixed", value=token[1:-1])
        if token[0].isalpha() or token[0] == "_":
            return Node("ref", value=token)
        raise ContractError(f"unexpected EBNF token {token!r}")


def number_nodes(productions: list[Production]) -> tuple[Node, ...]:
    """Assign stable production/path identity and a dense preorder index."""
    output: list[Node] = []

    def visit(node: Node, production: int, path: str) -> None:
        node.production = production
        node.path = path
        node.index = len(output)
        output.append(node)
        for child_index, child in enumerate(node.children):
            visit(child, production, f"{path}.{child_index}")

    for production_index, production in enumerate(productions):
        visit(production.root, production_index, "0")
    return tuple(output)


def expand_fixed(value: str) -> tuple[bytes, ...]:
    """Expand one quoted atom into exact raw formed-token spellings."""
    raw = value.encode("ascii")
    if raw == b"[0-9]+":
        return ()
    output: list[bytes] = []
    cursor = 0
    while cursor < len(raw):
        if raw[cursor : cursor + 2] in (b"->", b"=>"):
            output.append(raw[cursor : cursor + 2])
            cursor += 2
        elif 0x61 <= raw[cursor] <= 0x7A:
            end = cursor + 1
            while end < len(raw) and (
                0x61 <= raw[end] <= 0x7A
                or 0x30 <= raw[end] <= 0x39
                or raw[end] == 0x5F
            ):
                end += 1
            output.append(raw[cursor:end])
            cursor = end
        elif raw[cursor] in b"(){}[]<>,:;.=&#":
            output.append(raw[cursor : cursor + 1])
            cursor += 1
        else:
            raise ContractError(f"unknown fixed atom byte in {value!r}")
    return tuple(output)


def fixed_variants(rust: str) -> dict[bytes, str]:
    """Recover the already-bound fixed spelling to Rust variant mapping."""
    output: dict[bytes, str] = {}
    for variant, spelling in SPELLING_ARM.findall(rust):
        raw = spelling.encode("ascii")
        if raw in output:
            raise ContractError(f"duplicate fixed spelling {raw!r}")
        output[raw] = variant
    if len(output) != 64:
        raise ContractError(f"expected 64 fixed variants, got {len(output)}")
    return output


def terminal_descriptor(predicate: str, fixed: dict[bytes, str]) -> str:
    """Return the Rust expression for one closed predicate descriptor."""
    if predicate == "End":
        return "LookaheadPredicateV0_9::SourceEnd"
    if predicate.startswith("Fixed:"):
        spelling = bytes.fromhex(predicate.removeprefix("Fixed:"))
        return (
            "LookaheadPredicateV0_9::Terminal(TerminalPredicateV0_9::Fixed("
            f"FixedTerminalV0_9::{fixed[spelling]}))"
        )
    return f"LookaheadPredicateV0_9::Terminal(TerminalPredicateV0_9::{predicate})"


def node_terminal_words(node: Node, fixed: dict[bytes, str]) -> Words:
    """Return the direct terminal word for one terminal-bearing node."""
    if node.kind == "fixed":
        if node.value == "[0-9]+":
            predicates = ("Digits",)
        else:
            predicates = tuple(f"Fixed:{item.hex()}" for item in expand_fixed(node.value or ""))
            for item in expand_fixed(node.value or ""):
                if item not in fixed:
                    raise ContractError(f"fixed spelling {item!r} is absent from terminal data")
    elif node.kind == "ref" and node.value in EXTERNAL:
        predicates = (EXTERNAL[node.value or ""],)
    else:
        raise ContractError("node is not a terminal occurrence")
    return frozenset(
        [
            tuple(
                Atom(
                    predicate=predicate,
                    node=node.index,
                    transparent_name=predicate if predicate in NAME_PREDICATES else None,
                    atom_only=False,
                )
                for predicate in predicates
            )
        ]
    )


EPSILON: Words = frozenset([()])


def concat(left: Words, right: Words) -> Words:
    """Concatenate provenance words and truncate to two positions."""
    return frozenset((lhs + rhs)[:2] for lhs in left for rhs in right)


def repeat(words: Words) -> Words:
    """Compute the bounded Kleene closure of a nonnullable word language."""
    output = EPSILON
    while True:
        extended = output | concat(output, words)
        if extended == output:
            return output
        output = frozenset(extended)


def block_transparent(words: Words) -> Words:
    """Mark source-choice traversal as opaque to mandatory-name attribution."""
    return frozenset(
        tuple(replace(atom, transparent_name=None) for atom in word) for word in words
    )


def is_atom_only_reference(node: Node) -> bool:
    """Return whether this exact `atom` occurrence is GRAM-9 restricted."""
    return (
        node.kind == "ref"
        and node.value == "atom"
        and (
            PRODUCTION_NAMES[node.production] in {"atom_list", "fieldinit"}
            or (PRODUCTION_NAMES[node.production] == "pbase" and node.path == "0.2.7")
        )
    )


def nullable(node: Node, production_nullable: dict[str, bool], names: set[str]) -> bool:
    """Evaluate nullability under the current production fixpoint."""
    if node.kind == "ref":
        return node.value in names and production_nullable[node.value or ""]
    if node.kind == "fixed":
        return False
    if node.kind == "sequence":
        return all(nullable(child, production_nullable, names) for child in node.children)
    if node.kind == "choice":
        return any(nullable(child, production_nullable, names) for child in node.children)
    if node.kind in {"group", "repeat1"}:
        return nullable(node.children[0], production_nullable, names)
    if node.kind in {"optional", "repeat0"}:
        return True
    raise ContractError(f"unknown node kind {node.kind}")


def prefix(node: Node, first: dict[str, Words], names: set[str], fixed: dict[bytes, str]) -> Words:
    """Compute provenance-carrying FIRST_2 words for one EBNF node."""
    if node.kind == "ref":
        if node.value in names:
            words = first[node.value or ""]
            if is_atom_only_reference(node):
                return frozenset(
                    tuple(replace(atom, atom_only=True) for atom in word) for word in words
                )
            return words
        if node.value not in EXTERNAL:
            raise ContractError(f"unknown grammar reference {node.value!r}")
        return node_terminal_words(node, fixed)
    if node.kind == "fixed":
        return node_terminal_words(node, fixed)
    if node.kind == "sequence":
        output = EPSILON
        for child in node.children:
            output = concat(output, prefix(child, first, names, fixed))
        return output
    if node.kind == "choice":
        output: set[Word] = set()
        for child in node.children:
            output.update(block_transparent(prefix(child, first, names, fixed)))
        return frozenset(output)
    if node.kind == "group":
        return prefix(node.children[0], first, names, fixed)
    if node.kind == "optional":
        return EPSILON | prefix(node.children[0], first, names, fixed)
    if node.kind == "repeat0":
        return repeat(prefix(node.children[0], first, names, fixed))
    if node.kind == "repeat1":
        body = prefix(node.children[0], first, names, fixed)
        return concat(body, repeat(body))
    raise ContractError(f"unknown node kind {node.kind}")


def first_sets(productions: tuple[Production, ...], fixed: dict[bytes, str]) -> dict[str, Words]:
    """Compute complete provenance-carrying FIRST_2 production languages."""
    names = {production.name for production in productions}
    output = {name: frozenset() for name in names}
    while True:
        changed = False
        for production in productions:
            value = output[production.name] | prefix(production.root, output, names, fixed)
            if value != output[production.name]:
                output[production.name] = value
                changed = True
        if not changed:
            return output


def suffix_words(
    children: Iterable[Node],
    first: dict[str, Words],
    names: set[str],
    fixed: dict[bytes, str],
    outer: Words,
) -> Words:
    """Return FIRST_2(children followed by outer)."""
    output = EPSILON
    for child in children:
        output = concat(output, prefix(child, first, names, fixed))
    return concat(output, outer)


def propagate_follow(
    node: Node,
    outer: Words,
    first: dict[str, Words],
    follow: dict[str, Words],
    names: set[str],
    fixed: dict[bytes, str],
) -> bool:
    """Propagate one occurrence's provenance-carrying continuation."""
    changed = False
    if node.kind == "ref":
        if node.value in names:
            previous = follow[node.value or ""]
            follow[node.value or ""] = previous | outer
            return previous != follow[node.value or ""]
        return False
    if node.kind == "fixed":
        return False
    if node.kind == "sequence":
        for index, child in enumerate(node.children):
            continuation = suffix_words(node.children[index + 1 :], first, names, fixed, outer)
            changed |= propagate_follow(child, continuation, first, follow, names, fixed)
        return changed
    if node.kind == "choice":
        for child in node.children:
            changed |= propagate_follow(child, outer, first, follow, names, fixed)
        return changed
    if node.kind in {"group", "optional"}:
        return propagate_follow(node.children[0], outer, first, follow, names, fixed)
    if node.kind in {"repeat0", "repeat1"}:
        body = prefix(node.children[0], first, names, fixed)
        continuation = concat(repeat(body), outer)
        return propagate_follow(node.children[0], continuation, first, follow, names, fixed)
    raise ContractError(f"unknown node kind {node.kind}")


def follow_sets(
    productions: tuple[Production, ...],
    first: dict[str, Words],
    fixed: dict[bytes, str],
) -> dict[str, Words]:
    """Compute complete provenance-carrying FOLLOW_2 production languages."""
    names = {production.name for production in productions}
    end = Atom("End", None, None, False)
    output = {name: frozenset() for name in names}
    output["program"] = frozenset([((end), (end))])
    while True:
        changed = False
        for production in productions:
            changed |= propagate_follow(
                production.root,
                output[production.name],
                first,
                output,
                names,
                fixed,
            )
        if not changed:
            return output


def decision_context(node: Node) -> str:
    """Classify only the closed DIAG-1 entry frontiers."""
    production = PRODUCTION_NAMES[node.production]
    if production == "program" and node.path == "0":
        return "ProgramItems"
    if production in {"item", "stmt", "requires_entry"} and node.path == "0":
        return "ConstructEntry"
    if node.kind in {"repeat0", "repeat1"}:
        body = node.children[0]
        if body.kind == "ref" and body.value in {"stmt", "requires_entry"}:
            return "ConstructEntry"
    return "Ordinary"


def marked_words(inside: Words, outside: Words) -> tuple[tuple[SelectAtom, ...], ...]:
    """Concatenate arm and caller words while retaining their boundary."""
    output: set[tuple[SelectAtom, ...]] = set()
    for lhs in inside:
        for rhs in outside:
            values = tuple(
                SelectAtom(
                    atom.predicate,
                    atom.node,
                    True,
                    atom.transparent_name,
                    atom.atom_only,
                )
                for atom in lhs
            )
            if len(values) < 2:
                values += tuple(
                    SelectAtom(
                        atom.predicate,
                        atom.node,
                        False,
                        atom.transparent_name,
                        atom.atom_only,
                    )
                    for atom in rhs[: 2 - len(values)]
                )
            output.add(values[:2])
    return tuple(sorted(output))


def rows_for_arms(arms: tuple[tuple[SelectAtom, ...], ...]) -> tuple[SelectRow, ...]:
    """Flatten already-marked arm words into ordered decision rows."""
    output: list[SelectRow] = []
    for arm, words in enumerate(arms):
        for word in words:
            if len(word) != 2:
                raise ContractError("SELECT row does not contain exactly two positions")
            output.append(SelectRow(arm, word[0], word[1]))
    return tuple(output)


def predicates_overlap(left: str, right: str) -> bool:
    """Return exact v0.9 token-set intersection for predictive predicates."""
    if left == right:
        return True
    return {left, right} == {"Fixed:756e6974", "Literal"}


def validate_disjoint(decision: Decision) -> None:
    """Require pairwise-disjoint arm languages under set-valued membership."""
    for left_index, left in enumerate(decision.rows):
        for right in decision.rows[left_index + 1 :]:
            if left.arm == right.arm:
                continue
            if predicates_overlap(left.first.predicate, right.first.predicate) and predicates_overlap(
                left.second.predicate, right.second.predicate
            ):
                raise ContractError(
                    f"predictive conflict at {PRODUCTION_NAMES[decision.production]} "
                    f"node {decision.node}, arms {left.arm} and {right.arm}"
                )


def collect_decisions(
    productions: tuple[Production, ...],
    first: dict[str, Words],
    follow: dict[str, Words],
    fixed: dict[bytes, str],
) -> tuple[Decision, ...]:
    """Build every exact strong-LL(2) source decision with provenance."""
    names = {production.name for production in productions}
    output: list[Decision] = []

    def walk(node: Node, outer: Words) -> None:
        if node.kind in {"ref", "fixed"}:
            return
        if node.kind == "sequence":
            for index, child in enumerate(node.children):
                continuation = suffix_words(node.children[index + 1 :], first, names, fixed, outer)
                walk(child, continuation)
            return
        if node.kind == "group":
            walk(node.children[0], outer)
            return
        if node.kind == "choice":
            arms = tuple(
                marked_words(prefix(child, first, names, fixed), outer) for child in node.children
            )
        elif node.kind == "optional":
            arms = (
                marked_words(prefix(node.children[0], first, names, fixed), outer),
                marked_words(EPSILON, outer),
            )
        elif node.kind in {"repeat0", "repeat1"}:
            body = prefix(node.children[0], first, names, fixed)
            repeated_outer = concat(repeat(body), outer)
            arms = (marked_words(body, repeated_outer), marked_words(EPSILON, outer))
        else:
            raise ContractError(f"unknown decision node kind {node.kind}")
        decision = Decision(
            node=node.index,
            production=node.production,
            kind=node.kind,
            context=decision_context(node),
            arm_count=len(arms),
            rows=rows_for_arms(arms),
        )
        validate_disjoint(decision)
        output.append(decision)
        if node.kind == "choice":
            for child in node.children:
                walk(child, outer)
        elif node.kind == "optional":
            walk(node.children[0], outer)
        else:
            body = prefix(node.children[0], first, names, fixed)
            walk(node.children[0], concat(repeat(body), outer))

    for production in productions:
        walk(production.root, follow[production.name])
    return tuple(output)


def build_grammar(specification: bytes, terminal_rust: str) -> Grammar:
    """Construct the complete immutable exact-v0.9 grammar contract."""
    digest = hashlib.sha256(specification).hexdigest()
    if digest != SPEC_SHA256:
        raise ContractError(f"v0.9 identity drifted to {digest}")
    fixed = fixed_variants(terminal_rust)
    productions = [
        Production(name, RULE_OWNER[name], EbnfParser(rhs).parse())
        for name, rhs in production_texts(specification)
    ]
    nodes = number_nodes(productions)
    names = {production.name for production in productions}
    nullable_map = {name: False for name in names}
    while True:
        changed = False
        for production in productions:
            value = nullable(production.root, nullable_map, names)
            if value and not nullable_map[production.name]:
                nullable_map[production.name] = True
                changed = True
        if not changed:
            break
    for node in nodes:
        if node.kind in {"repeat0", "repeat1"} and nullable(node.children[0], nullable_map, names):
            raise ContractError(f"nullable repetition body at {node.production}:{node.path}")
    first = first_sets(tuple(productions), fixed)
    follow = follow_sets(tuple(productions), first, fixed)
    decisions = collect_decisions(tuple(productions), first, follow, fixed)
    children: list[int] = []
    terminals: list[str] = []
    for node in nodes:
        children.extend(child.index for child in node.children)
        if node.kind == "fixed" or (node.kind == "ref" and node.value not in names):
            word = next(iter(node_terminal_words(node, fixed)))
            terminals.extend(atom.predicate for atom in word)
    order: list[str] = []
    seen: set[str] = set()
    for node in nodes:
        if node.kind == "fixed" or (node.kind == "ref" and node.value not in names):
            for atom in next(iter(node_terminal_words(node, fixed))):
                if atom.predicate not in seen:
                    seen.add(atom.predicate)
                    order.append(atom.predicate)
    if len(decisions) != 72:
        raise ContractError(f"expected 72 decisions, got {len(decisions)}")
    projection = {
        (decision.production, nodes[decision.node].path, decision.kind, row.arm,
         row.first.predicate, row.second.predicate)
        for decision in decisions
        for row in decision.rows
    }
    if len(projection) != 1253:
        raise ContractError(f"expected 1253 projected SELECT rows, got {len(projection)}")
    projection_bytes = "".join(
        "\t".join(
            (
                PRODUCTION_NAMES[production],
                path,
                kind,
                str(arm),
                first,
                second,
            )
        )
        + "\n"
        for production, path, kind, arm, first, second in sorted(projection)
    ).encode("ascii")
    projection_digest = hashlib.sha256(projection_bytes).hexdigest()
    if projection_digest != SELECT2_PROJECTION_SHA256:
        raise ContractError(
            f"SELECT2 projection identity drifted to {projection_digest}"
        )
    return Grammar(
        tuple(productions), nodes, tuple(children), tuple(terminals), fixed, decisions, tuple(order)
    )


def atom_rust(atom: SelectAtom, fixed: dict[bytes, str]) -> str:
    """Render one SELECT position."""
    node = "None" if atom.node is None else f"Some(GrammarNodeIdV0_9::new({atom.node}))"
    name = (
        "None"
        if atom.transparent_name is None
        else f"Some(NamePredicateV0_9::{atom.transparent_name})"
    )
    return (
        "SelectAtomV0_9::new("
        f"{terminal_descriptor(atom.predicate, fixed)}, {node}, "
        f"{str(atom.inside_arm).lower()}, {name}, {str(atom.atom_only).lower()})"
    )


def render(grammar: Grammar) -> str:
    """Render the checked grammar into dependency-free immutable Rust data."""
    names = {production.name for production in grammar.productions}
    child_cursor = 0
    terminal_cursor = 0
    decision_by_node = {decision.node: index for index, decision in enumerate(grammar.decisions)}
    lines = [
        "// @generated by tools/generate_syntax_contract.py; do not edit by hand.",
        "use crate::{",
        "    DecisionContextV0_9, DecisionKindV0_9, DecisionV0_9, GrammarNodeIdV0_9, GrammarNodeKindV0_9,",
        "    GrammarNodeV0_9, LookaheadPredicateV0_9, NamePredicateV0_9, RuleOwnerV0_9, SelectAtomV0_9,",
        "    SelectRowV0_9,",
        "};",
        "use whitefoot_language_data::{FixedTerminalV0_9, TerminalPredicateV0_9};",
        "",
        "/// One normative production in specification-definition order.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub enum ProductionV0_9 {",
    ]
    for production in grammar.productions:
        lines.append(f"    /// The `{production.name}` production.")
        lines.append(f"    {camel(production.name)},")
    lines.extend(["}", "", "impl ProductionV0_9 {", "    pub(crate) const fn index(self) -> usize {", "        self as usize", "    }", "}", ""])
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const PRODUCTIONS: [ProductionV0_9; {len(grammar.productions)}] = [")
    lines.extend(f"    ProductionV0_9::{camel(item.name)}," for item in grammar.productions)
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const PRODUCTION_ROOTS: [GrammarNodeIdV0_9; {len(grammar.productions)}] = [")
    lines.extend(f"    GrammarNodeIdV0_9::new({item.root.index})," for item in grammar.productions)
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const PRODUCTION_OWNERS: [RuleOwnerV0_9; {len(grammar.productions)}] = [")
    lines.extend(f"    RuleOwnerV0_9::{item.owner}," for item in grammar.productions)
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const GRAMMAR_CHILDREN: [GrammarNodeIdV0_9; {len(grammar.children)}] = [")
    for child in grammar.children:
        lines.append(f"    GrammarNodeIdV0_9::new({child}),")
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const GRAMMAR_TERMINALS: [LookaheadPredicateV0_9; {len(grammar.terminals)}] = [")
    for terminal in grammar.terminals:
        lines.append(f"    {terminal_descriptor(terminal, grammar.fixed_variants)},")
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const GRAMMAR_NODES: [GrammarNodeV0_9; {len(grammar.nodes)}] = [")
    for node in grammar.nodes:
        decision = decision_by_node.get(node.index)
        decision_value = "None" if decision is None else f"Some({decision})"
        if node.kind == "ref" and node.value in names:
            kind = f"GrammarNodeKindV0_9::Production(ProductionV0_9::{camel(node.value or '')})"
            count = 0
            start = 0
        elif node.kind == "fixed" or (node.kind == "ref" and node.value not in names):
            kind = "GrammarNodeKindV0_9::TerminalSequence"
            word = next(iter(node_terminal_words(node, grammar.fixed_variants)))
            start = terminal_cursor
            count = len(word)
            terminal_cursor += count
        else:
            kind = {
                "sequence": "GrammarNodeKindV0_9::Sequence",
                "choice": "GrammarNodeKindV0_9::Choice",
                "group": "GrammarNodeKindV0_9::Group",
                "optional": "GrammarNodeKindV0_9::Optional",
                "repeat0": "GrammarNodeKindV0_9::RepeatZero",
                "repeat1": "GrammarNodeKindV0_9::RepeatOne",
            }[node.kind]
            start = child_cursor
            count = len(node.children)
            child_cursor += count
        atom_only = is_atom_only_reference(node)
        lines.append(
            "    GrammarNodeV0_9::new("
            f"{kind}, {start}, {count}, {decision_value}, {str(atom_only).lower()}),"
        )
    lines.append("];")
    lines.append("")
    row_cursor = 0
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const DECISIONS: [DecisionV0_9; {len(grammar.decisions)}] = [")
    for decision in grammar.decisions:
        lines.append(
            "    DecisionV0_9::new("
            f"GrammarNodeIdV0_9::new({decision.node}), "
            f"ProductionV0_9::{camel(PRODUCTION_NAMES[decision.production])}, "
            f"DecisionKindV0_9::{camel(decision.kind)}, "
            f"DecisionContextV0_9::{decision.context}, {decision.arm_count}, "
            f"{row_cursor}, {len(decision.rows)}),"
        )
        row_cursor += len(decision.rows)
    lines.append("];")
    lines.append("")
    all_rows = [row for decision in grammar.decisions for row in decision.rows]
    atoms: list[SelectAtom] = []
    atom_indices: dict[SelectAtom, int] = {}
    for row in all_rows:
        for atom in (row.first, row.second):
            if atom not in atom_indices:
                atom_indices[atom] = len(atoms)
                atoms.append(atom)
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const SELECT_ATOMS: [SelectAtomV0_9; {len(atoms)}] = [")
    for atom in atoms:
        lines.append(f"    {atom_rust(atom, grammar.fixed_variants)},")
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub(crate) const SELECT_ROWS: [SelectRowV0_9; {len(all_rows)}] = [")
    for row in all_rows:
        lines.append(
            f"    SelectRowV0_9::new({row.arm}, {atom_indices[row.first]}, "
            f"{atom_indices[row.second]}),"
        )
    lines.append("];")
    lines.append("")
    lines.append("#[rustfmt::skip]")
    lines.append(
        f"pub(crate) const DIAGNOSTIC_ORDER: [LookaheadPredicateV0_9; {len(grammar.diagnostic_order)}] = ["
    )
    lines.extend(
        f"    {terminal_descriptor(predicate, grammar.fixed_variants)},"
        for predicate in grammar.diagnostic_order
    )
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def expected_generated() -> str:
    """Return the exact generated Rust file for the live bound inputs."""
    grammar = build_grammar(SPEC.read_bytes(), TERMINALS.read_text(encoding="utf-8"))
    return render(grammar)
