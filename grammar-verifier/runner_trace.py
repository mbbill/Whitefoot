"""Bounded structural validation for Oracle source-derivation traces."""

from __future__ import annotations

from runner_common_schema import DocumentSchema
from runner_common_wire import PATH, canonical_hex, canonical_uint
from runner_inputs import Inputs, fail


NODE_KINDS = {
    b"choice",
    b"fixed",
    b"group",
    b"optional",
    b"pattern",
    b"ref",
    b"repeat0",
    b"repeat1",
    b"sequence",
}


def _hex_bytes(value: bytes) -> bytes:
    canonical_hex(value)
    return bytes.fromhex(value.decode("ascii"))


Expected = tuple[bytes, ...]


def _maximum_trace_depth(raw_length: int, inputs: Inputs) -> int:
    """Return a sound stack bound for one encoded derivation tree."""

    return min(inputs.limits["oracle_max_proof_nodes"], raw_length)


def _node_label(label: bytes, schema: DocumentSchema) -> tuple[Expected, ...]:
    fields = label.split(b":")
    if len(fields) != 5 or fields[0] != b"node":
        fail("oracle_trace", "an Oracle trace has an unknown node label")
    _hex_bytes(fields[1])
    if not PATH.fullmatch(fields[2]) or fields[3] not in NODE_KINDS:
        fail("oracle_trace", "an Oracle trace has a noncanonical node identity")
    lhs, path, kind, variant = fields[1], fields[2], fields[3], fields[4]
    record = schema.nodes.get((lhs, path))
    if record is None or record[0] != kind:
        fail("oracle_trace", "an Oracle trace node is absent from the common grammar ledger")
    value = record[1]
    if kind in (b"ref", b"fixed"):
        _hex_bytes(variant)
        if variant != value:
            fail("oracle_trace", "an Oracle trace leaf variant disagrees with the grammar ledger")
    elif kind == b"pattern":
        if variant != b"5b302d395d2b" or variant != value:
            fail("oracle_trace", "an Oracle trace has an unknown pattern variant")
    elif kind in (b"sequence", b"group"):
        if variant != b"-":
            fail("oracle_trace", "an Oracle trace structural node has a variant")
    elif kind == b"choice":
        canonical_uint(variant)
    elif kind == b"optional":
        if variant not in (b"empty", b"present"):
            fail("oracle_trace", "an Oracle trace has an unknown optional variant")
    elif kind == b"repeat0":
        if variant not in (b"empty", b"more"):
            fail("oracle_trace", "an Oracle trace has an unknown repeat0 variant")
    elif variant not in (b"one", b"more"):
        fail("oracle_trace", "an Oracle trace has an unknown repeat1 variant")
    grammar_children = schema.children[(lhs, path)]
    child_node = lambda child_path: (
        b"node",
        lhs,
        child_path,
        schema.nodes[(lhs, child_path)][0],
    )
    if kind == b"ref":
        target = schema.references[(lhs, path)]
        if target in schema.productions:
            return ((b"production", target),)
        if target not in schema.lexical_names:
            fail("oracle_trace", "an Oracle trace reference has an unknown target")
        return ((b"token", b"lexical", target),)
    if kind == b"fixed":
        return tuple(
            (b"token", b"fixed", spelling)
            for spelling in schema.fixed_expansions[(lhs, path)]
        )
    if kind == b"pattern":
        return ((b"token", b"pattern", b"5b302d395d2b"),)
    if kind == b"sequence":
        return tuple(child_node(child) for child in grammar_children)
    if kind == b"choice":
        arm = canonical_uint(variant)
        if arm >= len(grammar_children):
            fail("oracle_trace", "an Oracle trace selects a nonexistent grammar arm")
        return (child_node(grammar_children[arm]),)
    if kind == b"group":
        return (child_node(grammar_children[0]),)
    if kind == b"optional":
        return () if variant == b"empty" else (child_node(grammar_children[0]),)
    identity = (b"node", lhs, path, kind)
    if kind == b"repeat0":
        return () if variant == b"empty" else (child_node(grammar_children[0]), identity)
    return (
        (child_node(grammar_children[0]),)
        if variant == b"one"
        else (child_node(grammar_children[0]), identity)
    )


def _label_child_count(
    label: bytes,
    source: bytes,
    spans: list[tuple[int, int]],
    maximum_tokens: int,
    schema: DocumentSchema,
) -> tuple[Expected, tuple[Expected, ...]]:
    if label.startswith(b"production:"):
        fields = label.split(b":")
        if len(fields) != 2:
            fail("oracle_trace", "an Oracle trace has a malformed production label")
        _hex_bytes(fields[1])
        root = (fields[1], b"0")
        if fields[1] not in schema.productions or root not in schema.nodes:
            fail("oracle_trace", "an Oracle production is absent from the common grammar ledger")
        return (
            (b"production", fields[1]),
            ((b"node", fields[1], b"0", schema.nodes[root][0]),),
        )
    if label.startswith(b"node:"):
        children = _node_label(label, schema)
        fields = label.split(b":")
        return (b"node", fields[1], fields[2], fields[3]), children
    fields = label.split(b":")
    if len(fields) != 5 or fields[0] != b"token" or fields[1] not in (b"fixed", b"pattern", b"lexical"):
        fail("oracle_trace", "an Oracle trace has an unknown event label")
    start = canonical_uint(fields[3])
    end = canonical_uint(fields[4])
    if start >= end or end > len(source):
        fail("oracle_trace", "an Oracle trace token span is outside its source")
    spelling = source[start:end]
    if fields[1] == b"fixed":
        if _hex_bytes(fields[2]) != spelling:
            fail("oracle_trace", "an Oracle fixed-token label disagrees with its source span")
    elif fields[1] == b"pattern":
        if fields[2] != b"5b302d395d2b" or not spelling.isdigit():
            fail("oracle_trace", "an Oracle pattern-token label disagrees with its source span")
    else:
        _hex_bytes(fields[2])
        if fields[2] not in schema.lexical_names:
            fail("oracle_trace", "an Oracle lexical-token label is absent from the common ledger")
    if len(spans) >= maximum_tokens:
        fail("oracle_trace", "an Oracle trace exceeds its source-token bound")
    spans.append((start, end))
    return (b"token", fields[1], fields[2]), ()


def _next_child(raw: bytes, cursor: int, parent_end: int) -> tuple[int, int]:
    if cursor + 8 > parent_end:
        fail("oracle_trace", "an Oracle trace omits a child length")
    length = int.from_bytes(raw[cursor : cursor + 8], "big")
    child_start = cursor + 8
    child_end = child_start + length
    if length == 0 or child_end > parent_end:
        fail("oracle_trace", "an Oracle trace child length is outside its parent")
    return child_start, child_end


def validate_trace_hex(
    value: bytes,
    start_hex: bytes,
    source: bytes,
    inputs: Inputs,
    schema: DocumentSchema,
) -> None:
    """Validate one complete tree iteratively against the agreed grammar ledger."""

    raw = _hex_bytes(value)
    maximum_nodes = inputs.limits["oracle_max_proof_nodes"]
    # A derivation can cross many production/ref aliases without consuming a
    # token or increasing one production's EBNF depth. Total proof nodes, not
    # source tokens plus grammar depth, is therefore the sound nesting bound.
    # The encoded byte length is a second, tighter bound because every nested
    # node consumes at least one byte.
    maximum_depth = _maximum_trace_depth(len(raw), inputs)
    cursor = 0
    current_end = len(raw)
    stack: list[tuple[int, tuple[Expected, ...], int]] = []
    spans: list[tuple[int, int]] = []
    nodes = 0
    expected: Expected = (b"production", start_hex)
    while True:
        if cursor + 9 > current_end or raw[cursor] != 0x54:
            fail("oracle_trace", "an Oracle trace node is truncated or lacks its T tag")
        label_length = int.from_bytes(raw[cursor + 1 : cursor + 5], "big")
        label_start = cursor + 5
        label_end = label_start + label_length
        if label_length == 0 or label_end + 4 > current_end or label_length > inputs.limits["max_line_bytes"]:
            fail("oracle_trace", "an Oracle trace label length is invalid")
        label = raw[label_start:label_end]
        try:
            label.decode("ascii")
        except UnicodeDecodeError:
            fail("oracle_trace", "an Oracle trace label is not ASCII")
        child_count = int.from_bytes(raw[label_end : label_end + 4], "big")
        identity, expected_children = _label_child_count(
            label,
            source,
            spans,
            inputs.limits["oracle_max_source_tokens"],
            schema,
        )
        if identity != expected:
            fail("oracle_trace", "an Oracle trace edge disagrees with the common grammar ledger")
        if child_count != len(expected_children):
            fail("oracle_trace", "an Oracle trace label has a noncanonical child count")
        nodes += 1
        if nodes > maximum_nodes:
            fail("oracle_trace", "an Oracle trace exceeds its node bound")
        cursor = label_end + 4
        if child_count:
            if len(stack) + 1 > maximum_depth:
                fail("oracle_trace", "an Oracle trace exceeds its depth bound")
            child_start, child_end = _next_child(raw, cursor, current_end)
            stack.append((current_end, expected_children, 1))
            expected = expected_children[0]
            cursor, current_end = child_start, child_end
            continue
        if cursor != current_end:
            fail("oracle_trace", "an Oracle leaf does not consume its declared byte range")
        while stack:
            parent_end, siblings, next_index = stack.pop()
            if next_index < len(siblings):
                child_start, child_end = _next_child(raw, cursor, parent_end)
                stack.append((parent_end, siblings, next_index + 1))
                expected = siblings[next_index]
                cursor, current_end = child_start, child_end
                break
            if cursor != parent_end:
                fail("oracle_trace", "an Oracle trace parent has trailing or missing bytes")
            current_end = parent_end
        else:
            break
    if cursor != len(raw):
        fail("oracle_trace", "an Oracle trace has trailing bytes")
    previous = 0
    for start, end in spans:
        if start < previous or any(byte not in (0x0A, 0x20) for byte in source[previous:start]):
            fail("oracle_trace", "Oracle trace token spans are reordered or skip source bytes")
        previous = end
    if any(byte not in (0x0A, 0x20) for byte in source[previous:]):
        fail("oracle_trace", "Oracle trace token spans do not cover the complete source")
