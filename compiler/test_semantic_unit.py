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
from test_ast_validate import AstValidationReport, validate
from test_semantic_body import (
    AST,
    BODY_CAPACITY,
    BODY_CLEAN,
    BODY_INVALID_AST_TAPE,
    BODY_INVALID_VALIDATION,
    BODY_UNKNOWN_NAME,
    BODY_UNSUPPORTED,
    SemanticBodyReport,
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
from test_symbols import SYMBOL_NONE, SYMBOL_TYPE, SymbolTape


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
    library.semantic_reader_run.argtypes = [
        Buffer,
        ctypes.POINTER(TokenTape),
        ctypes.POINTER(AstTape),
        ctypes.POINTER(SymbolTape),
        ctypes.c_uint64,
        ctypes.POINTER(SemanticBodyScratch),
        ctypes.POINTER(SemanticBodyReport),
    ]
    library.semantic_reader_run.restype = None
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
        scratch_caps=(scratch_capacity,) * 5,
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


# Source ordinals of the functions the classifier admits CLEAN over the wfc
# corpus. 0-17 are the original fixed-width/scanner/linear/reader-Bool set; the
# remainder are the F1 general-signature + general-enum-match slice, the first
# F2 loop/local-mutation slice, and the bounded F3 writer slices (general
# `own` scalar/enum params, shared buffer borrows, pure or reads+traps effects,
# exhaustive/exact multi-variant enum matches, scalar/enum returns, and typed
# tag-only-enum buffer reads, exact arbitrary-arity call-region substitution,
# structured loop flow, innermost labeled break, owned-let mutation, and exact
# writes-only direct field assignment through one or more same-region exclusive
# struct borrows, plus exact trapping indexed assignment through a buffer field
# of one of those roots with an own-u64 parameter, canonical u64 literal, or
# prior immutable u64 constant subscript, with
# direct own values, canonical u8/u64 literals, prior direct u64 constants, or
# exact nullary tag constructors, plus one exact two-region mixed writer whose
# call RHS values carry explicit read-region attribution, and one exact
# zero-cursor loop that clears indexed rows before direct field tail writes).
# The first F4 profile additionally admits one unique struct parent whose direct
# field children are reborrowed into distinct one-statement local regions and
# passed alone to independently-proven flat writes-only unit callees.
# The second F4 profile admits the exact same-region byte push body and a
# one-statement whole-parent unique reborrow to that independently-proven
# callee, with one canonical u8 literal argument. The third F4 profile admits
# the exact guarded eight-step chunk body whose statement-scoped whole-parent
# calls pass the eight owned u8 parameters to that same independently-proven
# callee. The fourth F4 profile admits an exact one-region wrapper around that
# chunk callee with one whole-parent unique reborrow, one canonical u64 literal,
# and eight canonical u8 literals. Every listed function is validated legal by
# the stage-0 reference checker at build time.
COMPILER_CLEAN_ORDINALS = (
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    21, 22, 23, 24, 29, 30, 31, 34, 63, 64, 65, 88, 89, 90, 91, 96, 98, 102,
    104, 105, 106, 110, 111, 118, 119,
    123, 124, 125, 126, 143, 144, 145, 146, 147, 148, 149, 150, 152, 158, 159,
    162, 163, 164, 175,
    185, 190, 191, 204, 207, 208, 209, 214, 216, 228, 229, 230, 235, 287, 288,
    296, 378, 387, 389, 395, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428,
    430, 431, 441, 442, 443, 444, 445, 446, 447, 450, 451, 452, 455, 456, 457,
    458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 473, 474,
    475, 476, 477, 478, 479, 480, 481, 482, 483, 484, 485, 487, 488, 490, 494,
    497, 498, 499, 501, 502, 503, 504, 509, 516, 536, 551, 579, 596, 597, 619,
)


def assert_compiler_coverage(library):
    data = compiler_source().encode("ascii")
    case = parsed(library, data)
    functions = top_level_functions(case)
    assert len(functions) == 626

    work = make_work(library, case[5].count)
    first = invoke_unit(library, case, work)
    expected = (
        UNIT_CLEAN,
        626,
        152,
        474,
        0,
        functions[18],
        AST_NONE,
    )
    assert unit_report_tuple(first) == expected
    assert_no_unit_diagnostic(first)
    assert function_name(data, case, first.first_unsupported) == (
        b"lexer_scan_string"
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
    assert tuple(clean_ordinals) == COMPILER_CLEAN_ORDINALS

    second = invoke_unit(library, case, work)
    assert unit_report_tuple(second) == unit_report_tuple(first)
    assert_no_unit_diagnostic(second)
    assert_output_guards(work)
    return case, work


def assert_legal_nonprofile_is_unsupported(library):
    # A legal function outside every capability profile stays unsupported. F2
    # admits mutation of an owned `let` binding; mutation of an owned parameter
    # remains outside that bounded capability and therefore stays fail-closed.
    data = (
        b"fn accumulate (value: own u64) -> own u64 traps {\n"
        b"  set value = iadd.trap<u64>(value, 1_u64);\n"
        b"  return value;\n"
        b"}\n"
    )
    case = parsed(library, data)
    (function,) = top_level_functions(case)
    assert len(children_of(case[4], function)) == 6
    work = make_work(library, case[5].count)

    kind, report = invoke_dispatch(library, case, function, work)
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
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

    name_collision = data.replace(b"accumulate", b"lexer_match3")
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


def assert_reader_loops_and_local_set(library):
    def classify(source):
        case = parsed(library, source)
        functions = top_level_functions(case)
        work = make_work(library, case[5].count)
        results = [
            invoke_dispatch(library, case, function, work)
            for function in functions
        ]
        return case, functions, work, results

    def assert_clean(source):
        case, functions, work, results = classify(source)
        for function, (kind, report) in zip(functions, results):
            assert (kind, report.status, report.function, report.related) == (
                CAPABILITY_ACYCLIC,
                BODY_CLEAN,
                function,
                AST_NONE,
            ), (kind, report.status, report.function, report.related)
            assert_no_capability_diagnostic(report)
        assert_output_guards(work)
        return case, functions, work

    def assert_unsupported(source):
        case, functions, work, results = classify(source)
        function = functions[-1]
        kind, report = results[-1]
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            function,
        ), (kind, report.status, report.function, report.related)
        assert_no_capability_diagnostic(report)
        assert_output_guards(work)
        return case

    local_set = (
        b"fn accumulate (value: own u64) -> own u64 traps {\n"
        b"  let total: own u64 = value;\n"
        b"  set total = iadd.trap<u64>(total, 1_u64);\n"
        b"  return total;\n"
        b"}\n"
    )
    assert_clean(local_set)

    loop_break = (
        b"fn count (value: own u64) -> own u64 traps {\n"
        b"  let total: own u64 = value;\n"
        b"  loop @counting {\n"
        b"    set total = iadd.trap<u64>(total, 1_u64);\n"
        b"    match ige<u64>(total, 3_u64) {\n"
        b"      True() => { break @counting; }\n"
        b"      False() => {}\n"
        b"    }\n"
        b"  }\n"
        b"  return total;\n"
        b"}\n"
    )
    assert_clean(loop_break)

    # A no-break loop cannot fall through. This bounded slice requires at
    # least one return path and treats the remaining paths as divergence.
    assert_clean(
        b"fn decide (flag: own Bool) -> own Bool pure {\n"
        b"  loop @deciding {\n"
        b"    match flag {\n"
        b"      True() => { return True(); }\n"
        b"      False() => {}\n"
        b"    }\n"
        b"  }\n"
        b"}\n"
    )

    # Pure divergence has no return witness in the bounded flow summary and
    # stays unsupported until a later slice models it explicitly.
    assert_unsupported(
        b"fn forever () -> own u64 pure {\n"
        b"  loop @forever {}\n"
        b"}\n"
    )

    # Nested loops are admitted when each break names the innermost loop and
    # every label is unique in the function.
    assert_clean(
        b"fn nested (value: own u64) -> own u64 pure {\n"
        b"  loop @outer {\n"
        b"    loop @inner { break @inner; }\n"
        b"    break @outer;\n"
        b"  }\n"
        b"  return value;\n"
        b"}\n"
    )

    # Traps-only signatures and callees are exact effect rows, not an implicit
    # widening of pure functions.
    assert_clean(
        b"fn bump (value: own u64) -> own u64 traps {\n"
        b"  let next: own u64 = value;\n"
        b"  set next = iadd.trap<u64>(next, 1_u64);\n"
        b"  return next;\n"
        b"}\n"
        b"fn forward (value: own u64) -> own u64 traps {\n"
        b"  return bump(value: value);\n"
        b"}\n"
    )

    # Exact local types include Bool but never permit cross-type mutation.
    assert_clean(
        b"fn flip () -> own Bool pure {\n"
        b"  let flag: own Bool = True();\n"
        b"  set flag = False();\n"
        b"  return flag;\n"
        b"}\n"
    )
    assert_unsupported(
        local_set.replace(
            b"iadd.trap<u64>(total, 1_u64)", b"True()"
        )
    )

    # The bounded F2 mutation surface is an owned `let` place. Parameters,
    # unknown places, and loop-local bindings after the loop stay unsupported.
    assert_unsupported(
        b"fn parameter_set (value: own u64) -> own u64 traps {\n"
        b"  set value = iadd.trap<u64>(value, 1_u64);\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn unknown_set (value: own u64) -> own u64 traps {\n"
        b"  set missing = iadd.trap<u64>(value, 1_u64);\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn loop_scope (value: own u64) -> own u64 pure {\n"
        b"  loop @once {\n"
        b"    let inside: own u64 = value;\n"
        b"    break @once;\n"
        b"  }\n"
        b"  return inside;\n"
        b"}\n"
    )

    # Breaks are lexical and terminal. Outer-target breaks in nested loops are
    # conservatively deferred because this slice carries only the innermost
    # target through its control summary.
    assert_unsupported(
        b"fn stray_break (value: own u64) -> own u64 pure {\n"
        b"  break @missing;\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_unsupported(
        loop_break.replace(b"break @counting", b"break @other")
    )
    assert_unsupported(
        b"fn outer_break (value: own u64) -> own u64 pure {\n"
        b"  loop @outer {\n"
        b"    loop @inner { break @outer; }\n"
        b"  }\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn duplicate_labels (value: own u64) -> own u64 pure {\n"
        b"  loop @same { break @same; }\n"
        b"  loop @same { break @same; }\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn after_break (value: own u64) -> own u64 traps {\n"
        b"  let total: own u64 = value;\n"
        b"  loop @once {\n"
        b"    break @once;\n"
        b"    set total = iadd.trap<u64>(total, 1_u64);\n"
        b"  }\n"
        b"  return total;\n"
        b"}\n"
    )

    # EFF-2 remains bidirectional across a set RHS and loop body.
    assert_unsupported(local_set.replace(b"traps {", b"pure {"))
    assert_unsupported(
        b"fn false_traps (value: own u64) -> own u64 traps {\n"
        b"  let total: own u64 = value;\n"
        b"  set total = value;\n"
        b"  return total;\n"
        b"}\n"
    )

    def assert_direct_mutation_rejected(source, mutate):
        case, (function,), baseline_work = assert_clean(source)
        mutate(case)
        hostile_work = make_work(library, case[5].count)
        hostile = SemanticBodyReport(99, 123, 456, 99, 99, 789)
        library.semantic_reader_run(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            ctypes.byref(case[9]),
            function,
            ctypes.byref(hostile_work[6]),
            ctypes.byref(hostile),
        )
        assert (hostile.status, hostile.rule, hostile.fix) == (
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        )
        assert_output_guards(baseline_work)
        assert_output_guards(hostile_work)

    def break_set_target(case):
        statement = next(
            node
            for node in range(case[5].count)
            if case[4][0][node] == AST["AstSet"]
        )
        target = children_of(case[4], statement)[0]
        case[4][0][target] = AST["AstFieldPlace"]

    def break_loop_label(case):
        loop = next(
            node
            for node in range(case[5].count)
            if case[4][0][node] == AST["AstLoop"]
        )
        label = children_of(case[4], loop)[0]
        case[4][0][label] = AST["AstBreakLabel"]

    def break_break_label(case):
        statement = next(
            node
            for node in range(case[5].count)
            if case[4][0][node] == AST["AstBreak"]
        )
        label = children_of(case[4], statement)[0]
        case[4][0][label] = AST["AstLoopLabel"]

    assert_direct_mutation_rejected(local_set, break_set_target)
    assert_direct_mutation_rejected(loop_break, break_loop_label)
    assert_direct_mutation_rejected(loop_break, break_break_label)




def assert_reader_bool_equality_rejected(library):
    # Regression: the acyclic (reader) profile must reject integer-only equality
    # and inequality (ieq/ine, defined by OP-1 over "all int T") applied to Bool
    # operands. Bool equality uses eeq/ene; ieq<Bool> remains ill-typed. An earlier
    # revision set operation_ok unconditionally in the integer-equality branch, so
    # a reader carrying `ieq<Bool>` walked to a false CLEAN. The branch now pins the
    # operand to a scalar, exactly like the ordered-comparison branch.
    def reader_probe(third_let):
        # The head load reads 's and bounds-traps, so the exhibited effect row
        # equals the declared reads('s), traps -- keeping the fixture EFF-2-valid
        # and isolating the Bool-equality operand as the only reason to reject.
        return (
            b"fn reader_probe ['s] (source: &'s buffer<u8>, start: own u64, "
            b"size: own u64) -> own u64 reads('s), traps {\n"
            b"  let head: own u8 = index<u8>(deref(source), start);\n"
            b"  let flag_a: own Bool = ige<u8>(head, 32_u8);\n"
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
        integer_eq = reader_probe(
            b"  let bad: own Bool = " + legal_op + b"<u64>(start, size);\n"
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

    # v0.8 tag-only equality and inequality include Bool and use the distinct
    # eeq/ene family. The otherwise-identical reader shape must be CLEAN.
    for legal_op in (b"eeq", b"ene"):
        bool_eq = reader_probe(
            b"  let bad: own Bool = " + legal_op + b"<Bool>(flag_a, flag_b);\n"
        )
        bool_case = parsed(library, bool_eq)
        (bool_function,) = top_level_functions(bool_case)
        bool_work = make_work(library, bool_case[5].count)
        kind, report = invoke_dispatch(
            library, bool_case, bool_function, bool_work
        )
        assert (kind, report.status, report.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            bool_function,
        ), legal_op
        assert_output_guards(bool_work)

    # eeq/ene do not widen to integers; ieq/ine remain their sole equality
    # family even when both operands have the explicit integer type argument.
    for illegal_op in (b"eeq", b"ene"):
        source = reader_probe(
            b"  let bad: own Bool = " + illegal_op + b"<u64>(start, size);\n"
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
        assert_output_guards(work)


def assert_reader_bool_returns_admitted(library):
    # The acyclic (reader) profile admits own Bool-returning functions in
    # addition to own u64. This exercises the two Bool-only surfaces the
    # extension adds: the unary bnot operator (OP-1: bnot is (Bool)->Bool, the
    # sole unary op) and the True()/False() nullary-constructor literal returns.
    # The negatives pin the guarantees: bnot with a non-Bool type argument, and
    # a Bool-returning function with a falling-through path, must stay
    # Unsupported (fail-closed) -- never a false CLEAN.
    def bool_reader(tail):
        return (
            b"fn reader_bool_probe ['s] (source: &'s buffer<u8>, start: own u64, "
            b"size: own u64) -> own Bool reads('s), traps {\n"
            b"  let flag: own Bool = ige<u64>(start, size);\n"
            b"  match flag {\n"
            b"    True() => {\n"
            b"      return False();\n"
            b"    }\n"
            b"    False() => {\n"
            b"    }\n"
            b"  }\n"
            b"  let byte: own u8 = index<u8>(deref(source), start);\n"
            b"  let hit: own Bool = ieq<u8>(byte, 65_u8);\n"
            + tail
        )

    # Positive: bnot<Bool> and True()/False() literal returns -> clean acyclic.
    clean_source = bool_reader(
        b"  let neg: own Bool = bnot<Bool>(hit);\n"
        b"  match neg {\n"
        b"    True() => {\n"
        b"      return True();\n"
        b"    }\n"
        b"    False() => {\n"
        b"    }\n"
        b"  }\n"
        b"  return False();\n"
        b"}\n"
    )
    clean_case = parsed(library, clean_source)
    (clean_function,) = top_level_functions(clean_case)
    clean_work = make_work(library, clean_case[5].count)
    kind, report = invoke_dispatch(
        library, clean_case, clean_function, clean_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_ACYCLIC,
        BODY_CLEAN,
        clean_function,
    )
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
    assert_output_guards(clean_work)

    # Negative: identical shape and flow, but the bnot type argument is u64.
    # bnot is (Bool)->Bool by OP-1, so bnot<u64> is ill-typed. The only change
    # from the clean case is the operand domain, proving the bnot path pins its
    # operand to Bool rather than admitting the shape.
    wrong_operand = clean_source.replace(
        b"bnot<Bool>(hit)", b"bnot<u64>(hit)"
    )
    wrong_case = parsed(library, wrong_operand)
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
    assert_output_guards(wrong_work)

    # Negative: a Bool-returning function whose False arm falls through -> the
    # block does not return on every path, so the all-paths-return flow gate
    # keeps it Unsupported (not a false CLEAN just because the return type is
    # now admitted).
    non_returning = (
        b"fn reader_bool_probe ['s] (source: &'s buffer<u8>, start: own u64, "
        b"size: own u64) -> own Bool reads('s), traps {\n"
        b"  let flag: own Bool = ige<u64>(start, size);\n"
        b"  match flag {\n"
        b"    True() => {\n"
        b"      return True();\n"
        b"    }\n"
        b"    False() => {\n"
        b"    }\n"
        b"  }\n"
        b"}\n"
    )
    non_returning_case = parsed(library, non_returning)
    (non_returning_function,) = top_level_functions(non_returning_case)
    non_returning_work = make_work(library, non_returning_case[5].count)
    kind, report = invoke_dispatch(
        library, non_returning_case, non_returning_function, non_returning_work
    )
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        non_returning_function,
    )
    assert_output_guards(non_returning_work)

    # Positive: the F1 general-signature slice widens the admitted return set to
    # own u64, own u8, and own Bool. A u8-returning reader whose body returns a
    # bounds-checked byte load is now clean acyclic (own u8 is type-checked end to
    # end: the index place types u8 and the return type-id equality holds).
    u8_return = (
        b"fn reader_bool_probe ['s] (source: &'s buffer<u8>, start: own u64, "
        b"size: own u64) -> own u8 reads('s), traps {\n"
        b"  return index<u8>(deref(source), start);\n"
        b"}\n"
    )
    u8_case = parsed(library, u8_return)
    (u8_function,) = top_level_functions(u8_case)
    u8_work = make_work(library, u8_case[5].count)
    kind, report = invoke_dispatch(library, u8_case, u8_function, u8_work)
    assert (kind, report.status, report.function) == (
        CAPABILITY_ACYCLIC,
        BODY_CLEAN,
        u8_function,
    )
    assert_output_guards(u8_work)


def assert_reader_general_signature_and_enum_match(library):
    # F1 first slice: the acyclic reader admits general signatures (any number of
    # own scalar params u8/u64/Bool, by-value own <Enum> params, shared buffer
    # borrows, multiple region params; pure OR reads(<declared>)+traps; primitive
    # own u64/u8/Bool return) and general multi-variant enum matches that are
    # EXHAUSTIVE and EXACT over the scrutinee enum's declared variants. Positives
    # admit; every negative fails closed (never a false CLEAN).
    def under_test(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        (kind, report) = invoke_dispatch(library, case, function, work)
        return kind, report, function

    def assert_clean(source):
        kind, report, function = under_test(source)
        assert (kind, report.status, report.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            function,
        ), (kind, report.status)
        assert (
            report.rule,
            report.fix,
            report.primary_node,
            report.related_node,
            report.path_count,
        ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, 0)
        assert_path_guard(report)

    def assert_unsupported(source):
        kind, report, function = under_test(source)
        assert report.status == BODY_UNSUPPORTED, (kind, report.status)
        assert kind in (CAPABILITY_ACYCLIC, CAPABILITY_UNSUPPORTED), kind

    sig = b"enum Sig {\n  SigLo();\n  SigMid();\n  SigHi();\n}\n\n"
    other = b"enum Other {\n  OtherA();\n  OtherB();\n}\n\n"

    def rank(arms):
        return sig + b"fn rank (s: own Sig) -> own u64 pure {\n" + arms + b"}\n"

    lo = b"    SigLo() => {\n      return 0_u64;\n    }\n"
    mid = b"    SigMid() => {\n      return 1_u64;\n    }\n"
    hi = b"    SigHi() => {\n      return 2_u64;\n    }\n"

    # Positive: general signature -- several own scalar params (u8/u64/Bool) plus
    # a shared buffer borrow, reads+traps, u64 return, acyclic read-only body. The
    # exhibited row equals the declared reads('s), traps (the byte load reads 's
    # and bounds-traps).
    assert_clean(
        b"fn probe ['s] (source: &'s buffer<u8>, start: own u64, "
        b"a: own u8, b: own u8, flag: own Bool) -> own u64 reads('s), traps {\n"
        b"  let byte: own u8 = index<u8>(deref(source), start);\n"
        b"  let hit: own Bool = ieq<u8>(byte, a);\n"
        b"  let same: own Bool = ieq<u8>(a, b);\n"
        b"  let both: own Bool = band<Bool>(hit, same);\n"
        b"  let gate: own Bool = bor<Bool>(both, flag);\n"
        b"  match gate {\n"
        b"    True() => {\n      return start;\n    }\n"
        b"    False() => {\n      return 0_u64;\n    }\n"
        b"  }\n"
        b"}\n"
    )

    # Positive: multiple region params + multiple shared buffers, own u8 return.
    # The declared reads('a 'b) EQUALS the exhibited row -- the body reads both
    # 'a (via x) and 'b (via y) and traps (bounds-checked index).
    assert_clean(
        b"fn two ['a, 'b] (x: &'a buffer<u8>, y: &'b buffer<u8>, i: own u64) "
        b"-> own u8 reads('a 'b), traps {\n"
        b"  let bx: own u8 = index<u8>(deref(x), i);\n"
        b"  let by: own u8 = index<u8>(deref(y), i);\n"
        b"  let same: own Bool = ieq<u8>(bx, by);\n"
        b"  match same {\n"
        b"    True() => {\n      return bx;\n    }\n"
        b"    False() => {\n      return by;\n    }\n"
        b"  }\n"
        b"}\n"
    )

    # A one-region read-only callee attributes its exhibited row to the exact
    # explicit substitution, including a non-first region of a multi-region
    # caller. The actual shared argument must have that same provenance.
    witness = (
        b"fn c1 ['r] (source: &'r buffer<u8>, at: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(source), at);\n"
        b"}\n"
        b"fn hole ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own u8 reads('a), traps {\n"
        b"  return c1<'a>(source: b, at: at);\n"
        b"}\n"
    )
    kind, report, function = under_test(witness)
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        function,
    ), (kind, report.status)

    assert_clean(
        b"fn c1 ['r] (source: &'r buffer<u8>, at: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(source), at);\n"
        b"}\n"
        b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own u8 reads('b), traps {\n"
        b"  return c1<'b>(source: b, at: at);\n"
        b"}\n"
    )
    assert_clean(
        b"fn c1 ['r] (source: &'r buffer<u8>, at: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(source), at);\n"
        b"}\n"
        b"fn both ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  let x: own u8 = c1<'a>(source: a, at: at);\n"
        b"  let y: own u8 = c1<'b>(source: b, at: at);\n"
        b"  return ieq<u8>(x, y);\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn c1 ['r] (source: &'r buffer<u8>, at: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(source), at);\n"
        b"}\n"
        b"fn wrong ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own u8 reads('a), traps {\n"
        b"  return c1<'b>(source: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        b"fn c1 ['r] (source: &'r buffer<u8>, at: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(source), at);\n"
        b"}\n"
        b"fn wrong ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own u8 reads('b) {\n"
        b"  return c1<'b>(source: b, at: at);\n"
        b"}\n"
    )

    # Reads-without-traps is a distinct exact row. The caller neither gains nor
    # loses traps when it instantiates the callee's sole region with 'b.
    read_only_callee = (
        b"fn width ['r] (source: &'r buffer<u8>) -> own u64 reads('r) {\n"
        b"  return len<u8>(deref(source));\n"
        b"}\n"
    )
    assert_clean(
        read_only_callee
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 reads('b) {\n"
        b"  return width<'b>(source: b);\n"
        b"}\n"
    )
    assert_unsupported(
        read_only_callee
        + b"fn wrong ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 reads('b), traps {\n"
        b"  return width<'b>(source: b);\n"
        b"}\n"
    )

    # Region substitution applies to pure calls too. The explicit 'a argument
    # instantiates c's 'r, so only a place rooted in 'a can satisfy x: &'r.
    pure_callee = (
        b"fn c ['r] (x: &'r buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
    )
    assert_clean(
        pure_callee
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 pure {\n"
        b"  return c<'a>(x: a);\n"
        b"}\n"
    )
    assert_clean(
        pure_callee
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 pure {\n"
        b"  return c<'b>(x: b);\n"
        b"}\n"
    )
    assert_unsupported(
        pure_callee
        + b"fn hole ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 pure {\n"
        b"  return c<'a>(x: b);\n"
        b"}\n"
    )

    # Named own-u64 call arguments use the same bounded canonical parser as
    # ordinary u64 expressions; they are not restricted to one-digit values.
    takes_u64 = (
        b"fn takes_u64 (value: own u64) -> own u64 pure {\n"
        b"  return value;\n"
        b"}\n"
    )
    assert_clean(
        takes_u64
        + b"fn literal_call () -> own u64 pure {\n"
        b"  return takes_u64(value: 12_u64);\n"
        b"}\n"
    )
    assert_unsupported(
        takes_u64
        + b"fn noncanonical_call () -> own u64 pure {\n"
        b"  return takes_u64(value: 01_u64);\n"
        b"}\n"
    )

    # Even when the callee does not use its formal region, the explicit actual
    # must be a declared live caller region; an unused region cannot legalize an
    # undeclared spelling.
    assert_unsupported(
        b"fn unused ['r] () -> own u64 pure {\n  return 0_u64;\n}\n"
        b"fn wrong ['a] (a: &'a buffer<u8>) -> own u64 pure {\n"
        b"  return unused<'z>();\n"
        b"}\n"
    )

    # A shared formal must name the callee's declared region before the call
    # can substitute it. An undeclared formal region fails closed.
    assert_unsupported(
        b"fn malformed ['r] (x: &'q buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
        b"fn relay ['a] (a: &'a buffer<u8>) -> own u64 pure {\n"
        b"  return malformed<'a>(x: a);\n"
        b"}\n"
    )

    # Positive: general 3-variant enum match; every declared variant appears
    # exactly once, tag-only, and every arm returns.
    assert_clean(rank(b"  match s {\n" + lo + mid + hi + b"  }\n"))

    # Negative: non-exhaustive (SigHi missing) -> Unsupported.
    assert_unsupported(rank(b"  match s {\n" + lo + mid + b"  }\n"))

    # Negative: foreign variant (OtherA is a different enum's variant) with the
    # correct arm count -> not an exact bijection -> Unsupported.
    foreign = b"    OtherA() => {\n      return 2_u64;\n    }\n"
    assert_unsupported(
        other + b"fn rank (s: own Sig) -> own u64 pure {\n"
        b"  match s {\n" + lo + mid + foreign + b"  }\n}\n"
    )

    # Negative: duplicate variant (SigLo twice, SigHi missing) with the correct
    # arm count -> Unsupported.
    assert_unsupported(
        rank(b"  match s {\n" + lo + lo.replace(b"0_u64", b"1_u64") + mid + b"  }\n")
    )

    # Negative: exhaustive match but the SigMid arm falls through -> not all paths
    # return -> Unsupported.
    assert_unsupported(
        rank(b"  match s {\n" + lo + b"    SigMid() => {\n    }\n" + hi + b"  }\n")
    )

    # Negative: exclusive (&uniq) borrow parameter is deferred -> Unsupported.
    assert_unsupported(
        b"fn probe ['s] (source: &uniq 's buffer<u8>, start: own u64) "
        b"-> own u64 reads('s), traps {\n"
        b"  return start;\n"
        b"}\n"
    )

    # F2 positive: exact-type mutation of an owned `let` binding.
    assert_clean(
        b"fn probe (x: own u64) -> own u64 traps {\n"
        b"  let y: own u64 = x;\n"
        b"  set y = iadd.trap<u64>(y, 1_u64);\n"
        b"  return y;\n"
        b"}\n"
    )

    # Negative: the loop is structurally supported, but this traps-only row
    # exhibits no trap and therefore remains an EFF-2 mismatch.
    assert_unsupported(
        b"fn probe (x: own u64) -> own u64 traps {\n"
        b"  loop @scan {\n"
        b"    break @scan;\n"
        b"  }\n"
        b"  return x;\n"
        b"}\n"
    )

    # Negative: aggregate (struct) return -> Unsupported.
    assert_unsupported(
        b"struct Pair {\n  lo: u64;\n  hi: u64;\n}\n\n"
        b"fn probe (x: own u64) -> own Pair pure {\n"
        b"  return Pair(lo: x, hi: x);\n"
        b"}\n"
    )

    # Negative: writes effect is outside the read-only slice -> Unsupported.
    assert_unsupported(
        b"fn probe ['s] (buf: &uniq 's buffer<u8>, i: own u64) "
        b"-> own u64 writes('s), traps {\n"
        b"  return i;\n"
        b"}\n"
    )

    # EFF-2 effect-row reconciliation: the DECLARED row must EQUAL the row the
    # body EXHIBITS (both ways). The classifier trusts the declared row as an
    # optimizer fact (EFF-3), so an under-declared effect is a soundness hole.
    # Every mismatch below must be Unsupported (never a false CLEAN).

    # Body reads 'a and bounds-traps, but declares `pure`.
    assert_unsupported(
        b"fn probe ['a] (x: &'a buffer<u8>, i: own u64) -> own u8 pure {\n"
        b"  return index<u8>(deref(x), i);\n"
        b"}\n"
    )

    # Body traps (iadd.trap), but declares `pure`.
    assert_unsupported(
        b"fn probe (start: own u64, size: own u64) -> own u64 pure {\n"
        b"  let s: own u64 = iadd.trap<u64>(start, size);\n"
        b"  return s;\n"
        b"}\n"
    )

    # Body reads 'a, but declares reads('b) -- undeclared read of 'a.
    assert_unsupported(
        b"fn probe ['a, 'b] (x: &'a buffer<u8>, y: &'b buffer<u8>, i: own u64) "
        b"-> own u8 reads('b), traps {\n"
        b"  return index<u8>(deref(x), i);\n"
        b"}\n"
    )

    # Body reads only 's, but declares reads('t) -- undeclared read of 's.
    assert_unsupported(
        b"fn probe ['s, 't] (a: &'s buffer<u8>, b: &'t buffer<u8>, i: own u64) "
        b"-> own u8 reads('t), traps {\n"
        b"  return index<u8>(deref(a), i);\n"
        b"}\n"
    )

    # Over-declared reads: body reads only 'a, but declares reads('a 'b) --
    # declared-but-unexhibited 'b is an EFF-2 error too.
    assert_unsupported(
        b"fn probe ['a, 'b] (x: &'a buffer<u8>, y: &'b buffer<u8>, i: own u64) "
        b"-> own u8 reads('a 'b), traps {\n"
        b"  return index<u8>(deref(x), i);\n"
        b"}\n"
    )

    # Over-declared traps: body is a pure enum map (no index/.trap/call) but
    # declares reads('a), traps -- both directions of the row are wrong.
    assert_unsupported(
        sig +
        b"fn rank ['a] (x: &'a buffer<u8>, s: own Sig) -> own u64 reads('a), "
        b"traps {\n"
        b"  match s {\n" + lo + mid + hi + b"  }\n}\n"
    )


def assert_reader_multi_region_calls(library):
    def under_test(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        assert_output_guards(work)
        return kind, report, function

    def assert_clean(source):
        kind, report, function = under_test(source)
        assert (kind, report.status, report.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            function,
        ), (kind, report.status)
        assert (
            report.rule,
            report.fix,
            report.primary_node,
            report.related_node,
            report.path_count,
        ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, 0)
        assert_path_guard(report)

    def assert_unsupported(source):
        kind, report, function = under_test(source)
        assert report.status == BODY_UNSUPPORTED, (kind, report.status)
        assert kind in (CAPABILITY_ACYCLIC, CAPABILITY_UNSUPPORTED), kind
        assert report.function == function

    pair = (
        b"fn pair ['l, 'r] (left: &'l buffer<u8>, right: &'r buffer<u8>, "
        b"at: own u64) -> own Bool reads('l 'r), traps {\n"
        b"  let lv: own u8 = index<u8>(deref(left), at);\n"
        b"  let rv: own u8 = index<u8>(deref(right), at);\n"
        b"  return ieq<u8>(lv, rv);\n"
        b"}\n"
    )

    # Identity, a positional permutation, and a non-injective substitution all
    # preserve exact shared provenance and instantiate the complete read row.
    assert_clean(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'a, 'b>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_clean(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'b, 'a>(left: b, right: a, at: at);\n"
        b"}\n"
    )
    assert_clean(
        pair
        + b"fn relay ['a] (a: &'a buffer<u8>, at: own u64) "
        b"-> own Bool reads('a), traps {\n"
        b"  return pair<'a, 'a>(left: a, right: a, at: at);\n"
        b"}\n"
    )

    # The same positional mappings reject unswapped or conflicting shared
    # values. Region substitution is ordinal and may repeat, but provenance is
    # never inferred from the value arguments.
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'b, 'a>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a), traps {\n"
        b"  return pair<'a, 'a>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'a, 'a>(left: a, right: a, at: at);\n"
        b"}\n"
    )

    widths = (
        b"fn widths ['l, 'r] (left: &'l buffer<u8>, right: &'r buffer<u8>) "
        b"-> own u64 reads('l 'r) {\n"
        b"  let ll: own u64 = len<u8>(deref(left));\n"
        b"  let rr: own u64 = len<u8>(deref(right));\n"
        b"  return iadd.wrap<u64>(ll, rr);\n"
        b"}\n"
    )
    assert_clean(
        widths
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 reads('a 'b) {\n"
        b"  return widths<'a, 'b>(left: a, right: b);\n"
        b"}\n"
    )
    assert_unsupported(
        widths
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 reads('a 'b), traps {\n"
        b"  return widths<'a, 'b>(left: a, right: b);\n"
        b"}\n"
    )

    pure_pair = (
        b"fn pure_pair ['l, 'r] (left: &'l buffer<u8>, "
        b"right: &'r buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
    )
    assert_clean(
        pure_pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 pure {\n"
        b"  return pure_pair<'b, 'a>(left: b, right: a);\n"
        b"}\n"
    )

    tri = (
        b"fn tri ['x, 'y, 'z] (x: &'x buffer<u8>, y: &'y buffer<u8>, "
        b"z: &'z buffer<u8>, at: own u64) -> own u8 "
        b"reads('x 'y 'z), traps {\n"
        b"  let xv: own u8 = index<u8>(deref(x), at);\n"
        b"  let yv: own u8 = index<u8>(deref(y), at);\n"
        b"  let zv: own u8 = index<u8>(deref(z), at);\n"
        b"  return xv;\n"
        b"}\n"
    )
    assert_clean(
        tri
        + b"fn relay ['a, 'b, 'c] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"c: &'c buffer<u8>, at: own u64) -> own u8 "
        b"reads('a 'b 'c), traps {\n"
        b"  return tri<'c, 'a, 'b>(x: c, y: a, z: b, at: at);\n"
        b"}\n"
    )

    five = (
        b"fn five ['r0, 'r1, 'r2, 'r3, 'r4] () -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
    )
    assert_clean(
        five
        + b"fn relay ['a, 'b, 'c, 'd, 'e] () -> own u64 pure {\n"
        b"  return five<'e, 'd, 'c, 'b, 'a>();\n"
        b"}\n"
    )

    # Exact EFF-2 reconciliation rejects a missing read, an extra read, a
    # missing trap, and a spurious trap after substitution.
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a), traps {\n"
        b"  return pair<'a, 'b>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b, 'c] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"c: &'c buffer<u8>, at: own u64) -> own Bool "
        b"reads('a 'b 'c), traps {\n"
        b"  return pair<'a, 'b>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b) {\n"
        b"  return pair<'a, 'b>(left: a, right: b, at: at);\n"
        b"}\n"
    )

    # Region actuals have exact arity and must all be live caller regions.
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'a>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b, 'c] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"c: &'c buffer<u8>, at: own u64) -> own Bool "
        b"reads('a 'b), traps {\n"
        b"  return pair<'a, 'b, 'c>(left: a, right: b, at: at);\n"
        b"}\n"
    )
    assert_unsupported(
        pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>, "
        b"at: own u64) -> own Bool reads('a 'b), traps {\n"
        b"  return pair<'a, 'z>(left: a, right: b, at: at);\n"
        b"}\n"
    )

    # The narrow fact boundary requires reads of every formal exactly once in
    # declaration order. Subsets, permutations, duplicates, undeclared names,
    # and duplicate formal declarations remain unsupported.
    row_prefix = b"fn row ['l, 'r] (left: &'l buffer<u8>, right: &'r buffer<u8>) -> own u64 "
    row_body = b" {\n  return 0_u64;\n}\n"
    caller = (
        b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 reads('a 'b), traps {\n"
        b"  return row<'a, 'b>(left: a, right: b);\n"
        b"}\n"
    )
    for row in (
        b"reads('l), traps",
        b"reads('r 'l), traps",
        b"reads('l 'l), traps",
        b"reads('l 'q), traps",
    ):
        assert_unsupported(row_prefix + row + row_body + caller)

    duplicate_source = (
        pure_pair
        + b"fn relay ['a, 'b] (a: &'a buffer<u8>, b: &'b buffer<u8>) "
        b"-> own u64 pure {\n"
        b"  return pure_pair<'a, 'b>(left: a, right: b);\n"
        b"}\n"
    )
    duplicate_case = parsed(library, duplicate_source)
    duplicate_callee, duplicate_caller = top_level_functions(duplicate_case)
    duplicate_regions = next(
        child
        for child in children_of(duplicate_case[4], duplicate_callee)
        if duplicate_case[4][0][child] == AST["AstRegionParameters"]
    )
    first_region, second_region = children_of(duplicate_case[4], duplicate_regions)
    first_head = duplicate_case[4][1][first_region]
    second_head = duplicate_case[4][1][second_region]
    first_start = duplicate_case[2][1][first_head]
    first_end = duplicate_case[2][2][first_head]
    second_start = duplicate_case[2][1][second_head]
    second_end = duplicate_case[2][2][second_head]
    assert first_end - first_start == second_end - second_start
    for offset in range(first_end - first_start):
        duplicate_case[0][second_start + offset] = duplicate_case[0][
            first_start + offset
        ]
    duplicate_work = make_work(library, duplicate_case[5].count)
    duplicate_kind, duplicate_report = invoke_dispatch(
        library, duplicate_case, duplicate_caller, duplicate_work
    )
    assert (duplicate_kind, duplicate_report.status, duplicate_report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        duplicate_caller,
    ), (duplicate_kind, duplicate_report.status, duplicate_report.function)
    assert_output_guards(duplicate_work)

    def assert_direct_terminal_rejected(source, mutate):
        case = parsed(library, source)
        caller_function = top_level_functions(case)[-1]
        baseline_work = make_work(library, case[5].count)
        baseline_kind, baseline = invoke_dispatch(
            library, case, caller_function, baseline_work
        )
        assert (baseline_kind, baseline.status, baseline.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            caller_function,
        )
        mutate(case)
        hostile_work = make_work(library, case[5].count)
        hostile = SemanticBodyReport(99, 123, 456, 99, 99, 789)
        library.semantic_reader_run(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            ctypes.byref(case[9]),
            caller_function,
            ctypes.byref(hostile_work[6]),
            ctypes.byref(hostile),
        )
        assert (hostile.status, hostile.rule, hostile.fix) == (
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        )
        assert_output_guards(baseline_work)
        assert_output_guards(hostile_work)

    pure_terminal_source = (
        b"fn callee ['l, 'r] () -> own u64 pure {\n  return 0_u64;\n}\n"
        b"fn caller ['a, 'b] () -> own u64 pure {\n"
        b"  return callee<'a, 'b>();\n}\n"
    )

    def break_formal_terminal(case):
        callee_function = top_level_functions(case)[0]
        regions = next(
            child
            for child in children_of(case[4], callee_function)
            if case[4][0][child] == AST["AstRegionParameters"]
        )
        formal_regions = children_of(case[4], regions)
        case[4][6][formal_regions[-1]] = formal_regions[0]

    def break_call_terminal(case):
        caller_function = top_level_functions(case)[-1]
        caller_block = children_of(case[4], caller_function)[-1]
        caller_return = children_of(case[4], caller_block)[0]
        call = children_of(case[4], caller_return)[0]
        call_children = children_of(case[4], call)
        case[4][6][call_children[-1]] = call_children[0]

    assert_direct_terminal_rejected(pure_terminal_source, break_formal_terminal)
    assert_direct_terminal_rejected(pure_terminal_source, break_call_terminal)

    reads_terminal_source = (
        b"fn callee ['l, 'r] () -> own u64 reads('l 'r), traps {\n"
        b"  return 0_u64;\n}\n"
        b"fn caller ['a, 'b] () -> own u64 reads('a 'b), traps {\n"
        b"  return callee<'a, 'b>();\n}\n"
    )

    def break_reads_terminal(case):
        callee_function = top_level_functions(case)[0]
        reads = next(
            child
            for child in children_of(case[4], callee_function)
            if case[4][0][child] == AST["AstReadsEffect"]
        )
        read_regions = children_of(case[4], reads)
        case[4][6][read_regions[-1]] = read_regions[0]

    assert_direct_terminal_rejected(reads_terminal_source, break_reads_terminal)


def assert_reader_region_argument_leaf(library):
    # The kind-agnostic structural validator permits a uniquely reachable child
    # beneath any node. The semantic call reader must still enforce the grammar
    # invariant that an explicit region argument is a leaf.
    data = (
        b"fn c ['r] (x: &'r buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
        b"fn relay ['a] (a: &'a buffer<u8>) -> own u64 pure {\n"
        b"  return c<'a>(x: a);\n"
        b"}\n"
    )
    case = parsed(library, data)
    callee, caller = top_level_functions(case)
    callee_block = children_of(case[4], callee)[-1]
    stolen_statement = children_of(case[4], callee_block)[0]
    caller_block = children_of(case[4], caller)[-1]
    caller_return = children_of(case[4], caller_block)[0]
    call = children_of(case[4], caller_return)[0]
    region_argument = next(
        child
        for child in children_of(case[4], call)
        if case[4][0][child] == AST["AstRegionArgument"]
    )

    case[4][4][callee_block] = AST_NONE
    case[4][5][callee_block] = AST_NONE
    case[4][4][region_argument] = stolen_statement
    case[4][5][region_argument] = stolen_statement
    case[4][6][stolen_statement] = AST_NONE

    refreshed = validate(library, len(data), case[3].count, case[5])
    assert refreshed.status == 0
    forged = list(case)
    forged[6] = refreshed
    forged = tuple(forged)
    work = make_work(library, forged[5].count)
    kind, report = invoke_dispatch(library, forged, caller, work)
    assert (kind, report.status, report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        caller,
    ), (kind, report.status)
    assert_output_guards(work)

    # Text equality alone is not a region argument: the head token must retain
    # the REGIONID lexical kind even when its bytes still spell 'a.
    token_case = parsed(library, data)
    _, token_caller = top_level_functions(token_case)
    token_block = children_of(token_case[4], token_caller)[-1]
    token_return = children_of(token_case[4], token_block)[0]
    token_call = children_of(token_case[4], token_return)[0]
    token_region_argument = next(
        child
        for child in children_of(token_case[4], token_call)
        if token_case[4][0][child] == AST["AstRegionArgument"]
    )
    token_head = token_case[4][1][token_region_argument]
    token_case[2][0][token_head] = 1
    token_work = make_work(library, token_case[5].count)
    token_kind, token_report = invoke_dispatch(
        library, token_case, token_caller, token_work
    )
    assert (token_kind, token_report.status, token_report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        token_caller,
    ), (token_kind, token_report.status)
    assert_output_guards(token_work)


def assert_reader_callee_signature_singletons(library):
    data = (
        b"fn c ['r] (x: &'r buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
        b"fn relay ['a] (a: &'a buffer<u8>) -> own u64 pure {\n"
        b"  return c<'a>(x: a);\n"
        b"}\n"
    )

    def assert_forged_unsupported(case, caller):
        refreshed = validate(library, len(data), case[3].count, case[5])
        assert refreshed.status == 0
        forged = list(case)
        forged[6] = refreshed
        forged = tuple(forged)
        work = make_work(library, forged[5].count)
        kind, report = invoke_dispatch(library, forged, caller, work)
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            caller,
        ), (kind, report.status)
        assert_output_guards(work)

    # A second region-parameter node cannot replace an earlier malformed one.
    # Keep the tree uniquely reachable while moving the callee's return subtree
    # into a second, well-shaped region declaration.
    region_case = parsed(library, data)
    region_callee, region_caller = top_level_functions(region_case)
    region_children = children_of(region_case[4], region_callee)
    original_regions = next(
        child
        for child in region_children
        if region_case[4][0][child] == AST["AstRegionParameters"]
    )
    original_region = children_of(region_case[4], original_regions)[0]
    region_block = region_children[-1]
    moved_regions = children_of(region_case[4], region_block)[0]
    moved_region = children_of(region_case[4], moved_regions)[0]
    region_case[4][4][region_block] = AST_NONE
    region_case[4][5][region_block] = AST_NONE
    region_case[4][6][region_block] = moved_regions
    region_case[4][5][region_callee] = moved_regions
    region_case[4][6][moved_regions] = AST_NONE
    region_case[4][0][moved_regions] = AST["AstRegionParameters"]
    region_case[4][0][moved_region] = AST["AstRegion"]
    region_case[4][1][original_region], region_case[4][1][moved_region] = (
        region_case[4][1][moved_region],
        region_case[4][1][original_region],
    )
    assert_forged_unsupported(region_case, region_caller)

    # Every other singleton signature component is counted too. A second pure
    # effect leaf must not be accepted merely because it appears last.
    pure_case = parsed(library, data)
    pure_callee, pure_caller = top_level_functions(pure_case)
    pure_children = children_of(pure_case[4], pure_callee)
    original_pure = next(
        child
        for child in pure_children
        if pure_case[4][0][child] == AST["AstPureEffect"]
    )
    pure_block = pure_children[-1]
    return_statement = children_of(pure_case[4], pure_block)[0]
    moved_pure = children_of(pure_case[4], return_statement)[0]
    pure_case[4][4][return_statement] = AST_NONE
    pure_case[4][5][return_statement] = AST_NONE
    pure_case[4][6][pure_block] = moved_pure
    pure_case[4][5][pure_callee] = moved_pure
    pure_case[4][6][moved_pure] = AST_NONE
    pure_case[4][0][moved_pure] = AST["AstPureEffect"]
    pure_case[4][1][original_pure], pure_case[4][1][moved_pure] = (
        pure_case[4][1][moved_pure],
        pure_case[4][1][original_pure],
    )
    assert_forged_unsupported(pure_case, pure_caller)


def assert_reader_callee_signature_shapes(library):
    pure = (
        b"fn c ['r] (x: &'r buffer<u8>) -> own u64 pure {\n"
        b"  return 0_u64;\n"
        b"}\n"
        b"fn relay ['a] (a: &'a buffer<u8>) -> own u64 pure {\n"
        b"  return c<'a>(x: a);\n"
        b"}\n"
    )
    effectful = (
        b"fn c ['r] (x: &'r buffer<u8>, i: own u64) -> own u8 "
        b"reads('r), traps {\n"
        b"  return index<u8>(deref(x), i);\n"
        b"}\n"
        b"fn relay ['a] (x: &'a buffer<u8>, i: own u64) -> own u8 "
        b"reads('a), traps {\n"
        b"  return c<'a>(x: x, i: i);\n"
        b"}\n"
    )

    def assert_mutation_unsupported(label, source, mutate):
        case = parsed(library, source)
        callee, caller = top_level_functions(case)
        direct = children_of(case[4], callee)
        by_kind = {}
        for node in direct:
            by_kind.setdefault(case[4][0][node], []).append(node)
        mutate(case, callee, direct, by_kind)
        refreshed = validate(library, len(source), case[3].count, case[5])
        assert refreshed.status == 0, label
        forged = list(case)
        forged[6] = refreshed
        forged = tuple(forged)
        work = make_work(library, forged[5].count)
        kind, report = invoke_dispatch(library, forged, caller, work)
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            caller,
        ), (label, kind, report.status)
        assert_output_guards(work)

    def attach_block_statement_to(kind_name):
        def mutate(case, callee, direct, by_kind):
            block = by_kind[AST["AstBlock"]][0]
            target = by_kind[AST[kind_name]][0]
            statement = children_of(case[4], block)[0]
            case[4][4][block] = AST_NONE
            case[4][5][block] = AST_NONE
            case[4][4][target] = statement
            case[4][5][target] = statement
            case[4][6][statement] = AST_NONE

        return mutate

    def swap_heads(left_name, right_name):
        def mutate(case, callee, direct, by_kind):
            left = by_kind[AST[left_name]][0]
            right = by_kind[AST[right_name]][0]
            case[4][1][left], case[4][1][right] = (
                case[4][1][right],
                case[4][1][left],
            )

        return mutate

    def move_parameter_before_regions(case, callee, direct, by_kind):
        region = by_kind[AST["AstRegionParameters"]][0]
        parameter = by_kind[AST["AstParameter"]][0]
        order = list(direct)
        region_index = order.index(region)
        parameter_index = order.index(parameter)
        order[region_index], order[parameter_index] = (
            order[parameter_index],
            order[region_index],
        )
        case[4][4][callee] = order[0]
        case[4][5][callee] = order[-1]
        for current, following in zip(order, order[1:]):
            case[4][6][current] = following
        case[4][6][order[-1]] = AST_NONE

    cases = (
        ("pure nonleaf", pure, attach_block_statement_to("AstPureEffect")),
        (
            "traps nonleaf",
            effectful,
            attach_block_statement_to("AstTrapsEffect"),
        ),
        (
            "region parent head",
            pure,
            swap_heads("AstRegionParameters", "AstBlock"),
        ),
        ("return mode head", pure, swap_heads("AstOwnMode", "AstBlock")),
        ("function name head", pure, swap_heads("AstFunctionName", "AstBlock")),
        ("pure effect head", pure, swap_heads("AstPureEffect", "AstBlock")),
        ("parameter before regions", pure, move_parameter_before_regions),
    )
    for label, source, mutate in cases:
        assert_mutation_unsupported(label, source, mutate)

    # The callee root is syntax-bearing too: its head must remain the exact
    # WORD token `fn`, not merely an in-range token with the same bytes.
    root_case = parsed(library, pure)
    root_callee, root_caller = top_level_functions(root_case)
    root_head = root_case[4][1][root_callee]
    root_case[2][0][root_head] = 12
    root_work = make_work(library, root_case[5].count)
    root_kind, root_report = invoke_dispatch(
        library, root_case, root_caller, root_work
    )
    assert (root_kind, root_report.status, root_report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        root_caller,
    ), (root_kind, root_report.status)
    assert_output_guards(root_work)

    # A symbol-table hit must still agree with the callee's validated AST name.
    # Swap two valid identifier heads while preserving unique reachability.
    name_source = (
        b"fn c (x: own u64) -> own u64 pure {\n"
        b"  let other: own u64 = x;\n"
        b"  return other;\n"
        b"}\n"
        b"fn relay (x: own u64) -> own u64 pure {\n"
        b"  return c(x: x);\n"
        b"}\n"
    )
    name_case = parsed(library, name_source)
    name_callee, name_caller = top_level_functions(name_case)
    function_name = next(
        child
        for child in children_of(name_case[4], name_callee)
        if name_case[4][0][child] == AST["AstFunctionName"]
    )
    binding_name = next(
        node
        for node in range(name_case[5].count)
        if name_case[4][0][node] == AST["AstBindingName"]
    )
    name_case[4][1][function_name], name_case[4][1][binding_name] = (
        name_case[4][1][binding_name],
        name_case[4][1][function_name],
    )
    name_validation = validate(
        library, len(name_source), name_case[3].count, name_case[5]
    )
    assert name_validation.status == 0
    forged_name = list(name_case)
    forged_name[6] = name_validation
    forged_name = tuple(forged_name)
    name_work = make_work(library, forged_name[5].count)
    name_kind, name_report = invoke_dispatch(
        library, forged_name, name_caller, name_work
    )
    assert (name_kind, name_report.status, name_report.function) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        name_caller,
    ), (name_kind, name_report.status)
    assert_output_guards(name_work)

    # Distinct formal names are part of the canonical signature. Matching a
    # forged duplicate formal list with forged duplicate named arguments must
    # not make the call appear exact.
    duplicate_source = (
        b"fn c (x: own u64, y: own u64) -> own u64 pure {\n"
        b"  return x;\n"
        b"}\n"
        b"fn relay (a: own u64, b: own u64) -> own u64 pure {\n"
        b"  let x: own u64 = a;\n"
        b"  return c(x: a, y: b);\n"
        b"}\n"
    )
    duplicate_case = parsed(library, duplicate_source)
    duplicate_callee, duplicate_caller = top_level_functions(duplicate_case)
    callee_parameters = [
        child
        for child in children_of(duplicate_case[4], duplicate_callee)
        if duplicate_case[4][0][child] == AST["AstParameter"]
    ]
    callee_block = children_of(duplicate_case[4], duplicate_callee)[-1]
    callee_return = children_of(duplicate_case[4], callee_block)[0]
    callee_place = children_of(duplicate_case[4], callee_return)[0]
    duplicate_case[4][1][callee_parameters[1]], duplicate_case[4][1][callee_place] = (
        duplicate_case[4][1][callee_place],
        duplicate_case[4][1][callee_parameters[1]],
    )
    caller_block = children_of(duplicate_case[4], duplicate_caller)[-1]
    caller_statements = children_of(duplicate_case[4], caller_block)
    caller_binding = next(
        node
        for node in range(duplicate_case[5].count)
        if duplicate_case[4][0][node] == AST["AstBindingName"]
    )
    caller_call = children_of(duplicate_case[4], caller_statements[-1])[0]
    caller_arguments = [
        child
        for child in children_of(duplicate_case[4], caller_call)
        if duplicate_case[4][0][child] == AST["AstNamedArgument"]
    ]
    duplicate_case[4][1][caller_arguments[1]], duplicate_case[4][1][caller_binding] = (
        duplicate_case[4][1][caller_binding],
        duplicate_case[4][1][caller_arguments[1]],
    )
    duplicate_validation = validate(
        library,
        len(duplicate_source),
        duplicate_case[3].count,
        duplicate_case[5],
    )
    assert duplicate_validation.status == 0
    forged_duplicate = list(duplicate_case)
    forged_duplicate[6] = duplicate_validation
    forged_duplicate = tuple(forged_duplicate)
    duplicate_work = make_work(library, forged_duplicate[5].count)
    duplicate_kind, duplicate_report = invoke_dispatch(
        library, forged_duplicate, duplicate_caller, duplicate_work
    )
    assert (
        duplicate_kind,
        duplicate_report.status,
        duplicate_report.function,
    ) == (
        CAPABILITY_UNSUPPORTED,
        BODY_UNSUPPORTED,
        duplicate_caller,
    ), (duplicate_kind, duplicate_report.status)
    assert_output_guards(duplicate_work)


def assert_reader_struct_field_and_typed_index(library):
    # F1 slice 2 (tape reads): the acyclic reader admits reading fields of a
    # borrowed struct -- scalar/enum fields directly (reads(region), no trap) --
    # and buffer<u8|u64> fields via a typed index<T>(deref(structparam).field, i)
    # (reads(region)+traps). The index element T must EXACTLY match the buffer
    # element; every mismatch or under-declaration fails closed (never a false
    # CLEAN).
    def under_test(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        (kind, report) = invoke_dispatch(library, case, function, work)
        return kind, report

    def assert_clean(source):
        kind, report = under_test(source)
        assert (kind, report.status) == (CAPABILITY_ACYCLIC, BODY_CLEAN), (
            kind,
            report.status,
        )

    def assert_unsupported(source):
        kind, report = under_test(source)
        assert report.status == BODY_UNSUPPORTED, (kind, report.status)

    def assert_type_declaration_redirect_unsupported(source, query, target):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        baseline_work = make_work(library, case[5].count)
        baseline_kind, baseline = invoke_dispatch(
            library, case, function, baseline_work
        )
        assert (baseline_kind, baseline.status) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
        ), (query, target, baseline_kind, baseline.status)

        type_slots = {}
        token_starts = case[2][1]
        token_ends = case[2][2]
        for slot in range(case[9].count):
            if (
                case[7][0][slot] != SYMBOL_TYPE
                or case[7][1][slot] != SYMBOL_NONE
            ):
                continue
            name_token = case[7][2][slot]
            spelling = source[
                token_starts[name_token] : token_ends[name_token]
            ]
            if spelling in (query, target):
                assert spelling not in type_slots, (query, target, spelling)
                type_slots[spelling] = slot
        assert set(type_slots) == {query, target}, (query, target, type_slots)
        case[7][3][type_slots[query]] = case[7][3][type_slots[target]]

        redirected_work = make_work(library, case[5].count)
        redirected_kind, redirected = invoke_dispatch(
            library, case, function, redirected_work
        )
        assert (redirected_kind, redirected.status) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
        ), (query, target, redirected_kind, redirected.status)
        assert_output_guards(baseline_work)
        assert_output_guards(redirected_work)

    tape = b"struct Tape {\n  count: u64;\n  data: buffer<u64>;\n  bytes: buffer<u8>;\n}\n\n"
    # scalar field read declaring reads('s) (no trap) -> CLEAN
    assert_clean(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 reads('s) {\n  return deref(t).count;\n}\n"
    )
    # under-declared: field read but declares pure -> reject
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 pure {\n  return deref(t).count;\n}\n"
    )
    # nonexistent field / &uniq struct param -> reject
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 reads('s) {\n  return deref(t).nope;\n}\n"
    )
    assert_unsupported(
        tape + b"fn probe ['s] (t: &uniq 's Tape) -> own u64 reads('s) {\n  return deref(t).count;\n}\n"
    )
    # typed index over buffer<u64>/buffer<u8> fields -> CLEAN
    assert_clean(
        tape + b"fn probe ['s] (t: &'s Tape, i: own u64) -> own u64 reads('s), traps {\n  return index<u64>(deref(t).data, i);\n}\n"
    )
    assert_clean(
        tape + b"fn probe ['s] (t: &'s Tape, i: own u64) -> own u8 reads('s), traps {\n  return index<u8>(deref(t).bytes, i);\n}\n"
    )
    # element mismatch either direction -> reject (no type confusion)
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape, i: own u64) -> own u64 reads('s), traps {\n  return index<u64>(deref(t).bytes, i);\n}\n"
    )
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape, i: own u64) -> own u8 reads('s), traps {\n  return index<u8>(deref(t).data, i);\n}\n"
    )
    # index on a non-buffer (scalar) field -> reject
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape, i: own u64) -> own u64 reads('s), traps {\n  return index<u64>(deref(t).count, i);\n}\n"
    )
    # existing buffer-param index still CLEAN; wrong element on it rejected
    assert_clean(
        b"fn probe ['s] (src: &'s buffer<u8>, i: own u64) -> own u8 reads('s), traps {\n  return index<u8>(deref(src), i);\n}\n"
    )
    assert_unsupported(
        b"fn probe ['s] (src: &'s buffer<u8>, i: own u64) -> own u64 reads('s), traps {\n  return index<u64>(deref(src), i);\n}\n"
    )
    # len<u8|u64> over a buffer field/param: reads(region) with NO trap.
    assert_clean(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 reads('s) {\n  return len<u64>(deref(t).data);\n}\n"
    )
    assert_clean(
        b"fn probe ['s] (b: &'s buffer<u8>) -> own u64 reads('s) {\n  return len<u8>(deref(b));\n}\n"
    )
    # len exhibits reads -> declaring pure is under-declared -> reject
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 pure {\n  return len<u64>(deref(t).data);\n}\n"
    )
    # len element must match the buffer element; non-buffer operand -> reject
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 reads('s) {\n  return len<u8>(deref(t).data);\n}\n"
    )
    assert_unsupported(
        tape + b"fn probe ['s] (t: &'s Tape) -> own u64 reads('s) {\n  return len<u64>(deref(t).count);\n}\n"
    )

    # A structurally valid type-symbol row cannot redirect a queried struct
    # name to a different declaration and expose fields of that other nominal
    # type. The reader binds the lookup result back to its AstStructName.
    assert_type_declaration_redirect_unsupported(
        b"struct AlphaStruct {\n  value: u64;\n}\n\n"
        b"struct BetaStruct {\n  other: u64;\n}\n\n"
        b"fn probe ['s] (item: &'s BetaStruct) -> own u64 reads('s) {\n"
        b"  return deref(item).value;\n"
        b"}\n",
        b"BetaStruct",
        b"AlphaStruct",
    )

    # Enum-typed fields use the same canonical leaf-enum resolver as params,
    # locals, returns, and type arguments; a redirected field type must not be
    # reinterpreted as a same-width distinct nominal enum.
    assert_type_declaration_redirect_unsupported(
        b"enum AlphaTag {\n  AlphaLow();\n  AlphaHigh();\n}\n\n"
        b"enum BetaTag {\n  BetaLow();\n  BetaHigh();\n}\n\n"
        b"struct Holder {\n  tag: BetaTag;\n}\n\n"
        b"fn probe ['s] (item: &'s Holder) -> own AlphaTag reads('s) {\n"
        b"  return deref(item).tag;\n"
        b"}\n",
        b"BetaTag",
        b"AlphaTag",
    )


def assert_reader_enum_values_and_buffers(library):
    # F1 enum-value slice: tag-only enum buffer elements may flow through typed
    # index/len, own locals, own returns, and already-supported user calls. The
    # encoded enum identity remains exact, payload-enum buffers fail closed at
    # the copy boundary, and OP-1 comparisons stay integer-only.
    def under_test(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        return kind, report, function, work

    def assert_clean(source):
        kind, report, function, work = under_test(source)
        assert (kind, report.status, report.function) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            function,
        ), (kind, report.status)
        assert (
            report.rule,
            report.fix,
            report.primary_node,
            report.related_node,
            report.path_count,
        ) == (RULE_NONE, FIX_NONE, AST_NONE, AST_NONE, 0)
        assert_path_guard(report)
        assert_output_guards(work)

    def assert_unsupported(source):
        kind, report, _, work = under_test(source)
        assert report.status == BODY_UNSUPPORTED, (kind, report.status)
        assert kind in (CAPABILITY_ACYCLIC, CAPABILITY_UNSUPPORTED), kind
        assert_path_guard(report)
        assert_output_guards(work)

    tag = b"enum Tag {\n  TagA();\n  TagB();\n}\n\n"
    other = b"enum OtherTag {\n  OtherA();\n  OtherB();\n}\n\n"
    tape = (
        tag
        + other
        + b"struct TagTape {\n  tags: buffer<Tag>;\n}\n\n"
    )

    # v0.8 enum equality is admitted only through eeq/ene on one exact
    # tag-only nominal type. Both operations return Bool.
    for legal_op in (b"eeq", b"ene"):
        assert_clean(
            tag
            + b"fn compare (left: own Tag, right: own Tag) -> own Bool pure {\n"
            b"  return " + legal_op + b"<Tag>(left, right);\n"
            b"}\n"
        )

    # A same-width but distinct enum cannot be smuggled through the explicit
    # Tag type argument: both operands must have that exact nominal type.
    for illegal_op in (b"eeq", b"ene"):
        assert_unsupported(
            tag
            + other
            + b"fn wrong (left: own Tag, right: own OtherTag) -> own Bool pure {\n"
            b"  return " + illegal_op + b"<Tag>(left, right);\n"
            b"}\n"
        )

    # A structurally valid SymbolTape must not redirect one nominal type name
    # to another declaration. The lookup columns and stored name token remain
    # valid in this witness; only OtherTag's declaration ordinal is forged to
    # point at Tag. The reader must bind each resolved declaration back to its
    # canonical AstEnumName before it can classify equality as CLEAN.
    for hostile_op in (b"eeq", b"ene"):
        source = (
            tag
            + other
            + b"fn wrong (left: own Tag, right: own OtherTag) -> own Bool pure {\n"
            b"  return " + hostile_op + b"<Tag>(left, right);\n"
            b"}\n"
        )
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        baseline_work = make_work(library, case[5].count)
        baseline_kind, baseline = invoke_dispatch(
            library, case, function, baseline_work
        )
        assert (baseline_kind, baseline.status) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
        ), (hostile_op, baseline_kind, baseline.status)

        type_slots = {}
        token_starts = case[2][1]
        token_ends = case[2][2]
        for slot in range(case[9].count):
            if (
                case[7][0][slot] != SYMBOL_TYPE
                or case[7][1][slot] != SYMBOL_NONE
            ):
                continue
            name_token = case[7][2][slot]
            spelling = source[
                token_starts[name_token] : token_ends[name_token]
            ]
            if spelling in (b"Tag", b"OtherTag"):
                assert spelling not in type_slots, (hostile_op, spelling)
                type_slots[spelling] = slot
        assert set(type_slots) == {b"Tag", b"OtherTag"}, (
            hostile_op,
            type_slots,
        )
        case[7][3][type_slots[b"OtherTag"]] = case[7][3][type_slots[b"Tag"]]

        hostile_work = make_work(library, case[5].count)
        hostile_kind, hostile = invoke_dispatch(
            library, case, function, hostile_work
        )
        assert (hostile_kind, hostile.status) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
        ), (hostile_op, hostile_kind, hostile.status)
        assert_output_guards(baseline_work)
        assert_output_guards(hostile_work)

    # Own enum locals and return types preserve exact enum identity.
    assert_clean(
        tag
        + b"fn keep (tag: own Tag) -> own Tag pure {\n"
        b"  let held: own Tag = tag;\n"
        b"  return held;\n"
        b"}\n"
    )

    # Enum-returning user-call typing composes with enum locals.
    assert_clean(
        tag
        + b"fn keep (tag: own Tag) -> own Tag pure {\n  return tag;\n}\n"
        b"fn relay (tag: own Tag) -> own Tag pure {\n"
        b"  let result: own Tag = keep(tag: tag);\n"
        b"  return result;\n"
        b"}\n"
    )

    # Typed enum index reads and bounds-traps; typed enum len reads without a
    # trap. Both accept only the field's exact tag-only enum element.
    assert_clean(
        tape
        + b"fn read ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  let result: own Tag = index<Tag>(deref(t).tags, i);\n"
        b"  return result;\n"
        b"}\n"
    )
    assert_clean(
        tape
        + b"fn width ['s] (t: &'s TagTape) -> own u64 reads('s) {\n"
        b"  return len<Tag>(deref(t).tags);\n"
        b"}\n"
    )

    # An enum-returning effectful call retains the already-supported single
    # region's exact reads+traps row.
    assert_clean(
        tape
        + b"fn read ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  return index<Tag>(deref(t).tags, i);\n"
        b"}\n"
        b"fn relay ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  let result: own Tag = read<'s>(t: t, i: i);\n"
        b"  return result;\n"
        b"}\n"
    )

    # Exact type identity: a distinct enum, an integer, a struct, and an
    # unresolved type argument cannot reinterpret buffer<Tag>.
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own OtherTag "
        b"reads('s), traps {\n"
        b"  return index<OtherTag>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own u64 "
        b"reads('s), traps {\n"
        b"  return index<u64>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  return index<TagTape>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  return index<MissingTag>(deref(t).tags, i);\n"
        b"}\n"
    )

    # Enum locals and returns also require exact identity.
    assert_unsupported(
        tag
        + other
        + b"fn wrong (tag: own Tag) -> own Tag pure {\n"
        b"  let other: own OtherTag = tag;\n"
        b"  return tag;\n"
        b"}\n"
    )
    assert_unsupported(
        tag
        + other
        + b"fn wrong (tag: own Tag) -> own OtherTag pure {\n"
        b"  return tag;\n"
        b"}\n"
    )

    # Today's wfc parser accepts only nullary variant declarations. Exercise the
    # payload boundary adversarially: start from a CLEAN source, attach a child
    # to one AstVariant in the already indexed tape, then invoke the reader
    # directly so the stale-validation guard does not mask its own fail-closed
    # tag-only check.
    def assert_hostile_payload_rejected(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        baseline_work = make_work(library, case[5].count)
        baseline_kind, baseline = invoke_dispatch(
            library, case, function, baseline_work
        )
        assert (baseline_kind, baseline.status) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
        ), (baseline_kind, baseline.status)
        enum_declarations = [
            node
            for node in children_of(case[4], case[5].root)
            if case[4][0][node] == AST["AstEnumDecl"]
        ]
        assert enum_declarations
        enum_children = children_of(case[4], enum_declarations[0])
        enum_names = [
            node for node in enum_children if case[4][0][node] == AST["AstEnumName"]
        ]
        enum_variants = [
            node for node in enum_children if case[4][0][node] == AST["AstVariant"]
        ]
        assert len(enum_names) == 1 and enum_variants
        payload_variant = enum_variants[0]
        case[4][4][payload_variant] = enum_names[0]
        case[4][5][payload_variant] = enum_names[0]

        hostile_work = make_work(library, case[5].count)
        hostile = SemanticBodyReport(99, 123, 456, 99, 99, 789)
        library.semantic_reader_run(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            ctypes.byref(case[9]),
            function,
            ctypes.byref(hostile_work[6]),
            ctypes.byref(hostile),
        )
        assert (hostile.status, hostile.rule, hostile.fix) == (
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        )
        assert_output_guards(baseline_work)
        assert_output_guards(hostile_work)

    # Centralized resolution rejects payload-like enums at parameter, return,
    # local/type-argument, index, and len boundaries.
    assert_hostile_payload_rejected(
        tag
        + b"fn param (tag: own Tag) -> own u64 pure {\n  return 0_u64;\n}\n"
    )
    assert_hostile_payload_rejected(
        tape
        + b"fn result ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s), traps {\n"
        b"  return index<Tag>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_hostile_payload_rejected(
        tape
        + b"fn local ['s] (t: &'s TagTape, i: own u64) -> own u64 "
        b"reads('s), traps {\n"
        b"  let tag: own Tag = index<Tag>(deref(t).tags, i);\n"
        b"  return i;\n"
        b"}\n"
    )
    assert_hostile_payload_rejected(
        tape
        + b"fn width ['s] (t: &'s TagTape) -> own u64 reads('s) {\n"
        b"  return len<Tag>(deref(t).tags);\n"
        b"}\n"
    )
    for hostile_op in (b"eeq", b"ene"):
        assert_hostile_payload_rejected(
            tag
            + b"fn compare (left: own Tag, right: own Tag) -> own Bool pure {\n"
            b"  return " + hostile_op + b"<Tag>(left, right);\n"
            b"}\n"
        )

    # Integer equality and ordering remain closed over integer types. Bool
    # rejection is pinned separately; ieq/ine and ordering reject enums here.
    for illegal_op in (b"ieq", b"ine", b"ilt", b"ile", b"igt", b"ige"):
        assert_unsupported(
            tag
            + b"fn wrong (left: own Tag, right: own Tag) -> own Bool pure {\n"
            b"  return " + illegal_op + b"<Tag>(left, right);\n"
            b"}\n"
        )

    # Exact EFF-2 rows remain type-agnostic: index needs reads+traps, while len
    # needs reads and specifically does not exhibit traps.
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own Tag pure {\n"
        b"  return index<Tag>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape, i: own u64) -> own Tag "
        b"reads('s) {\n"
        b"  return index<Tag>(deref(t).tags, i);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape) -> own u64 pure {\n"
        b"  return len<Tag>(deref(t).tags);\n"
        b"}\n"
    )
    assert_unsupported(
        tape
        + b"fn wrong ['s] (t: &'s TagTape) -> own u64 reads('s), traps {\n"
        b"  return len<Tag>(deref(t).tags);\n"
        b"}\n"
    )


def assert_reader_enum_type_id_partition_guard():
    # Constructing an AST with at least 1e9 addressable nodes is not practical
    # in this test process. Pin the central pre-encoding guard instead: no enum
    # declaration index may cross from the enum ID interval into the struct ID
    # interval before the declaration index is added to the enum base.
    reader = (Path(__file__).parent / "src" / "semantic_reader.wf").read_text()
    start = reader.index("fn semantic_reader_leaf_enum_id")
    end = reader.index("\nfn ", start + 3)
    helper = reader[start:end]
    limit = (
        "let enum_decl_limit: own u64 = isub.wrap<u64>("
        "semantic_reader_struct_type_base, semantic_reader_enum_type_base);"
    )
    guard = (
        "let enum_decl_in_range: own Bool = "
        "ilt<u64>(enum_decl, enum_decl_limit);"
    )
    encode = "iadd.trap<u64>(enum_decl, semantic_reader_enum_type_base)"
    assert limit in helper
    assert guard in helper
    assert helper.index(limit) < helper.index(guard) < helper.index(encode)


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
        626,
        152,
        474,
        0,
        top_level_functions(case)[18],
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
        assert_reader_loops_and_local_set(library)
        assert_reader_bool_equality_rejected(library)
        assert_reader_bool_returns_admitted(library)
        assert_reader_general_signature_and_enum_match(library)
        assert_reader_multi_region_calls(library)
        assert_reader_region_argument_leaf(library)
        assert_reader_callee_signature_singletons(library)
        assert_reader_callee_signature_shapes(library)
        assert_reader_struct_field_and_typed_index(library)
        assert_reader_enum_values_and_buffers(library)
        assert_reader_enum_type_id_partition_guard()
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
        "semantic unit: compiler 626 total / 152 clean / 474 unsupported / "
        "0 rejected; exact clean ordinals, source-order frontier, legal "
        "nonprofile, reader bool-equality rejection, reader bool-return "
        "admission, exact arbitrary-arity call-region attribution, general signatures "
        "and multi-variant enum matches, "
        "structured loops and owned-let mutation, "
        "enum values and tag-only-enum buffer reads, "
        "flat field, exact trapping indexed writers, exact indexed "
        "row-clearing control writers, and exact one-child statement-scoped "
        "reborrow writers, exact same-region byte push and whole-parent-u8 "
        "reborrow calls, exact guarded eight-byte chunk reborrows and exact "
        "one-region fixed-literal chunk wrappers, "
        "structural rename, real "
        "reject, deterministic repeat, "
        "fresh validation, bounded paths, transactional diagnostics, "
        "dynamic/hostile capacities, inputs, and guards pass"
    )


if __name__ == "__main__":
    main()
