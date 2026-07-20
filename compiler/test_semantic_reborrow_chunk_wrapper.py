#!/usr/bin/env python3
"""Audit exact one-region fixed-literal wrappers around the chunk callee."""

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
from test_semantic_reborrow_chunk import REBORROW_CHUNK, replace_last
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


WRAPPER = (
    b"fn emit_fixed ['o] (out: &uniq 'o PushTape) "
    b"-> own unit reads('o), writes('o), traps {\n"
    b"  region 'fixed_chunk {\n"
    b"    emit_chunk(out: &uniq 'fixed_chunk deref(out), count: 4_u64, "
    b"b0: 119_u8, b1: 102_u8, b2: 99_u8, b3: 32_u8, "
    b"b4: 32_u8, b5: 32_u8, b6: 32_u8, b7: 32_u8);\n"
    b"  }\n"
    b"  return unit;\n"
    b"}\n"
)
REBORROW_CHUNK_WRAPPER = REBORROW_CHUNK + WRAPPER

PAIR_WRAPPER = (
    b"fn emit_pair ['o] (out: &uniq 'o PushTape) "
    b"-> own unit reads('o), writes('o), traps {\n"
    b"  region 'pair_head {\n"
    b"    emit_chunk(out: &uniq 'pair_head deref(out), count: 8_u64, "
    b"b0: 101_u8, b1: 120_u8, b2: 116_u8, b3: 114_u8, "
    b"b4: 97_u8, b5: 99_u8, b6: 116_u8, b7: 118_u8);\n"
    b"  }\n"
    b"  region 'pair_tail {\n"
    b"    emit_chunk(out: &uniq 'pair_tail deref(out), count: 3_u64, "
    b"b0: 98_u8, b1: 108_u8, b2: 101_u8, b3: 32_u8, "
    b"b4: 32_u8, b5: 32_u8, b6: 32_u8, b7: 32_u8);\n"
    b"  }\n"
    b"  return unit;\n"
    b"}\n"
)
REBORROW_CHUNK_PAIR_WRAPPER = REBORROW_CHUNK + PAIR_WRAPPER


def assert_reborrow_chunk_wrapper_boundary(library):
    def classify(source=REBORROW_CHUNK_WRAPPER):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        return case, function, work, kind, report

    def assert_clean(source=REBORROW_CHUNK_WRAPPER):
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

    assert_clean()
    assert_clean(
        REBORROW_CHUNK_WRAPPER.replace(b"push_byte", b"put_octet")
        .replace(b"emit_chunk", b"send_octets")
        .replace(b"emit_fixed", b"send_fixed")
        .replace(b"'fixed_chunk", b"'fixed_scope")
    )
    assert_clean(
        REBORROW_CHUNK_WRAPPER.replace(b"count: 4_u64", b"count: 9_u64", 1)
    )
    assert_clean(REBORROW_CHUNK_PAIR_WRAPPER)
    assert_clean(
        REBORROW_CHUNK_PAIR_WRAPPER.replace(b"push_byte", b"put_octet")
        .replace(b"emit_chunk", b"send_octets")
        .replace(b"emit_pair", b"send_pair")
        .replace(b"'pair_head", b"'first_scope")
        .replace(b"'pair_tail", b"'second_scope")
    )

    # The paired profile is exact: both sequential statement-confined calls
    # are independently proven, their local regions are distinct, and a third
    # region or any composite statement remains outside the boundary.
    for source in (
        replace_last(
            REBORROW_CHUNK_PAIR_WRAPPER,
            b"out: &uniq 'o PushTape",
            b"out: &'o PushTape",
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"count: 3_u64", b"count: 03_u64", 1
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"b0: 98_u8", b"b0: 256_u8", 1
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"count: 3_u64, b0: 98_u8",
            b"b0: 98_u8, count: 3_u64",
            1,
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"&uniq 'pair_tail deref(out)",
            b"&'pair_tail deref(out)",
            1,
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"&uniq 'pair_tail deref(out)",
            b"&uniq 'pair_tail deref(out).count",
            1,
        ),
        replace_last(
            REBORROW_CHUNK_PAIR_WRAPPER,
            b"emit_chunk(out:",
            b"emit_chunk<'pair_tail>(out:",
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"    emit_chunk(out: &uniq 'pair_tail",
            b"    let ignored: own unit = emit_chunk(out: &uniq 'pair_tail",
            1,
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"b7: 32_u8);\n  }\n  return unit;",
            b"b7: 32_u8);\n    return unit;\n  }\n  return unit;",
            1,
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"    emit_chunk(out: &uniq 'pair_head",
            b"    push_byte(out: &uniq 'pair_head",
            1,
        ),
        replace_last(
            REBORROW_CHUNK_PAIR_WRAPPER,
            b"emit_chunk(out:",
            b"missing_chunk(out:",
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"'pair_tail", b"'pair_head"
        ),
        REBORROW_CHUNK_PAIR_WRAPPER.replace(
            b"'pair_tail", b"'o"
        ),
        replace_last(
            REBORROW_CHUNK_PAIR_WRAPPER,
            b"  return unit;\n}\n",
            b"  region 'pair_third {\n"
            b"    emit_chunk(out: &uniq 'pair_third deref(out), "
            b"count: 0_u64, b0: 0_u8, b1: 0_u8, b2: 0_u8, b3: 0_u8, "
            b"b4: 0_u8, b5: 0_u8, b6: 0_u8, b7: 0_u8);\n"
            b"  }\n  return unit;\n}\n",
        ),
    ):
        assert_unsupported(source)

    # The caller keeps the exact one-parent same-region signature.
    for source in (
        replace_last(
            REBORROW_CHUNK_WRAPPER,
            b"out: &uniq 'o PushTape",
            b"out: &'o PushTape",
        ),
        replace_last(
            REBORROW_CHUNK_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"writes('o), traps {",
        ),
        replace_last(
            REBORROW_CHUNK_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"reads('o), traps {",
        ),
        replace_last(
            REBORROW_CHUNK_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"reads('o), writes('o) {",
        ),
    ):
        assert_unsupported(source)

    # All ten named arguments are in formal order. Count is a canonical u64;
    # every byte is a canonical in-range u8 literal.
    for old, new in (
        (b"count: 4_u64", b"count: 04_u64"),
        (b"count: 4_u64", b"count: 4_u8"),
        (b"b0: 119_u8", b"b0: 0119_u8"),
        (b"b0: 119_u8", b"b0: 119_u64"),
        (b"b0: 119_u8", b"b0: 256_u8"),
        (b"count: 4_u64, b0: 119_u8", b"b0: 119_u8, count: 4_u64"),
        (b"b0: 119_u8, b1: 102_u8", b"b1: 102_u8, b0: 119_u8"),
        (b"b7: 32_u8", b"last: 32_u8"),
        (b"&uniq 'fixed_chunk deref(out)", b"&'fixed_chunk deref(out)"),
        (b"&uniq 'fixed_chunk deref(out)", b"&uniq 'other deref(out)"),
        (b"&uniq 'fixed_chunk deref(out)", b"&uniq 'fixed_chunk deref(out).count"),
        (b"emit_chunk(out:", b"emit_chunk<'fixed_chunk>(out:"),
    ):
        assert_unsupported(REBORROW_CHUNK_WRAPPER.replace(old, new, 1))
    assert_unsupported(
        REBORROW_CHUNK_WRAPPER.replace(
            b"    emit_chunk(out: &uniq 'fixed_chunk",
            b"    let ignored: own unit = emit_chunk(out: &uniq 'fixed_chunk",
            1,
        )
    )
    assert_unsupported(
        REBORROW_CHUNK_WRAPPER.replace(
            b"b7: 32_u8);\n",
            b"b7: 32_u8);\n    return unit;\n",
            1,
        )
    )
    # A declaration alone never lends effects: both the chunk and its nested
    # push callee are independently re-proven.
    assert_unsupported(
        REBORROW_CHUNK_WRAPPER.replace(
            b"ile<u64>(count, 8_u64)", b"ilt<u64>(count, 8_u64)", 1
        )
    )
    assert_unsupported(
        REBORROW_CHUNK_WRAPPER.replace(
            b"ilt<u64>(slot, capacity)", b"ile<u64>(slot, capacity)", 1
        )
    )

    def nodes(case, function, region_ordinal=0):
        columns = case[4]
        direct = children_of(columns, function)
        outer_region = children_of(columns, direct[1])[0]
        outer_parameter = direct[2]
        reads_region = children_of(columns, direct[5])[0]
        writes_region = children_of(columns, direct[6])[0]
        region_block = children_of(columns, direct[8])[region_ordinal]
        local_region, inner_block = children_of(columns, region_block)
        expression = children_of(columns, inner_block)[0]
        call = children_of(columns, expression)[0]
        arguments = children_of(columns, call)
        output_argument = arguments[0]
        borrow = children_of(columns, output_argument)[0]
        child_region, deref_place = children_of(columns, borrow)
        parent = children_of(columns, deref_place)[0]
        count_argument = arguments[1]
        count_literal = children_of(columns, count_argument)[0]
        byte_argument = arguments[2]
        byte_literal = children_of(columns, byte_argument)[0]
        callee = top_level_functions(case)[1]
        callee_direct = children_of(columns, callee)
        return {
            "outer_region": outer_region,
            "outer_parameter": outer_parameter,
            "reads_region": reads_region,
            "writes_region": writes_region,
            "region_block": region_block,
            "local_region": local_region,
            "child_region": child_region,
            "call": call,
            "callee_name": callee_direct[0],
            "output_argument": output_argument,
            "callee_output": callee_direct[2],
            "count_argument": count_argument,
            "callee_count": callee_direct[3],
            "byte_argument": byte_argument,
            "callee_byte": callee_direct[4],
            "parent": parent,
            "count_literal": count_literal,
            "byte_literal": byte_literal,
            "borrow": borrow,
        }

    def assert_direct_mutation_rejected(
        mutate, source=REBORROW_CHUNK_WRAPPER
    ):
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

    def redirect_head(target, source):
        def mutate(case, function):
            selected = nodes(case, function)
            case[4][1][selected[target]] = case[4][1][selected[source]]

        return mutate

    def shared_borrow(case, function):
        selected = nodes(case, function)
        case[4][0][selected["borrow"]] = AST["AstSharedBorrow"]

    def argument_cycle(case, function):
        selected = nodes(case, function)
        case[4][6][selected["output_argument"]] = selected["output_argument"]

    def pair_region_cycle(case, function):
        selected = nodes(case, function)
        case[4][6][selected["region_block"]] = selected["region_block"]

    for mutate in (
        redirect_head("reads_region", "outer_region"),
        redirect_head("writes_region", "outer_region"),
        redirect_head("local_region", "child_region"),
        redirect_head("child_region", "local_region"),
        redirect_head("call", "callee_name"),
        redirect_head("callee_name", "call"),
        redirect_head("output_argument", "callee_output"),
        redirect_head("callee_output", "output_argument"),
        redirect_head("count_argument", "callee_count"),
        redirect_head("callee_count", "count_argument"),
        redirect_head("byte_argument", "callee_byte"),
        redirect_head("callee_byte", "byte_argument"),
        redirect_head("parent", "outer_parameter"),
        redirect_head("outer_parameter", "parent"),
        redirect_head("count_literal", "count_argument"),
        redirect_head("byte_literal", "byte_argument"),
        shared_borrow,
        argument_cycle,
    ):
        assert_direct_mutation_rejected(mutate)
    assert_direct_mutation_rejected(
        pair_region_cycle, source=REBORROW_CHUNK_PAIR_WRAPPER
    )


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_reborrow_chunk_wrapper_boundary(library)
    print(
        "semantic reborrow chunk wrapper: exact one- and two-region "
        "fixed-literal ten-argument calls, independent nested callee proof, "
        "source anchoring, topology, and closed higher-region/composite "
        "shapes pass"
    )


if __name__ == "__main__":
    main()
