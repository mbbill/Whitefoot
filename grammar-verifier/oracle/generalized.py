"""Independent token lattice and bounded generalized derivation counter."""

from __future__ import annotations

from collections import defaultdict, deque
from dataclasses import dataclass
import struct
from typing import DefaultDict

from core import Limits, fail
from ebnf import Node
from extract import GrammarDocument


@dataclass(frozen=True)
class BnfRule:
    identifier: int
    lhs: str
    rhs: tuple[str, ...]
    event: str | None


@dataclass(frozen=True)
class CompiledGrammar:
    rules: tuple[BnfRule, ...]
    terminal_symbols: dict[str, str]
    starts: dict[str, str]


@dataclass(frozen=True)
class TokenSlot:
    start: int
    end: int
    labels: tuple[str, ...]


@dataclass(frozen=True)
class Tokenization:
    slots: tuple[TokenSlot, ...]
    complete: bool


@dataclass(frozen=True)
class Proof:
    identifier: int
    event: str | None
    children: tuple[int, ...]
    forest: tuple[int, ...]


@dataclass(frozen=True)
class SourceNode:
    identifier: int
    label: str
    children: tuple[int, ...]


@dataclass(frozen=True)
class ParseResult:
    classification: str
    traces: tuple[bytes, ...]
    source_tokens: int
    chart_items: int
    packed_edges: int
    proof_nodes: int


@dataclass
class OracleMetrics:
    parsed_streams: int = 0
    source_tokens: int = 0
    chart_items: int = 0
    packed_edges: int = 0
    proof_nodes: int = 0

    def include(self, result: ParseResult) -> None:
        self.parsed_streams += 1
        self.source_tokens += result.source_tokens
        self.chart_items += result.chart_items
        self.packed_edges += result.packed_edges
        self.proof_nodes += result.proof_nodes


def _hex_text(text: str) -> str:
    return text.encode("ascii").hex()


def _node_event(lhs: str, path: str, kind: str, variant: str = "-") -> str:
    return f"node:{_hex_text(lhs)}:{path}:{kind}:{variant}"


class _Compiler:
    def __init__(self, grammar: GrammarDocument) -> None:
        self.grammar = grammar
        self.rules: list[BnfRule] = []
        self.terminals: dict[str, str] = {}

    def compile(self) -> CompiledGrammar:
        starts = {name: f"production:{_hex_text(name)}" for name in self.grammar.production_by_name}
        for production in self.grammar.productions:
            root_symbol = self._compile_node(production.lhs, "0", production.root)
            self._add(
                starts[production.lhs],
                (root_symbol,),
                f"production:{_hex_text(production.lhs)}",
            )
        return CompiledGrammar(tuple(self.rules), self.terminals, starts)

    def _add(self, lhs: str, rhs: tuple[str, ...], event: str | None) -> None:
        if len(rhs) > 2:
            fail("internal", "non_binary_rule")
        self.rules.append(BnfRule(len(self.rules), lhs, rhs, event))

    def _terminal(self, label: str) -> str:
        symbol = f"terminal:{label}"
        existing = self.terminals.get(symbol)
        if existing is not None and existing != label:
            fail("internal", "terminal_collision")
        self.terminals[symbol] = label
        return symbol

    def _sequence(
        self,
        lhs: str,
        members: tuple[str, ...],
        event: str | None,
        helper_prefix: str,
    ) -> None:
        if not members:
            self._add(lhs, (), event)
            return
        if len(members) <= 2:
            self._add(lhs, members, event)
            return
        next_symbol = f"helper:{helper_prefix}:1"
        self._add(lhs, (members[0], next_symbol), event)
        for index in range(1, len(members) - 2):
            following = f"helper:{helper_prefix}:{index + 1}"
            self._add(next_symbol, (members[index], following), None)
            next_symbol = following
        self._add(next_symbol, members[-2:], None)

    def _compile_node(self, lhs: str, path: str, node: Node) -> str:
        symbol = f"node:{_hex_text(lhs)}:{path}"
        child_symbols = tuple(
            self._compile_node(lhs, f"{path}.{index}", child)
            for index, child in enumerate(node.children)
        )
        if node.kind == "ref":
            if node.value is None:
                fail("internal", "reference_value")
            if node.value in self.grammar.production_by_name:
                target = f"production:{_hex_text(node.value)}"
            else:
                label = f"lexical:{_hex_text(node.value)}"
                target = self._terminal(label)
            self._add(symbol, (target,), _node_event(lhs, path, "ref", _hex_text(node.value)))
        elif node.kind == "fixed":
            occurrence = next(
                item
                for item in self.grammar.fixed
                if item.lhs == lhs and item.path == path
            )
            members = tuple(
                self._terminal(f"fixed:{atom.spelling.hex()}")
                for atom in occurrence.expansion
            )
            self._sequence(
                symbol,
                members,
                _node_event(lhs, path, "fixed", occurrence.spelling.hex()),
                f"{_hex_text(lhs)}:{path}:fixed",
            )
        elif node.kind == "pattern":
            label = "pattern:5b302d395d2b"
            self._add(
                symbol,
                (self._terminal(label),),
                _node_event(lhs, path, "pattern", "5b302d395d2b"),
            )
        elif node.kind == "sequence":
            self._sequence(
                symbol,
                child_symbols,
                _node_event(lhs, path, "sequence"),
                f"{_hex_text(lhs)}:{path}:sequence",
            )
        elif node.kind == "choice":
            for index, child in enumerate(child_symbols):
                self._add(
                    symbol,
                    (child,),
                    _node_event(lhs, path, "choice", str(index)),
                )
        elif node.kind == "group":
            self._add(symbol, child_symbols, _node_event(lhs, path, "group"))
        elif node.kind == "optional":
            self._add(symbol, (), _node_event(lhs, path, "optional", "empty"))
            self._add(symbol, child_symbols, _node_event(lhs, path, "optional", "present"))
        elif node.kind == "repeat0":
            self._add(symbol, (), _node_event(lhs, path, "repeat0", "empty"))
            self._add(
                symbol,
                (child_symbols[0], symbol),
                _node_event(lhs, path, "repeat0", "more"),
            )
        elif node.kind == "repeat1":
            self._add(
                symbol,
                child_symbols,
                _node_event(lhs, path, "repeat1", "one"),
            )
            self._add(
                symbol,
                (child_symbols[0], symbol),
                _node_event(lhs, path, "repeat1", "more"),
            )
        else:
            fail("internal", "compile_node_kind")
        return symbol


def compile_grammar(grammar: GrammarDocument) -> CompiledGrammar:
    return _Compiler(grammar).compile()


def tokenize_source(
    grammar: GrammarDocument,
    source: bytes,
    limits: Limits,
) -> Tokenization:
    """Build a maximal-munch slot stream while retaining category alternatives."""

    fixed_lowerwords = frozenset(grammar.expanded_lowerwords)
    cursor = 0
    slots: list[TokenSlot] = []
    while cursor < len(source):
        while cursor < len(source) and source[cursor] in (0x0A, 0x20):
            cursor += 1
        if cursor == len(source):
            break
        candidates: list[tuple[int, str]] = []
        for spelling in grammar.fixed_token_spellings:
            if source.startswith(spelling, cursor):
                candidates.append((cursor + len(spelling), f"fixed:{spelling.hex()}"))
        if grammar.has_number_pattern and 0x30 <= source[cursor] <= 0x39:
            end = cursor + 1
            while end < len(source) and 0x30 <= source[end] <= 0x39:
                end += 1
            candidates.append((end, "pattern:5b302d395d2b"))
        for definition in grammar.lexical:
            end = definition.match_prefix(source, cursor, fixed_lowerwords)
            if end is not None:
                candidates.append((end, f"lexical:{_hex_text(definition.name)}"))
        if not candidates:
            return Tokenization(tuple(slots), False)
        longest = max(end for end, _label in candidates)
        labels = tuple(sorted({label for end, label in candidates if end == longest}))
        limits.require("oracle_max_source_tokens", len(slots) + 1)
        slots.append(TokenSlot(cursor, longest, labels))
        cursor = longest
    return Tokenization(tuple(slots), True)


class _Chart:
    def __init__(
        self,
        compiled: CompiledGrammar,
        tokenization: Tokenization,
        limits: Limits,
    ) -> None:
        self.compiled = compiled
        self.tokens = tokenization.slots
        self.limits = limits
        self.cells: dict[tuple[str, int, int], list[Proof]] = {}
        self.by_start: DefaultDict[tuple[str, int], list[tuple[int, Proof]]] = defaultdict(list)
        self.by_end: DefaultDict[tuple[str, int], list[tuple[int, Proof]]] = defaultdict(list)
        self.agenda: deque[tuple[str, int, int, Proof]] = deque()
        self.proofs: list[Proof] = []
        self.proof_intern: dict[
            tuple[str, int, int, str | None, tuple[int, ...]], Proof
        ] = {}
        self.source_nodes: list[SourceNode] = []
        self.source_intern: dict[tuple[str, tuple[int, ...]], int] = {}
        self.chart_items = 0
        self.packed_edges = 0

        self.unary: DefaultDict[str, list[BnfRule]] = defaultdict(list)
        self.binary_left: DefaultDict[str, list[BnfRule]] = defaultdict(list)
        self.binary_right: DefaultDict[str, list[BnfRule]] = defaultdict(list)
        self.epsilon: list[BnfRule] = []
        for rule in compiled.rules:
            if len(rule.rhs) == 0:
                self.epsilon.append(rule)
            elif len(rule.rhs) == 1:
                self.unary[rule.rhs[0]].append(rule)
            else:
                self.binary_left[rule.rhs[0]].append(rule)
                self.binary_right[rule.rhs[1]].append(rule)

    def run(self, start_symbol: str) -> tuple[bytes, ...]:
        boundaries = range(len(self.tokens) + 1)
        for position in boundaries:
            for rule in self.epsilon:
                self._add(rule.lhs, position, position, rule.event, ())
        symbols_by_label = {label: symbol for symbol, label in self.compiled.terminal_symbols.items()}
        for index, slot in enumerate(self.tokens):
            for label in slot.labels:
                symbol = symbols_by_label.get(label)
                if symbol is None:
                    continue
                event = f"token:{label}:{slot.start}:{slot.end}"
                self._add(symbol, index, index + 1, event, ())
        while self.agenda:
            symbol, start, end, proof = self.agenda.popleft()
            for rule in self.unary.get(symbol, ()):
                self._add(rule.lhs, start, end, rule.event, (proof,))
            for rule in self.binary_left.get(symbol, ()):
                right_symbol = rule.rhs[1]
                for right_end, right in tuple(self.by_start.get((right_symbol, end), ())):
                    self._add(rule.lhs, start, right_end, rule.event, (proof, right))
            for rule in self.binary_right.get(symbol, ()):
                left_symbol = rule.rhs[0]
                for left_start, left in tuple(self.by_end.get((left_symbol, start), ())):
                    self._add(rule.lhs, left_start, end, rule.event, (left, proof))
        roots = self.cells.get((start_symbol, 0, len(self.tokens)), [])
        sizes = self._expanded_tree_sizes()
        traces: list[bytes] = []
        for proof in roots:
            if len(proof.forest) != 1:
                fail("internal", "root_tree_shape")
            traces.append(self._encode_tree(proof.forest[0], sizes))
        return tuple(sorted(traces))

    def _expanded_tree_sizes(self) -> tuple[int, ...]:
        limit = self.limits.get("max_engine_output_bytes")
        sizes: list[int] = []
        for node in self.source_nodes:
            if node.identifier != len(sizes):
                fail("internal", "source_node_order")
            size = 9 + len(node.label.encode("ascii"))
            for child in node.children:
                if child >= node.identifier:
                    fail("internal", "source_tree_cycle")
                size += 8 + sizes[child]
                if size > limit:
                    size = limit + 1
                    break
            sizes.append(size)
        return tuple(sizes)

    def _encode_tree(self, root: int, sizes: tuple[int, ...]) -> bytes:
        if root >= len(self.source_nodes):
            fail("internal", "source_tree_root")
        self.limits.require("max_engine_output_bytes", sizes[root])
        output = bytearray()
        actions: list[tuple[bool, int]] = [(False, root)]
        while actions:
            is_length, value = actions.pop()
            if is_length:
                output.extend(struct.pack(">Q", value))
                continue
            node = self.source_nodes[value]
            label = node.label.encode("ascii")
            output.extend(b"T")
            output.extend(struct.pack(">I", len(label)))
            output.extend(label)
            output.extend(struct.pack(">I", len(node.children)))
            for child in reversed(node.children):
                actions.append((False, child))
                actions.append((True, sizes[child]))
        if len(output) != sizes[root]:
            fail("internal", "source_tree_size")
        return bytes(output)

    def _add(
        self,
        symbol: str,
        start: int,
        end: int,
        event: str | None,
        children: tuple[Proof, ...],
    ) -> None:
        child_forest = tuple(item for child in children for item in child.forest)
        source_key = None if event is None else (event, child_forest)
        source_identifier = (
            None if source_key is None else self.source_intern.get(source_key)
        )
        forest = (
            child_forest
            if event is None
            else None if source_identifier is None else (source_identifier,)
        )
        key = (symbol, start, end)
        cell = self.cells.get(key)
        if cell is not None:
            if (
                forest is not None
                and any(existing.forest == forest for existing in cell)
            ) or len(cell) >= 2:
                return
        else:
            self.limits.require("oracle_max_chart_items", self.chart_items + 1)
        self.limits.require("oracle_max_proof_nodes", len(self.proofs) + 1)
        self.limits.require("oracle_max_packed_edges", self.packed_edges + 1)

        if source_key is not None and source_identifier is None:
            source_identifier = len(self.source_nodes)
            self.source_nodes.append(
                SourceNode(source_identifier, source_key[0], source_key[1])
            )
            self.source_intern[source_key] = source_identifier
            forest = (source_identifier,)
        if forest is None:
            fail("internal", "source_forest")

        proof_key = (symbol, start, end, event, tuple(child.identifier for child in children))
        proof = self.proof_intern.get(proof_key)
        if proof is not None:
            fail("internal", "proof_intern_mismatch")
        proof = Proof(len(self.proofs), event, proof_key[4], forest)
        self.proofs.append(proof)
        self.proof_intern[proof_key] = proof
        if cell is None:
            cell = []
            self.cells[key] = cell
            self.chart_items += 1
        cell.append(proof)
        self.packed_edges += 1
        self.by_start[(symbol, start)].append((end, proof))
        self.by_end[(symbol, end)].append((start, proof))
        self.agenda.append((symbol, start, end, proof))


def parse_source(
    grammar: GrammarDocument,
    compiled: CompiledGrammar,
    start: str,
    source: bytes,
    limits: Limits,
) -> ParseResult:
    start_symbol = compiled.starts.get(start)
    if start_symbol is None:
        fail("extraction", "unknown_start_nonterminal")
    tokenization = tokenize_source(grammar, source, limits)
    if not tokenization.complete:
        return ParseResult("zero", (), len(tokenization.slots), 0, 0, 0)
    chart = _Chart(compiled, tokenization, limits)
    traces = chart.run(start_symbol)
    classification = ("zero", "one", "many")[min(len(traces), 2)]
    return ParseResult(
        classification,
        traces[:2],
        len(tokenization.slots),
        chart.chart_items,
        chart.packed_edges,
        len(chart.proofs),
    )
