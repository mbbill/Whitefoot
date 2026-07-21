from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from form2_independent_inputs import IndependentInputs, IndependentSource, digest
from form2_independent_lex import lex_independently
from form2_independent_parse import parse_independently


class IndependentRepairError(RuntimeError):
    pass


@dataclass(frozen=True)
class ExactEdit:
    before: bytes
    after: bytes
    count: int = 1


@dataclass(frozen=True)
class FailureContract:
    identifier: str | None
    path: str
    expected_rule: str | None
    disposition: str
    edits: tuple[ExactEdit, ...]
    proposed_repair: bool = False


def _edit(before: bytes, after: bytes, count: int = 1) -> ExactEdit:
    return ExactEdit(before, after, count)


FAILURE_CONTRACTS = (
    FailureContract(
        "const1-neg-noninteger",
        "conformance/cases/const1-neg-noninteger.wf",
        "CONST-1",
        "protected-source-syntax-mismatch",
        (_edit(b"NAME", b"name", 3),),
        True,
    ),
    FailureContract(
        "form1-neg-unknown-construct",
        "conformance/cases/form1-neg-unknown-construct.wf",
        "FORM-1",
        "expected-rule-derivation-boundary",
        (_edit(b"  gimble x;\n", b"  let x: own unit = unit;\n"),),
    ),
    FailureContract(
        "form3-neg-opname-bad-suffix",
        "conformance/cases/form3-neg-opname-bad-suffix.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (_edit(b"iadd.bogus", b"iadd.wrap"),),
    ),
    FailureContract(
        "form3-neg-region-param-missing-apostrophe",
        "conformance/cases/form3-neg-region-param-missing-apostrophe.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (_edit(b"[scope]", b"['scope]"),),
    ),
    FailureContract(
        "form3-neg-requires-binding",
        "conformance/cases/form3-neg-requires-binding.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (
            _edit(b"requires: own i32", b"value: own i32"),
            _edit(b"return requires;", b"return value;"),
        ),
    ),
    FailureContract(
        "form3-neg-reserved-mode-field",
        "conformance/cases/form3-neg-reserved-mode-field.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (_edit(b"  trap: i32;", b"  value: i32;"),),
    ),
    FailureContract(
        "form3-neg-typeid-fn-name",
        "conformance/cases/form3-neg-typeid-fn-name.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (_edit(b"fn Main ", b"fn main "),),
    ),
    FailureContract(
        "form4-neg-comment",
        "conformance/cases/form4-neg-comment.wf",
        "FORM-4",
        "expected-rule-lexical-boundary",
        (_edit(b"  // comments do not exist in whitefoot\n", b""),),
    ),
    FailureContract(
        "form5-neg-missing-suffix",
        "conformance/cases/form5-neg-missing-suffix.wf",
        "FORM-5",
        "expected-rule-derivation-boundary",
        (_edit(b"= 42;", b"= 42_i32;"),),
    ),
    FailureContract(
        "gram9-neg-constructor-in-call-argument",
        "conformance/cases/gram9-neg-constructor-in-call-argument.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        (
            _edit(
                b"  consume(value: True());",
                b"  let value: own Bool = True();\n  consume(value: value);",
            ),
        ),
    ),
    FailureContract(
        "gram9-neg-constructor-in-constructor-field",
        "conformance/cases/gram9-neg-constructor-in-constructor-field.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        (
            _edit(
                b"  let outer: own Outer = OuterValue(value: InnerValue());",
                b"  let inner: own Inner = InnerValue();\n"
                b"  let outer: own Outer = OuterValue(value: inner);",
            ),
        ),
    ),
    FailureContract(
        "gram9-neg-nested-call",
        "conformance/cases/gram9-neg-nested-call.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        (
            _edit(
                b"  let p: own i32 = imul.wrap<i32>(iadd.wrap<i32>(3_i32, 4_i32), 2_i32);",
                b"  let s: own i32 = iadd.wrap<i32>(3_i32, 4_i32);\n"
                b"  let p: own i32 = imul.wrap<i32>(s, 2_i32);",
            ),
        ),
    ),
    FailureContract(
        None,
        "conformance/cases/pending-const2-item.wf",
        None,
        "protected-source-syntax-mismatch",
        (_edit(b"LIMIT", b"limit"),),
        True,
    ),
    FailureContract(
        "type7-neg-match-borrow-expression",
        "conformance/cases/type7-neg-match-borrow-expression.wf",
        "TYPE-7",
        "protected-source-syntax-mismatch",
        (
            _edit(
                b'        doc "A second arm statement pins rejection before temporary-borrow cleanup.";\n',
                b"",
            ),
        ),
        True,
    ),
    FailureContract(
        "x-eff-pure-combined-with-traps",
        "conformance/cases/x-eff-pure-combined-with-traps.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        (_edit(b" pure, traps {", b" traps {"),),
    ),
    FailureContract(
        "x-eff-trailing-comma-row",
        "conformance/cases/x-eff-trailing-comma-row.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        (_edit(b" traps, {", b" traps {"),),
    ),
    FailureContract(
        "x-eff-writes-missing-region",
        "conformance/cases/x-eff-writes-missing-region.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        (_edit(b" writes {", b" pure {"),),
    ),
    FailureContract(
        "x-form-form2-tab-indent",
        "conformance/cases/x-form-form2-tab-indent.wf",
        "FORM-2",
        "isolated-form2-format-negative",
        (_edit(b"\t", b"  ", 2),),
    ),
    FailureContract(
        "x-form-form3-enum-name-ident",
        "conformance/cases/x-form-form3-enum-name-ident.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        (_edit(b"enum sign ", b"enum Sign "),),
    ),
    FailureContract(
        "x-form-form4-block-comment",
        "conformance/cases/x-form-form4-block-comment.wf",
        "FORM-4",
        "expected-rule-lexical-boundary",
        (_edit(b"  /* block comment */\n", b""),),
    ),
    FailureContract(
        "x-form-form5-op-arg-missing-suffix",
        "conformance/cases/x-form-form5-op-arg-missing-suffix.wf",
        "FORM-5",
        "expected-rule-derivation-boundary",
        (_edit(b", 2);", b", 2_i32);"),),
    ),
    FailureContract(
        "x-gram-nested-op-in-construct-field",
        "conformance/cases/x-gram-nested-op-in-construct-field.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        (
            _edit(
                b"  let x: own Wrap = W(inner: iadd.wrap<i32>(3_i32, 4_i32));",
                b"  let inner: own i32 = iadd.wrap<i32>(3_i32, 4_i32);\n"
                b"  let x: own Wrap = W(inner: inner);",
            ),
        ),
    ),
    FailureContract(
        "x-gram-nested-ucall-in-call-arg",
        "conformance/cases/x-gram-nested-ucall-in-call-arg.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        (
            _edit(
                b"  let r: own i32 = add2(x: inc(y: 1_i32), y: 2_i32);",
                b"  let inner: own i32 = inc(y: 1_i32);\n"
                b"  let r: own i32 = add2(x: inner, y: 2_i32);",
            ),
        ),
    ),
)


def _positions(raw: bytes, needle: bytes) -> list[int]:
    if not needle:
        raise IndependentRepairError("an exact-edit source fragment is empty")
    result: list[int] = []
    cursor = 0
    while True:
        offset = raw.find(needle, cursor)
        if offset < 0:
            return result
        result.append(offset)
        cursor = offset + len(needle)


def _apply_exact_edits(
    raw: bytes, edits: tuple[ExactEdit, ...]
) -> tuple[bytes, list[dict[str, Any]]]:
    spans: list[tuple[int, int, bytes]] = []
    for edit in edits:
        positions = _positions(raw, edit.before)
        if len(positions) != edit.count:
            raise IndependentRepairError(
                f"exact-edit occurrence count changed: expected {edit.count}, "
                f"found {len(positions)}"
            )
        spans.extend(
            (start, start + len(edit.before), edit.after) for start in positions
        )
    spans.sort()
    if any(left[1] > right[0] for left, right in zip(spans, spans[1:])):
        raise IndependentRepairError("exact-edit source fragments overlap")
    output = bytearray()
    records: list[dict[str, Any]] = []
    cursor = 0
    for start, end, replacement in spans:
        output.extend(raw[cursor:start])
        output.extend(replacement)
        records.append(
            {
                "after_hex": replacement.hex(),
                "before_hex": raw[start:end].hex(),
                "byte_end_before": end,
                "byte_start_before": start,
            }
        )
        cursor = end
    output.extend(raw[cursor:])
    return bytes(output), records


def apply_exact_edits(raw: bytes, edits: tuple[ExactEdit, ...]) -> bytes:
    return _apply_exact_edits(raw, edits)[0]


def audit_failure_controls(
    inputs: IndependentInputs,
    raw_failure_paths: frozenset[str],
) -> tuple[list[dict[str, Any]], dict[str, bytes]]:
    contracts = {contract.path: contract for contract in FAILURE_CONTRACTS}
    if len(contracts) != len(FAILURE_CONTRACTS):
        raise IndependentRepairError("failure-contract paths are not unique")
    if raw_failure_paths != frozenset(contracts):
        raise IndependentRepairError(
            "raw failure inventory changed: "
            f"missing={sorted(set(contracts) - raw_failure_paths)!r}, "
            f"extra={sorted(raw_failure_paths - set(contracts))!r}"
        )
    sources = {source.path: source for source in inputs.sources}
    records: list[dict[str, Any]] = []
    repairs: dict[str, bytes] = {}
    for path in sorted(contracts, key=str.encode):
        contract = contracts[path]
        source: IndependentSource = sources[path]
        if source.identifier != contract.identifier:
            raise IndependentRepairError(f"failure identifier changed: {path}")
        expected = None if source.manifest is None else source.manifest["expect"]
        required = (
            None
            if contract.expected_rule is None
            else {"kind": "reject", "rule": contract.expected_rule}
        )
        if expected != required:
            raise IndependentRepairError(f"expected verdict changed: {path}")
        control, edit_records = _apply_exact_edits(source.raw, contract.edits)
        if control == source.raw:
            raise IndependentRepairError(f"control edit changed no bytes: {path}")
        tokens = lex_independently(control)
        parse_independently(tokens, len(control))
        record = {
            "control_sha256": digest(control),
            "disposition": contract.disposition,
            "expected_rule": contract.expected_rule,
            "id": contract.identifier,
            "edits": edit_records,
            "path": path,
            "predictive_control_parse_complete": True,
            "proposed_repair": contract.proposed_repair,
            "source_sha256": source.sha256,
        }
        records.append(record)
        if contract.proposed_repair:
            repairs[path] = control
    if len(repairs) != 3:
        raise IndependentRepairError("protected syntax-repair inventory is not exactly three")
    return records, repairs
