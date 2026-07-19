#!/usr/bin/env python3
"""Exercise the first fixed buffer type rows and lexer_match3 signature gate."""

import ctypes
import subprocess
import sys
import tempfile
from pathlib import Path

from test_ast_validate import AstValidationReport
from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import AST_NONE, AstTape, children_of, parse
from test_parser_expressions import enum_ordinals
from test_semantic_body import (
    SCRATCH_COLUMNS,
    SemanticBodyReport,
    SemanticBodyScratch,
    assert_scratch_guards,
    configure as configure_semantic_body,
    make_scratch,
    parsed as semantic_parsed,
)
from test_semantic_facts import (
    NODE_COLUMNS,
    TYPE_COLUMNS,
    NodeFacts,
    TypeTape,
    assert_guards,
    make_tape,
    snapshot,
)
from test_symbols import SymbolTape


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT / "prototype" / "democ"))

import democ


SEMANTIC_FACTS_CLEAN = 0
SEMANTIC_FACTS_INVALID_SHAPE = 1
SEMANTIC_FACTS_CAPACITY = 2
SEMANTIC_TYPE_U8 = 2
SEMANTIC_TYPE_U64 = 3
SEMANTIC_TYPE_BOOL = 4
SEMANTIC_TYPE_BUFFER = 7
PRELUDE_TYPE_UNKNOWN = 0
PRELUDE_TYPE_BOOL = 1
TYPE_U8 = 0
TYPE_BOOL = 1
TYPE_U64 = 2
TYPE_BUFFER_U8 = 3
MODE_NONE = 0
MODE_OWN = 1
MODE_SHARED = 2
OP_NONE = 0
OP_IADD_TRAP = 1
OP_IEQ = 5
OP_BAND = 10
PRELUDE_CONSTRUCTOR_UNKNOWN = 0
BODY_CLEAN = 0
BODY_CAPACITY = 8
BODY_MALFORMED = 9
U64_MAX = (1 << 64) - 1
AST = enum_ordinals("AstKind")


def module_is_wired():
    entries = {
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    }
    return "src/semantic_buffer.wf" in entries


def build_focused_library(directory):
    if module_is_wired():
        return build_library(directory)
    source = compiler_source()
    source += "\n" + (HERE / "src" / "semantic_buffer.wf").read_text()
    ir = democ.compile_program(source, alias=False)
    ll = directory / "semantic_buffer.ll"
    library_path = directory / (
        "semantic_buffer.dylib" if sys.platform == "darwin" else "semantic_buffer.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected semantic buffer IR:\n{result.stderr}")
    return ctypes.CDLL(str(library_path))


def configure(library):
    configure_semantic_body(library)
    library.lexer_run.argtypes = [Buffer, ctypes.POINTER(TokenTape)]
    library.lexer_run.restype = None
    library.parser_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
    ]
    library.parser_run.restype = None
    library.semantic_type_tape_reset.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_reset.restype = None
    library.semantic_buffer_type_capacity_valid.argtypes = [
        ctypes.POINTER(TypeTape)
    ]
    library.semantic_buffer_type_capacity_valid.restype = ctypes.c_bool
    library.semantic_buffer_initialize_types.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_buffer_initialize_types.restype = None
    library.semantic_buffer_match3_signature_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_match3_signature_valid.restype = ctypes.c_bool
    library.semantic_buffer_match3_body_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_match3_body_valid.restype = ctypes.c_bool
    library.semantic_buffer_fixed_width_profile.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_fixed_width_profile.restype = ctypes.c_uint64
    library.semantic_buffer_fixed_width_signature_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_fixed_width_signature_valid.restype = ctypes.c_bool
    library.semantic_buffer_fixed_width_body_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_buffer_fixed_width_body_valid.restype = ctypes.c_bool
    library.semantic_buffer_match3_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SymbolTape),
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(SemanticBodyScratch),
        ctypes.POINTER(SemanticBodyReport),
    ]
    library.semantic_buffer_match3_run.restype = None
    library.semantic_buffer_fixed_width_run.argtypes = (
        library.semantic_buffer_match3_run.argtypes
    )
    library.semantic_buffer_fixed_width_run.restype = None


def assert_type_initializer(library):
    capacities = (4,) * len(TYPE_COLUMNS)
    storage, types = make_tape(TypeTape, TYPE_COLUMNS, capacities)
    library.semantic_type_tape_reset(ctypes.byref(types))
    assert library.semantic_buffer_type_capacity_valid(ctypes.byref(types))

    library.semantic_buffer_initialize_types(ctypes.byref(types))

    assert list(storage[0][:4]) == [
        SEMANTIC_TYPE_U8,
        SEMANTIC_TYPE_BOOL,
        SEMANTIC_TYPE_U64,
        SEMANTIC_TYPE_BUFFER,
    ]
    assert list(storage[1][:4]) == [U64_MAX] * 4
    assert list(storage[2][:4]) == [U64_MAX, U64_MAX, U64_MAX, 0]
    assert list(storage[3][:4]) == [U64_MAX] * 4
    assert list(storage[4][:4]) == [U64_MAX] * 4
    assert list(storage[5][:4]) == [
        PRELUDE_TYPE_UNKNOWN,
        PRELUDE_TYPE_BOOL,
        PRELUDE_TYPE_UNKNOWN,
        PRELUDE_TYPE_UNKNOWN,
    ]
    assert (types.count, types.status, types.node, types.related) == (
        4,
        SEMANTIC_FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert_guards(storage, TYPE_COLUMNS, capacities)

    for short_column in range(len(TYPE_COLUMNS)):
        short_capacities = [4] * len(TYPE_COLUMNS)
        short_capacities[short_column] = 3
        short_storage, short_types = make_tape(
            TypeTape, TYPE_COLUMNS, tuple(short_capacities)
        )
        library.semantic_type_tape_reset(ctypes.byref(short_types))
        assert not library.semantic_buffer_type_capacity_valid(
            ctypes.byref(short_types)
        )
        assert_guards(short_storage, TYPE_COLUMNS, tuple(short_capacities))

    library.semantic_type_tape_reset(ctypes.byref(types))
    types.count = 1
    assert not library.semantic_buffer_type_capacity_valid(ctypes.byref(types))
    assert_guards(storage, TYPE_COLUMNS, capacities)

    types.count = U64_MAX
    assert not library.semantic_buffer_type_capacity_valid(ctypes.byref(types))
    assert_guards(storage, TYPE_COLUMNS, capacities)


def real_match_source(width):
    lexer = (HERE / "src" / "lexer.wf").read_bytes()
    start = lexer.index(f"fn lexer_match{width} ".encode())
    end = lexer.index(b"\nfn ", start)
    return lexer[start:end].rstrip() + b"\n"


def renamed_match4_source():
    exact = real_match_source(4)
    renamed = exact.replace(b"'s", b"'input_region")
    renamed = renamed.replace(b"source", b"input")
    renamed = renamed.replace(b"start", b"offset")
    renamed = renamed.replace(b"d: own u8", b"needle: own u8")
    renamed = renamed.replace(b"b3, d)", b"b3, needle)")
    renamed = renamed.replace(b"p3", b"position3")
    assert renamed != exact
    return renamed


def real_match3_source():
    return real_match_source(3)


def parsed_match3(library):
    data = real_match3_source()
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    functions = children_of(columns, ast.root)
    assert len(functions) == 1
    function = functions[0]
    direct = children_of(columns, function)
    assert len(direct) == 12
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
    return source_storage, source, token_storage, tokens, columns, ast, function


def signature_valid(library, source, tokens, ast, function):
    return bool(
        library.semantic_buffer_match3_signature_valid(
            source,
            ctypes.byref(tokens),
            ctypes.byref(ast),
            function,
        )
    )


def assert_real_signature(library):
    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    assert signature_valid(library, source, tokens, ast, function)


def signature_nodes(columns, function):
    direct = children_of(columns, function)
    regions = direct[1]
    source_parameter = direct[2]
    source_mode = children_of(columns, source_parameter)[0]
    reads_effect = direct[9]
    return {
        "regions": regions,
        "source mode": source_mode,
        "reads effect": reads_effect,
    }


def assert_hostile_ast_fails_closed(library):
    # These are the three containers whose absent child used to feed AST_NONE
    # into a later head lookup. Each corruption must be rejected before that lookup.
    for label in ("regions", "source mode", "reads effect"):
        fixture = parsed_match3(library)
        _, source, _, tokens, columns, ast, function = fixture
        node = signature_nodes(columns, function)[label]
        columns[4][node] = AST_NONE
        columns[5][node] = AST_NONE
        assert not signature_valid(library, source, tokens, ast, function), label

    # A non-terminating direct-child chain is malformed but remains in-bounds.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    direct = children_of(columns, function)
    columns[6][direct[0]] = direct[0]
    assert not signature_valid(library, source, tokens, ast, function)

    # Invalid public ordinals and truncated advertised columns fail before reads.
    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, _ = fixture
    assert not signature_valid(library, source, tokens, ast, ast.count)
    assert not signature_valid(library, source, tokens, ast, U64_MAX)

    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    tokens.ends.length = tokens.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    fixture = parsed_match3(library)
    _, source, _, tokens, _, ast, function = fixture
    ast.heads.length = ast.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    # Out-of-range child links must be rejected without indexing that child.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    regions = children_of(columns, function)[1]
    columns[4][regions] = ast.count
    columns[5][regions] = ast.count
    assert not signature_valid(library, source, tokens, ast, function)

    # A one-byte parameter head aimed at the zero-width EOF token used to reach
    # parser_word_byte eagerly. It must now fail at the width gate.
    fixture = parsed_match3(library)
    _, source, _, tokens, columns, ast, function = fixture
    a_parameter = children_of(columns, function)[4]
    columns[1][a_parameter] = tokens.count - 1
    assert not signature_valid(library, source, tokens, ast, function)

    # Keep the twelve-child function shape while corrupting gated node kinds.
    for child_ordinal, wrong_kind in (
        (10, AST["AstPureEffect"]),
        (11, AST["AstReturn"]),
    ):
        fixture = parsed_match3(library)
        _, source, _, tokens, columns, ast, function = fixture
        child = children_of(columns, function)[child_ordinal]
        columns[0][child] = wrong_kind
        assert not signature_valid(library, source, tokens, ast, function)


def assert_wrong_signature_fails(library):
    variants = (
        (b"lexer_match3", b"lexer_matchx"),
        (b", c: own u8)", b", c: own u8, d: own u8)"),
        (b"['s]", b"['r]"),
        (b"source: &'s buffer<u8>", b"source: own buffer<u8>"),
        (b"source: &'s buffer<u8>", b"source: &'s buffer<u64>"),
        (b"start: own u64", b"start: own u8"),
        (b"b: own u8", b"b: own u64"),
        (b"c: own u8", b"c: own Bool"),
        (b"-> own Bool", b"-> &'s Bool"),
        (b"-> own Bool", b"-> own u8"),
        (b"reads('s), traps", b"reads('other), traps"),
        (b"reads('s), traps", b"reads('s), writes('s)"),
        (b"reads('s), traps", b"reads('s)"),
    )
    exact = real_match3_source()
    for before, after in variants:
        data = exact.replace(before, after, 1)
        assert data != exact
        source_storage, token_storage, tokens, columns, ast = parse(library, data)
        assert ast.status == 0
        function = children_of(columns, ast.root)[0]
        source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
        assert not signature_valid(library, source, tokens, ast, function)
        assert token_storage and columns


def semantic_case(library, data=None):
    if data is None:
        data = real_match3_source()
    return semantic_parsed(library, data)


def make_buffer_outputs(library, ast_count, scratch_count=14):
    type_caps = (4,) * len(TYPE_COLUMNS)
    fact_caps = (ast_count,) * len(NODE_COLUMNS)
    scratch_caps = (scratch_count,) * len(SCRATCH_COLUMNS)
    type_storage, types = make_tape(TypeTape, TYPE_COLUMNS, type_caps)
    fact_storage, facts = make_tape(NodeFacts, NODE_COLUMNS, fact_caps)
    library.semantic_type_tape_reset(ctypes.byref(types))
    library.semantic_node_facts_reset(ctypes.byref(facts))
    scratch_storage, scratch_physical, scratch = make_scratch(scratch_caps)
    return {
        "type_storage": type_storage,
        "types": types,
        "fact_storage": fact_storage,
        "facts": facts,
        "scratch_storage": scratch_storage,
        "scratch_physical": scratch_physical,
        "scratch": scratch,
        "type_caps": type_caps,
        "fact_caps": fact_caps,
    }


def invoke_run(library, case, function, outputs, generic=False):
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    run = (
        library.semantic_buffer_fixed_width_run
        if generic
        else library.semantic_buffer_match3_run
    )
    run(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        function,
        ctypes.byref(case[6]),
        ctypes.byref(case[9]),
        ctypes.byref(outputs["types"]),
        ctypes.byref(outputs["facts"]),
        ctypes.byref(outputs["scratch"]),
        ctypes.byref(report),
    )
    return report


def output_payload_snapshot(outputs):
    return (
        snapshot(outputs["type_storage"], outputs["type_caps"]),
        snapshot(outputs["fact_storage"], outputs["fact_caps"]),
        tuple(
            tuple(column[:capacity])
            for column, capacity in zip(
                outputs["scratch_storage"], outputs["scratch_physical"]
            )
        ),
    )


def output_snapshot(outputs):
    types = outputs["types"]
    facts = outputs["facts"]
    scratch = outputs["scratch"]
    return (
        output_payload_snapshot(outputs),
        (types.count, types.status, types.node, types.related),
        (facts.count, facts.status, facts.node, facts.related),
        scratch.count,
    )


def assert_output_guards(outputs):
    assert_guards(
        outputs["type_storage"], TYPE_COLUMNS, outputs["type_caps"]
    )
    assert_guards(
        outputs["fact_storage"], NODE_COLUMNS, outputs["fact_caps"]
    )
    assert_scratch_guards(
        outputs["scratch_storage"], outputs["scratch_physical"]
    )


def match3_nodes(columns, function):
    direct = children_of(columns, function)
    assert len(direct) == 12
    parameters = direct[2:7]
    return_mode, return_type = direct[7:9]
    block = direct[11]
    statements = children_of(columns, block)
    assert len(statements) == 10
    lets = statements[:9]
    return_node = statements[9]
    let_children = [children_of(columns, node) for node in lets]
    assert all(len(nodes) == 4 for nodes in let_children)
    names = [nodes[0] for nodes in let_children]
    modes = [nodes[1] for nodes in let_children]
    types = [nodes[2] for nodes in let_children]
    values = [nodes[3] for nodes in let_children]
    value_children = [children_of(columns, node) for node in values]
    assert all(len(nodes) == 3 for nodes in value_children)
    (return_value,) = children_of(columns, return_node)
    return_value_children = children_of(columns, return_value)
    assert len(return_value_children) == 3
    return {
        "parameters": parameters,
        "return_mode": return_mode,
        "return_type": return_type,
        "block": block,
        "lets": lets,
        "return": return_node,
        "names": names,
        "modes": modes,
        "types": types,
        "values": values,
        "value_children": value_children,
        "return_value": return_value,
        "return_value_children": return_value_children,
    }


def assert_dense_facts(case, outputs, nodes):
    ast = case[5]
    count = ast.count
    type_ids = [U64_MAX] * count
    resolved = [U64_MAX] * count
    ordinals = [U64_MAX] * count
    operations = [OP_NONE] * count
    constants = [U64_MAX] * count
    targets = [U64_MAX] * count
    modes = [MODE_NONE] * count
    constructors = [PRELUDE_CONSTRUCTOR_UNKNOWN] * count

    parameter_types = [TYPE_BUFFER_U8, TYPE_U64, TYPE_U8, TYPE_U8, TYPE_U8]
    parameter_modes = [MODE_SHARED, MODE_OWN, MODE_OWN, MODE_OWN, MODE_OWN]
    for ordinal, (parameter, type_id, mode) in enumerate(
        zip(nodes["parameters"], parameter_types, parameter_modes)
    ):
        mode_node, type_node = children_of(case[4], parameter)
        type_ids[parameter] = type_id
        ordinals[parameter] = ordinal
        modes[parameter] = mode
        type_ids[mode_node] = type_id
        modes[mode_node] = mode
        type_ids[type_node] = type_id
    source_type = children_of(case[4], nodes["parameters"][0])[1]
    (source_element,) = children_of(case[4], source_type)
    type_ids[source_element] = TYPE_U8
    type_ids[nodes["return_mode"]] = TYPE_BOOL
    modes[nodes["return_mode"]] = MODE_OWN
    type_ids[nodes["return_type"]] = TYPE_BOOL

    let_types = [
        TYPE_U64,
        TYPE_U64,
        TYPE_U8,
        TYPE_U8,
        TYPE_U8,
        TYPE_BOOL,
        TYPE_BOOL,
        TYPE_BOOL,
        TYPE_BOOL,
    ]
    for ordinal, (declaration, name, mode_node, type_node, type_id) in enumerate(
        zip(
            nodes["lets"],
            nodes["names"],
            nodes["modes"],
            nodes["types"],
            let_types,
        )
    ):
        type_ids[declaration] = type_id
        ordinals[declaration] = ordinal
        modes[declaration] = MODE_OWN
        type_ids[name] = type_id
        type_ids[mode_node] = type_id
        modes[mode_node] = MODE_OWN
        type_ids[type_node] = type_id

    start = nodes["parameters"][1]
    source = nodes["parameters"][0]
    for ordinal, literal_value in ((0, 1), (1, 2)):
        call = nodes["values"][ordinal]
        type_node, left, literal = nodes["value_children"][ordinal]
        type_ids[call] = TYPE_U64
        ordinals[call] = ordinal
        operations[call] = OP_IADD_TRAP
        modes[call] = MODE_OWN
        type_ids[type_node] = TYPE_U64
        type_ids[left] = TYPE_U64
        resolved[left] = start
        modes[left] = MODE_OWN
        type_ids[literal] = TYPE_U64
        constants[literal] = literal_value
        modes[literal] = MODE_OWN

    subscript_declarations = [start, nodes["lets"][0], nodes["lets"][1]]
    for index in range(3):
        ordinal = index + 2
        root = nodes["values"][ordinal]
        type_node, deref_node, subscript = nodes["value_children"][ordinal]
        (source_place,) = children_of(case[4], deref_node)
        type_ids[root] = TYPE_U8
        ordinals[root] = ordinal
        modes[root] = MODE_OWN
        type_ids[type_node] = TYPE_U8
        type_ids[deref_node] = TYPE_BUFFER_U8
        modes[deref_node] = MODE_SHARED
        type_ids[source_place] = TYPE_BUFFER_U8
        resolved[source_place] = source
        modes[source_place] = MODE_SHARED
        type_ids[subscript] = TYPE_U64
        resolved[subscript] = subscript_declarations[index]
        modes[subscript] = MODE_OWN

    eq_right_declarations = nodes["parameters"][2:5]
    for index in range(3):
        ordinal = index + 5
        call = nodes["values"][ordinal]
        type_node, left, right = nodes["value_children"][ordinal]
        type_ids[call] = TYPE_BOOL
        ordinals[call] = ordinal
        operations[call] = OP_IEQ
        modes[call] = MODE_OWN
        type_ids[type_node] = TYPE_U8
        type_ids[left] = TYPE_U8
        resolved[left] = nodes["lets"][index + 2]
        modes[left] = MODE_OWN
        type_ids[right] = TYPE_U8
        resolved[right] = eq_right_declarations[index]
        modes[right] = MODE_OWN

    first_call = nodes["values"][8]
    first_type, first_left, first_right = nodes["value_children"][8]
    type_ids[first_call] = TYPE_BOOL
    ordinals[first_call] = 8
    operations[first_call] = OP_BAND
    modes[first_call] = MODE_OWN
    type_ids[first_type] = TYPE_BOOL
    type_ids[first_left] = TYPE_BOOL
    resolved[first_left] = nodes["lets"][5]
    modes[first_left] = MODE_OWN
    type_ids[first_right] = TYPE_BOOL
    resolved[first_right] = nodes["lets"][6]
    modes[first_right] = MODE_OWN

    final_call = nodes["return_value"]
    final_type, final_left, final_right = nodes["return_value_children"]
    type_ids[final_call] = TYPE_BOOL
    ordinals[final_call] = 9
    operations[final_call] = OP_BAND
    modes[final_call] = MODE_OWN
    type_ids[final_type] = TYPE_BOOL
    type_ids[final_left] = TYPE_BOOL
    resolved[final_left] = nodes["lets"][8]
    modes[final_left] = MODE_OWN
    type_ids[final_right] = TYPE_BOOL
    resolved[final_right] = nodes["lets"][7]
    modes[final_right] = MODE_OWN
    type_ids[nodes["return"]] = TYPE_BOOL
    ordinals[nodes["return"]] = 9
    modes[nodes["return"]] = MODE_OWN

    expected = (
        type_ids,
        resolved,
        ordinals,
        operations,
        constants,
        targets,
        modes,
        constructors,
    )
    observed = tuple(
        list(column[:count]) for column in outputs["fact_storage"]
    )
    assert observed == expected
    facts = outputs["facts"]
    assert (facts.count, facts.status, facts.node, facts.related) == (
        count,
        SEMANTIC_FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )


def assert_dense_scratch(case, outputs, nodes):
    heads = case[4][1]
    name_nodes = nodes["parameters"] + nodes["names"]
    declarations = nodes["parameters"] + nodes["lets"]
    types = [
        TYPE_BUFFER_U8,
        TYPE_U64,
        TYPE_U8,
        TYPE_U8,
        TYPE_U8,
        TYPE_U64,
        TYPE_U64,
        TYPE_U8,
        TYPE_U8,
        TYPE_U8,
        TYPE_BOOL,
        TYPE_BOOL,
        TYPE_BOOL,
        TYPE_BOOL,
    ]
    modes = [MODE_SHARED] + [MODE_OWN] * 13
    expected = (
        [heads[node] for node in name_nodes],
        declarations,
        types,
        modes,
        [SCRATCH_COLUMNS[4][1]] * 14,
    )
    observed = tuple(
        list(column[:14]) for column in outputs["scratch_storage"]
    )
    assert observed == expected
    assert outputs["scratch"].count == 14
    assert outputs["scratch"].loop_count == 0


def fixed_width_nodes(columns, function, width):
    direct = children_of(columns, function)
    assert len(direct) == width + 9
    parameters = direct[2 : width + 4]
    return_mode = direct[width + 4]
    return_type = direct[width + 5]
    block = direct[width + 8]
    statements = children_of(columns, block)
    assert len(statements) == 4 * width - 2
    lets = statements[:-1]
    return_node = statements[-1]
    let_children = [children_of(columns, node) for node in lets]
    assert all(len(row) == 4 for row in let_children)
    values = [row[3] for row in let_children]
    value_children = [children_of(columns, node) for node in values]
    assert all(len(row) == 3 for row in value_children)
    (return_value,) = children_of(columns, return_node)
    return_value_children = children_of(columns, return_value)
    assert len(return_value_children) == 3
    return {
        "parameters": parameters,
        "return_mode": return_mode,
        "return_type": return_type,
        "block": block,
        "lets": lets,
        "return": return_node,
        "names": [row[0] for row in let_children],
        "modes": [row[1] for row in let_children],
        "types": [row[2] for row in let_children],
        "values": values,
        "value_children": value_children,
        "return_value": return_value,
        "return_value_children": return_value_children,
    }


def assert_fixed_width_dense_facts(case, outputs, nodes, width):
    count = case[5].count
    type_ids = [U64_MAX] * count
    resolved = [U64_MAX] * count
    ordinals = [U64_MAX] * count
    operations = [OP_NONE] * count
    constants = [U64_MAX] * count
    targets = [U64_MAX] * count
    modes = [MODE_NONE] * count
    constructors = [PRELUDE_CONSTRUCTOR_UNKNOWN] * count

    parameter_types = [TYPE_BUFFER_U8, TYPE_U64] + [TYPE_U8] * width
    parameter_modes = [MODE_SHARED] + [MODE_OWN] * (width + 1)
    for ordinal, (parameter, type_id, mode) in enumerate(
        zip(nodes["parameters"], parameter_types, parameter_modes)
    ):
        mode_node, type_node = children_of(case[4], parameter)
        type_ids[parameter] = type_id
        ordinals[parameter] = ordinal
        modes[parameter] = mode
        type_ids[mode_node] = type_id
        modes[mode_node] = mode
        type_ids[type_node] = type_id
    source_type = children_of(case[4], nodes["parameters"][0])[1]
    (source_element,) = children_of(case[4], source_type)
    type_ids[source_element] = TYPE_U8
    type_ids[nodes["return_mode"]] = TYPE_BOOL
    modes[nodes["return_mode"]] = MODE_OWN
    type_ids[nodes["return_type"]] = TYPE_BOOL

    add_count = width - 1
    load_start = add_count
    eq_start = load_start + width
    band_start = eq_start + width
    band_count = width - 2
    let_types = (
        [TYPE_U64] * add_count
        + [TYPE_U8] * width
        + [TYPE_BOOL] * (2 * width - 2)
    )
    assert len(let_types) == 4 * width - 3
    for ordinal, (declaration, name, mode_node, type_node, type_id) in enumerate(
        zip(
            nodes["lets"],
            nodes["names"],
            nodes["modes"],
            nodes["types"],
            let_types,
        )
    ):
        type_ids[declaration] = type_id
        ordinals[declaration] = ordinal
        modes[declaration] = MODE_OWN
        type_ids[name] = type_id
        type_ids[mode_node] = type_id
        modes[mode_node] = MODE_OWN
        type_ids[type_node] = type_id

    source_parameter, start_parameter = nodes["parameters"][:2]
    for index in range(add_count):
        call = nodes["values"][index]
        type_node, left, literal = nodes["value_children"][index]
        type_ids[call] = TYPE_U64
        ordinals[call] = index
        operations[call] = OP_IADD_TRAP
        modes[call] = MODE_OWN
        type_ids[type_node] = TYPE_U64
        type_ids[left] = TYPE_U64
        resolved[left] = start_parameter
        modes[left] = MODE_OWN
        type_ids[literal] = TYPE_U64
        constants[literal] = index + 1
        modes[literal] = MODE_OWN

    for index in range(width):
        ordinal = load_start + index
        root = nodes["values"][ordinal]
        type_node, deref_node, subscript = nodes["value_children"][ordinal]
        (source_place,) = children_of(case[4], deref_node)
        subscript_declaration = (
            start_parameter if index == 0 else nodes["lets"][index - 1]
        )
        type_ids[root] = TYPE_U8
        ordinals[root] = ordinal
        modes[root] = MODE_OWN
        type_ids[type_node] = TYPE_U8
        type_ids[deref_node] = TYPE_BUFFER_U8
        modes[deref_node] = MODE_SHARED
        type_ids[source_place] = TYPE_BUFFER_U8
        resolved[source_place] = source_parameter
        modes[source_place] = MODE_SHARED
        type_ids[subscript] = TYPE_U64
        resolved[subscript] = subscript_declaration
        modes[subscript] = MODE_OWN

    def set_binary(ordinal, operand_type, operation, left_decl, right_decl):
        call = nodes["values"][ordinal]
        type_node, left, right = nodes["value_children"][ordinal]
        type_ids[call] = TYPE_BOOL
        ordinals[call] = ordinal
        operations[call] = operation
        modes[call] = MODE_OWN
        type_ids[type_node] = operand_type
        type_ids[left] = operand_type
        resolved[left] = left_decl
        modes[left] = MODE_OWN
        type_ids[right] = operand_type
        resolved[right] = right_decl
        modes[right] = MODE_OWN

    for index in range(width):
        set_binary(
            eq_start + index,
            TYPE_U8,
            OP_IEQ,
            nodes["lets"][load_start + index],
            nodes["parameters"][2 + index],
        )

    pair_count = width // 2
    for index in range(band_count):
        if index < pair_count:
            left_decl = nodes["lets"][eq_start + 2 * index]
            right_decl = nodes["lets"][eq_start + 2 * index + 1]
        else:
            left_ordinal = band_start if index == pair_count else band_start + index - 1
            right_ordinal = band_start + (index - pair_count) + 1
            left_decl = nodes["lets"][left_ordinal]
            right_decl = nodes["lets"][right_ordinal]
        set_binary(
            band_start + index,
            TYPE_BOOL,
            OP_BAND,
            left_decl,
            right_decl,
        )

    final_ordinal = len(nodes["lets"])
    final_type, final_left, final_right = nodes["return_value_children"]
    final_left_index = band_count - 1 if band_count > pair_count else 0
    final_left_decl = nodes["lets"][band_start + final_left_index]
    final_right_decl = (
        nodes["lets"][eq_start + width - 1]
        if width % 2
        else nodes["lets"][band_start + pair_count - 1]
    )
    type_ids[nodes["return_value"]] = TYPE_BOOL
    ordinals[nodes["return_value"]] = final_ordinal
    operations[nodes["return_value"]] = OP_BAND
    modes[nodes["return_value"]] = MODE_OWN
    type_ids[final_type] = TYPE_BOOL
    type_ids[final_left] = TYPE_BOOL
    resolved[final_left] = final_left_decl
    modes[final_left] = MODE_OWN
    type_ids[final_right] = TYPE_BOOL
    resolved[final_right] = final_right_decl
    modes[final_right] = MODE_OWN
    type_ids[nodes["return"]] = TYPE_BOOL
    ordinals[nodes["return"]] = final_ordinal
    modes[nodes["return"]] = MODE_OWN

    expected = (
        type_ids,
        resolved,
        ordinals,
        operations,
        constants,
        targets,
        modes,
        constructors,
    )
    observed = tuple(list(column[:count]) for column in outputs["fact_storage"])
    assert observed == expected
    facts = outputs["facts"]
    assert (facts.count, facts.status, facts.node, facts.related) == (
        count,
        SEMANTIC_FACTS_CLEAN,
        U64_MAX,
        U64_MAX,
    )


def assert_fixed_width_dense_scratch(case, outputs, nodes, width):
    needed = 5 * width - 1
    heads = case[4][1]
    parameter_types = [TYPE_BUFFER_U8, TYPE_U64] + [TYPE_U8] * width
    let_types = (
        [TYPE_U64] * (width - 1)
        + [TYPE_U8] * width
        + [TYPE_BOOL] * (2 * width - 2)
    )
    name_nodes = nodes["parameters"] + nodes["names"]
    declarations = nodes["parameters"] + nodes["lets"]
    expected = (
        [heads[node] for node in name_nodes],
        declarations,
        parameter_types + let_types,
        [MODE_SHARED] + [MODE_OWN] * (needed - 1),
        [SCRATCH_COLUMNS[4][1]] * needed,
    )
    observed = tuple(
        list(column[:needed]) for column in outputs["scratch_storage"]
    )
    assert observed == expected
    assert outputs["scratch"].count == needed
    assert outputs["scratch"].loop_count == 0


def assert_fixed_width_capacity_retry(
    library, case, function, width, expected_success
):
    needed = 5 * width - 1
    groups = (
        ("types", tuple(name for name, *_ in TYPE_COLUMNS), 3, 4),
        (
            "facts",
            tuple(name for name, *_ in NODE_COLUMNS),
            case[5].count - 1,
            case[5].count,
        ),
        (
            "scratch",
            ("name_tokens", "declarations", "type_ids", "modes"),
            needed - 1,
            needed,
        ),
    )
    for object_name, fields, short_length, full_length in groups:
        for field in fields:
            outputs = make_buffer_outputs(library, case[5].count, needed)
            before = output_payload_snapshot(outputs)
            target = outputs[object_name]
            getattr(target, field).length = short_length
            report = invoke_run(library, case, function, outputs, generic=True)
            assert (report.status, report.node, report.related) == (
                BODY_CAPACITY,
                function,
                needed,
            ), (width, object_name, field, report.status, report.related)
            assert output_payload_snapshot(outputs) == before
            assert outputs["types"].count == 0
            assert outputs["facts"].count == 0
            assert outputs["scratch"].count == 0
            assert outputs["types"].status == SEMANTIC_FACTS_CAPACITY
            assert outputs["facts"].status == SEMANTIC_FACTS_CAPACITY
            assert_output_guards(outputs)

            getattr(target, field).length = full_length
            library.semantic_type_tape_reset(ctypes.byref(outputs["types"]))
            library.semantic_node_facts_reset(ctypes.byref(outputs["facts"]))
            retry = invoke_run(library, case, function, outputs, generic=True)
            assert (retry.status, retry.node, retry.related) == (
                BODY_CLEAN,
                U64_MAX,
                U64_MAX,
            )
            assert output_snapshot(outputs) == expected_success
            assert_output_guards(outputs)


def assert_all_fixed_width_profiles(library):
    for width in (3, 4, 6, 7):
        data = real_match_source(width)
        case = semantic_case(library, data)
        function = children_of(case[4], case[5].root)[0]
        nodes = fixed_width_nodes(case[4], function, width)
        assert case[5].count == 36 * width
        assert library.semantic_buffer_fixed_width_profile(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        ) == width
        assert library.semantic_buffer_fixed_width_signature_valid(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        )
        assert library.semantic_buffer_fixed_width_body_valid(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        )
        assert bool(
            library.semantic_buffer_match3_signature_valid(
                case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
            )
        ) == (width == 3)
        assert bool(
            library.semantic_buffer_match3_body_valid(
                case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
            )
        ) == (width == 3)

        needed = 5 * width - 1
        first = make_buffer_outputs(library, case[5].count, needed)
        first_report = invoke_run(
            library, case, function, first, generic=True
        )
        assert (first_report.status, first_report.node, first_report.related) == (
            BODY_CLEAN,
            U64_MAX,
            U64_MAX,
        )
        assert_fixed_width_dense_facts(case, first, nodes, width)
        assert_fixed_width_dense_scratch(case, first, nodes, width)
        assert_output_guards(first)

        second = make_buffer_outputs(library, case[5].count, needed)
        second_report = invoke_run(
            library, case, function, second, generic=True
        )
        assert (second_report.status, second_report.node, second_report.related) == (
            BODY_CLEAN,
            U64_MAX,
            U64_MAX,
        )
        expected_success = output_snapshot(first)
        assert output_snapshot(second) == expected_success
        assert_output_guards(second)
        assert_fixed_width_capacity_retry(
            library, case, function, width, expected_success
        )

        if width != 3:
            rejected = make_buffer_outputs(library, case[5].count, needed)
            rejected_before = output_payload_snapshot(rejected)
            rejected_report = invoke_run(
                library, case, function, rejected, generic=False
            )
            assert rejected_report.status != BODY_CLEAN
            assert output_payload_snapshot(rejected) == rejected_before
            assert rejected["types"].count == 0
            assert rejected["facts"].count == 0
            assert rejected["scratch"].count == 0


def assert_generic_renames_and_profile_mismatches(library):
    exact = real_match_source(4)
    renamed = renamed_match4_source()
    renamed_case = semantic_case(library, renamed)
    renamed_function = children_of(
        renamed_case[4], renamed_case[5].root
    )[0]
    assert library.semantic_buffer_fixed_width_body_valid(
        renamed_case[1],
        ctypes.byref(renamed_case[3]),
        ctypes.byref(renamed_case[5]),
        renamed_function,
    )
    renamed_nodes = fixed_width_nodes(
        renamed_case[4], renamed_function, 4
    )
    renamed_outputs = make_buffer_outputs(
        library, renamed_case[5].count, 5 * 4 - 1
    )
    renamed_report = invoke_run(
        library,
        renamed_case,
        renamed_function,
        renamed_outputs,
        generic=True,
    )
    assert (
        renamed_report.status,
        renamed_report.node,
        renamed_report.related,
    ) == (BODY_CLEAN, U64_MAX, U64_MAX)
    assert_fixed_width_dense_facts(
        renamed_case, renamed_outputs, renamed_nodes, 4
    )
    assert_fixed_width_dense_scratch(
        renamed_case, renamed_outputs, renamed_nodes, 4
    )
    assert_output_guards(renamed_outputs)

    one_sided = exact.replace(
        b"source: &'s buffer<u8>", b"input: &'s buffer<u8>", 1
    )
    source_storage, _, tokens, columns, ast = parse(library, one_sided)
    function = children_of(columns, ast.root)[0]
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(one_sided))
    assert library.semantic_buffer_fixed_width_signature_valid(
        source, ctypes.byref(tokens), ctypes.byref(ast), function
    )
    assert not library.semantic_buffer_fixed_width_body_valid(
        source, ctypes.byref(tokens), ctypes.byref(ast), function
    )

    for replacement, expected_profile in (
        (b"lexer_match3", 3),
        (b"lexer_match5", U64_MAX),
        (b"lexer_matchx", U64_MAX),
    ):
        mismatched = exact.replace(b"lexer_match4", replacement, 1)
        mismatch_storage, _, mismatch_tokens, mismatch_columns, mismatch_ast = parse(
            library, mismatched
        )
        mismatch_function = children_of(
            mismatch_columns, mismatch_ast.root
        )[0]
        mismatch_source = Buffer(
            ctypes.cast(mismatch_storage, ctypes.c_void_p), len(mismatched)
        )
        assert library.semantic_buffer_fixed_width_profile(
            mismatch_source,
            ctypes.byref(mismatch_tokens),
            ctypes.byref(mismatch_ast),
            mismatch_function,
        ) == expected_profile
        assert not library.semantic_buffer_fixed_width_signature_valid(
            mismatch_source,
            ctypes.byref(mismatch_tokens),
            ctypes.byref(mismatch_ast),
            mismatch_function,
        )
        assert not library.semantic_buffer_fixed_width_body_valid(
            mismatch_source,
            ctypes.byref(mismatch_tokens),
            ctypes.byref(mismatch_ast),
            mismatch_function,
        )


def assert_nonclean_validation_is_atomic(library):
    width = 4
    case = semantic_case(library, real_match_source(width))
    function = children_of(case[4], case[5].root)[0]
    case[6].status = 1
    outputs = make_buffer_outputs(library, case[5].count, 5 * width - 1)
    before = output_payload_snapshot(outputs)
    report = invoke_run(library, case, function, outputs, generic=True)
    assert report.status != BODY_CLEAN
    assert output_payload_snapshot(outputs) == before
    assert outputs["types"].count == 0
    assert outputs["facts"].count == 0
    assert outputs["scratch"].count == 0


def assert_width7_body_mutations_fail_atomically(library):
    width = 7
    exact = real_match_source(width)
    variants = (
        (
            "last add literal",
            exact.replace(
                b"iadd.trap<u64>(start, 6_u64)",
                b"iadd.trap<u64>(start, 5_u64)",
                1,
            ),
        ),
        (
            "last load subscript",
            exact.replace(
                b"index<u8>(deref(source), p6)",
                b"index<u8>(deref(source), p5)",
                1,
            ),
        ),
        (
            "last equality binding",
            exact.replace(b"ieq<u8>(b6, g)", b"ieq<u8>(b6, f)", 1),
        ),
        (
            "deep band binding",
            exact.replace(
                b"band<Bool>(first_four, pair2)",
                b"band<Bool>(first_four, pair1)",
                1,
            ),
        ),
        (
            "final band binding",
            exact.replace(
                b"return band<Bool>(first_six, e6);",
                b"return band<Bool>(first_four, e6);",
                1,
            ),
        ),
    )
    for label, data in variants:
        assert data != exact, label
        case = semantic_case(library, data)
        function = children_of(case[4], case[5].root)[0]
        assert library.semantic_buffer_fixed_width_profile(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        ) == width, label
        assert not library.semantic_buffer_fixed_width_body_valid(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        ), label
        outputs = make_buffer_outputs(
            library, case[5].count, 5 * width - 1
        )
        before = output_payload_snapshot(outputs)
        report = invoke_run(
            library, case, function, outputs, generic=True
        )
        assert report.status == BODY_MALFORMED, (
            label,
            report.status,
            report.node,
            report.related,
        )
        assert output_payload_snapshot(outputs) == before, label
        assert outputs["types"].count == 0
        assert outputs["facts"].count == 0
        assert outputs["scratch"].count == 0
        assert outputs["types"].status == SEMANTIC_FACTS_INVALID_SHAPE
        assert outputs["facts"].status == SEMANTIC_FACTS_INVALID_SHAPE
        assert_output_guards(outputs)


def assert_full_semantic_pass(library):
    case = semantic_case(library)
    function = children_of(case[4], case[5].root)[0]
    nodes = match3_nodes(case[4], function)
    assert case[5].count == 108
    assert library.semantic_buffer_match3_body_valid(
        case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
    )

    first = make_buffer_outputs(library, case[5].count)
    first_report = invoke_run(library, case, function, first)
    assert (first_report.status, first_report.node, first_report.related) == (
        BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert list(first["type_storage"][0][:4]) == [
        SEMANTIC_TYPE_U8,
        SEMANTIC_TYPE_BOOL,
        SEMANTIC_TYPE_U64,
        SEMANTIC_TYPE_BUFFER,
    ]
    assert list(first["type_storage"][1][:4]) == [U64_MAX] * 4
    assert list(first["type_storage"][2][:4]) == [
        U64_MAX,
        U64_MAX,
        U64_MAX,
        TYPE_U8,
    ]
    assert list(first["type_storage"][3][:4]) == [U64_MAX] * 4
    assert list(first["type_storage"][4][:4]) == [U64_MAX] * 4
    assert list(first["type_storage"][5][:4]) == [0, 1, 0, 0]
    assert (
        first["types"].count,
        first["types"].status,
        first["types"].node,
        first["types"].related,
    ) == (4, SEMANTIC_FACTS_CLEAN, U64_MAX, U64_MAX)
    assert_dense_facts(case, first, nodes)
    assert_dense_scratch(case, first, nodes)
    assert_output_guards(first)

    second = make_buffer_outputs(library, case[5].count)
    second_report = invoke_run(library, case, function, second)
    assert (second_report.status, second_report.node, second_report.related) == (
        BODY_CLEAN,
        U64_MAX,
        U64_MAX,
    )
    assert output_snapshot(first) == output_snapshot(second)
    assert_output_guards(second)
    return case, function, output_snapshot(first)


def assert_capacity_retry(library, case, function, expected_success):
    groups = (
        (
            "types",
            tuple(name for name, *_ in TYPE_COLUMNS),
            3,
            4,
        ),
        (
            "facts",
            tuple(name for name, *_ in NODE_COLUMNS),
            case[5].count - 1,
            case[5].count,
        ),
        (
            "scratch",
            ("name_tokens", "declarations", "type_ids", "modes"),
            13,
            14,
        ),
    )
    for object_name, fields, short_length, full_length in groups:
        for field in fields:
            outputs = make_buffer_outputs(library, case[5].count)
            before = output_payload_snapshot(outputs)
            target = outputs[object_name]
            getattr(target, field).length = short_length
            report = invoke_run(library, case, function, outputs)
            assert (report.status, report.node, report.related) == (
                BODY_CAPACITY,
                function,
                14,
            ), (object_name, field, report.status, report.node, report.related)
            assert output_payload_snapshot(outputs) == before
            assert outputs["types"].count == 0
            assert outputs["facts"].count == 0
            assert outputs["scratch"].count == 0
            assert outputs["types"].status == SEMANTIC_FACTS_CAPACITY
            assert outputs["facts"].status == SEMANTIC_FACTS_CAPACITY
            assert_output_guards(outputs)

            getattr(target, field).length = full_length
            library.semantic_type_tape_reset(ctypes.byref(outputs["types"]))
            library.semantic_node_facts_reset(ctypes.byref(outputs["facts"]))
            retry = invoke_run(library, case, function, outputs)
            assert (retry.status, retry.node, retry.related) == (
                BODY_CLEAN,
                U64_MAX,
                U64_MAX,
            )
            assert output_snapshot(outputs) == expected_success
            assert_output_guards(outputs)


def swapped_body_order(data):
    lines = data.splitlines(keepends=True)
    p1 = next(index for index, line in enumerate(lines) if b"let p1:" in line)
    p2 = next(index for index, line in enumerate(lines) if b"let p2:" in line)
    lines[p1], lines[p2] = lines[p2], lines[p1]
    return b"".join(lines)


def assert_body_mutations_fail_atomically(library):
    exact = real_match3_source()
    variants = [
        ("body order", swapped_body_order(exact)),
        ("operation", exact.replace(b"iadd.trap", b"iadd.wrap", 1)),
        ("binding type", exact.replace(b"let p1: own u64", b"let p1: own u8", 1)),
        ("binding name", exact.replace(b"let p1:", b"let q1:", 1)),
        ("literal", exact.replace(b"1_u64", b"3_u64", 1)),
        ("index base", exact.replace(b"deref(source)", b"deref(start)", 1)),
        (
            "index subscript",
            exact.replace(
                b"index<u8>(deref(source), start)",
                b"index<u8>(deref(source), p1)",
                1,
            ),
        ),
        (
            "return",
            exact.replace(
                b"return band<Bool>(first, e2);",
                b"return band<Bool>(e0, e2);",
                1,
            ),
        ),
    ]
    for label, data in variants:
        assert data != exact, label
        case = semantic_case(library, data)
        function = children_of(case[4], case[5].root)[0]
        assert not library.semantic_buffer_match3_body_valid(
            case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
        ), label
        outputs = make_buffer_outputs(library, case[5].count)
        before = output_payload_snapshot(outputs)
        report = invoke_run(library, case, function, outputs)
        assert report.status == BODY_MALFORMED, (
            label,
            report.status,
            report.node,
            report.related,
        )
        assert output_payload_snapshot(outputs) == before, label
        assert outputs["types"].count == 0
        assert outputs["facts"].count == 0
        assert outputs["scratch"].count == 0
        assert outputs["types"].status == SEMANTIC_FACTS_INVALID_SHAPE
        assert outputs["facts"].status == SEMANTIC_FACTS_INVALID_SHAPE
        assert_output_guards(outputs)


def assert_mutated_profile_fails_atomically(
    library, case, function, label
):
    assert not library.semantic_buffer_match3_body_valid(
        case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
    ), label
    outputs = make_buffer_outputs(library, case[5].count)
    before = output_payload_snapshot(outputs)
    report = invoke_run(library, case, function, outputs)
    assert report.status != BODY_CLEAN, (
        label,
        report.status,
        report.node,
        report.related,
    )
    assert output_payload_snapshot(outputs) == before, label
    assert outputs["types"].count == 0
    assert outputs["facts"].count == 0
    assert outputs["scratch"].count == 0
    assert outputs["types"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert outputs["facts"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert_output_guards(outputs)


def assert_topology_and_token_regressions(library):
    root_cases = (
        ("root first none", 4, "none"),
        ("root last none", 5, "none"),
        ("root first self", 4, "root"),
        ("root last self", 5, "root"),
        ("root first out of range", 4, "count"),
        ("root last out of range", 5, "count"),
    )
    for label, column_index, value_kind in root_cases:
        case = semantic_case(library)
        ast = case[5]
        function = children_of(case[4], ast.root)[0]
        value = AST_NONE
        if value_kind == "root":
            value = ast.root
        elif value_kind == "count":
            value = ast.count
        case[4][column_index][ast.root] = value
        assert_mutated_profile_fails_atomically(
            library, case, function, label
        )

    for label, next_value in (
        ("function next self", "self"),
        ("function next out of range", "count"),
    ):
        case = semantic_case(library)
        ast = case[5]
        function = children_of(case[4], ast.root)[0]
        value = function if next_value == "self" else ast.count
        case[4][6][function] = value
        assert_mutated_profile_fails_atomically(
            library, case, function, label
        )

    for label, head_value in (
        ("region container head out of range", "count"),
        ("region container head max", "max"),
    ):
        case = semantic_case(library)
        ast = case[5]
        function = children_of(case[4], ast.root)[0]
        regions = children_of(case[4], function)[1]
        value = case[3].count if head_value == "count" else U64_MAX
        case[4][1][regions] = value
        assert_mutated_profile_fails_atomically(
            library, case, function, label
        )

    case = semantic_case(library)
    ast = case[5]
    function = children_of(case[4], ast.root)[0]
    nodes = match3_nodes(case[4], function)
    start_operand = nodes["value_children"][0][1]
    operand_token = case[4][1][start_operand]
    case[2][0][operand_token] = 2  # TokWord -> TokOpName, bytes unchanged.
    assert_mutated_profile_fails_atomically(
        library, case, function, "operand token kind"
    )


def assert_full_compiler_membership(library):
    data = compiler_source().encode("utf-8")
    source_storage, token_storage, tokens, columns, ast = parse(library, data)
    assert ast.status == 0
    matches = []
    for declaration in children_of(columns, ast.root):
        if columns[0][declaration] != AST["AstFunction"]:
            continue
        direct = children_of(columns, declaration)
        if not direct:
            continue
        name = direct[0]
        if data[columns[2][name] : columns[3][name]] == b"lexer_match3":
            matches.append(declaration)
    assert len(matches) == 1
    source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(data))
    assert library.semantic_buffer_match3_body_valid(
        source, ctypes.byref(tokens), ctypes.byref(ast), matches[0]
    )
    assert token_storage and columns


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_focused_library(Path(raw_directory))
        configure(library)
        assert_type_initializer(library)
        assert_real_signature(library)
        assert_hostile_ast_fails_closed(library)
        assert_wrong_signature_fails(library)
        case, function, expected_success = assert_full_semantic_pass(library)
        assert_capacity_retry(library, case, function, expected_success)
        assert_all_fixed_width_profiles(library)
        assert_generic_renames_and_profile_mismatches(library)
        assert_nonclean_validation_is_atomic(library)
        assert_width7_body_mutations_fail_atomically(library)
        assert_body_mutations_fail_atomically(library)
        assert_topology_and_token_regressions(library)
        assert_full_compiler_membership(library)
    print(
        "semantic buffer: canonical 4-row types; real lexer_match3/4/6/7 "
        "profiles; dense facts; 5N-1 scratch; asymmetric capacities/retry; "
        "renamed analysis, profile mismatches, and width-aware hostile "
        "mutations pass"
    )


if __name__ == "__main__":
    main()
