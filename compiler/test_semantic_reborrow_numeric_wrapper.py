#!/usr/bin/env python3
"""Audit exact fixed-chunk plus recursive-number wrappers."""

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
from test_semantic_reborrow_u64 import DIGITS, EMITTER
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


WRAPPER_HEAD = (
    b"fn emit_numbered ['o] (out: &uniq 'o PushTape, id: own u64) "
    b"-> own unit reads('o), writes('o), traps {\n"
)
CHUNK_REGION = (
    b"  region 'number_prefix {\n"
    b"    emit_chunk(out: &uniq 'number_prefix deref(out), count: 2_u64, "
    b"b0: 37_u8, b1: 118_u8, b2: 32_u8, b3: 32_u8, "
    b"b4: 32_u8, b5: 32_u8, b6: 32_u8, b7: 32_u8);\n"
    b"  }\n"
)
NUMBER_REGION = (
    b"  region 'number_value {\n"
    b"    emit_u64(out: &uniq 'number_value deref(out), value: id);\n"
    b"  }\n"
)
WRAPPER_TAIL = b"  return unit;\n}\n"
WRAPPER = WRAPPER_HEAD + CHUNK_REGION + NUMBER_REGION + WRAPPER_TAIL
REBORROW_NUMERIC_WRAPPER = REBORROW_CHUNK + DIGITS + EMITTER + WRAPPER


def assert_reborrow_numeric_wrapper_boundary(library):
    def classify(source=REBORROW_NUMERIC_WRAPPER):
        case = parsed(library, source)
        function = top_level_functions(case)[-1]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        return case, function, work, kind, report

    def assert_clean(source=REBORROW_NUMERIC_WRAPPER):
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
        REBORROW_NUMERIC_WRAPPER.replace(b"push_byte", b"put_octet")
        .replace(b"emit_chunk", b"send_octets")
        .replace(b"decimal_digits", b"number_glyphs")
        .replace(b"emit_u64", b"write_number")
        .replace(b"emit_numbered", b"write_numbered")
        .replace(b"id: own u64", b"number: own u64", 1)
        .replace(b"value: id", b"value: number", 1)
        .replace(b"'number_prefix", b"'header_scope")
        .replace(b"'number_value", b"'number_scope")
    )

    # The wrapper signature, ordered pair of statement-confined children,
    # owned numeric argument, and both exact callees form one closed profile.
    reversed_wrapper = (
        REBORROW_CHUNK
        + DIGITS
        + EMITTER
        + WRAPPER_HEAD
        + NUMBER_REGION
        + CHUNK_REGION
        + WRAPPER_TAIL
    )
    third_region = replace_last(
        REBORROW_NUMERIC_WRAPPER,
        WRAPPER_TAIL,
        CHUNK_REGION.replace(b"number_prefix", b"extra_prefix") + WRAPPER_TAIL,
    )
    for source in (
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"out: &uniq 'o PushTape",
            b"out: &'o PushTape",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER, b"id: own u64", b"id: own u8"
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"writes('o), traps {",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"reads('o), traps {",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"reads('o), writes('o), traps {",
            b"reads('o), writes('o) {",
        ),
        replace_last(REBORROW_NUMERIC_WRAPPER, b"2_u64", b"02_u64"),
        replace_last(REBORROW_NUMERIC_WRAPPER, b"37_u8", b"256_u8"),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"count: 2_u64, b0: 37_u8",
            b"b0: 37_u8, count: 2_u64",
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"&uniq 'number_prefix deref(out)",
            b"&'number_prefix deref(out)",
            1,
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"&uniq 'number_prefix deref(out)",
            b"&uniq 'number_prefix deref(out).count",
            1,
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"emit_chunk(out:",
            b"emit_chunk<'number_prefix>(out:",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"    emit_chunk(out:",
            b"    let ignored: own unit = emit_chunk(out:",
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"&uniq 'number_value deref(out)",
            b"&'number_value deref(out)",
            1,
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"&uniq 'number_value deref(out)",
            b"&uniq 'number_value deref(out).count",
            1,
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"emit_u64(out:",
            b"emit_u64<'number_value>(out:",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"    emit_u64(out:",
            b"    let ignored: own unit = emit_u64(out:",
        ),
        replace_last(REBORROW_NUMERIC_WRAPPER, b"value: id", b"value: out"),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"emit_u64(out:",
            b"missing_number(out:",
        ),
        REBORROW_NUMERIC_WRAPPER.replace(b"'number_value", b"'number_prefix"),
        REBORROW_NUMERIC_WRAPPER.replace(b"'number_value", b"'o"),
        reversed_wrapper,
        third_region,
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"  return unit;\n}\n",
            b"  let extra: own u64 = 0_u64;\n  return unit;\n}\n",
        ),
        replace_last(
            REBORROW_NUMERIC_WRAPPER,
            b"  return unit;\n}\n",
            b"  return id;\n}\n",
        ),
    ):
        assert_unsupported(source)

    # Declarations do not lend effects: both callees and their nested proof
    # dependencies are independently revalidated at the wrapper boundary.
    for source in (
        REBORROW_NUMERIC_WRAPPER.replace(
            b"ile<u64>(count, 8_u64)", b"ilt<u64>(count, 8_u64)", 1
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"ilt<u64>(slot, capacity)", b"ile<u64>(slot, capacity)", 1
        ),
        REBORROW_NUMERIC_WRAPPER.replace(
            b"irem.trap<u64>(value, 10_u64)",
            b"idiv.trap<u64>(value, 10_u64)",
            1,
        ),
        REBORROW_NUMERIC_WRAPPER.replace(b"array<u8, 10>", b"array<u8, 9>", 1),
    ):
        assert_unsupported(source)

    def nodes(case, function):
        columns = case[4]
        direct = children_of(columns, function)
        body = direct[9]
        chunk_region, numeric_region, _ = children_of(columns, body)

        def region_nodes(region):
            local, inner = children_of(columns, region)
            expression = children_of(columns, inner)[0]
            call = children_of(columns, expression)[0]
            output, value = children_of(columns, call)[:2]
            borrow = children_of(columns, output)[0]
            child, deref_place = children_of(columns, borrow)
            parent = children_of(columns, deref_place)[0]
            return local, call, output, value, borrow, child, parent

        chunk = region_nodes(chunk_region)
        numeric = region_nodes(numeric_region)
        callees = top_level_functions(case)
        return {
            "outer_region": children_of(columns, direct[1])[0],
            "output_parameter": direct[2],
            "value_parameter": direct[3],
            "chunk_region": chunk_region,
            "chunk_local": chunk[0],
            "chunk_call": chunk[1],
            "chunk_output": chunk[2],
            "chunk_borrow": chunk[4],
            "chunk_child": chunk[5],
            "chunk_parent": chunk[6],
            "numeric_region": numeric_region,
            "numeric_local": numeric[0],
            "numeric_call": numeric[1],
            "numeric_output": numeric[2],
            "numeric_value": children_of(columns, numeric[3])[0],
            "numeric_borrow": numeric[4],
            "numeric_child": numeric[5],
            "numeric_parent": numeric[6],
            "chunk_callee": children_of(columns, callees[1])[0],
            "numeric_callee": children_of(columns, callees[2])[0],
        }

    def assert_direct_mutation_rejected(mutate):
        case, function = assert_clean()
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

    def shared_numeric_borrow(case, function):
        selected = nodes(case, function)
        case[4][0][selected["numeric_borrow"]] = AST["AstSharedBorrow"]

    def argument_cycle(case, function):
        selected = nodes(case, function)
        case[4][6][selected["numeric_output"]] = selected["numeric_output"]

    def region_cycle(case, function):
        selected = nodes(case, function)
        case[4][6][selected["chunk_region"]] = selected["chunk_region"]

    for mutate in (
        redirect_head("chunk_local", "chunk_child"),
        redirect_head("chunk_child", "chunk_local"),
        redirect_head("chunk_call", "chunk_callee"),
        redirect_head("chunk_callee", "chunk_call"),
        redirect_head("chunk_parent", "output_parameter"),
        redirect_head("numeric_local", "numeric_child"),
        redirect_head("numeric_child", "numeric_local"),
        redirect_head("numeric_call", "numeric_callee"),
        redirect_head("numeric_callee", "numeric_call"),
        redirect_head("numeric_parent", "output_parameter"),
        redirect_head("numeric_value", "value_parameter"),
        redirect_head("numeric_local", "outer_region"),
        shared_numeric_borrow,
        argument_cycle,
        region_cycle,
    ):
        assert_direct_mutation_rejected(mutate)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_reborrow_numeric_wrapper_boundary(library)
    print(
        "semantic numeric wrapper: exact fixed chunk then owned u64 emission, "
        "independent nested proofs, source anchoring, topology, and closed "
        "nearby shapes pass"
    )


if __name__ == "__main__":
    main()
