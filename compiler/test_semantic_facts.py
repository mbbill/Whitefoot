#!/usr/bin/env python3
"""Exercise the compact, fixed-capacity semantic fact tapes."""

import ctypes
import re
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AstTape, parse


HERE = Path(__file__).resolve().parent
U64_MAX = (1 << 64) - 1
ENUM_POISON = 0x55555555
ENUM_GUARD = 0x66666666
U64_POISON = 0x7777777777777777
U64_GUARD = 0x8888888888888888

FACTS_CLEAN = 0


class TypeTape(ctypes.Structure):
    _fields_ = [
        ("kinds", Buffer),
        ("declarations", Buffer),
        ("element_types", Buffer),
        ("array_lengths", Buffer),
        ("source_nodes", Buffer),
        ("prelude_types", Buffer),
        ("count", ctypes.c_uint64),
        ("status", ctypes.c_int32),
        ("node", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
    ]


class NodeFacts(ctypes.Structure):
    _fields_ = [
        ("type_ids", Buffer),
        ("resolved_declarations", Buffer),
        ("ordinals", Buffer),
        ("operations", Buffer),
        ("constant_values", Buffer),
        ("control_targets", Buffer),
        ("modes", Buffer),
        ("prelude_constructors", Buffer),
        ("count", ctypes.c_uint64),
        ("status", ctypes.c_int32),
        ("node", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
    ]


TYPE_COLUMNS = (
    ("kinds", ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
    ("declarations", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("element_types", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("array_lengths", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("source_nodes", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("prelude_types", ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
)

NODE_COLUMNS = (
    ("type_ids", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("resolved_declarations", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("ordinals", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("operations", ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
    ("constant_values", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("control_targets", ctypes.c_uint64, U64_POISON, U64_GUARD),
    ("modes", ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
    ("prelude_constructors", ctypes.c_int32, ENUM_POISON, ENUM_GUARD),
)


def enum_members(name):
    source = (HERE / "src" / "semantic_facts.xl").read_text()
    match = re.search(rf"enum {name} \{{(.*?)\n\}}", source, re.DOTALL)
    assert match is not None, name
    return tuple(re.findall(r"\b([A-Z][A-Za-z0-9_]*)\(\);", match.group(1)))


def assert_enum_ordinals():
    assert enum_members("SemanticTypeKind") == (
        "SemanticTypeUnknown",
        "SemanticTypeUnit",
        "SemanticTypeU8",
        "SemanticTypeU64",
        "SemanticTypeBool",
        "SemanticTypeEnum",
        "SemanticTypeStruct",
        "SemanticTypeBuffer",
        "SemanticTypeArray",
    )
    assert enum_members("SemanticValueMode") == (
        "SemanticModeNone",
        "SemanticModeOwn",
        "SemanticModeShared",
        "SemanticModeUniq",
    )
    assert enum_members("SemanticOperation") == (
        "SemanticOperationNone",
        "SemanticOperationIaddTrap",
        "SemanticOperationIsubWrap",
        "SemanticOperationIdivTrap",
        "SemanticOperationIremTrap",
        "SemanticOperationIeq",
        "SemanticOperationIlt",
        "SemanticOperationIle",
        "SemanticOperationIgt",
        "SemanticOperationIge",
        "SemanticOperationBand",
        "SemanticOperationBor",
        "SemanticOperationBnot",
        "SemanticOperationLen",
        "SemanticOperationBufferNew",
    )
    assert enum_members("SemanticFactsStatus") == (
        "SemanticFactsClean",
        "SemanticFactsInvalidShape",
        "SemanticFactsCapacity",
    )


def assert_abi_layout():
    assert [name for name, _ in TypeTape._fields_] == [
        "kinds",
        "declarations",
        "element_types",
        "array_lengths",
        "source_nodes",
        "prelude_types",
        "count",
        "status",
        "node",
        "related",
    ]
    assert [getattr(TypeTape, name).offset for name, _ in TypeTape._fields_] == [
        0,
        16,
        32,
        48,
        64,
        80,
        96,
        104,
        112,
        120,
    ]
    assert ctypes.sizeof(TypeTape) == 128

    assert [name for name, _ in NodeFacts._fields_] == [
        "type_ids",
        "resolved_declarations",
        "ordinals",
        "operations",
        "constant_values",
        "control_targets",
        "modes",
        "prelude_constructors",
        "count",
        "status",
        "node",
        "related",
    ]
    assert [getattr(NodeFacts, name).offset for name, _ in NodeFacts._fields_] == [
        0,
        16,
        32,
        48,
        64,
        80,
        96,
        112,
        128,
        136,
        144,
        152,
    ]
    assert ctypes.sizeof(NodeFacts) == 160


def make_tape(tape_type, specs, capacities):
    assert len(specs) == len(capacities)
    storage = []
    buffers = []
    for (_, ctype, poison, guard), capacity in zip(specs, capacities):
        column = (ctype * (capacity + 1))()
        for slot in range(capacity):
            column[slot] = poison
        column[capacity] = guard
        storage.append(column)
        buffers.append(Buffer(ctypes.cast(column, ctypes.c_void_p), capacity))
    tape = tape_type(*buffers, U64_MAX, 99, 123, 456)
    return tuple(storage), tape


def snapshot(storage, capacities):
    return tuple(tuple(column[:capacity]) for column, capacity in zip(storage, capacities))


def assert_guards(storage, specs, capacities):
    for column, (_, _, _, guard), capacity in zip(storage, specs, capacities):
        assert column[capacity] == guard


def configure(library):
    library.semantic_type_tape_reset.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_reset.restype = None
    library.semantic_type_tape_shape_valid.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_shape_valid.restype = ctypes.c_bool
    library.semantic_type_tape_capacity_valid.argtypes = [
        ctypes.POINTER(TypeTape),
        ctypes.c_uint64,
    ]
    library.semantic_type_tape_capacity_valid.restype = ctypes.c_bool
    library.semantic_node_facts_reset.argtypes = [ctypes.POINTER(NodeFacts)]
    library.semantic_node_facts_reset.restype = None
    library.semantic_node_facts_shape_valid.argtypes = [ctypes.POINTER(NodeFacts)]
    library.semantic_node_facts_shape_valid.restype = ctypes.c_bool
    library.semantic_node_facts_capacity_valid.argtypes = [
        ctypes.POINTER(NodeFacts),
        ctypes.c_uint64,
    ]
    library.semantic_node_facts_capacity_valid.restype = ctypes.c_bool
    library.parser_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
    ]
    library.parser_run.restype = None


def assert_reset(library, tape_type, specs, reset):
    capacities = (3,) * len(specs)
    storage, tape = make_tape(tape_type, specs, capacities)
    before = snapshot(storage, capacities)
    for _ in range(2):
        reset(ctypes.byref(tape))
        assert (tape.count, tape.status, tape.node, tape.related) == (
            0,
            FACTS_CLEAN,
            U64_MAX,
            U64_MAX,
        )
        assert snapshot(storage, capacities) == before
        assert_guards(storage, specs, capacities)
        tape.count = U64_MAX
        tape.status = 99
        tape.node = 123
        tape.related = 456


def assert_capacity_cases(library, tape_type, specs, reset, shape, capacity):
    column_count = len(specs)

    # A zero-capacity tape is a valid empty shape and accepts exactly zero rows.
    zero = (0,) * column_count
    storage, tape = make_tape(tape_type, specs, zero)
    reset(ctypes.byref(tape))
    before = snapshot(storage, zero)
    assert shape(ctypes.byref(tape))
    assert capacity(ctypes.byref(tape), 0)
    assert not capacity(ctypes.byref(tape), 1)
    assert snapshot(storage, zero) == before
    assert_guards(storage, specs, zero)

    # Equal full columns accept exactly their remaining capacity.
    full = (3,) * column_count
    storage, tape = make_tape(tape_type, specs, full)
    reset(ctypes.byref(tape))
    before = snapshot(storage, full)
    assert shape(ctypes.byref(tape))
    assert capacity(ctypes.byref(tape), 3)
    assert not capacity(ctypes.byref(tape), 4)
    tape.count = 3
    assert shape(ctypes.byref(tape))
    assert capacity(ctypes.byref(tape), 0)
    assert not capacity(ctypes.byref(tape), 1)
    assert snapshot(storage, full) == before
    assert_guards(storage, specs, full)

    # Every column independently limits the atomic row capacity.
    for short_index in range(column_count):
        asymmetric = [3] * column_count
        asymmetric[short_index] = 2
        capacities = tuple(asymmetric)
        storage, tape = make_tape(tape_type, specs, capacities)
        reset(ctypes.byref(tape))
        before = snapshot(storage, capacities)
        assert shape(ctypes.byref(tape))
        assert capacity(ctypes.byref(tape), 2)
        assert not capacity(ctypes.byref(tape), 3)
        tape.count = 2
        assert shape(ctypes.byref(tape))
        assert capacity(ctypes.byref(tape), 0)
        assert not capacity(ctypes.byref(tape), 1)
        assert snapshot(storage, capacities) == before
        assert_guards(storage, specs, capacities)

    # A forged count is rejected before wrapped subtraction can make it fit.
    hostile = (2,) * column_count
    for forged_count in (3, U64_MAX):
        storage, tape = make_tape(tape_type, specs, hostile)
        tape.count = forged_count
        before = snapshot(storage, hostile)
        metadata = (tape.count, tape.status, tape.node, tape.related)
        assert not shape(ctypes.byref(tape))
        assert not capacity(ctypes.byref(tape), 0)
        assert not capacity(ctypes.byref(tape), U64_MAX)
        assert (tape.count, tape.status, tape.node, tape.related) == metadata
        assert snapshot(storage, hostile) == before
        assert_guards(storage, specs, hostile)


def assert_current_compiler_capacity(library):
    data = compiler_source().encode("ascii")
    source_storage, token_storage, tokens, ast_storage, ast = parse(library, data)
    assert ast.status == 0
    assert 0 < ast.count <= tokens.count

    type_capacities = (ast.count,) * len(TYPE_COLUMNS)
    type_storage, types = make_tape(TypeTape, TYPE_COLUMNS, type_capacities)
    library.semantic_type_tape_reset(ctypes.byref(types))
    assert library.semantic_type_tape_capacity_valid(ctypes.byref(types), ast.count)
    assert_guards(type_storage, TYPE_COLUMNS, type_capacities)

    fact_capacities = (ast.count,) * len(NODE_COLUMNS)
    fact_storage, facts = make_tape(NodeFacts, NODE_COLUMNS, fact_capacities)
    library.semantic_node_facts_reset(ctypes.byref(facts))
    assert library.semantic_node_facts_capacity_valid(ctypes.byref(facts), ast.count)
    assert_guards(fact_storage, NODE_COLUMNS, fact_capacities)

    # Every fact column really must have one slot per AST node.
    for short_index in range(len(NODE_COLUMNS)):
        short = [ast.count] * len(NODE_COLUMNS)
        short[short_index] -= 1
        short_storage, short_facts = make_tape(NodeFacts, NODE_COLUMNS, tuple(short))
        library.semantic_node_facts_reset(ctypes.byref(short_facts))
        assert not library.semantic_node_facts_capacity_valid(
            ctypes.byref(short_facts), ast.count
        )
        assert_guards(short_storage, NODE_COLUMNS, tuple(short))

    # Keep every allocation alive until the native calls have returned.
    assert source_storage and token_storage and ast_storage


def main():
    assert_enum_ordinals()
    assert_abi_layout()
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_reset(
            library,
            TypeTape,
            TYPE_COLUMNS,
            library.semantic_type_tape_reset,
        )
        assert_reset(
            library,
            NodeFacts,
            NODE_COLUMNS,
            library.semantic_node_facts_reset,
        )
        assert_capacity_cases(
            library,
            TypeTape,
            TYPE_COLUMNS,
            library.semantic_type_tape_reset,
            library.semantic_type_tape_shape_valid,
            library.semantic_type_tape_capacity_valid,
        )
        assert_capacity_cases(
            library,
            NodeFacts,
            NODE_COLUMNS,
            library.semantic_node_facts_reset,
            library.semantic_node_facts_shape_valid,
            library.semantic_node_facts_capacity_valid,
        )
        assert_current_compiler_capacity(library)
    print(
        "semantic facts: exact ABI/enums, deterministic reset, asymmetric "
        "capacity, hostile counts, guards, and compiler-sized invariant pass"
    )


if __name__ == "__main__":
    main()
