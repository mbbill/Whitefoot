from __future__ import annotations

from dataclasses import dataclass


class ZeroDerivationAuditError(RuntimeError):
    pass


@dataclass(frozen=True)
class ExactEdit:
    before: bytes
    after: bytes
    purpose: str
    expected_count: int = 1


@dataclass(frozen=True)
class ZeroCase:
    path: str
    expected_rule: str | None
    disposition: str
    control_purpose: str
    control_edits: tuple[ExactEdit, ...]
    repair_rationale: str | None = None


def _edit(
    before: bytes,
    after: bytes,
    purpose: str,
    expected_count: int = 1,
) -> ExactEdit:
    return ExactEdit(before, after, purpose, expected_count)


ZERO_CASES = {
    "const1-neg-noninteger": ZeroCase(
        "conformance/cases/const1-neg-noninteger.wf",
        "CONST-1",
        "protected-source-syntax-mismatch",
        (
            "Use the legal IDENT spelling consistently; the intended "
            "non-integer CONST-1 case remains."
        ),
        (
            _edit(
                b"NAME",
                b"name",
                "Change the consistently used TYPEID-shaped constant name to IDENT.",
                3,
            ),
        ),
        (
            "This changes only the spelling class of one binding and its two uses. "
            "The unit type and unit value that make the constant non-integer are unchanged."
        ),
    ),
    "form1-neg-unknown-construct": ZeroCase(
        "conformance/cases/form1-neg-unknown-construct.wf",
        "FORM-1",
        "expected-rule-derivation-boundary",
        "Replace only the unknown construct with a valid statement.",
        (
            _edit(
                b"  gimble x;\n",
                b"  let x: own unit = unit;\n",
                "Remove the intended unknown-construct violation for the control parse.",
            ),
        ),
    ),
    "form3-neg-opname-bad-suffix": ZeroCase(
        "conformance/cases/form3-neg-opname-bad-suffix.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Replace only the illegal operation suffix with a closed-set suffix.",
        (
            _edit(
                b"iadd.bogus",
                b"iadd.wrap",
                "Remove the intended bad-OPNAME violation for the control parse.",
            ),
        ),
    ),
    "form3-neg-region-param-missing-apostrophe": ZeroCase(
        "conformance/cases/form3-neg-region-param-missing-apostrophe.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Add only the required REGIONID apostrophe.",
        (
            _edit(
                b"[scope]",
                b"['scope]",
                "Remove the intended REGIONID spelling violation for the control parse.",
            ),
        ),
    ),
    "form3-neg-requires-binding": ZeroCase(
        "conformance/cases/form3-neg-requires-binding.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Rename only the reserved parameter and its use.",
        (
            _edit(
                b"requires: own i32",
                b"value: own i32",
                "Remove the intended reserved-binding violation from the parameter.",
            ),
            _edit(
                b"return requires;",
                b"return value;",
                "Keep the control binding use consistent.",
            ),
        ),
    ),
    "form3-neg-reserved-mode-field": ZeroCase(
        "conformance/cases/form3-neg-reserved-mode-field.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Rename only the reserved field.",
        (
            _edit(
                b"  trap: i32;",
                b"  value: i32;",
                "Remove the intended reserved-field violation for the control parse.",
            ),
        ),
    ),
    "form3-neg-typeid-fn-name": ZeroCase(
        "conformance/cases/form3-neg-typeid-fn-name.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Change only the TYPEID-shaped function name to IDENT.",
        (
            _edit(
                b"fn Main ",
                b"fn main ",
                "Remove the intended function-name spelling violation for the control parse.",
            ),
        ),
    ),
    "form4-neg-comment": ZeroCase(
        "conformance/cases/form4-neg-comment.wf",
        "FORM-4",
        "expected-rule-lexical-boundary",
        "Remove only the forbidden line comment.",
        (
            _edit(
                b"  // comments do not exist in whitefoot\n",
                b"",
                "Remove the intended comment violation for the control parse.",
            ),
        ),
    ),
    "form5-neg-missing-suffix": ZeroCase(
        "conformance/cases/form5-neg-missing-suffix.wf",
        "FORM-5",
        "expected-rule-derivation-boundary",
        "Add only the mandatory integer suffix.",
        (
            _edit(
                b"= 42;",
                b"= 42_i32;",
                "Remove the intended unsuffixed-literal violation for the control parse.",
            ),
        ),
    ),
    "gram9-neg-constructor-in-call-argument": ZeroCase(
        "conformance/cases/gram9-neg-constructor-in-call-argument.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        "Let-bind only the nested constructor used as a call argument.",
        (
            _edit(
                b"  consume(value: True());",
                b"  let value: own Bool = True();\n  consume(value: value);",
                "Remove the intended nested-constructor violation for the control parse.",
            ),
        ),
    ),
    "gram9-neg-constructor-in-constructor-field": ZeroCase(
        "conformance/cases/gram9-neg-constructor-in-constructor-field.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        "Let-bind only the inner constructor.",
        (
            _edit(
                b"  let outer: own Outer = OuterValue(value: InnerValue());",
                b"  let inner: own Inner = InnerValue();\n"
                b"  let outer: own Outer = OuterValue(value: inner);",
                "Remove the intended nested-constructor violation for the control parse.",
            ),
        ),
    ),
    "gram9-neg-nested-call": ZeroCase(
        "conformance/cases/gram9-neg-nested-call.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        "Let-bind only the inner operation call.",
        (
            _edit(
                b"  let p: own i32 = imul.wrap<i32>(iadd.wrap<i32>(3_i32, 4_i32), 2_i32);",
                b"  let s: own i32 = iadd.wrap<i32>(3_i32, 4_i32);\n"
                b"  let p: own i32 = imul.wrap<i32>(s, 2_i32);",
                "Remove the intended nested-call violation for the control parse.",
            ),
        ),
    ),
    "pending-const2-item": ZeroCase(
        "conformance/cases/pending-const2-item.wf",
        None,
        "protected-source-syntax-mismatch",
        "Use the legal IDENT spelling for the intended named constant item.",
        (
            _edit(
                b"LIMIT",
                b"limit",
                "Change the TYPEID-shaped constant name to IDENT.",
            ),
        ),
        (
            "This unmanifested legacy source has no protected expected verdict. "
            "The edit changes only its unused binding spelling; type and value are unchanged."
        ),
    ),
    "type7-neg-match-borrow-expression": ZeroCase(
        "conformance/cases/type7-neg-match-borrow-expression.wf",
        "TYPE-7",
        "protected-source-syntax-mismatch",
        "Remove the doc production from an arm body, where only stmt productions are admitted.",
        (
            _edit(
                b'        doc "A second arm statement pins rejection before '
                b'temporary-borrow cleanup.";\n',
                b"",
                "Remove documentation from the stmt-only arm body.",
            ),
        ),
        (
            "A doc production has no runtime semantics. The direct borrow match and the "
            "following set statement remain unchanged, so the intended TYPE-7 case remains."
        ),
    ),
    "x-eff-pure-combined-with-traps": ZeroCase(
        "conformance/cases/x-eff-pure-combined-with-traps.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        "Remove only pure from the otherwise valid traps row.",
        (
            _edit(
                b" pure, traps {",
                b" traps {",
                "Remove the intended mixed-pure-row violation for the control parse.",
            ),
        ),
    ),
    "x-eff-trailing-comma-row": ZeroCase(
        "conformance/cases/x-eff-trailing-comma-row.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        "Remove only the trailing effect-row comma.",
        (
            _edit(
                b" traps, {",
                b" traps {",
                "Remove the intended trailing-comma violation for the control parse.",
            ),
        ),
    ),
    "x-eff-writes-missing-region": ZeroCase(
        "conformance/cases/x-eff-writes-missing-region.wf",
        "EFF-1",
        "expected-rule-derivation-boundary",
        "Replace only the incomplete writes effect with the empty row.",
        (
            _edit(
                b" writes {",
                b" pure {",
                "Remove the intended missing-region violation for the control parse.",
            ),
        ),
    ),
    "x-form-form2-tab-indent": ZeroCase(
        "conformance/cases/x-form-form2-tab-indent.wf",
        "FORM-2",
        "isolated-form2-format-negative",
        "Replace only the two tab indentation bytes with canonical spaces.",
        (
            _edit(
                b"\t",
                b"  ",
                "Remove the intended tab-indentation violation for the control parse.",
                2,
            ),
        ),
    ),
    "x-form-form3-enum-name-ident": ZeroCase(
        "conformance/cases/x-form-form3-enum-name-ident.wf",
        "FORM-3",
        "expected-rule-derivation-boundary",
        "Change only the IDENT-shaped enum name to TYPEID.",
        (
            _edit(
                b"enum sign ",
                b"enum Sign ",
                "Remove the intended enum-name spelling violation for the control parse.",
            ),
        ),
    ),
    "x-form-form4-block-comment": ZeroCase(
        "conformance/cases/x-form-form4-block-comment.wf",
        "FORM-4",
        "expected-rule-lexical-boundary",
        "Remove only the forbidden block comment.",
        (
            _edit(
                b"  /* block comment */\n",
                b"",
                "Remove the intended block-comment violation for the control parse.",
            ),
        ),
    ),
    "x-form-form5-op-arg-missing-suffix": ZeroCase(
        "conformance/cases/x-form-form5-op-arg-missing-suffix.wf",
        "FORM-5",
        "expected-rule-derivation-boundary",
        "Add only the mandatory integer suffix to the operation operand.",
        (
            _edit(
                b", 2);",
                b", 2_i32);",
                "Remove the intended unsuffixed-literal violation for the control parse.",
            ),
        ),
    ),
    "x-gram-nested-op-in-construct-field": ZeroCase(
        "conformance/cases/x-gram-nested-op-in-construct-field.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        "Let-bind only the operation nested in the constructor field.",
        (
            _edit(
                b"  let x: own Wrap = W(inner: iadd.wrap<i32>(3_i32, 4_i32));",
                b"  let inner: own i32 = iadd.wrap<i32>(3_i32, 4_i32);\n"
                b"  let x: own Wrap = W(inner: inner);",
                "Remove the intended nested-call violation for the control parse.",
            ),
        ),
    ),
    "x-gram-nested-ucall-in-call-arg": ZeroCase(
        "conformance/cases/x-gram-nested-ucall-in-call-arg.wf",
        "GRAM-9",
        "expected-rule-derivation-boundary",
        "Let-bind only the user call nested in another call argument.",
        (
            _edit(
                b"  let r: own i32 = add2(x: inc(y: 1_i32), y: 2_i32);",
                b"  let inner: own i32 = inc(y: 1_i32);\n"
                b"  let r: own i32 = add2(x: inner, y: 2_i32);",
                "Remove the intended nested-user-call violation for the control parse.",
            ),
        ),
    ),
}

PROTECTED_REPAIR_KEYS = frozenset(
    {
        "const1-neg-noninteger",
        "pending-const2-item",
        "type7-neg-match-borrow-expression",
    }
)
