#!/usr/bin/env python3
"""Audit the exact flat indexed-writer capability and its fail-closed fences."""

import ctypes
import tempfile
from pathlib import Path

from test_lexer import build_library
from test_parser import AST_NONE, children_of
from test_semantic_body import (
    AST,
    BODY_CLEAN,
    BODY_UNSUPPORTED,
    SemanticBodyReport,
    assert_output_guards,
    parsed,
)
from test_semantic_unit import (
    CAPABILITY_ACYCLIC,
    CAPABILITY_UNSUPPORTED,
    FIX_NONE,
    RULE_NONE,
    assert_no_capability_diagnostic,
    configure,
    invoke_dispatch,
    make_work,
    top_level_functions,
)


INDEX_WRITER = (
    b"enum WriterIndexMode {\n"
    b"  WriterIndexOwn();\n"
    b"  WriterIndexShared();\n"
    b"}\n"
    b"struct WriterIndexRows {\n"
    b"  type_ids: buffer<u64>;\n"
    b"  modes: buffer<WriterIndexMode>;\n"
    b"}\n"
    b"fn writer_index ['w] (rows: &uniq 'w WriterIndexRows, node: own u64, "
    b"type_id: own u64, mode: own WriterIndexMode) -> own unit "
    b"writes('w), traps {\n"
    b"  set index<u64>(deref(rows).type_ids, node) = type_id;\n"
    b"  set index<WriterIndexMode>(deref(rows).modes, node) = mode;\n"
    b"  set index<WriterIndexMode>(deref(rows).modes, node) = "
    b"WriterIndexOwn();\n"
    b"  return unit;\n"
    b"}\n"
)

LITERAL_INDEX_WRITER = (
    b"struct WriterLiteralRows {\n"
    b"  type_ids: buffer<u64>;\n"
    b"}\n"
    b"fn writer_literal_index ['w] (rows: &uniq 'w WriterLiteralRows, "
    b"value: own u64) -> own unit writes('w), traps {\n"
    b"  set index<u64>(deref(rows).type_ids, 0_u64) = value;\n"
    b"  set index<u64>(deref(rows).type_ids, 1_u64) = value;\n"
    b"  return unit;\n"
    b"}\n"
)

GLOBAL_INDEX_WRITER = (
    b"const writer_index_zero: u64 = 0_u64;\n"
    b"struct WriterGlobalRows {\n"
    b"  type_ids: buffer<u64>;\n"
    b"}\n"
    b"fn writer_global_index ['w] (rows: &uniq 'w WriterGlobalRows, "
    b"value: own u64) -> own unit writes('w), traps {\n"
    b"  set index<u64>(deref(rows).type_ids, writer_index_zero) = value;\n"
    b"  return unit;\n"
    b"}\n"
)


def assert_index_writer_boundary(library):
    def classify(source):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        return case, function, work, kind, report

    def assert_clean(source):
        case, function, work, kind, report = classify(source)
        assert (kind, report.status, report.function, report.related) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            function,
            AST_NONE,
        ), (kind, report.status, report.function, report.related)
        assert_no_capability_diagnostic(report)
        assert_output_guards(work)
        return case, function

    def assert_unsupported(source):
        _, function, work, kind, report = classify(source)
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            function,
        ), (kind, report.status, report.function, report.related)
        assert_no_capability_diagnostic(report)
        assert_output_guards(work)

    assert_clean(INDEX_WRITER)
    assert_clean(LITERAL_INDEX_WRITER)
    assert_clean(GLOBAL_INDEX_WRITER)

    # The indexed target itself exhibits the bounds trap, but it does not read
    # the exclusive root. Both declared rows remain exact in both directions.
    assert_unsupported(INDEX_WRITER.replace(b", traps {", b" {"))
    assert_unsupported(
        INDEX_WRITER.replace(b"writes('w), traps", b"reads('w), writes('w), traps")
    )
    assert_unsupported(
        INDEX_WRITER.replace(b"['w]", b"['w, 'x]").replace(
            b"writes('w)", b"writes('x)"
        )
    )
    assert_unsupported(INDEX_WRITER.replace(b"&uniq 'w", b"&'w"))
    assert_unsupported(
        b"struct WriterIndexRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn writer_index_cross ['w, 'x] (rows: &uniq 'w WriterIndexRows, "
        b"unused: &uniq 'x WriterIndexRows, node: own u64, value: own u64) "
        b"-> own unit writes('w), traps {\n"
        b"  set index<u64>(deref(rows).type_ids, node) = value;\n"
        b"  return unit;\n}\n"
    )

    # A trapping row is not a license for an ordinary field-only writer.
    assert_unsupported(
        b"struct WriterCell {\n  value: u64;\n}\n"
        b"fn writer_field_traps ['w] (cell: &uniq 'w WriterCell, "
        b"value: own u64) -> own unit writes('w), traps {\n"
        b"  set deref(cell).value = value;\n"
        b"  return unit;\n}\n"
    )
    # Targets remain confined to buffer fields below an exclusive struct root.
    # Subscripts are direct own-u64 parameters, canonical u64 literals, or prior
    # direct immutable u64 constants.
    assert_unsupported(
        b"fn writer_direct ['w] (values: &uniq 'w buffer<u8>, node: own u64, "
        b"value: own u8) -> own unit writes('w), traps {\n"
        b"  set index<u8>(deref(values), node) = value;\n"
        b"  return unit;\n}\n"
    )
    assert_unsupported(
        b"struct WriterIndexRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn writer_u8_subscript ['w] (rows: &uniq 'w WriterIndexRows, "
        b"node: own u8, value: own u64) -> own unit writes('w), traps {\n"
        b"  set index<u64>(deref(rows).type_ids, node) = value;\n"
        b"  return unit;\n}\n"
    )
    assert_unsupported(LITERAL_INDEX_WRITER.replace(b"0_u64", b"00_u64"))
    assert_unsupported(LITERAL_INDEX_WRITER.replace(b"0_u64", b"0_u8"))
    assert_unsupported(
        LITERAL_INDEX_WRITER.replace(
            b"0_u64", b"18446744073709551616_u64"
        )
    )
    assert_unsupported(GLOBAL_INDEX_WRITER.replace(b"0_u64", b"00_u64"))
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"0_u64", b"18446744073709551616_u64"
        )
    )
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"const writer_index_zero: u64 = 0_u64;\n", b""
        )
        + b"const writer_index_zero: u64 = 0_u64;\n"
    )
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"const writer_index_zero: u64 = 0_u64;",
            b"const writer_index_zero: u8 = 0_u8;",
        )
    )
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"const writer_index_zero: u64 = 0_u64;",
            b"fn writer_index_zero () -> own u64 pure { return 0_u64; }",
        )
    )
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"value: own u64)",
            b"writer_index_zero: own u8, value: own u64)",
        )
    )
    assert_unsupported(
        GLOBAL_INDEX_WRITER.replace(
            b"value: own u64)",
            b"writer_index_zero: &uniq 'w u64, value: own u64)",
        )
    )

    # Writer-only subscripts do not widen the read-only expression profile.
    assert_unsupported(
        b"struct WriterLiteralRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn reader_literal_index ['r] (rows: &'r WriterLiteralRows) "
        b"-> own u64 reads('r), traps {\n"
        b"  return index<u64>(deref(rows).type_ids, 0_u64);\n"
        b"}\n"
    )
    assert_unsupported(
        b"const writer_index_zero: u64 = 0_u64;\n"
        b"struct WriterGlobalRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn reader_global_index ['r] (rows: &'r WriterGlobalRows) "
        b"-> own u64 reads('r), traps {\n"
        b"  return index<u64>(deref(rows).type_ids, writer_index_zero);\n"
        b"}\n"
    )

    # Element and constructor typing remains exact and nominal.
    assert_unsupported(
        INDEX_WRITER.replace(
            b"index<u64>(deref(rows).type_ids, node) = type_id",
            b"index<u8>(deref(rows).type_ids, node) = type_id",
        )
    )
    assert_unsupported(
        b"enum WriterIndexMode {\n  WriterIndexOwn();\n}\n"
        b"enum WriterIndexOther {\n  WriterIndexOtherOwn();\n}\n"
        b"struct WriterIndexRows {\n  modes: buffer<WriterIndexMode>;\n}\n"
        b"fn writer_wrong_constructor ['w] (rows: &uniq 'w WriterIndexRows, "
        b"node: own u64) -> own unit writes('w), traps {\n"
        b"  set index<WriterIndexMode>(deref(rows).modes, node) = "
        b"WriterIndexOtherOwn();\n"
        b"  return unit;\n}\n"
    )

    # A read/write profile does not inherit indexed targets in this slice, and
    # a hidden RHS read cannot be concealed by the target's trap.
    assert_unsupported(
        b"struct WriterInput {\n  values: buffer<u64>;\n}\n"
        b"struct WriterIndexRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn writer_spurious_read ['r, 'w] (input: &'r WriterInput, "
        b"rows: &uniq 'w WriterIndexRows, node: own u64, value: own u64) "
        b"-> own unit reads('r), writes('w), traps {\n"
        b"  set index<u64>(deref(rows).type_ids, node) = value;\n"
        b"  return unit;\n}\n"
    )
    assert_unsupported(
        b"struct WriterInput {\n  values: buffer<u64>;\n}\n"
        b"struct WriterIndexRows {\n  type_ids: buffer<u64>;\n}\n"
        b"fn writer_hidden_read ['r, 'w] (input: &'r WriterInput, "
        b"rows: &uniq 'w WriterIndexRows, node: own u64) -> own unit "
        b"reads('r), writes('w), traps {\n"
        b"  set index<u64>(deref(rows).type_ids, node) = "
        b"index<u64>(deref(input).values, node);\n"
        b"  return unit;\n}\n"
    )

    def assert_direct_reader_mutation_rejected(mutate, source=INDEX_WRITER):
        case, function = assert_clean(source)
        mutate(case, function)
        work = make_work(library, case[5].count)
        report = SemanticBodyReport(99, 123, 456, 99, 99, 789)
        library.semantic_reader_run(
            case[1],
            ctypes.byref(case[3]),
            ctypes.byref(case[5]),
            ctypes.byref(case[9]),
            function,
            ctypes.byref(work[6]),
            ctypes.byref(report),
        )
        assert (report.status, report.rule, report.fix) == (
            BODY_UNSUPPORTED,
            RULE_NONE,
            FIX_NONE,
        )
        assert_output_guards(work)

    def first_index_target(case, function):
        block = next(
            node
            for node in children_of(case[4], function)
            if case[4][0][node] == AST["AstBlock"]
        )
        statement = next(
            node
            for node in children_of(case[4], block)
            if case[4][0][node] == AST["AstSet"]
        )
        return children_of(case[4], statement)[0]

    def index_targets(case, function):
        block = next(
            node
            for node in children_of(case[4], function)
            if case[4][0][node] == AST["AstBlock"]
        )
        return [
            children_of(case[4], statement)[0]
            for statement in children_of(case[4], block)
            if case[4][0][statement] == AST["AstSet"]
        ]

    def target_kind(case, function):
        target = first_index_target(case, function)
        case[4][0][target] = AST["AstFieldPlace"]

    def target_terminal(case, function):
        target = first_index_target(case, function)
        children = children_of(case[4], target)
        case[4][6][children[-1]] = children[0]

    def type_kind(case, function):
        target = first_index_target(case, function)
        type_node = children_of(case[4], target)[0]
        case[4][0][type_node] = AST["AstParameterType"]

    def base_kind(case, function):
        target = first_index_target(case, function)
        base = children_of(case[4], target)[1]
        case[4][0][base] = AST["AstDerefPlace"]

    def deref_kind(case, function):
        target = first_index_target(case, function)
        base = children_of(case[4], target)[1]
        deref_node = children_of(case[4], base)[0]
        case[4][0][deref_node] = AST["AstPlaceUse"]

    def root_kind(case, function):
        target = first_index_target(case, function)
        base = children_of(case[4], target)[1]
        deref_node = children_of(case[4], base)[0]
        root = children_of(case[4], deref_node)[0]
        case[4][0][root] = AST["AstNumericLiteral"]

    def subscript_kind(case, function):
        target = first_index_target(case, function)
        subscript = children_of(case[4], target)[2]
        case[4][0][subscript] = AST["AstNumericLiteral"]

    def target_head(case, function):
        target = first_index_target(case, function)
        base = children_of(case[4], target)[1]
        deref_node = children_of(case[4], base)[0]
        case[4][1][target] = case[4][1][deref_node]

    def target_same_word_head(case, function):
        targets = index_targets(case, function)
        case[4][1][targets[0]] = case[4][1][targets[1]]

    def field_same_word_head(case, function):
        targets = index_targets(case, function)
        second_base = children_of(case[4], targets[1])[1]
        third_base = children_of(case[4], targets[2])[1]
        case[4][1][second_base] = case[4][1][third_base]

    def subscript_same_word_head(case, function):
        targets = index_targets(case, function)
        first_subscript = children_of(case[4], targets[0])[2]
        second_subscript = children_of(case[4], targets[1])[2]
        case[4][1][first_subscript] = case[4][1][second_subscript]

    def literal_subscript_kind(case, function):
        target = first_index_target(case, function)
        subscript = children_of(case[4], target)[2]
        case[4][0][subscript] = AST["AstPlaceUse"]

    def global_subscript_same_word_head(case, function):
        declaration = next(
            node
            for node in children_of(case[4], case[5].root)
            if case[4][0][node] == AST["AstConstDecl"]
        )
        name = children_of(case[4], declaration)[0]
        target = first_index_target(case, function)
        subscript = children_of(case[4], target)[2]
        case[4][1][subscript] = case[4][1][name]

    for mutate in (
        target_kind,
        target_terminal,
        type_kind,
        base_kind,
        deref_kind,
        root_kind,
        subscript_kind,
        target_head,
        target_same_word_head,
        field_same_word_head,
        subscript_same_word_head,
    ):
        assert_direct_reader_mutation_rejected(mutate)

    literal_same_word = LITERAL_INDEX_WRITER.replace(b"1_u64", b"0_u64")
    assert_direct_reader_mutation_rejected(
        literal_subscript_kind, literal_same_word
    )
    assert_direct_reader_mutation_rejected(
        subscript_same_word_head, literal_same_word
    )
    assert_direct_reader_mutation_rejected(
        global_subscript_same_word_head, GLOBAL_INDEX_WRITER
    )


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_index_writer_boundary(library)
    print(
        "semantic indexed writer: exact exclusive struct-field buffer targets, "
        "own-u64, canonical literal, or prior immutable-u64 subscripts, target "
        "traps, nominal values, and hostile effect, provenance, binding, source "
        "anchoring, shape, topology, and deferred-profile fences pass"
    )


if __name__ == "__main__":
    main()
