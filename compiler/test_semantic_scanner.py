#!/usr/bin/env python3
"""Exercise the shared semantic slice for the three real lexer scanners."""

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
    PRELUDE_CONSTRUCTOR_FALSE,
    PRELUDE_CONSTRUCTOR_TRUE,
    SCRATCH_COLUMNS,
    SemanticBodyReport,
    SemanticBodyScratch,
    configure as configure_semantic_body,
    find_function_by_text,
    parsed as semantic_parsed,
)
from test_semantic_buffer import (
    BODY_CAPACITY,
    BODY_CLEAN,
    MODE_NONE,
    MODE_OWN,
    MODE_SHARED,
    OP_IADD_TRAP,
    OP_NONE,
    PRELUDE_CONSTRUCTOR_UNKNOWN,
    SEMANTIC_FACTS_CAPACITY,
    SEMANTIC_FACTS_CLEAN,
    SEMANTIC_FACTS_INVALID_SHAPE,
    SEMANTIC_TYPE_BOOL,
    SEMANTIC_TYPE_BUFFER,
    SEMANTIC_TYPE_U64,
    SEMANTIC_TYPE_U8,
    TYPE_BOOL,
    TYPE_BUFFER_U8,
    TYPE_U8,
    TYPE_U64,
    U64_MAX,
    assert_output_guards,
    make_buffer_outputs,
    output_payload_snapshot,
    output_snapshot,
)
from test_semantic_facts import NODE_COLUMNS, TYPE_COLUMNS, NodeFacts, TypeTape
from test_symbols import SymbolTape


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT / "prototype" / "democ"))

import democ


AST = enum_ordinals("AstKind")
OP_IGE = 9
SCANNERS = (
    (b"lexer_scan_ident", b"lexer_is_ident_tail"),
    (b"lexer_scan_typeid", b"lexer_is_type_tail"),
    (b"lexer_scan_number", b"lexer_is_number_tail"),
)


def module_is_wired():
    entries = {
        line.strip()
        for line in (HERE / "sources.txt").read_text().splitlines()
        if line.strip()
    }
    return "src/semantic_scanner.wf" in entries


def build_focused_library(directory):
    if module_is_wired():
        return build_library(directory)
    source = compiler_source()
    source += "\n" + (HERE / "src" / "semantic_scanner.wf").read_text()
    ir = democ.compile_program(source, alias=False)
    ll = directory / "semantic_scanner.ll"
    library_path = directory / (
        "semantic_scanner.dylib" if sys.platform == "darwin" else "semantic_scanner.so"
    )
    ll.write_text(ir)
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    command = [cc, "-O2"]
    command += ["-dynamiclib"] if sys.platform == "darwin" else ["-shared", "-fPIC"]
    command += [str(ll), "-o", str(library_path)]
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(f"clang rejected scanner semantic IR:\n{result.stderr}")
    return ctypes.CDLL(str(library_path))


def configure(library):
    configure_semantic_body(library)
    library.semantic_type_tape_reset.argtypes = [ctypes.POINTER(TypeTape)]
    library.semantic_type_tape_reset.restype = None
    library.semantic_node_facts_reset.argtypes = [ctypes.POINTER(NodeFacts)]
    library.semantic_node_facts_reset.restype = None
    library.semantic_scanner_signature_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
    ]
    library.semantic_scanner_signature_valid.restype = ctypes.c_bool
    library.semantic_scanner_body_valid.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.c_uint64,
        ctypes.POINTER(SymbolTape),
    ]
    library.semantic_scanner_body_valid.restype = ctypes.c_bool
    library.semantic_scanner_run.argtypes = [
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
    library.semantic_scanner_run.restype = None


def function_source(name):
    lexer = (HERE / "src" / "lexer.wf").read_bytes()
    start = lexer.index(b"fn " + name + b" ")
    end = lexer.index(b"\nfn ", start)
    return lexer[start:end].rstrip() + b"\n"


def scanner_fixture(scanner, predicate):
    return function_source(predicate) + b"\n" + function_source(scanner)


def invoke(library, case, function, outputs):
    report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
    library.semantic_scanner_run(
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


def scanner_nodes(columns, function):
    direct = children_of(columns, function)
    assert len(direct) == 10
    block = direct[9]
    cursor_let, loop, return_node = children_of(columns, block)
    cursor_children = children_of(columns, cursor_let)
    loop_label, loop_block = children_of(columns, loop)
    guard_match, byte_let, keep_let, keep_match = children_of(columns, loop_block)
    guard_call, guard_true, guard_false = children_of(columns, guard_match)
    (guard_true_block,) = children_of(columns, guard_true)
    (guard_false_block,) = children_of(columns, guard_false)
    (guard_break,) = children_of(columns, guard_true_block)
    (guard_break_label,) = children_of(columns, guard_break)
    byte_children = children_of(columns, byte_let)
    keep_children = children_of(columns, keep_let)
    predicate_call = keep_children[3]
    (predicate_argument,) = children_of(columns, predicate_call)
    (predicate_actual,) = children_of(columns, predicate_argument)
    keep_scrutinee, keep_true, keep_false = children_of(columns, keep_match)
    (keep_true_block,) = children_of(columns, keep_true)
    (keep_false_block,) = children_of(columns, keep_false)
    (set_node,) = children_of(columns, keep_true_block)
    set_target, increment = children_of(columns, set_node)
    (keep_break,) = children_of(columns, keep_false_block)
    (keep_break_label,) = children_of(columns, keep_break)
    (return_value,) = children_of(columns, return_node)
    return {
        "function": function,
        "direct": direct,
        "block": block,
        "cursor_let": cursor_let,
        "cursor_children": cursor_children,
        "loop": loop,
        "loop_label": loop_label,
        "loop_block": loop_block,
        "guard_match": guard_match,
        "guard_call": guard_call,
        "guard_true": guard_true,
        "guard_false": guard_false,
        "guard_true_block": guard_true_block,
        "guard_false_block": guard_false_block,
        "guard_break": guard_break,
        "guard_break_label": guard_break_label,
        "byte_let": byte_let,
        "byte_children": byte_children,
        "keep_let": keep_let,
        "keep_children": keep_children,
        "predicate_call": predicate_call,
        "predicate_argument": predicate_argument,
        "predicate_actual": predicate_actual,
        "keep_match": keep_match,
        "keep_scrutinee": keep_scrutinee,
        "keep_true": keep_true,
        "keep_false": keep_false,
        "keep_true_block": keep_true_block,
        "keep_false_block": keep_false_block,
        "set": set_node,
        "set_target": set_target,
        "increment": increment,
        "keep_break": keep_break,
        "keep_break_label": keep_break_label,
        "return": return_node,
        "return_value": return_value,
    }


def assert_dense_facts(case, outputs, nodes, predicate):
    count = case[5].count
    type_ids = [U64_MAX] * count
    resolved = [U64_MAX] * count
    ordinals = [U64_MAX] * count
    operations = [OP_NONE] * count
    constants = [U64_MAX] * count
    targets = [U64_MAX] * count
    modes = [MODE_NONE] * count
    constructors = [PRELUDE_CONSTRUCTOR_UNKNOWN] * count

    def set_place(node, type_id, declaration, mode, target=U64_MAX):
        type_ids[node] = type_id
        resolved[node] = declaration
        modes[node] = mode
        targets[node] = target

    def set_parameter(parameter, type_id, ordinal, mode):
        mode_node, type_node = children_of(case[4], parameter)
        type_ids[parameter] = type_id
        ordinals[parameter] = ordinal
        modes[parameter] = mode
        type_ids[mode_node] = type_id
        modes[mode_node] = mode
        type_ids[type_node] = type_id

    def set_let(declaration, type_id, ordinal):
        name, mode_node, type_node, _ = children_of(case[4], declaration)
        type_ids[declaration] = type_id
        ordinals[declaration] = ordinal
        modes[declaration] = MODE_OWN
        type_ids[name] = type_id
        type_ids[mode_node] = type_id
        modes[mode_node] = MODE_OWN
        type_ids[type_node] = type_id

    def set_match(match_node, continuation, true_target, false_target):
        scrutinee, true_arm, false_arm = children_of(case[4], match_node)
        (true_block,) = children_of(case[4], true_arm)
        (false_block,) = children_of(case[4], false_arm)
        targets[match_node] = continuation
        targets[scrutinee] = match_node
        type_ids[true_arm] = TYPE_BOOL
        ordinals[true_arm] = 0
        constants[true_arm] = 1
        targets[true_arm] = true_target
        constructors[true_arm] = PRELUDE_CONSTRUCTOR_TRUE
        type_ids[false_arm] = TYPE_BOOL
        ordinals[false_arm] = 1
        constants[false_arm] = 0
        targets[false_arm] = false_target
        constructors[false_arm] = PRELUDE_CONSTRUCTOR_FALSE
        targets[true_block] = true_target
        targets[false_block] = false_target

    source, start, size = nodes["direct"][2:5]
    set_parameter(source, TYPE_BUFFER_U8, 0, MODE_SHARED)
    set_parameter(start, TYPE_U64, 1, MODE_OWN)
    set_parameter(size, TYPE_U64, 2, MODE_OWN)
    source_type = children_of(case[4], source)[1]
    (source_element,) = children_of(case[4], source_type)
    type_ids[source_element] = TYPE_U8
    type_ids[nodes["direct"][5]] = TYPE_U64
    modes[nodes["direct"][5]] = MODE_OWN
    type_ids[nodes["direct"][6]] = TYPE_U64

    set_let(nodes["cursor_let"], TYPE_U64, 0)
    set_place(nodes["cursor_children"][3], TYPE_U64, start, MODE_OWN)

    guard_type, guard_left, guard_right = children_of(
        case[4], nodes["guard_call"]
    )
    type_ids[nodes["guard_call"]] = TYPE_BOOL
    ordinals[nodes["guard_call"]] = 1
    operations[nodes["guard_call"]] = OP_IGE
    modes[nodes["guard_call"]] = MODE_OWN
    targets[nodes["guard_call"]] = nodes["guard_match"]
    type_ids[guard_type] = TYPE_U64
    set_place(guard_left, TYPE_U64, nodes["cursor_let"], MODE_OWN)
    set_place(guard_right, TYPE_U64, size, MODE_OWN)

    set_let(nodes["byte_let"], TYPE_U8, 2)
    index_node = nodes["byte_children"][3]
    index_type, deref_node, subscript = children_of(case[4], index_node)
    (source_place,) = children_of(case[4], deref_node)
    type_ids[index_node] = TYPE_U8
    ordinals[index_node] = 2
    modes[index_node] = MODE_OWN
    type_ids[index_type] = TYPE_U8
    type_ids[deref_node] = TYPE_BUFFER_U8
    modes[deref_node] = MODE_SHARED
    set_place(source_place, TYPE_BUFFER_U8, source, MODE_SHARED)
    set_place(subscript, TYPE_U64, nodes["cursor_let"], MODE_OWN)

    set_let(nodes["keep_let"], TYPE_BOOL, 3)
    predicate_formal = children_of(case[4], predicate)[1]
    type_ids[nodes["predicate_call"]] = TYPE_BOOL
    resolved[nodes["predicate_call"]] = predicate
    ordinals[nodes["predicate_call"]] = 3
    modes[nodes["predicate_call"]] = MODE_OWN
    type_ids[nodes["predicate_argument"]] = TYPE_U8
    resolved[nodes["predicate_argument"]] = predicate_formal
    ordinals[nodes["predicate_argument"]] = 0
    modes[nodes["predicate_argument"]] = MODE_OWN
    set_place(
        nodes["predicate_actual"], TYPE_U8, nodes["byte_let"], MODE_OWN
    )
    set_place(
        nodes["keep_scrutinee"],
        TYPE_BOOL,
        nodes["keep_let"],
        MODE_OWN,
        nodes["keep_match"],
    )

    increment_type, increment_left, increment_literal = children_of(
        case[4], nodes["increment"]
    )
    type_ids[nodes["increment"]] = TYPE_U64
    ordinals[nodes["increment"]] = 4
    operations[nodes["increment"]] = OP_IADD_TRAP
    modes[nodes["increment"]] = MODE_OWN
    targets[nodes["increment"]] = nodes["guard_call"]
    type_ids[increment_type] = TYPE_U64
    set_place(increment_left, TYPE_U64, nodes["cursor_let"], MODE_OWN)
    type_ids[increment_literal] = TYPE_U64
    constants[increment_literal] = 1
    modes[increment_literal] = MODE_OWN
    type_ids[nodes["set"]] = TYPE_U64
    ordinals[nodes["set"]] = 4
    targets[nodes["set"]] = nodes["loop"]
    set_place(nodes["set_target"], TYPE_U64, nodes["cursor_let"], MODE_OWN)

    set_match(
        nodes["guard_match"],
        nodes["byte_let"],
        nodes["guard_break"],
        nodes["byte_let"],
    )
    set_match(
        nodes["keep_match"],
        nodes["loop"],
        nodes["set"],
        nodes["keep_break"],
    )
    targets[nodes["loop"]] = nodes["return"]
    targets[nodes["loop_block"]] = nodes["guard_match"]
    targets[nodes["guard_break"]] = nodes["return"]
    resolved[nodes["guard_break_label"]] = nodes["loop_label"]
    targets[nodes["keep_break"]] = nodes["return"]
    resolved[nodes["keep_break_label"]] = nodes["loop_label"]

    set_place(
        nodes["return_value"],
        TYPE_U64,
        nodes["cursor_let"],
        MODE_OWN,
        nodes["return"],
    )
    type_ids[nodes["return"]] = TYPE_U64
    ordinals[nodes["return"]] = 0
    targets[nodes["return"]] = nodes["function"]
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
    defaults = (
        U64_MAX,
        U64_MAX,
        U64_MAX,
        OP_NONE,
        U64_MAX,
        U64_MAX,
        MODE_NONE,
        PRELUDE_CONSTRUCTOR_UNKNOWN,
    )
    assert sum(
        any(expected[column][node] != defaults[column] for column in range(8))
        for node in range(count)
    ) == 62
    assert (
        outputs["facts"].count,
        outputs["facts"].status,
        outputs["facts"].node,
        outputs["facts"].related,
    ) == (count, SEMANTIC_FACTS_CLEAN, U64_MAX, U64_MAX)


def assert_dense_scratch(case, outputs, nodes):
    source, start, size = nodes["direct"][2:5]
    name_nodes = [
        source,
        start,
        size,
        nodes["cursor_children"][0],
        nodes["byte_children"][0],
        nodes["keep_children"][0],
    ]
    declarations = [
        source,
        start,
        size,
        nodes["cursor_let"],
        nodes["byte_let"],
        nodes["keep_let"],
    ]
    expected = (
        [case[4][1][node] for node in name_nodes],
        declarations,
        [TYPE_BUFFER_U8, TYPE_U64, TYPE_U64, TYPE_U64, TYPE_U8, TYPE_BOOL],
        [MODE_SHARED, MODE_OWN, MODE_OWN, MODE_OWN, MODE_OWN, MODE_OWN],
        [SCRATCH_COLUMNS[4][1]] * 6,
    )
    observed = tuple(list(column[:6]) for column in outputs["scratch_storage"])
    assert observed == expected
    assert outputs["scratch"].count == 6
    assert outputs["scratch"].loop_count == 0


def assert_real_scanners(library):
    for scanner_name, predicate_name in SCANNERS:
        isolated = function_source(scanner_name)
        source_storage, _, tokens, columns, ast = parse(library, isolated)
        assert ast.status == 0
        assert ast.count == 73
        function = children_of(columns, ast.root)[0]
        source = Buffer(ctypes.cast(source_storage, ctypes.c_void_p), len(isolated))
        assert library.semantic_scanner_signature_valid(
            source, ctypes.byref(tokens), ctypes.byref(ast), function
        )

        data = scanner_fixture(scanner_name, predicate_name)
        case = semantic_parsed(library, data)
        scanner = find_function_by_text(
            data, case[4], case[5], scanner_name
        )
        predicate = find_function_by_text(
            data, case[4], case[5], predicate_name
        )
        assert library.semantic_scanner_body_valid(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            scanner,
            ctypes.byref(case[9]),
        )
        outputs = make_buffer_outputs(library, case[5].count, 6)
        report = invoke(library, case, scanner, outputs)
        assert (report.status, report.node, report.related) == (
            BODY_CLEAN,
            U64_MAX,
            U64_MAX,
        )
        nodes = scanner_nodes(case[4], scanner)
        assert outputs["types"].count == 4
        assert tuple(list(column[:4]) for column in outputs["type_storage"]) == (
            [
                SEMANTIC_TYPE_U8,
                SEMANTIC_TYPE_BOOL,
                SEMANTIC_TYPE_U64,
                SEMANTIC_TYPE_BUFFER,
            ],
            [U64_MAX] * 4,
            [U64_MAX, U64_MAX, U64_MAX, TYPE_U8],
            [U64_MAX] * 4,
            [U64_MAX] * 4,
            [0, 1, 0, 0],
        )
        assert outputs["facts"].count == case[5].count
        assert outputs["scratch"].count == 6
        assert outputs["fact_storage"][1][nodes["predicate_call"]] == predicate
        assert outputs["fact_storage"][5][nodes["increment"]] == nodes["guard_call"]
        assert_dense_facts(case, outputs, nodes, predicate)
        assert_dense_scratch(case, outputs, nodes)
        assert_output_guards(outputs)

        repeat = make_buffer_outputs(library, case[5].count, 6)
        repeat_report = invoke(library, case, scanner, repeat)
        assert repeat_report.status == BODY_CLEAN
        assert output_snapshot(repeat) == output_snapshot(outputs)
        assert_output_guards(repeat)


def assert_capacity_retry(library):
    scanner_name, predicate_name = SCANNERS[0]
    data = scanner_fixture(scanner_name, predicate_name)
    case = semantic_parsed(library, data)
    scanner = find_function_by_text(data, case[4], case[5], scanner_name)
    full = make_buffer_outputs(library, case[5].count, 6)
    full_report = invoke(library, case, scanner, full)
    assert full_report.status == BODY_CLEAN
    expected = output_snapshot(full)
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
            5,
            6,
        ),
    )
    for object_name, fields, short_length, full_length in groups:
        for field in fields:
            outputs = make_buffer_outputs(library, case[5].count, 6)
            before = output_payload_snapshot(outputs)
            target = outputs[object_name]
            getattr(target, field).length = short_length
            report = invoke(library, case, scanner, outputs)
            assert (report.status, report.node, report.related) == (
                BODY_CAPACITY,
                scanner,
                6,
            ), (object_name, field, report.status, report.related)
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
            retry = invoke(library, case, scanner, outputs)
            assert (retry.status, retry.node, retry.related) == (
                BODY_CLEAN,
                U64_MAX,
                U64_MAX,
            )
            assert output_snapshot(outputs) == expected
            assert_output_guards(outputs)


def assert_rejected_atomically(library, case, scanner, label):
    assert not library.semantic_scanner_body_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        scanner,
        ctypes.byref(case[9]),
    ), label
    outputs = make_buffer_outputs(library, case[5].count, 6)
    before = output_payload_snapshot(outputs)
    report = invoke(library, case, scanner, outputs)
    assert report.status != BODY_CLEAN, (label, report.status)
    assert output_payload_snapshot(outputs) == before, label
    assert outputs["types"].count == 0
    assert outputs["facts"].count == 0
    assert outputs["scratch"].count == 0
    assert outputs["types"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert outputs["facts"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert_output_guards(outputs)


def assert_invalid_validation_is_atomic(library):
    scanner_name, predicate_name = SCANNERS[0]
    data = scanner_fixture(scanner_name, predicate_name)
    case = semantic_parsed(library, data)
    scanner = find_function_by_text(data, case[4], case[5], scanner_name)
    case[6].status = 1
    outputs = make_buffer_outputs(library, case[5].count, 6)
    before = output_payload_snapshot(outputs)
    report = invoke(library, case, scanner, outputs)
    assert report.status != BODY_CLEAN
    assert output_payload_snapshot(outputs) == before
    assert outputs["types"].count == 0
    assert outputs["facts"].count == 0
    assert outputs["scratch"].count == 0
    assert outputs["types"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert outputs["facts"].status == SEMANTIC_FACTS_INVALID_SHAPE
    assert_output_guards(outputs)


def assert_consistent_renames(library):
    predicate = function_source(b"lexer_is_ident_tail")
    scanner = function_source(b"lexer_scan_ident")
    predicate = predicate.replace(b"lexer_is_ident_tail", b"custom_predicate")
    predicate = predicate.replace(b"(c: own u8)", b"(value: own u8)")
    predicate = predicate.replace(b"(c: c)", b"(c: value)")
    predicate = predicate.replace(b"(c, 95_u8)", b"(value, 95_u8)")
    replacements = (
        (b"lexer_scan_ident", b"scan_custom"),
        (b"'s", b"'scan_region"),
        (b"source", b"input"),
        (b"start", b"begin"),
        (b"size", b"limit"),
        (b"cursor", b"position"),
        (b"byte", b"element"),
        (b"keep", b"continue_flag"),
        (b"@ident", b"@scan"),
        (b"lexer_is_ident_tail", b"custom_predicate"),
        (b"c: element", b"value: element"),
    )
    for before, after in replacements:
        scanner = scanner.replace(before, after)
    data = predicate + b"\n" + scanner
    case = semantic_parsed(library, data)
    function = find_function_by_text(data, case[4], case[5], b"scan_custom")
    assert library.semantic_scanner_signature_valid(
        case[1], ctypes.byref(case[3]), ctypes.byref(case[5]), function
    )
    assert library.semantic_scanner_body_valid(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        function,
        ctypes.byref(case[9]),
    )
    outputs = make_buffer_outputs(library, case[5].count, 6)
    report = invoke(library, case, function, outputs)
    assert report.status == BODY_CLEAN
    assert_output_guards(outputs)


def assert_hostile_token_heads(library):
    scanner_name, predicate_name = SCANNERS[0]
    data = scanner_fixture(scanner_name, predicate_name)
    cases = (
        ("scanner fn", "function", "cursor_let"),
        ("callee fn", "predicate", "predicate_name"),
        ("reads", "reads", "traps"),
        ("traps", "traps", "reads"),
        ("top block", "block", "loop"),
        ("loop", "loop", "guard_match"),
        ("loop block", "loop_block", "loop"),
        ("guard match", "guard_match", "guard_call"),
        ("guard arm block", "guard_true_block", "loop"),
        ("set", "set", "return"),
        ("return", "return", "set"),
        ("callee pure", "predicate_effect", "predicate_block"),
        ("callee block", "predicate_block", "predicate_effect"),
    )
    for label, target_role, replacement_role in cases:
        case = semantic_parsed(library, data)
        scanner = find_function_by_text(data, case[4], case[5], scanner_name)
        predicate = find_function_by_text(data, case[4], case[5], predicate_name)
        nodes = scanner_nodes(case[4], scanner)
        predicate_direct = children_of(case[4], predicate)
        roles = dict(nodes)
        roles.update(
            {
                "predicate": predicate,
                "predicate_name": predicate_direct[0],
                "predicate_effect": predicate_direct[4],
                "predicate_block": predicate_direct[5],
                "reads": nodes["direct"][7],
                "traps": nodes["direct"][8],
            }
        )
        case[4][1][roles[target_role]] = case[4][1][roles[replacement_role]]
        assert_rejected_atomically(library, case, scanner, label)


def assert_body_mutations(library):
    predicate = function_source(b"lexer_is_ident_tail")
    scanner = function_source(b"lexer_scan_ident")
    variants = (
        ("guard operation", scanner.replace(b"ige<u64>", b"ile<u64>", 1)),
        (
            "wrong index base",
            scanner.replace(b"deref(source)", b"deref(start)", 1),
        ),
        ("one-sided label", scanner.replace(b"break @ident", b"break @other", 1)),
        (
            "wrong set target",
            scanner.replace(b"set cursor =", b"set size =", 1),
        ),
        ("wrong increment", scanner.replace(b"1_u64", b"2_u64", 1)),
        (
            "duplicate local",
            scanner.replace(b"let byte:", b"let cursor:", 1),
        ),
    )
    for label, mutated_scanner in variants:
        data = predicate + b"\n" + mutated_scanner
        case = semantic_parsed(library, data)
        scanner_node = find_function_by_text(
            data, case[4], case[5], b"lexer_scan_ident"
        )
        assert_rejected_atomically(library, case, scanner_node, label)


def assert_impure_predicate_rejected(library):
    predicate = function_source(b"lexer_is_ident_tail").replace(
        b") -> own Bool pure {", b") -> own Bool traps {", 1
    )
    scanner = function_source(b"lexer_scan_ident")
    data = predicate + b"\n" + scanner
    case = semantic_parsed(library, data)
    scanner_node = find_function_by_text(
        data, case[4], case[5], b"lexer_scan_ident"
    )
    assert_rejected_atomically(
        library, case, scanner_node, "impure predicate"
    )


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_focused_library(Path(raw_directory))
        configure(library)
        assert_real_scanners(library)
        assert_capacity_retry(library)
        assert_invalid_validation_is_atomic(library)
        assert_consistent_renames(library)
        assert_hostile_token_heads(library)
        assert_body_mutations(library)
        assert_impure_predicate_rejected(library)
    print(
        "semantic scanner: real ident/typeid/number loops share one resolved "
        "predicate profile with dense deterministic facts, atomic capacities, "
        "renames, and hostile-mutation rejection"
    )


if __name__ == "__main__":
    main()
