from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct
import sys
from typing import Iterator

from form2_inputs import MAX_SOURCE_BYTES, _read_bounded, sha256


GRAMMAR_ROOT = Path(__file__).resolve().parent.parent
ORACLE_ROOT = GRAMMAR_ROOT / "oracle"
if str(ORACLE_ROOT) not in sys.path:
    sys.path.insert(0, str(ORACLE_ROOT))

from core import Failure, Limits  # noqa: E402
from extract import GrammarDocument, extract_document  # noqa: E402
from generalized import (  # noqa: E402
    CompiledGrammar,
    compile_grammar,
    parse_source,
    tokenize_source,
)
from ingress import parse_limits  # noqa: E402


CANDIDATE_PATH = "grammar-verifier/proposal/kernel-spec-successor-candidate.md"
LIMITS_PATH = "grammar-verifier/limits.txt"
MAX_TRACE_BYTES = 8_388_608
MAX_TRACE_DEPTH = 512
MAX_TRACE_NODES = 1_000_000


class TreeError(RuntimeError):
    pass


@dataclass(frozen=True)
class TraceNode:
    label: str
    children: tuple["TraceNode", ...]


@dataclass(frozen=True)
class Terminal:
    index: int
    start: int
    end: int
    label: str
    raw: bytes


@dataclass(frozen=True)
class ProductionNode:
    name: str
    first_token: int
    last_token: int
    direct_tokens: tuple[int, ...]
    children: tuple["ProductionNode", ...]

    def walk(self) -> Iterator["ProductionNode"]:
        yield self
        for child in self.children:
            yield from child.walk()


@dataclass(frozen=True)
class Derivation:
    source_length: int
    items: tuple[ProductionNode, ...]
    terminals: tuple[Terminal, ...]
    source_trace_forest_projection_sha256: str
    source_forest_projection_sha256: str
    terminal_projection_sha256: str


@dataclass(frozen=True)
class ParseAttempt:
    classification: str
    stage: str
    derivation: Derivation | None
    source_token_count: int
    chart_item_count: int
    packed_edge_count: int
    proof_node_count: int
    failure: str | None = None


@dataclass(frozen=True)
class CandidateParser:
    source: bytes
    source_sha256: str
    limits_raw: bytes
    limits_sha256: str
    limits: Limits
    grammar: GrammarDocument
    compiled: CompiledGrammar


def _structural_limits(raw: bytes) -> Limits:
    parsed = parse_limits(raw)
    values = dict(parsed.values)
    values.update(
        oracle_max_source_tokens=4_096,
        oracle_max_chart_items=4_000_000,
        oracle_max_packed_edges=4_000_000,
        oracle_max_proof_nodes=4_000_000,
    )
    return Limits(values)


def build_candidate_parser(source: bytes, limits_raw: bytes) -> CandidateParser:
    limits = _structural_limits(limits_raw)
    try:
        grammar = extract_document("successor-candidate", source, limits)
        compiled = compile_grammar(grammar)
    except Failure as error:
        raise TreeError(
            f"cannot extract successor candidate: {error.family}:{error.code}"
        ) from error
    return CandidateParser(
        source=source,
        source_sha256=sha256(source),
        limits_raw=limits_raw,
        limits_sha256=sha256(limits_raw),
        limits=limits,
        grammar=grammar,
        compiled=compiled,
    )


def load_candidate_parser() -> CandidateParser:
    source = _read_bounded(CANDIDATE_PATH, MAX_SOURCE_BYTES)
    limits_raw = _read_bounded(LIMITS_PATH, 16_384)
    return build_candidate_parser(source, limits_raw)


def decode_trace(raw: bytes) -> TraceNode:
    if len(raw) > MAX_TRACE_BYTES:
        raise TreeError("derivation trace exceeds its byte limit")
    node_count = 0

    def decode(offset: int, limit: int, depth: int) -> tuple[TraceNode, int]:
        nonlocal node_count
        if depth > MAX_TRACE_DEPTH:
            raise TreeError("derivation trace exceeds its depth limit")
        node_count += 1
        if node_count > MAX_TRACE_NODES:
            raise TreeError("derivation trace exceeds its node limit")
        if offset >= limit or raw[offset : offset + 1] != b"T":
            raise TreeError("derivation trace has an invalid node marker")
        offset += 1
        if offset + 4 > limit:
            raise TreeError("derivation trace truncates a label length")
        label_length = struct.unpack(">I", raw[offset : offset + 4])[0]
        offset += 4
        label_end = offset + label_length
        if label_end + 4 > limit:
            raise TreeError("derivation trace truncates a node header")
        try:
            label = raw[offset:label_end].decode("ascii")
        except UnicodeDecodeError as error:
            raise TreeError("derivation trace label is not ASCII") from error
        offset = label_end
        child_count = struct.unpack(">I", raw[offset : offset + 4])[0]
        offset += 4
        children: list[TraceNode] = []
        for _ in range(child_count):
            if offset + 8 > limit:
                raise TreeError("derivation trace truncates a child length")
            child_length = struct.unpack(">Q", raw[offset : offset + 8])[0]
            offset += 8
            child_end = offset + child_length
            if child_end > limit:
                raise TreeError("derivation trace child exceeds its parent")
            child, consumed = decode(offset, child_end, depth + 1)
            if consumed != child_end:
                raise TreeError("derivation trace child has trailing bytes")
            children.append(child)
            offset = child_end
        return TraceNode(label, tuple(children)), offset

    node, consumed = decode(0, len(raw), 0)
    if consumed != len(raw):
        raise TreeError("derivation trace has trailing bytes")
    return node


def _production_name(label: str) -> str | None:
    if not label.startswith("production:"):
        return None
    encoded = label.removeprefix("production:")
    try:
        raw = bytes.fromhex(encoded)
        name = raw.decode("ascii")
    except (ValueError, UnicodeDecodeError) as error:
        raise TreeError("derivation trace has a malformed production label") from error
    if not name or not all(character.islower() or character == "_" for character in name):
        raise TreeError("derivation trace has an invalid production name")
    return name


def _terminal_label(label: str) -> tuple[str, int, int] | None:
    if not label.startswith("token:"):
        return None
    body, separator, encoded_end = label.rpartition(":")
    if not separator:
        raise TreeError("derivation trace has a malformed token end")
    body, separator, encoded_start = body.rpartition(":")
    if not separator:
        raise TreeError("derivation trace has a malformed token start")
    try:
        start = int(encoded_start)
        end = int(encoded_end)
    except ValueError as error:
        raise TreeError("derivation trace has a non-decimal token span") from error
    return body.removeprefix("token:"), start, end


def _trace_projection(node: TraceNode) -> bytes:
    output = bytearray()
    stack = [node]
    while stack:
        current = stack.pop()
        terminal = _terminal_label(current.label)
        label = current.label if terminal is None else f"token:{terminal[0]}"
        encoded = label.encode("ascii")
        output.extend(struct.pack(">I", len(encoded)))
        output.extend(encoded)
        output.extend(struct.pack(">I", len(current.children)))
        stack.extend(reversed(current.children))
    return bytes(output)


def _terminal_projection(terminals: tuple[Terminal, ...]) -> bytes:
    output = bytearray()
    for terminal in terminals:
        output.extend(struct.pack(">Q", len(terminal.raw)))
        output.extend(terminal.raw)
    return bytes(output)


def _production_projection(node: ProductionNode) -> bytes:
    output = bytearray()

    def encode(current: ProductionNode) -> None:
        name = current.name.encode("ascii")
        output.extend(struct.pack(">I", len(name)))
        output.extend(name)
        output.extend(struct.pack(">QQ", current.first_token, current.last_token))
        output.extend(struct.pack(">I", len(current.direct_tokens)))
        for index in current.direct_tokens:
            output.extend(struct.pack(">Q", index))
        output.extend(struct.pack(">I", len(current.children)))
        for child in current.children:
            encode(child)

    encode(node)
    return bytes(output)


def source_forest_projection(items: tuple[ProductionNode, ...]) -> bytes:
    output = bytearray(struct.pack(">I", len(items)))
    for item in items:
        projection = _production_projection(item)
        output.extend(struct.pack(">Q", len(projection)))
        output.extend(projection)
    return bytes(output)


def _trace_item_forests(root: TraceNode) -> tuple[TraceNode, ...]:
    if _production_name(root.label) != "program":
        raise TreeError("derivation trace wrapper is not program")
    items: list[TraceNode] = []

    def descend(node: TraceNode) -> None:
        name = _production_name(node.label)
        if name == "item":
            items.append(node)
            return
        if name not in (None, "program"):
            raise TreeError("program trace reaches a production before item")
        for child in node.children:
            descend(child)

    for child in root.children:
        descend(child)
    return tuple(items)


def _trace_forest_projection(root: TraceNode) -> bytes:
    items = _trace_item_forests(root)
    output = bytearray(struct.pack(">I", len(items)))
    for item in items:
        projection = _trace_projection(item)
        output.extend(struct.pack(">Q", len(projection)))
        output.extend(projection)
    if b"program" in output or b"70726f6772616d" in output:
        raise TreeError("source trace forest retains the parser program wrapper")
    return bytes(output)


def _simplify_trace(root: TraceNode, source: bytes) -> Derivation:
    terminal_indices: dict[int, int] = {}
    terminals: list[Terminal] = []

    def collect(node: TraceNode) -> None:
        terminal = _terminal_label(node.label)
        if terminal is not None:
            label, start, end = terminal
            if node.children:
                raise TreeError("a terminal trace node has children")
            if start < 0 or end <= start or end > len(source):
                raise TreeError("a terminal span is outside its source")
            index = len(terminals)
            terminal_indices[id(node)] = index
            terminals.append(Terminal(index, start, end, label, source[start:end]))
            return
        for child in node.children:
            collect(child)

    collect(root)
    for left, right in zip(terminals, terminals[1:]):
        if left.end > right.start:
            raise TreeError("derivation terminal spans overlap or are out of order")

    def production(node: TraceNode) -> ProductionNode:
        name = _production_name(node.label)
        if name is None:
            raise TreeError("expected a production trace node")
        direct: list[int] = []
        children: list[ProductionNode] = []

        def descend(candidate: TraceNode) -> None:
            candidate_name = _production_name(candidate.label)
            if candidate_name is not None:
                children.append(production(candidate))
                return
            token_index = terminal_indices.get(id(candidate))
            if token_index is not None:
                direct.append(token_index)
                return
            for child in candidate.children:
                descend(child)

        for child in node.children:
            descend(child)
        covered = sorted(
            direct
            + [
                index
                for child in children
                for index in range(child.first_token, child.last_token + 1)
            ]
        )
        if covered:
            first, last = covered[0], covered[-1]
            if covered != list(range(first, last + 1)):
                raise TreeError(f"production {name} does not cover a contiguous token range")
        else:
            first = last = len(terminals)
        return ProductionNode(name, first, last, tuple(direct), tuple(children))

    wrapper = production(root)
    if wrapper.name != "program":
        raise TreeError("derivation root is not program")
    if wrapper.direct_tokens or any(child.name != "item" for child in wrapper.children):
        raise TreeError("program parser wrapper is not an item-only forest")
    terminal_tuple = tuple(terminals)
    items = wrapper.children
    covered = [
        index
        for item in items
        for index in range(item.first_token, item.last_token + 1)
    ]
    if covered != list(range(len(terminals))):
        raise TreeError("source item forest does not cover every terminal exactly once")
    forest_projection = source_forest_projection(items)
    if b"program" in forest_projection:
        raise TreeError("source forest projection retains a program node")
    return Derivation(
        source_length=len(source),
        items=items,
        terminals=terminal_tuple,
        source_trace_forest_projection_sha256=sha256(
            _trace_forest_projection(root)
        ),
        source_forest_projection_sha256=sha256(forest_projection),
        terminal_projection_sha256=sha256(_terminal_projection(terminal_tuple)),
    )


def parse_one(parser: CandidateParser, source: bytes) -> ParseAttempt:
    try:
        result = parse_source(
            parser.grammar,
            parser.compiled,
            "program",
            source,
            parser.limits,
        )
    except Failure as error:
        return ParseAttempt(
            "resource-failure",
            "resource-failure",
            None,
            0,
            0,
            0,
            0,
            f"{error.family}:{error.code}",
        )
    derivation = None
    if result.classification == "one":
        if len(result.traces) != 1:
            raise TreeError("unique parse did not retain exactly one trace")
        derivation = _simplify_trace(decode_trace(result.traces[0]), source)
        derivation = Derivation(
            source_length=derivation.source_length,
            items=derivation.items,
            terminals=derivation.terminals,
            source_trace_forest_projection_sha256=(
                derivation.source_trace_forest_projection_sha256
            ),
            source_forest_projection_sha256=(
                derivation.source_forest_projection_sha256
            ),
            terminal_projection_sha256=derivation.terminal_projection_sha256,
        )
    if result.classification == "one":
        stage = "unique-derivation"
    elif result.classification == "many":
        stage = "ambiguous-derivation"
    else:
        tokenization = tokenize_source(parser.grammar, source, parser.limits)
        stage = (
            "grammar-no-derivation"
            if tokenization.complete
            else "lexical-no-token-stream"
        )
    return ParseAttempt(
        classification=result.classification,
        stage=stage,
        derivation=derivation,
        source_token_count=result.source_tokens,
        chart_item_count=result.chart_items,
        packed_edge_count=result.packed_edges,
        proof_node_count=result.proof_nodes,
    )
