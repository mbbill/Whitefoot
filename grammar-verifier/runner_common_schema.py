"""Closed schema recovery for the neutral source-coverage ledger."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping

from runner_common_wire import (
    CLOSED_TABLE_NAME,
    REFERENCE_NAME,
    canonical_hex,
    canonical_uint,
    decoded_bytes,
)
from runner_inputs import Inputs, fail


EXPANSION_KINDS = (
    b"lowerword",
    b"identifier",
    b"ampersand",
    b"thin-arrow",
    b"fat-arrow",
    b"punctuation",
)
REGEX_PATTERNS = {
    b"IDENT": b"[a-z][a-z0-9_]*",
    b"TYPEID": b"[A-Z][A-Za-z0-9]*",
    b"REGIONID": b"'[a-z][a-z0-9_]*",
    b"LABEL": b"@[a-z][a-z0-9_]*",
    b"OPNAME": b"[a-z][a-z0-9_]*\\.(wrap|trap|checked|sat|strict)",
}
IDENT_MODIFIER = (
    b" excluding every lowercase token spelling produced by exact fixed grammar atoms "
    b"in the complete grammar"
)
CURRENT_LITERAL_DESCRIPTOR = (
    b"integer=-?[0-9]+_TYPE;float=-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE;"
    b"unit=unit;generic=0_T,1_T"
)
PROPOSAL_LITERAL_DESCRIPTOR = (
    b"integer=-?[0-9]+_TYPE;"
    b"float=-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE;"
    b"float-value=signed-zero-or-sign*C*10^(E-F);e-0=0;round=ieee-rne;"
    b"canonical=min-prefix-bytes,ascii-lex;finite=required;"
    b"unit=unit;generic=0_T,1_T"
)
BYTE_STRING_DESCRIPTOR = (
    b"range=32-126;exclude=34,92;escapes=backslash,quote,n;contexts=doc,check"
)
CLOSED_TABLE_DESCRIPTOR = b"associative(f),commutative(f),identity(f,e)"


@dataclass(frozen=True)
class ProductionRecord:
    definition: tuple[int, int]
    rhs: tuple[int, int]


@dataclass(frozen=True)
class NodeRecord:
    kind: bytes
    value: bytes
    span: tuple[int, int]


@dataclass(frozen=True)
class LexicalRecord:
    kind: bytes
    span: tuple[int, int]
    predicate: bytes


@dataclass(frozen=True)
class FixedRecord:
    spelling: bytes
    expansion: tuple[bytes, ...]
    span: tuple[int, int]


@dataclass(frozen=True)
class ReferenceRecord:
    target: bytes
    span: tuple[int, int]


@dataclass(frozen=True)
class DocumentSchema:
    """Closed grammar identities recovered from the agreed common ledger."""

    productions: frozenset[bytes]
    nodes: Mapping[tuple[bytes, bytes], tuple[bytes, bytes]]
    children: Mapping[tuple[bytes, bytes], tuple[bytes, ...]]
    lexical_names: frozenset[bytes]
    fixed_expansions: Mapping[tuple[bytes, bytes], tuple[bytes, ...]]
    references: Mapping[tuple[bytes, bytes], bytes]
    production_spans: Mapping[bytes, tuple[tuple[int, int], tuple[int, int]]]
    node_spans: Mapping[tuple[bytes, bytes], tuple[int, int]]

    def terminal_predicates(self) -> frozenset[bytes]:
        result = {b"end"}
        if any(kind == b"pattern" for kind, _value in self.nodes.values()):
            result.add(b"pattern:digits")
        result.update(b"lex:" + name for name in self.lexical_names)
        result.update(
            b"fixed:" + spelling
            for expansion in self.fixed_expansions.values()
            for spelling in expansion
        )
        return frozenset(result)


def fixed_expansion(value: bytes, spelling: bytes) -> tuple[bytes, ...]:
    canonical_hex(value)
    canonical_hex(spelling)
    spelling_bytes = bytes.fromhex(spelling.decode("ascii"))
    decoded = bytes.fromhex(value.decode("ascii"))
    atoms = decoded.split(b",")
    result: list[bytes] = []
    described: list[tuple[bytes, bytes]] = []
    for atom in atoms:
        kind, separator, atom_hex = atom.partition(b":")
        if separator != b":" or kind not in EXPANSION_KINDS:
            fail("report_fixed", "a fixed expansion has an unknown atom kind")
        canonical_hex(atom_hex)
        atom_bytes = bytes.fromhex(atom_hex.decode("ascii"))
        described.append((kind, atom_bytes))
        result.append(atom_hex)

    def symbol_start(byte: int) -> bool:
        return chr(byte).isascii() and (chr(byte).isalpha() or byte == 0x5F)

    def symbol_continue(byte: int) -> bool:
        return chr(byte).isascii() and (chr(byte).isalnum() or byte == 0x5F)

    def shape(part: bytes) -> bytes:
        if (
            part
            and 0x61 <= part[0] <= 0x7A
            and all(0x61 <= byte <= 0x7A or 0x30 <= byte <= 0x39 or byte == 0x5F for byte in part)
        ):
            return b"lowerword"
        if part and symbol_start(part[0]) and all(symbol_continue(byte) for byte in part):
            return b"identifier"
        return {
            b"&": b"ampersand",
            b"->": b"thin-arrow",
            b"=>": b"fat-arrow",
        }.get(part, b"punctuation")

    split = next((index for index, byte in enumerate(spelling_bytes) if symbol_start(byte)), None)
    if (
        split is not None
        and split > 0
        and all(not symbol_continue(byte) for byte in spelling_bytes[:split])
        and all(symbol_continue(byte) for byte in spelling_bytes[split:])
    ):
        parts = (spelling_bytes[:split], spelling_bytes[split:])
    else:
        parts = (spelling_bytes,)
    expected = [(shape(part), part) for part in parts]
    if described != expected:
        fail("report_fixed", "a fixed expansion does not use the canonical atom partition and kinds")
    return tuple(result)


def _require_source_signature(
    source_slice: bytes,
    prefix: bytes,
    suffix: bytes,
    fragments: tuple[bytes, ...],
) -> None:
    if not source_slice.startswith(prefix) or not source_slice.endswith(suffix):
        fail("report_lexical", "a lexical span does not bind its closed source form")
    cursor = 0
    for fragment in fragments:
        found = source_slice.find(fragment, cursor)
        if found < 0:
            fail("report_lexical", "a lexical span omits a closed source-form fragment")
        cursor = found + len(fragment)


def _validate_lexical(
    name_hex: bytes,
    record: LexicalRecord,
    source: bytes,
) -> None:
    name = decoded_bytes(name_hex)
    predicate = decoded_bytes(record.predicate)
    start, end = record.span
    source_slice = source[start:end]
    if record.kind == b"regex":
        pattern = REGEX_PATTERNS.get(name)
        if pattern is None:
            fail("report_lexical", "a regex lexical name is outside the closed form")
        base = name + b" `" + pattern + b"`"
        base_end = start + len(base)
        source_has_modifier = source.startswith(IDENT_MODIFIER, base_end)
        if source.startswith(b" excluding ", base_end) and not source_has_modifier:
            fail("report_lexical", "an IDENT exclusion modifier is not the exact closed form")
        ordinary = b"pattern=" + pattern + b";exclude=none"
        excluded = b"pattern=" + pattern + b";exclude=fixed-lowerwords"
        if predicate == ordinary:
            modifier = b""
        elif name == b"IDENT" and predicate == excluded:
            modifier = IDENT_MODIFIER
        else:
            fail("report_lexical", "a regex lexical predicate is outside the closed form")
        if source_has_modifier != bool(modifier):
            fail("report_lexical", "an IDENT predicate omits or invents its source modifier")
        expected = base + modifier
        if source_slice != expected:
            fail("report_lexical", "a regex predicate does not match its exact source span")
        return
    if record.kind == b"literal-union":
        if name != b"literal":
            fail("report_lexical", "a literal lexical predicate is outside the closed form")
        if predicate == CURRENT_LITERAL_DESCRIPTOR:
            float_fragments = (
                b"floats `-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE`",
            )
        elif predicate == PROPOSAL_LITERAL_DESCRIPTOR:
            float_fragments = (
                b"finite floats use the grammar "
                b"`-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE`",
                b"TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64)",
                "magnitude is C × 10^(E − F)".encode("utf-8"),
                b"IEEE 754 round-to-nearest, ties-to-even",
                b"fewest ASCII bytes before `_TYPE`",
                b"lexicographically least unsigned ASCII bytes",
            )
        else:
            fail("report_lexical", "a literal lexical predicate is outside the closed form")
        _require_source_signature(
            source_slice,
            b"Literals, exhaustively:",
            b"[OP-1].",
            (
                b"integers `-?[0-9]+_TYPE`",
                *float_fragments,
                b"`unit`",
                b"STRING `\"...\"`",
                b"Generic-numeric literals `0_T` and `1_T`",
            ),
        )
        return
    if record.kind == b"byte-string":
        if name != b"STRING" or predicate != BYTE_STRING_DESCRIPTOR:
            fail("report_lexical", "a byte-string lexical predicate is outside the closed form")
        _require_source_signature(
            source_slice,
            b"STRING `\"...\"`",
            b"diagnostic text is DEFERRED.",
            (
                b"U+0020..U+007E",
                b"other than `\"` and `\\`",
                b"exactly three escapes `\\\\ \\\" \\n`",
                b"STRING appears only in `doc` and `check` messages",
            ),
        )
        return
    if (
        record.kind != b"closed-table"
        or not CLOSED_TABLE_NAME.fullmatch(name)
        or predicate != CLOSED_TABLE_DESCRIPTOR
    ):
        fail("report_lexical", "a closed-table lexical predicate is outside the closed form")
    expected = (
        name
        + b" is a closed table: `associative(f)`, `commutative(f)`, `identity(f, e)`."
    )
    if source_slice != expected:
        fail("report_lexical", "a closed-table predicate does not match its exact source span")


def _required_ebnf_depth(nodes: Mapping[tuple[bytes, bytes], NodeRecord]) -> int:
    """Recover the shared parser's minimum call-depth budget from node paths."""

    required = 0
    for (lhs, path), _record in nodes.items():
        parts = path.split(b".")
        prefixes = [b".".join(parts[: index + 1]) for index in range(len(parts))]
        ancestors = [nodes.get((lhs, prefix)) for prefix in prefixes]
        if any(ancestor is None for ancestor in ancestors):
            fail("report_schema", "a structural node path has an unrecorded prefix")
        group_depth = sum(
            ancestor.kind == b"group" for ancestor in ancestors if ancestor is not None
        )
        # Both extractors enter choice, sequence, postfix and atom at each
        # parenthesized grammar level. Even a flat atom therefore requires 4.
        required = max(required, len(parts), 4 * (group_depth + 1))
    return required


def _enforce_semantic_limits(
    inputs: Inputs,
    rule_count: int,
    productions: Mapping[bytes, ProductionRecord],
    nodes: Mapping[tuple[bytes, bytes], NodeRecord],
    lexical: Mapping[bytes, LexicalRecord],
    references: Mapping[tuple[bytes, bytes], ReferenceRecord],
) -> None:
    counts = (
        ("max_rules", rule_count),
        ("max_definitions", len(productions)),
        ("max_grammar_nodes", len(nodes)),
        ("max_lexical_definitions", len(lexical)),
        (
            "max_terminal_occurrences",
            sum(record.kind in (b"fixed", b"pattern") for record in nodes.values()),
        ),
        ("max_ebnf_depth", _required_ebnf_depth(nodes)),
    )
    for name, observed in counts:
        if observed > inputs.limits[name]:
            fail("report_resource", f"the common ledger exceeds {name}")
    encoded_symbols = set(productions) | set(lexical)
    encoded_symbols.update(record.target for record in references.values())
    for symbol in encoded_symbols:
        if len(decoded_bytes(symbol)) > inputs.limits["max_symbol_bytes"]:
            fail("report_resource", "the common ledger exceeds max_symbol_bytes")


def finish_schema(
    source: bytes,
    inputs: Inputs,
    rule_count: int,
    productions: dict[bytes, ProductionRecord],
    nodes: dict[tuple[bytes, bytes], NodeRecord],
    lexical: dict[bytes, LexicalRecord],
    fixed: dict[tuple[bytes, bytes], FixedRecord],
    references: dict[tuple[bytes, bytes], ReferenceRecord],
) -> DocumentSchema:
    lexical_names = set(lexical)
    if set(productions) & lexical_names:
        fail("report_schema", "production and lexical symbol namespaces overlap")
    _enforce_semantic_limits(inputs, rule_count, productions, nodes, lexical, references)
    roots = {lhs for (lhs, path) in nodes if path == b"0"}
    if roots != set(productions) or any(lhs not in productions for lhs, _path in nodes):
        fail("report_schema", "the common ledger does not contain one root node per production")
    for lhs, production in productions.items():
        if nodes[(lhs, b"0")].span != production.rhs:
            fail("report_span", "a production root span does not equal its RHS span")
    fixed_nodes = {key for key, record in nodes.items() if record.kind == b"fixed"}
    reference_nodes = {key for key, record in nodes.items() if record.kind == b"ref"}
    if fixed_nodes != set(fixed) or reference_nodes != set(references):
        fail("report_schema", "the common leaf ledgers do not equal the structural nodes")
    direct_by_parent: dict[tuple[bytes, bytes], list[bytes]] = {key: [] for key in nodes}
    for lhs, path in nodes:
        if path == b"0":
            continue
        parent_path, separator, _index = path.rpartition(b".")
        parent = (lhs, parent_path)
        if separator != b"." or parent not in direct_by_parent:
            fail("report_schema", "a structural node has no recorded parent")
        direct_by_parent[parent].append(path)
    children: dict[tuple[bytes, bytes], tuple[bytes, ...]] = {}
    for key, record in nodes.items():
        kind, value = record.kind, record.value
        _lhs, path = key
        prefix = path + b"."
        direct = direct_by_parent[key]
        direct.sort(key=lambda child: canonical_uint(child[len(prefix) :]))
        if direct != [prefix + str(index).encode("ascii") for index in range(len(direct))]:
            fail("report_schema", "a structural node has noncontiguous child paths")
        expected_children = {
            b"ref": 0,
            b"fixed": 0,
            b"pattern": 0,
            b"group": 1,
            b"optional": 1,
            b"repeat0": 1,
            b"repeat1": 1,
        }.get(kind)
        if (
            (expected_children is not None and len(direct) != expected_children)
            or (kind == b"sequence" and not direct)
            or (kind == b"choice" and len(direct) < 2)
        ):
            fail("report_schema", "a structural node has an invalid grammar child count")
        parent_start, parent_end = record.span
        previous_end = parent_start
        for child_path in direct:
            child_start, child_end = nodes[(_lhs, child_path)].span
            if child_start < parent_start or child_end > parent_end:
                fail("report_span", "a structural child span is outside its parent")
            if child_start < previous_end:
                fail("report_span", "structural sibling spans overlap or are source-reordered")
            previous_end = child_end
        if kind in (b"ref", b"fixed", b"pattern"):
            canonical_hex(value)
            if kind == b"pattern" and value != b"5b302d395d2b":
                fail("report_schema", "a pattern node is outside the closed grammar notation")
        elif value != b"-":
            fail("report_schema", "a structural node unexpectedly carries a leaf value")
        children[key] = tuple(direct)
    for key, record in fixed.items():
        node = nodes[key]
        spelling = decoded_bytes(record.spelling)
        if (
            (node.kind, node.value) != (b"fixed", record.spelling)
            or node.span != record.span
        ):
            fail("report_schema", "a fixed leaf disagrees with its structural node")
        if source[record.span[0] : record.span[1]] != b'"' + spelling + b'"':
            fail("report_span", "a fixed leaf span does not equal its quoted source spelling")
    known_symbols = set(productions) | lexical_names
    for key, record in references.items():
        node = nodes[key]
        if (
            (node.kind, node.value) != (b"ref", record.target)
            or node.span != record.span
            or record.target not in known_symbols
        ):
            fail("report_schema", "a reference leaf is undefined or disagrees with its node")
        target = decoded_bytes(record.target)
        if not REFERENCE_NAME.fullmatch(target) or source[record.span[0] : record.span[1]] != target:
            fail("report_span", "a reference leaf span does not equal its source name")
    for record in nodes.values():
        if record.kind == b"pattern" and source[record.span[0] : record.span[1]] != b'"[0-9]+"':
            fail("report_span", "a pattern leaf span does not equal its quoted source pattern")
    for name, record in lexical.items():
        _validate_lexical(name, record, source)
    return DocumentSchema(
        frozenset(productions),
        {key: (record.kind, record.value) for key, record in nodes.items()},
        children,
        frozenset(lexical_names),
        {key: record.expansion for key, record in fixed.items()},
        {key: record.target for key, record in references.items()},
        {
            lhs: (record.definition, record.rhs)
            for lhs, record in productions.items()
        },
        {key: record.span for key, record in nodes.items()},
    )
