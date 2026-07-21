"""Exact FORM-5/FORM-7 float-contract extraction for the Oracle.

This module recognizes the immutable v0.8 wording and the one reviewed
successor wording as separate dialects.  It does not format or round floats;
it extracts the grammar predicate that the generalized grammar analysis needs.
"""

from __future__ import annotations

from dataclasses import dataclass
import re

from core import fail
from source import SourceScan


_INTEGER_CLAUSE = (
    b"Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; "
    b"a leading `-` is legal for signed TYPE, and the signed value must lie in TYPE's "
    b"range [FORM-7]; e.g. `42_i32`, `-2147483648_i32`); "
)
_CURRENT_FLOAT_CLAUSE = (
    b"floats `-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE` (a leading `-` is legal for the "
    b"value; the canonical spelling is the unique shortest decimal digit string that "
    b"round-trips under round-to-nearest-even, with at least one integer and one fraction "
    b"digit, lowercase `e`, and no leading zeros; `-0.0` is distinct from `0.0`; e.g. "
    b"`1.5_f64`, `6.022e23_f64`); "
)
_PROPOSAL_FLOAT_GRAMMAR = b"-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE"
_PROPOSAL_FLOAT_CLAUSE = (
    b"finite floats use the grammar `"
    + _PROPOSAL_FLOAT_GRAMMAR
    + b"`, where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), "
    b"positive exponents carry no sign, negative "
    b"exponents carry one `-`, and only the integer and exponent components have the "
    b"stated no-leading-zero form. Let C be the nonnegative integer formed by "
    b"concatenating the integer and fraction digits, let F be the number of fraction "
    b"digits, and let E be the signed integer formed by the exponent digits and their "
    b"optional `-`; when the exponent is absent E is zero, and `e-0` also gives E zero. "
    b"A matching decimal whose C is zero denotes signed decimal zero: a leading literal "
    b"`-` selects negative zero and its absence selects positive zero, independently of "
    b"E. Every other matching decimal denotes the exact nonzero rational whose magnitude "
    b"is C \xc3\x97 10^(E \xe2\x88\x92 F), with the leading literal sign applied. For one finite "
    b"bit pattern of TYPE, consider every matching decimal that rounds from that "
    b"signed zero or exact nonzero rational to the bit pattern under IEEE 754 "
    b"round-to-nearest, ties-to-even. Its canonical spelling is the candidate with the "
    b"fewest ASCII bytes "
    b"before `_TYPE`; a tie is resolved by lexicographically least unsigned ASCII bytes. "
    b"This selection is total, host-independent, and unique; in particular `0.0` and "
    b"`-0.0` remain distinct. Other examples are `1.5_f64` and `6.022e23_f64`. "
)
_LITERAL_SUFFIX = (
    b'`unit`; STRING `"..."` whose interior is a sequence of items, each one raw '
    b'ASCII-printable byte in U+0020..U+007E other than `"` and `\\`, or one of '
    b'exactly three escapes `\\\\ \\" \\n`; no other byte is legal, and each character '
    b"has exactly one spelling (the escape where one is defined, the raw byte otherwise). "
    b"STRING appears only in `doc` and `check` messages; non-ASCII diagnostic text is "
    b"DEFERRED. There are no boolean literals: `Bool` is a prelude enum (\xc2\xa715). "
    b"Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound "
    b"by a numeric contract (`Int` or `Float`, \xc2\xa715), denoting T's additive and "
    b"multiplicative identity; a concrete type uses `0_i32` and the like, so there is "
    b"no dual spelling. NaN and the infinities are not literals; they are the nullary "
    b"ops `fnan` and `finf` [OP-1]."
)

_CURRENT_FORM7 = (
    b"Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal "
    b"where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, "
    b"unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own "
    b"form, a leading `-` is legal for signed T, and `-0` is written `0`. A float literal "
    b"is legal where its round-to-nearest-even value in T is finite. An out-of-range integer, "
    b"a leading-zero integer, or a float that rounds to a non-finite value is a hard error at "
    b"check time [SCOPE-2]; a literal never denotes a wrapped, truncated, saturated, or "
    b"undefined value. The canonical decimal spelling of a float value is gated on the "
    b"FORM-1 reject-vs-canonicalize decision and DEFERRED."
)
_PROPOSAL_FORM7 = (
    b"Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal "
    b"where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, "
    b"unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own "
    b"form, a leading `-` is legal for signed T, and `-0` is written `0`. A float literal "
    b"is legal only when it has the unique canonical spelling selected by [FORM-5] and "
    b"denotes a finite value of its stated TYPE. An out-of-range integer, a leading-zero "
    b"integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite "
    b"value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, "
    b"truncated, saturated, or undefined value."
)

_CURRENT_FLOAT_PATTERN = rb"-?[0-9]+\.[0-9]+(?:e-?[0-9]+)?_(?:f32|f64)"
_PROPOSAL_FLOAT_PATTERN = (
    rb"-?(?:0|[1-9][0-9]*)\.[0-9]+(?:e-?(?:0|[1-9][0-9]*))?_(?:f32|f64)"
)


@dataclass(frozen=True)
class FloatContract:
    """One exact float-literal surface recognized from the specification."""

    dialect: str
    form5_payload: bytes
    form7_payload: bytes
    float_pattern: re.Pattern[bytes]
    literal_predicate: bytes


def _rule_payload(scan: SourceScan, owner: str) -> bytes:
    prefix = b"[" + owner.encode("ascii") + b"] "
    matches = [line.content for line in scan.lines if line.content.startswith(prefix)]
    if len(matches) != 1:
        fail("extraction", "float_contract_rule")
    return matches[0][len(prefix) :]


def extract_float_contract(scan: SourceScan) -> FloatContract:
    """Extract one closed current or proposal FORM-5/FORM-7 pair."""

    form5 = _rule_payload(scan, "FORM-5")
    form7 = _rule_payload(scan, "FORM-7")
    current_form5 = _INTEGER_CLAUSE + _CURRENT_FLOAT_CLAUSE + _LITERAL_SUFFIX
    proposal_form5 = _INTEGER_CLAUSE + _PROPOSAL_FLOAT_CLAUSE + _LITERAL_SUFFIX
    if form5 == current_form5 and form7 == _CURRENT_FORM7:
        return FloatContract(
            "v0.8-deferred-canonicality",
            form5,
            form7,
            re.compile(_CURRENT_FLOAT_PATTERN),
            (
                b"integer=-?[0-9]+_TYPE;float=-?[0-9]+\\.[0-9]+"
                b"(e-?[0-9]+)?_TYPE;unit=unit;generic=0_T,1_T"
            ),
        )
    if form5 == proposal_form5 and form7 == _PROPOSAL_FORM7:
        return FloatContract(
            "finite-rne-min-bytes-v1",
            form5,
            form7,
            re.compile(_PROPOSAL_FLOAT_PATTERN),
            (
                b"integer=-?[0-9]+_TYPE;float="
                + _PROPOSAL_FLOAT_GRAMMAR
                + b";float-value=signed-zero-or-sign*C*10^(E-F);e-0=0;round=ieee-rne;"
                b"canonical=min-prefix-bytes,ascii-lex;finite=required;"
                b"unit=unit;generic=0_T,1_T"
            ),
        )
    fail("extraction", "float_contract_shape")
