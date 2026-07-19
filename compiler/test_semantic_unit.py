#!/usr/bin/env python3
"""Audit the pure semantic capability frontier across a complete source unit."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import Buffer, TokenTape, build_library, compiler_source
from test_parser import (
    AST_FUNCTION,
    AST_FUNCTION_NAME,
    AST_NONE,
    AstTape,
    children_of,
)
from test_ast_validate import AstValidationReport
from test_semantic_body import (
    BODY_CAPACITY,
    BODY_CLEAN,
    BODY_INVALID_AST_TAPE,
    BODY_INVALID_VALIDATION,
    BODY_UNKNOWN_NAME,
    BODY_UNSUPPORTED,
    SemanticBodyScratch,
    assert_output_guards,
    configure as configure_semantic_body,
    find_function_by_text,
    fixture,
    make_outputs,
    parsed,
    user_call_fixture,
)
from test_semantic_facts import NodeFacts, TypeTape
from test_symbols import SymbolTape


UNIT_CLEAN = 0
UNIT_INVALID_TOKEN_TAPE = 1
UNIT_INVALID_AST_TAPE = 2
UNIT_INVALID_VALIDATION = 3
UNIT_INVALID_SYMBOL_TAPE = 4
UNIT_CAPACITY = 6
UNIT_INFRASTRUCTURE = 7

CAPABILITY_UNSUPPORTED = 4
CAPABILITY_FAILED = 0
CAPABILITY_LINEAR = 3
CAPABILITY_ACYCLIC = 5

RULE_NONE = 0
RULE_FORM7 = 1
RULE_GRAM11 = 2
RULE_TYPE5 = 3
RULE_TYPE6 = 4
RULE_TYPE7 = 5
RULE_EFFECT2 = 6

FIX_NONE = 0
FIX_CANONICAL_LITERAL = 1
FIX_NAME_ARGUMENTS = 2
FIX_MATCH_TYPE = 3
FIX_DECLARE_BEFORE_USE = 4
FIX_RENAME_BINDING = 5
FIX_DEREF_BORROW = 6
FIX_DECLARE_EFFECT = 7

PATH_POISON = 0xD1D1D1D1D1D1D1D1
PATH_GUARD = 0xE2E2E2E2E2E2E2E2

DIAGNOSTIC_READY = 0
DIAGNOSTIC_NONE = 1
DIAGNOSTIC_CAPACITY = 2
DIAGNOSTIC_INVALID_AST = 3


class SemanticCapabilityReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("function", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("rule", ctypes.c_int32),
        ("fix", ctypes.c_int32),
        ("primary_node", ctypes.c_uint64),
        ("related_node", ctypes.c_uint64),
        ("span_start", ctypes.c_uint64),
        ("span_end", ctypes.c_uint64),
        ("path", Buffer),
        ("path_count", ctypes.c_uint64),
    ]


class SemanticCapabilityResult(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("function", ctypes.c_uint64),
        ("related", ctypes.c_uint64),
        ("rule", ctypes.c_int32),
        ("fix", ctypes.c_int32),
        ("primary_node", ctypes.c_uint64),
        ("related_node", ctypes.c_uint64),
    ]


class SemanticDiagnosticPathResult(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("count", ctypes.c_uint64),
    ]


class SemanticUnitReport(ctypes.Structure):
    _fields_ = [
        ("status", ctypes.c_int32),
        ("total", ctypes.c_uint64),
        ("clean", ctypes.c_uint64),
        ("unsupported", ctypes.c_uint64),
        ("rejected", ctypes.c_uint64),
        ("first_unsupported", ctypes.c_uint64),
        ("first_rejected", ctypes.c_uint64),
        ("diagnostic_rule", ctypes.c_int32),
        ("diagnostic_fix", ctypes.c_int32),
        ("diagnostic_primary_node", ctypes.c_uint64),
        ("diagnostic_related_node", ctypes.c_uint64),
        ("diagnostic_span_start", ctypes.c_uint64),
        ("diagnostic_span_end", ctypes.c_uint64),
        ("diagnostic_path", Buffer),
        ("diagnostic_path_count", ctypes.c_uint64),
    ]


def configure(library):
    configure_semantic_body(library)
    common = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SymbolTape),
    ]
    work = [
        ctypes.POINTER(TypeTape),
        ctypes.POINTER(NodeFacts),
        ctypes.POINTER(SemanticBodyScratch),
    ]
    library.semantic_unit_dispatch.argtypes = (
        common
        + [ctypes.c_uint64]
        + work
        + [ctypes.POINTER(SemanticCapabilityReport)]
    )
    library.semantic_unit_dispatch.restype = ctypes.c_int32
    library.semantic_unit_run.argtypes = (
        common + work + [ctypes.POINTER(SemanticUnitReport)]
    )
    library.semantic_unit_run.restype = None
    library.semantic_diagnostic_write_path.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
        ctypes.c_uint64,
        Buffer,
    ]
    library.semantic_diagnostic_write_path.restype = SemanticDiagnosticPathResult
    library.semantic_diagnostic_publish_capability.argtypes = [
        ctypes.c_uint64,
        ctypes.c_uint64,
        ctypes.POINTER(AstTape),
        ctypes.POINTER(AstValidationReport),
        ctypes.POINTER(SemanticCapabilityResult),
        ctypes.POINTER(SemanticCapabilityReport),
    ]
    library.semantic_diagnostic_publish_capability.restype = ctypes.c_int32


def make_work(
    library,
    ast_count,
    *,
    type_capacity=4,
    fact_capacity=None,
    scratch_capacity=34,
):
    if fact_capacity is None:
        fact_capacity = ast_count
    return make_outputs(
        library,
        ast_count,
        type_caps=(type_capacity,) * 6,
        fact_caps=(fact_capacity,) * 8,
        scratch_caps=(scratch_capacity,) * 4,
    )


def unit_report_tuple(report):
    return (
        report.status,
        report.total,
        report.clean,
        report.unsupported,
        report.rejected,
        report.first_unsupported,
        report.first_rejected,
    )


def make_path(capacity):
    storage = (ctypes.c_uint64 * (capacity + 1))()
    for index in range(capacity):
        storage[index] = PATH_POISON
    storage[capacity] = PATH_GUARD
    return storage, Buffer(ctypes.cast(storage, ctypes.c_void_p), capacity)


def path_tuple(report, *, unit=False):
    storage = report._path_storage
    count = report.diagnostic_path_count if unit else report.path_count
    return tuple(storage[:count])


def assert_path_guard(report):
    assert report._path_storage[report._path_capacity] == PATH_GUARD


def assert_path_poisoned(report):
    assert all(
        value == PATH_POISON
        for value in report._path_storage[: report._path_capacity]
    )


def assert_no_capability_diagnostic(report):
    assert (
        report.rule,
        report.fix,
        report.primary_node,
        report.related_node,
        report.span_start,
        report.span_end,
        report.path_count,
    ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, AST_NONE, AST_NONE, 0)
    assert_path_guard(report)


def assert_no_unit_diagnostic(report):
    assert (
        report.diagnostic_rule,
        report.diagnostic_fix,
        report.diagnostic_primary_node,
        report.diagnostic_related_node,
        report.diagnostic_span_start,
        report.diagnostic_span_end,
        report.diagnostic_path_count,
    ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, AST_NONE, AST_NONE, 0)
    assert_path_guard(report)


def make_capability_report(path_capacity):
    path_storage, path = make_path(path_capacity)
    report = SemanticCapabilityReport(
        99, 123, 456, 99, 99, 123, 456, 789, 987, path, 99
    )
    report._path_storage = path_storage
    report._path_capacity = path_capacity
    return report


def invoke_unit(
    library,
    case,
    work,
    *,
    tokens=None,
    ast=None,
    validation=None,
    symbols=None,
    path_capacity=None,
):
    if tokens is None:
        tokens = case[3]
    if ast is None:
        ast = case[5]
    if validation is None:
        validation = case[6]
    if symbols is None:
        symbols = case[9]
    if path_capacity is None:
        path_capacity = ast.count
    path_storage, path = make_path(path_capacity)
    report = SemanticUnitReport(
        0x5A,
        *([0x5A] * 6),
        0x5A,
        0x5A,
        *([0x5A] * 4),
        path,
        0x5A,
    )
    library.semantic_unit_run(
        case[1],
        ctypes.byref(tokens),
        ctypes.byref(ast),
        ctypes.byref(validation),
        ctypes.byref(symbols),
        ctypes.byref(work[1]),
        ctypes.byref(work[3]),
        ctypes.byref(work[6]),
        ctypes.byref(report),
    )
    report._path_storage = path_storage
    report._path_capacity = path_capacity
    return report


def invoke_dispatch(library, case, function, work, *, path_capacity=None):
    if path_capacity is None:
        path_capacity = case[5].count
    path_storage, path = make_path(path_capacity)
    report = SemanticCapabilityReport(
        99, 123, 456, 99, 99, 123, 456, 789, 987, path, 99
    )
    kind = library.semantic_unit_dispatch(
        case[1],
        ctypes.byref(case[3]),
        ctypes.byref(case[5]),
        ctypes.byref(case[6]),
        ctypes.byref(case[9]),
        function,
        ctypes.byref(work[1]),
        ctypes.byref(work[3]),
        ctypes.byref(work[6]),
        ctypes.byref(report),
    )
    report._path_storage = path_storage
    report._path_capacity = path_capacity
    return kind, report


def clone_structure(value, **changes):
    fields = {
        name: getattr(value, name)
        for name, _ in value._fields_
    }
    fields.update(changes)
    return type(value)(*(fields[name] for name, _ in value._fields_))


def top_level_functions(case):
    columns = case[4]
    ast = case[5]
    return tuple(
        node
        for node in children_of(columns, ast.root)
        if columns[0][node] == AST_FUNCTION
    )


def function_name(data, case, function):
    columns = case[4]
    names = [
        node
        for node in children_of(columns, function)
        if columns[0][node] == AST_FUNCTION_NAME
    ]
    assert len(names) == 1, (function, names)
    name = names[0]
    return data[columns[2][name] : columns[3][name]]


def canonical_path(columns, root, target):
    reverse = []
    current = target
    while current != root:
        found = None
        for parent in range(len(columns[0])):
            children = children_of(columns, parent)
            if current in children:
                found = (parent, children.index(current))
                break
        assert found is not None, (root, target, current)
        current, ordinal = found
        reverse.append(ordinal)
    return tuple(reversed(reverse))


def assert_compiler_coverage(library):
    data = compiler_source().encode("ascii")
    case = parsed(library, data)
    functions = top_level_functions(case)
    assert len(functions) == 510

    work = make_work(library, case[5].count)
    first = invoke_unit(library, case, work)
    expected = (
        UNIT_CLEAN,
        510,
        17,
        493,
        0,
        functions[17],
        AST_NONE,
    )
    assert unit_report_tuple(first) == expected
    assert_no_unit_diagnostic(first)
    assert function_name(data, case, first.first_unsupported) == (
        b"lexer_ampuniq_at"
    )

    clean_ordinals = []
    for ordinal, function in enumerate(functions):
        _, report = invoke_dispatch(library, case, function, work)
        assert (
            report.rule,
            report.fix,
            report.primary_node,
            report.related_node,
            report.span_start,
            report.span_end,
            report.path_count,
        ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, AST_NONE, AST_NONE, 0)
        assert_path_guard(report)
        if report.status == BODY_CLEAN:
            clean_ordinals.append(ordinal)
        else:
            assert report.status == BODY_UNSUPPORTED, (
                ordinal,
                function_name(data, case, function),
                report.status,
                report.function,
                report.related,
            )
    assert tuple(clean_ordinals) == tuple(range(17))

    second = invoke_unit(library, case, work)
    assert unit_report_tuple(second) == unit_report_tuple(first)
    assert_no_unit_diagnostic(second)
    assert_output_guards(work)
    return case, work


def assert_legal_nonprofile_is_unsupported(library):
    data = (
        b"fn passthrough (value: own u64) -> own u64 pure {\n"
        b"  return value;\n"
        b"}\n"
    )
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    assert len(children_of(case[4], function)) == 6
    work = make_work(library, case[5].count)

    kind, report = invoke_dispatch(library, case, function, work)
    assert (kind, report.status, report.function, report.related) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        function,
        function,
    )
    assert (
        report.rule,
        report.fix,
        report.primary_node,
        report.related_node,
        report.path_count,
    ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, 0)
    assert_path_guard(report)
    unit = invoke_unit(library, case, work)
    assert unit_report_tuple(unit) == (
        UNIT_CLEAN,
        1,
        0,
        1,
        0,
        function,
        AST_NONE,
    )
    assert_no_unit_diagnostic(unit)
    assert_output_guards(work)

    name_collision = data.replace(b"passthrough", b"lexer_match3")
    collision_case = parsed(library, name_collision)
    (collision_function,) = top_level_functions(collision_case)
    collision_work = make_work(library, collision_case[5].count)
    kind, report = invoke_dispatch(
        library, collision_case, collision_function, collision_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        collision_function,
    )

    wrong_fixed_body = (
        b"fn lexer_match3 ['s] (source: &'s buffer<u8>, start: own u64, "
        b"a: own u8, b: own u8, c: own u8) -> own Bool "
        b"reads('s), traps {\n"
        b"  return True();\n"
        b"}\n"
    )
    wrong_case = parsed(library, wrong_fixed_body)
    (wrong_function,) = top_level_functions(wrong_case)
    wrong_work = make_work(library, wrong_case[5].count)
    kind, report = invoke_dispatch(
        library, wrong_case, wrong_function, wrong_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        wrong_function,
    )
    assert_output_guards(collision_work)
    assert_output_guards(wrong_work)


def assert_reader_bool_equality_rejected(library):
    # Regression: the acyclic (reader) profile must reject integer-only equality
    # and inequality (ieq/ine, defined by OP-1 over "all int T") applied to Bool
    # operands. There is no Bool equality op -- ieq<Bool> is ill-typed. An earlier
    # revision set operation_ok unconditionally in the equality branch, so a reader
    # carrying `ieq<Bool>` walked to a false CLEAN. The branch now pins the operand
    # to a scalar, exactly like the ordered-comparison branch.
    def reader_probe(third_let):
        return (
            b"fn reader_probe ['s] (source: &'s buffer<u8>, start: own u64, "
            b"size: own u64) -> own u64 reads('s), traps {\n"
            b"  let flag_a: own Bool = ige<u64>(start, size);\n"
            b"  let flag_b: own Bool = ilt<u64>(start, size);\n"
            + third_let
            + b"  match bad {\n"
            b"    True() => {\n"
            b"      return start;\n"
            b"    }\n"
            b"    False() => {\n"
            b"      return size;\n"
            b"    }\n"
            b"  }\n"
            b"}\n"
        )

    # Illegal: ieq/ine on Bool operands -> Unsupported (not a false CLEAN).
    for illegal_op in (b"ieq", b"ine"):
        source = reader_probe(
            b"  let bad: own Bool = " + illegal_op + b"<Bool>(flag_a, flag_b);\n"
        )
        case = parsed(library, source)
        (function,) = top_level_functions(case)
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            function,
        ), illegal_op
        assert (
            report.rule,
            report.fix,
            report.primary_node,
            report.related_node,
            report.span_start,
            report.span_end,
            report.path_count,
        ) == (
            RULE_NONE,
            FIX_NONE,
            AST_NONE,
            AST_NONE,
            AST_NONE,
            AST_NONE,
            0,
        ), illegal_op
        assert_path_guard(report)
        assert_output_guards(work)

    # Legal Bool logical op with identical signature and control flow -> clean
    # acyclic, proving the signature and flow are admitted and the rejection above
    # targets only the Bool equality operand, not the shape.
    legal_bool = reader_probe(
        b"  let bad: own Bool = band<Bool>(flag_a, flag_b);\n"
    )
    legal_case = parsed(library, legal_bool)
    (legal_function,) = top_level_functions(legal_case)
    legal_work = make_work(library, legal_case[5].count)
    kind, report = invoke_dispatch(
        library, legal_case, legal_function, legal_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_ACYCLIC,
        BODY_CLEAN,
        legal_function,
    )
    assert_output_guards(legal_work)

    # Legal integer equality/inequality -> clean acyclic, proving ieq/ine remain
    # admitted on the integer operands OP-1 allows (the fix narrows, not removes).
    for legal_op in (b"ieq", b"ine"):
        integer_eq = (
            b"fn reader_probe ['s] (source: &'s buffer<u8>, start: own u64, "
            b"size: own u64) -> own u64 reads('s), traps {\n"
            b"  let bad: own Bool = " + legal_op + b"<u64>(start, size);\n"
            b"  match bad {\n"
            b"    True() => {\n"
            b"      return start;\n"
            b"    }\n"
            b"    False() => {\n"
            b"      return size;\n"
            b"    }\n"
            b"  }\n"
            b"}\n"
        )
        integer_case = parsed(library, integer_eq)
        (integer_function,) = top_level_functions(integer_case)
        integer_work = make_work(library, integer_case[5].count)
        kind, report = invoke_dispatch(
            library, integer_case, integer_function, integer_work
        )
        assert (kind, report.status, report.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            integer_function,
        ), legal_op
        assert_output_guards(integer_work)


def assert_structural_profile_and_real_reject(library):
    renamed = fixture().replace(b"lexer_is_lower", b"arbitrary_predicate")
    renamed_case = parsed(library, renamed)
    (renamed_function,) = top_level_functions(renamed_case)
    renamed_work = make_work(library, renamed_case[5].count)
    kind, report = invoke_dispatch(
        library, renamed_case, renamed_function, renamed_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_LINEAR,
        BODY_CLEAN,
        renamed_function,
    )

    rejected = fixture(first_operand=b"missing")
    rejected_case = parsed(library, rejected)
    (rejected_function,) = top_level_functions(rejected_case)
    rejected_work = make_work(library, rejected_case[5].count)
    kind, report = invoke_dispatch(
        library, rejected_case, rejected_function, rejected_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_FAILED,
        BODY_UNKNOWN_NAME,
        rejected_function,
    )
    primary = report.primary_node
    assert report.rule == RULE_TYPE5
    assert report.fix == FIX_DECLARE_BEFORE_USE
    assert report.related_node == AST_NONE
    assert (report.span_start, report.span_end) == (
        rejected_case[4][2][primary],
        rejected_case[4][3][primary],
    )
    expected_path = canonical_path(
        rejected_case[4], rejected_case[5].root, primary
    )
    assert path_tuple(report) == expected_path
    assert_path_guard(report)
    unit = invoke_unit(library, rejected_case, rejected_work)
    assert unit_report_tuple(unit) == (
        UNIT_CLEAN,
        1,
        0,
        0,
        1,
        AST_NONE,
        rejected_function,
    )
    assert (
        unit.diagnostic_rule,
        unit.diagnostic_fix,
        unit.diagnostic_primary_node,
        unit.diagnostic_related_node,
        unit.diagnostic_span_start,
        unit.diagnostic_span_end,
    ) == (
        RULE_TYPE5,
        FIX_DECLARE_BEFORE_USE,
        primary,
        AST_NONE,
        rejected_case[4][2][primary],
        rejected_case[4][3][primary],
    )
    assert path_tuple(unit, unit=True) == expected_path
    assert_path_guard(unit)
    assert_output_guards(renamed_work)
    assert_output_guards(rejected_work)


def assert_diagnostic_path_capacity(library):
    data = fixture(first_operand=b"missing")
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    work = make_work(library, case[5].count)
    short_capacity = case[5].count - 1

    kind, capability = invoke_dispatch(
        library,
        case,
        function,
        work,
        path_capacity=short_capacity,
    )
    assert (kind, capability.status, capability.rule, capability.path_count) == (
        CAPABILITY_FAILED,
        BODY_CAPACITY,
        RULE_NONE,
        0,
    )
    assert all(
        value == PATH_POISON
        for value in capability._path_storage[:short_capacity]
    )
    assert_path_guard(capability)

    unit = invoke_unit(
        library,
        case,
        work,
        path_capacity=short_capacity,
    )
    assert unit.status == UNIT_CAPACITY
    assert_no_unit_diagnostic(unit)
    assert all(
        value == PATH_POISON for value in unit._path_storage[:short_capacity]
    )
    assert_output_guards(work)


def assert_site_specific_rule_ids(library):
    data = user_call_fixture(arguments=b"missing: c")
    case = parsed(library, data)
    caller = find_function_by_text(data, case[4], case[5], b"caller")
    work = make_work(library, case[5].count)

    kind, capability = invoke_dispatch(library, case, caller, work)
    assert (kind, capability.status) == (CAPABILITY_FAILED, BODY_UNKNOWN_NAME)
    assert (capability.rule, capability.fix) == (
        RULE_GRAM11,
        FIX_NAME_ARGUMENTS,
    )
    assert capability.related_node < case[5].count
    assert path_tuple(capability) == canonical_path(
        case[4], case[5].root, capability.primary_node
    )
    assert_path_guard(capability)

    unit = invoke_unit(library, case, work)
    assert unit_report_tuple(unit) == (
        UNIT_CLEAN,
        2,
        1,
        0,
        1,
        AST_NONE,
        caller,
    )
    assert (unit.diagnostic_rule, unit.diagnostic_fix) == (
        RULE_GRAM11,
        FIX_NAME_ARGUMENTS,
    )
    assert path_tuple(unit, unit=True) == canonical_path(
        case[4], case[5].root, unit.diagnostic_primary_node
    )
    assert_path_guard(unit)
    assert_output_guards(work)


def assert_stale_validation_is_rebound(library):
    data = fixture(first_operand=b"missing")
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    work = make_work(library, case[5].count)
    _, baseline = invoke_dispatch(library, case, function, work)
    primary = baseline.primary_node
    case[4][2][primary] = len(data) + 100

    kind, report = invoke_dispatch(library, case, function, work)
    assert (kind, report.status, case[6].status) == (
        CAPABILITY_FAILED,
        BODY_INVALID_VALIDATION,
        8,
    )
    assert_no_capability_diagnostic(report)
    assert_path_poisoned(report)

    unit = invoke_unit(library, case, work)
    assert unit_report_tuple(unit) == (
        UNIT_INVALID_VALIDATION,
        0,
        0,
        0,
        0,
        AST_NONE,
        AST_NONE,
    )
    assert_no_unit_diagnostic(unit)
    assert_path_poisoned(unit)
    assert_output_guards(work)


def assert_forged_edges_fail_closed(library):
    data = fixture(first_operand=b"missing")
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    block = children_of(case[4], function)[5]
    statement = children_of(case[4], block)[0]
    path_storage, path = make_path(case[5].count)
    path_result = library.semantic_diagnostic_write_path(
        len(data),
        case[3].count,
        ctypes.byref(case[5]),
        ctypes.byref(case[6]),
        statement,
        path,
    )
    expected = canonical_path(case[4], case[5].root, statement)
    assert (path_result.status, path_result.count) == (
        DIAGNOSTIC_READY,
        len(expected),
    )
    assert tuple(path_storage[: path_result.count]) == expected
    assert path_storage[-1] == PATH_GUARD

    for label, edge in (("cycle", "cycle"), ("out-of-range", "range")):
        data = fixture(first_operand=b"missing")
        case = parsed(library, data)
        (function,) = top_level_functions(case)
        block = children_of(case[4], function)[5]
        statement = children_of(case[4], block)[0]
        if edge == "cycle":
            case[4][6][statement] = statement
        else:
            case[4][6][statement] = case[5].count

        path_storage, path = make_path(case[5].count)
        path_result = library.semantic_diagnostic_write_path(
            len(data),
            case[3].count,
            ctypes.byref(case[5]),
            ctypes.byref(case[6]),
            statement,
            path,
        )
        assert (path_result.status, path_result.count) == (
            DIAGNOSTIC_INVALID_AST,
            0,
        ), label
        assert all(value == PATH_POISON for value in path_storage[:-1]), label
        assert path_storage[-1] == PATH_GUARD, label

        root_path_storage, root_path = make_path(case[5].count)
        root_path_result = library.semantic_diagnostic_write_path(
            len(data),
            case[3].count,
            ctypes.byref(case[5]),
            ctypes.byref(case[6]),
            case[5].root,
            root_path,
        )
        assert (root_path_result.status, root_path_result.count) == (
            DIAGNOSTIC_INVALID_AST,
            0,
        ), label
        assert all(
            value == PATH_POISON for value in root_path_storage[:-1]
        ), label
        assert root_path_storage[-1] == PATH_GUARD, label

        root_result = SemanticCapabilityResult(
            BODY_UNKNOWN_NAME,
            function,
            AST_NONE,
            RULE_TYPE5,
            FIX_DECLARE_BEFORE_USE,
            case[5].root,
            AST_NONE,
        )
        root_report = make_capability_report(case[5].count)
        root_published = library.semantic_diagnostic_publish_capability(
            len(data),
            case[3].count,
            ctypes.byref(case[5]),
            ctypes.byref(case[6]),
            ctypes.byref(root_result),
            ctypes.byref(root_report),
        )
        assert (root_published, root_report.status) == (
            DIAGNOSTIC_INVALID_AST,
            BODY_INVALID_AST_TAPE,
        ), label
        assert_no_capability_diagnostic(root_report)
        assert_path_poisoned(root_report)

        work = make_work(library, case[5].count)
        unit = invoke_unit(library, case, work)
        assert unit_report_tuple(unit) == (
            UNIT_INVALID_VALIDATION,
            0,
            0,
            0,
            0,
            AST_NONE,
            AST_NONE,
        ), label
        assert case[6].status != 0, label
        assert_no_unit_diagnostic(unit)
        assert_path_poisoned(unit)
        assert_output_guards(work)


def assert_publisher_validates_nodes(library):
    data = fixture(first_operand=b"missing")
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    work = make_work(library, case[5].count)
    _, baseline = invoke_dispatch(library, case, function, work)

    invalid_related = SemanticCapabilityResult(
        BODY_UNKNOWN_NAME,
        function,
        AST_NONE,
        RULE_TYPE5,
        FIX_DECLARE_BEFORE_USE,
        baseline.primary_node,
        case[5].count,
    )
    report = make_capability_report(case[5].count)
    published = library.semantic_diagnostic_publish_capability(
        len(data),
        case[3].count,
        ctypes.byref(case[5]),
        ctypes.byref(case[6]),
        ctypes.byref(invalid_related),
        ctypes.byref(report),
    )
    assert (published, report.status) == (
        DIAGNOSTIC_INVALID_AST,
        BODY_INVALID_AST_TAPE,
    )
    assert_no_capability_diagnostic(report)
    assert_path_poisoned(report)

    invalid_ast = clone_structure(
        case[5], count=case[5].kinds.length + 1
    )
    no_rule = SemanticCapabilityResult(
        BODY_INVALID_AST_TAPE,
        function,
        invalid_ast.count,
        RULE_NONE,
        FIX_NONE,
        invalid_ast.count,
        invalid_ast.count,
    )
    report = make_capability_report(case[5].count)
    published = library.semantic_diagnostic_publish_capability(
        len(data),
        case[3].count,
        ctypes.byref(invalid_ast),
        ctypes.byref(case[6]),
        ctypes.byref(no_rule),
        ctypes.byref(report),
    )
    assert (published, report.status) == (
        DIAGNOSTIC_NONE,
        BODY_INVALID_AST_TAPE,
    )
    assert_no_capability_diagnostic(report)
    assert_path_poisoned(report)
    assert_output_guards(work)


def assert_late_failure_discards_diagnostic(library):
    statements = [b"  let value0: own Bool = ige<u8>(c, 0_u8);\n"]
    for ordinal in range(1, 34):
        previous = ordinal - 1
        statements.append(
            f"  let value{ordinal}: own Bool = bor<Bool>("
            f"value{previous}, value{previous});\n".encode("ascii")
        )
    later_capacity = (
        b"fn many_values (c: own u8) -> own Bool pure {\n"
        + b"".join(statements)
        + b"  return value33;\n"
        + b"}\n"
    )
    data = fixture(first_operand=b"missing") + later_capacity
    case = parsed(library, data)
    work = make_work(library, case[5].count, scratch_capacity=34)
    report = invoke_unit(library, case, work)
    assert unit_report_tuple(report) == (
        UNIT_CAPACITY,
        0,
        0,
        0,
        0,
        AST_NONE,
        AST_NONE,
    )
    assert_no_unit_diagnostic(report)
    assert_path_poisoned(report)
    assert_output_guards(work)


def assert_dynamic_linear_capacity(library):
    statements = [
        b"  let value0: own Bool = ige<u8>(c, 0_u8);\n"
    ]
    for ordinal in range(1, 34):
        previous = ordinal - 1
        statements.append(
            f"  let value{ordinal}: own Bool = bor<Bool>("
            f"value{previous}, value{previous});\n".encode("ascii")
        )
    data = (
        b"fn many_values (c: own u8) -> own Bool pure {\n"
        + b"".join(statements)
        + b"  return value33;\n"
        + b"}\n"
    )
    case = parsed(library, data)
    short = make_work(library, case[5].count, scratch_capacity=34)
    report = invoke_unit(library, case, short)
    assert unit_report_tuple(report) == (
        UNIT_CAPACITY,
        0,
        0,
        0,
        0,
        AST_NONE,
        AST_NONE,
    )

    exact = make_work(library, case[5].count, scratch_capacity=35)
    report = invoke_unit(library, case, exact)
    assert unit_report_tuple(report) == (
        UNIT_CLEAN,
        1,
        1,
        0,
        0,
        AST_NONE,
        AST_NONE,
    )
    assert_output_guards(short)
    assert_output_guards(exact)


def assert_canonical_failure(library, case, work, status, **changes):
    report = invoke_unit(library, case, work, **changes)
    assert unit_report_tuple(report) == (
        status,
        0,
        0,
        0,
        0,
        AST_NONE,
        AST_NONE,
    )
    assert_output_guards(work)


def assert_hostile_inputs_and_capacities(library, case, full_work):
    tokens = clone_structure(
        case[3], count=case[3].kinds.length + 1
    )
    assert_canonical_failure(
        library,
        case,
        full_work,
        UNIT_INVALID_TOKEN_TAPE,
        tokens=tokens,
    )

    ast = clone_structure(case[5], count=case[5].kinds.length + 1)
    assert_canonical_failure(
        library,
        case,
        full_work,
        UNIT_INVALID_AST_TAPE,
        ast=ast,
    )

    validation = clone_structure(case[6], status=1, node=case[5].root)
    refreshed = invoke_unit(
        library,
        case,
        full_work,
        validation=validation,
    )
    assert unit_report_tuple(refreshed) == (
        UNIT_CLEAN,
        510,
        17,
        493,
        0,
        top_level_functions(case)[17],
        AST_NONE,
    )
    assert validation.status == 0
    assert_no_unit_diagnostic(refreshed)

    symbols = clone_structure(case[9], count=case[9].namespaces.length + 1)
    assert_canonical_failure(
        library,
        case,
        full_work,
        UNIT_INVALID_SYMBOL_TAPE,
        symbols=symbols,
    )

    short_work = (
        make_work(library, case[5].count, type_capacity=3),
        make_work(
            library,
            case[5].count,
            fact_capacity=case[5].count - 1,
        ),
        make_work(library, case[5].count, scratch_capacity=33),
    )
    for work in short_work:
        assert_canonical_failure(
            library, case, work, UNIT_CAPACITY
        )


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        case, work = assert_compiler_coverage(library)
        assert_legal_nonprofile_is_unsupported(library)
        assert_reader_bool_equality_rejected(library)
        assert_structural_profile_and_real_reject(library)
        assert_diagnostic_path_capacity(library)
        assert_site_specific_rule_ids(library)
        assert_stale_validation_is_rebound(library)
        assert_forged_edges_fail_closed(library)
        assert_publisher_validates_nodes(library)
        assert_late_failure_discards_diagnostic(library)
        assert_dynamic_linear_capacity(library)
        assert_hostile_inputs_and_capacities(library, case, work)
    print(
        "semantic unit: compiler 510 total / 17 clean / 493 unsupported / "
        "0 rejected; exact clean ordinals, source-order frontier, legal "
        "nonprofile, reader bool-equality rejection, structural rename, real "
        "reject, deterministic repeat, "
        "fresh validation, bounded paths, transactional diagnostics, "
        "dynamic/hostile capacities, inputs, and guards pass"
    )


if __name__ == "__main__":
    main()
