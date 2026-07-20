#!/usr/bin/env python3
"""Audit the exact same-region byte push and whole-parent call slice."""

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


REBORROW_RW = (
    b"enum PushStatus {\n"
    b"  PushClean();\n"
    b"  PushFull();\n"
    b"}\n"
    b"struct PushTape {\n"
    b"  bytes: buffer<u8>;\n"
    b"  count: u64;\n"
    b"  status: PushStatus;\n"
    b"}\n"
    b"fn push_byte ['o] (out: &uniq 'o PushTape, byte: own u8) "
    b"-> own unit reads('o), writes('o), traps {\n"
    b"  let slot: own u64 = deref(out).count;\n"
    b"  let capacity: own u64 = len<u8>(deref(out).bytes);\n"
    b"  let fits: own Bool = ilt<u64>(slot, capacity);\n"
    b"  match fits {\n"
    b"    True() => {\n"
    b"      set index<u8>(deref(out).bytes, slot) = byte;\n"
    b"    }\n"
    b"    False() => {\n"
    b"      set deref(out).status = PushFull();\n"
    b"    }\n"
    b"  }\n"
    b"  set deref(out).count = iadd.trap<u64>(slot, 1_u64);\n"
    b"  return unit;\n"
    b"}\n"
    b"fn emit_mark ['o] (out: &uniq 'o PushTape) "
    b"-> own unit reads('o), writes('o), traps {\n"
    b"  region 'mark_call {\n"
    b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);\n"
    b"  }\n"
    b"  return unit;\n"
    b"}\n"
)


def assert_reborrow_rw_boundary(library):
    def classify(source, ordinal=-1):
        case = parsed(library, source)
        function = top_level_functions(case)[ordinal]
        work = make_work(library, case[5].count)
        kind, report = invoke_dispatch(library, case, function, work)
        return case, function, work, kind, report

    def assert_clean(source=REBORROW_RW, ordinal=-1):
        case, function, work, kind, report = classify(source, ordinal)
        assert (kind, report.status, report.function, report.related) == (
            CAPABILITY_ACYCLIC,
            BODY_CLEAN,
            function,
            AST_NONE,
        ), (kind, report.status, report.function, report.related)
        assert_no_capability_diagnostic(report)
        assert_output_guards(work)
        return case, function

    def assert_unsupported(source, ordinal=-1):
        _, function, work, kind, report = classify(source, ordinal)
        assert (kind, report.status, report.function) == (
            CAPABILITY_UNSUPPORTED,
            BODY_UNSUPPORTED,
            function,
        ), (kind, report.status, report.function, report.related)
        assert_no_capability_diagnostic(report)
        assert_output_guards(work)

    # Both the independently validated callee and its exact wrapper are clean.
    assert_clean(ordinal=0)
    assert_clean()
    assert_clean(REBORROW_RW.replace(b"push_byte", b"put_octet"), ordinal=0)
    assert_clean(REBORROW_RW.replace(b"push_byte", b"put_octet"))

    other_rw = REBORROW_RW.replace(
        b"  let slot: own u64 = deref(out).count;\n"
        b"  let capacity: own u64 = len<u8>(deref(out).bytes);\n"
        b"  let fits: own Bool = ilt<u64>(slot, capacity);\n"
        b"  match fits {\n"
        b"    True() => {\n"
        b"      set index<u8>(deref(out).bytes, slot) = byte;\n"
        b"    }\n"
        b"    False() => {\n"
        b"      set deref(out).status = PushFull();\n"
        b"    }\n"
        b"  }\n"
        b"  set deref(out).count = iadd.trap<u64>(slot, 1_u64);\n",
        b"  let prior: own u64 = deref(out).count;\n"
        b"  let next: own u64 = iadd.trap<u64>(prior, 1_u64);\n"
        b"  set deref(out).count = next;\n",
        1,
    )
    assert_unsupported(other_rw, ordinal=0)

    # Same-region reads, writes, and traps are all required on both functions.
    for old, new, count in (
        (b"reads('o), writes('o), traps {", b"writes('o), traps {", 1),
        (b"reads('o), writes('o), traps {", b"reads('o), traps {", 1),
        (b"reads('o), writes('o), traps {", b"reads('o), writes('o) {", 1),
        (b"reads('o), writes('o), traps {", b"reads('o), writes('x), traps {", 1),
    ):
        assert_unsupported(REBORROW_RW.replace(old, new, count), ordinal=0)
    assert_unsupported(
        REBORROW_RW.replace(
            b"-> own unit reads('o), writes('o), traps {\n"
            b"  region 'mark_call",
            b"-> own unit writes('o), traps {\n  region 'mark_call",
        )
    )
    assert_unsupported(
        REBORROW_RW.replace(
            b"-> own unit reads('o), writes('o), traps {\n"
            b"  region 'mark_call",
            b"-> own unit reads('o), writes('o) {\n  region 'mark_call",
        )
    )

    # The callee is the exact structural push profile, not a name-based grant.
    for old, new in (
        (b"out: &uniq 'o PushTape, byte: own u8", b"out: &'o PushTape, byte: own u8"),
        (b"byte: own u8", b"byte: own u64"),
        (b"deref(out).count;", b"deref(out).status;"),
        (b"len<u8>(deref(out).bytes)", b"len<u8>(deref(out).count)"),
        (b"ilt<u64>(slot, capacity)", b"ile<u64>(slot, capacity)"),
        (b"index<u8>(deref(out).bytes, slot)", b"index<u8>(deref(out).bytes, capacity)"),
        (b"= byte;", b"= 1_u8;"),
        (b"= PushFull();", b"= byte;"),
        (b"iadd.trap<u64>(slot, 1_u64)", b"iadd.wrap<u64>(slot, 1_u64)"),
    ):
        assert_unsupported(REBORROW_RW.replace(old, new, 1), ordinal=0)
    assert_unsupported(
        REBORROW_RW.replace(b"bytes: buffer<u8>;", b"bytes: u64;"), ordinal=0
    )
    assert_unsupported(
        REBORROW_RW.replace(b"count: u64;", b"count: buffer<u8>;"), ordinal=0
    )

    # The wrapper lends one whole unique parent for one statement and passes
    # one canonical u8 literal. Other borrow and call profiles remain closed.
    for old, new in (
        (b"out: &uniq 'o PushTape) -> own unit", b"out: &'o PushTape) -> own unit"),
        (b"region 'mark_call", b"region 'o"),
        (b"&uniq 'mark_call deref(out)", b"&uniq 'other deref(out)"),
        (b"&uniq 'mark_call deref(out)", b"&'mark_call deref(out)"),
        (b"&uniq 'mark_call deref(out)", b"&uniq 'mark_call deref(out).count"),
        (b"push_byte(out:", b"push_byte<'mark_call>(out:"),
        (b"out: &uniq 'mark_call deref(out), byte: 33_u8", b"byte: 33_u8, out: &uniq 'mark_call deref(out)"),
        (b"byte: 33_u8", b"byte: 033_u8"),
        (b"byte: 33_u8", b"byte: 33_u64"),
        (b"byte: 33_u8", b"byte: 256_u8"),
    ):
        assert_unsupported(REBORROW_RW.replace(old, new, 1))
    assert_unsupported(
        REBORROW_RW.replace(
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);",
            b"    let ignored: own unit = push_byte("
            b"out: &uniq 'mark_call deref(out), byte: 33_u8);",
        )
    )
    assert_unsupported(
        REBORROW_RW.replace(
            b"  region 'mark_call {\n"
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);\n"
            b"  }\n"
            b"  return unit;\n",
            b"  region 'mark_call {\n"
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);\n"
            b"  }\n"
            b"  region 'second_call {\n"
            b"    push_byte(out: &uniq 'second_call deref(out), byte: 34_u8);\n"
            b"  }\n"
            b"  return unit;\n",
            1,
        )
    )
    assert_unsupported(
        REBORROW_RW.replace(
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);",
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 33_u8);\n"
            b"    push_byte(out: &uniq 'mark_call deref(out), byte: 34_u8);",
        )
    )

    def assert_direct_reader_mutation_rejected(mutate):
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

    def nodes(case, function):
        columns = case[4]
        wrapper_children = children_of(columns, function)
        outer_parameter = wrapper_children[2]
        outer_region = children_of(columns, wrapper_children[1])[0]
        region_block, _ = children_of(columns, wrapper_children[8])
        local_region, inner_block = children_of(columns, region_block)
        expression = children_of(columns, inner_block)[0]
        call = children_of(columns, expression)[0]
        output_argument, byte_argument = children_of(columns, call)
        borrow = children_of(columns, output_argument)[0]
        child_region, deref_place = children_of(columns, borrow)
        parent = children_of(columns, deref_place)[0]
        literal = children_of(columns, byte_argument)[0]
        callee = top_level_functions(case)[0]
        callee_children = children_of(columns, callee)
        callee_name = callee_children[0]
        callee_region = children_of(columns, callee_children[1])[0]
        output_parameter = callee_children[2]
        byte_parameter = callee_children[3]
        reads_region = children_of(columns, callee_children[6])[0]
        writes_region = children_of(columns, callee_children[7])[0]
        callee_block = callee_children[9]
        slot_let, _, _, match_node, tail_set, _ = children_of(columns, callee_block)
        slot_name = children_of(columns, slot_let)[0]
        true_arm = children_of(columns, match_node)[1]
        true_set = children_of(columns, children_of(columns, true_arm)[0])[0]
        index_target = children_of(columns, true_set)[0]
        tail_target = children_of(columns, tail_set)[0]
        struct_decl = next(
            item
            for item in children_of(columns, case[5].root)
            if columns[0][item] == AST["AstStructDecl"]
        )
        count_field = next(
            field
            for field in children_of(columns, struct_decl)
            if columns[0][field] == AST["AstField"]
            and bytes(case[0][columns[2][field] : columns[3][field]]).startswith(
                b"count:"
            )
        )
        return {
            "outer_parameter": outer_parameter,
            "outer_region": outer_region,
            "local_region": local_region,
            "call": call,
            "output_argument": output_argument,
            "byte_argument": byte_argument,
            "borrow": borrow,
            "child_region": child_region,
            "parent": parent,
            "literal": literal,
            "callee_name": callee_name,
            "callee_region": callee_region,
            "output_parameter": output_parameter,
            "byte_parameter": byte_parameter,
            "reads_region": reads_region,
            "writes_region": writes_region,
            "slot_name": slot_name,
            "index_target": index_target,
            "tail_target": tail_target,
            "count_field": count_field,
        }

    def redirect_head(target, source):
        def mutate(case, function):
            selected = nodes(case, function)
            case[4][1][selected[target]] = case[4][1][selected[source]]

        return mutate

    def borrow_kind_shared(case, function):
        selected = nodes(case, function)
        case[4][0][selected["borrow"]] = AST["AstSharedBorrow"]

    def named_argument_cycle(case, function):
        selected = nodes(case, function)
        case[4][6][selected["output_argument"]] = selected["output_argument"]

    for mutate in (
        redirect_head("local_region", "child_region"),
        redirect_head("child_region", "local_region"),
        redirect_head("call", "callee_name"),
        redirect_head("callee_name", "call"),
        redirect_head("output_argument", "output_parameter"),
        redirect_head("output_parameter", "output_argument"),
        redirect_head("byte_argument", "byte_parameter"),
        redirect_head("byte_parameter", "byte_argument"),
        redirect_head("parent", "outer_parameter"),
        redirect_head("outer_parameter", "parent"),
        redirect_head("reads_region", "callee_region"),
        redirect_head("writes_region", "callee_region"),
        redirect_head("slot_name", "output_parameter"),
        redirect_head("tail_target", "count_field"),
        redirect_head("index_target", "slot_name"),
        redirect_head("literal", "byte_parameter"),
        borrow_kind_shared,
        named_argument_cycle,
    ):
        assert_direct_reader_mutation_rejected(mutate)


def main():
    with tempfile.TemporaryDirectory() as raw_directory:
        library = build_library(Path(raw_directory))
        configure(library)
        assert_reborrow_rw_boundary(library)
    print(
        "semantic reborrow rw: exact same-region byte push, one whole-parent "
        "unique call plus canonical u8, independent callee proof, source "
        "anchoring, topology, and closed sibling/multi-call boundaries pass"
    )


if __name__ == "__main__":
    main()
