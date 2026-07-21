#!/usr/bin/env python3
"""Build or check independent exact-rational successor float evidence."""

from __future__ import annotations

import argparse
from fractions import Fraction
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

MODULE_ROOT = Path(__file__).resolve().parent
if str(MODULE_ROOT) not in sys.path:
    sys.path.insert(0, str(MODULE_ROOT))

from float_exact import (
    F32,
    F64,
    BinaryFormat,
    bits_hex,
    canonical_spelling,
    finite_value,
    parse_decimal_literal,
    round_rational,
    rounded_literal_bits,
    rounding_interval,
)


VERIFIER_ROOT = Path(__file__).resolve().parents[1]
CANDIDATE_PATH = VERIFIER_ROOT / "proposal" / "kernel-spec-successor-candidate.md"
EVIDENCE_PATH = VERIFIER_ROOT / "evidence" / "float-canonicality.json"
SOURCE_MANIFEST_PATH = VERIFIER_ROOT / "proposal" / "FLOAT_CANONICAL_SOURCES"
EVIDENCE_SOURCE_PATHS = (
    "proposal/float_canonical_evidence.py",
    "proposal/float_exact.py",
    "tests/test_float_canonicality.py",
)

_FLOAT_GRAMMAR = r"-?(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE"
_FLOAT_CLAUSE = (
    "finite floats use the grammar `"
    + _FLOAT_GRAMMAR
    + "`, where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), "
    "positive exponents carry no sign, negative exponents carry one `-`, and only the "
    "integer and exponent components have the stated no-leading-zero form. Let C be the "
    "nonnegative integer formed by concatenating the integer and fraction digits, let F "
    "be the number of fraction digits, and let E be the signed integer formed by the "
    "exponent digits and their optional `-`; when the exponent is absent E is zero, and "
    "`e-0` also gives E zero. A matching decimal whose C is zero denotes signed decimal "
    "zero: a leading literal `-` selects negative zero and its absence selects positive "
    "zero, independently of E. Every other matching decimal denotes the exact nonzero "
    "rational whose magnitude is C × 10^(E − F), with the leading literal sign applied. "
    "For one finite bit "
    "pattern of TYPE, consider every matching decimal that rounds from that "
    "signed zero or exact nonzero rational to the bit pattern under IEEE 754 "
    "round-to-nearest, ties-to-even. Its canonical spelling is the candidate with the "
    "fewest ASCII bytes "
    "before `_TYPE`; a tie is resolved by lexicographically least unsigned ASCII bytes. "
    "This selection is total, host-independent, and unique; in particular `0.0` and "
    "`-0.0` remain distinct. Other examples are `1.5_f64` and `6.022e23_f64`."
)
_FORM7_PAYLOAD = (
    "Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal "
    "where its signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, "
    "unsigned `[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own "
    "form, a leading `-` is legal for signed T, and `-0` is written `0`. A float literal "
    "is legal only when it has the unique canonical spelling selected by [FORM-5] and "
    "denotes a finite value of its stated TYPE. An out-of-range integer, a leading-zero "
    "integer, a noncanonical float spelling, or a float decimal that rounds to a non-finite "
    "value is a hard error at check time [SCOPE-2]; a literal never denotes a wrapped, "
    "truncated, saturated, or undefined value."
)

_CANONICAL_TARGETS = (
    ("f32-positive-zero", F32, 0x00000000, "signed-zero"),
    ("f32-negative-zero", F32, 0x80000000, "signed-zero"),
    ("f32-one", F32, 0x3F800000, "ordinary"),
    ("f32-negative-one", F32, 0xBF800000, "ordinary"),
    ("f32-one-and-half", F32, 0x3FC00000, "ordinary"),
    ("f32-minimum-subnormal", F32, 0x00000001, "subnormal-boundary"),
    ("f32-maximum-subnormal", F32, 0x007FFFFF, "normal-boundary"),
    ("f32-minimum-normal", F32, 0x00800000, "normal-boundary"),
    ("f32-maximum-finite", F32, 0x7F7FFFFF, "finite-boundary"),
    ("f32-fixed-vs-exponent", F32, 0x447A0000, "fixed-vs-exponent"),
    ("f64-positive-zero", F64, 0x0000000000000000, "signed-zero"),
    ("f64-negative-zero", F64, 0x8000000000000000, "signed-zero"),
    ("f64-one", F64, 0x3FF0000000000000, "ordinary"),
    ("f64-negative-one", F64, 0xBFF0000000000000, "ordinary"),
    ("f64-one-and-half", F64, 0x3FF8000000000000, "ordinary"),
    ("f64-minimum-subnormal", F64, 0x0000000000000001, "subnormal-boundary"),
    ("f64-maximum-subnormal", F64, 0x000FFFFFFFFFFFFF, "normal-boundary"),
    ("f64-minimum-normal", F64, 0x0010000000000000, "normal-boundary"),
    ("f64-maximum-finite", F64, 0x7FEFFFFFFFFFFFFF, "finite-boundary"),
    ("f64-fixed-vs-exponent", F64, 0x408F400000000000, "fixed-vs-exponent"),
)


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _rule_line(candidate: bytes, rule: str) -> tuple[int, bytes]:
    prefix = f"[{rule}] ".encode("ascii")
    matches = [
        (number, line)
        for number, line in enumerate(candidate.splitlines(), 1)
        if line.startswith(prefix)
    ]
    if len(matches) != 1:
        raise ValueError(f"candidate must contain exactly one {rule} line")
    return matches[0]


def extract_contract(candidate: bytes) -> dict[str, object]:
    """Independently extract and exact-shape-check the two normative rule lines."""

    try:
        candidate.decode("utf-8", "strict")
    except UnicodeDecodeError as error:
        raise ValueError("candidate is not UTF-8") from error
    form5_line, form5 = _rule_line(candidate, "FORM-5")
    form7_line, form7 = _rule_line(candidate, "FORM-7")
    float_clause = _FLOAT_CLAUSE.encode("utf-8")
    if form5.count(float_clause) != 1:
        raise ValueError("FORM-5 finite-float contract changed")
    before, after = form5.split(float_clause)
    if not before.endswith(b"); ") or not after.startswith(b" `unit`; STRING"):
        raise ValueError("FORM-5 float clause boundary changed")
    expected_form7 = b"[FORM-7] " + _FORM7_PAYLOAD.encode("utf-8")
    if form7 != expected_form7:
        raise ValueError("FORM-7 finite-float contract changed")
    return {
        "canonical_order": ["fewest-prefix-ascii-bytes", "unsigned-ascii-lexicographic"],
        "finite_only": True,
        "float_grammar": _FLOAT_GRAMMAR,
        "formats": {
            "f32": "IEEE 754 binary32",
            "f64": "IEEE 754 binary64",
        },
        "form5": {
            "line": form5_line,
            "sha256": _sha256(form5),
        },
        "form7": {
            "line": form7_line,
            "sha256": _sha256(form7),
        },
        "rounding": "IEEE 754 round-to-nearest-ties-to-even",
        "signed_zero": "all-zero-significand-sign-is-preserved-independent-of-exponent",
        "value_domain": "signed-zero-or-sign-times-C-times-10-to-E-minus-F",
        "zero_exponent": "absent-and-e-0-both-mean-E-equals-zero",
    }


def _rational(value: Fraction) -> dict[str, str]:
    return {
        "numerator": str(value.numerator),
        "denominator": str(value.denominator),
    }


def _canonical_vectors() -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for identifier, binary_format, bits, category in _CANONICAL_TARGETS:
        spelling = canonical_spelling(binary_format, bits)
        result.append(
            {
                "bits": bits_hex(binary_format, bits),
                "category": category,
                "id": identifier,
                "spelling": spelling,
                "target_exact_value": _rational(finite_value(binary_format, bits)),
            }
        )
    return result


def _tie_case(
    identifier: str,
    binary_format: BinaryFormat,
    lower_bits: int,
    upper_bits: int,
    expected_bits: int,
    *,
    negative: bool = False,
) -> dict[str, object]:
    lower = finite_value(binary_format, lower_bits)
    upper = finite_value(binary_format, upper_bits)
    midpoint = (lower + upper) / 2
    actual = round_rational(binary_format, midpoint, negative=negative)
    signed_expected = expected_bits | (binary_format.sign_mask if negative else 0)
    if actual != signed_expected:
        raise AssertionError(f"tie vector {identifier} did not round as declared")
    return {
        "expected_bits": bits_hex(binary_format, signed_expected),
        "format": binary_format.name,
        "id": identifier,
        "magnitude": _rational(midpoint),
        "negative": negative,
    }


def _tie_vectors() -> list[dict[str, object]]:
    vectors = [
        _tie_case("f32-tie-even-lower", F32, 0x3F800000, 0x3F800001, 0x3F800000),
        _tie_case("f32-tie-even-upper", F32, 0x3F800001, 0x3F800002, 0x3F800002),
        _tie_case(
            "f32-negative-tie-even-lower",
            F32,
            0x3F800000,
            0x3F800001,
            0x3F800000,
            negative=True,
        ),
        _tie_case(
            "f64-tie-even-lower",
            F64,
            0x3FF0000000000000,
            0x3FF0000000000001,
            0x3FF0000000000000,
        ),
        _tie_case(
            "f64-tie-even-upper",
            F64,
            0x3FF0000000000001,
            0x3FF0000000000002,
            0x3FF0000000000002,
        ),
    ]
    for binary_format in (F32, F64):
        zero_tie = rounding_interval(binary_format, 0).upper
        zero_bits = round_rational(binary_format, zero_tie)
        if zero_bits != 0:
            raise AssertionError("underflow midpoint did not select even zero")
        vectors.append(
            {
                "expected_bits": bits_hex(binary_format, 0),
                "format": binary_format.name,
                "id": f"{binary_format.name}-underflow-tie-to-zero",
                "magnitude": _rational(zero_tie),
                "negative": False,
            }
        )
        maximum = binary_format.maximum_finite_magnitude
        overflow_tie = rounding_interval(binary_format, maximum).upper
        infinity = round_rational(binary_format, overflow_tie)
        if infinity != binary_format.infinity_magnitude:
            raise AssertionError("overflow midpoint did not select infinity")
        vectors.append(
            {
                "expected_bits": bits_hex(binary_format, binary_format.infinity_magnitude),
                "format": binary_format.name,
                "id": f"{binary_format.name}-overflow-tie-to-infinity",
                "magnitude": _rational(overflow_tie),
                "negative": False,
            }
        )
    return sorted(vectors, key=lambda item: str(item["id"]))


def _classification_vectors() -> list[dict[str, object]]:
    sources = (
        ("canonical-f32-one", "1.0_f32", "canonical"),
        ("noncanonical-f32-one", "1.00_f32", "noncanonical-same-bits"),
        ("canonical-f64-example", "6.022e23_f64", "canonical"),
        ("exponent-negative-zero", "1.0e-0_f32", "noncanonical-same-bits"),
        ("signed-zero-exponent-independent", "-0.00e999_f64", "noncanonical-same-bits"),
        ("positive-exponent-has-no-plus", "1.0e+1_f32", "grammar-reject"),
        ("integer-leading-zero", "01.0_f32", "grammar-reject"),
        ("exponent-leading-zero", "1.0e01_f64", "grammar-reject"),
        ("rounds-to-infinity", "9.9e999_f32", "non-finite"),
    )
    result: list[dict[str, object]] = []
    for identifier, source, expected in sources:
        try:
            parsed = parse_decimal_literal(source)
        except ValueError:
            if expected != "grammar-reject":
                raise
            result.append({"id": identifier, "source": source, "verdict": expected})
            continue
        bits = rounded_literal_bits(source)
        finite = (bits & ~parsed.format.sign_mask) <= parsed.format.maximum_finite_magnitude
        if not finite:
            verdict = "non-finite"
            canonical = None
        else:
            canonical = canonical_spelling(parsed.format, bits)
            verdict = "canonical" if canonical == source else "noncanonical-same-bits"
        if verdict != expected:
            raise AssertionError(f"classification vector {identifier} changed")
        result.append(
            {
                "canonical": canonical,
                "id": identifier,
                "rounded_bits": bits_hex(parsed.format, bits),
                "source": source,
                "verdict": verdict,
            }
        )
    return result


def build_evidence(candidate: bytes) -> dict[str, object]:
    expected_manifest = "".join(path + "\n" for path in EVIDENCE_SOURCE_PATHS).encode(
        "ascii"
    )
    manifest = SOURCE_MANIFEST_PATH.read_bytes()
    if manifest != expected_manifest:
        raise ValueError("float evidence source manifest changed")
    source_revisions = {}
    for path in EVIDENCE_SOURCE_PATHS:
        raw = (VERIFIER_ROOT / path).read_bytes()
        source_revisions["grammar-verifier/" + path] = {
            "byte_length": len(raw),
            "sha256": _sha256(raw),
        }
    return {
        "authority": "proposal-only exact-rational evidence; not language or compiler authority",
        "candidate": {
            "byte_length": len(candidate),
            "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
            "sha256": _sha256(candidate),
        },
        "canonical_vectors": _canonical_vectors(),
        "classification_vectors": _classification_vectors(),
        "contract": extract_contract(candidate),
        "model": {
            "canonical_search": (
                "ascending prefix byte length; exhaustive grammar structures; "
                "exact rational coefficient intervals; unsigned ASCII tie-break"
            ),
            "host_float_parsing_or_formatting": False,
            "integer_and_fraction_arithmetic_only": True,
            "maximum_canonical_prefix_search_bytes": 64,
            "source_manifest": {
                "path": "grammar-verifier/proposal/FLOAT_CANONICAL_SOURCES",
                "sha256": _sha256(manifest),
            },
        },
        "rounding_vectors": _tie_vectors(),
        "schema": "whitefoot.float-canonicality-evidence.v1",
        "source_revisions": source_revisions,
    }


def canonical_json(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True) + "\n"
    ).encode("ascii")


def _atomic_write(path: Path, raw: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            os.fchmod(stream.fileno(), 0o644)
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--print", dest="print_evidence", action="store_true")
    mode.add_argument("--write", action="store_true")
    arguments = parser.parse_args()
    try:
        rendered = canonical_json(build_evidence(CANDIDATE_PATH.read_bytes()))
        if arguments.print_evidence:
            sys.stdout.buffer.write(rendered)
            return 0
        if arguments.write:
            _atomic_write(EVIDENCE_PATH, rendered)
            return 0
        if EVIDENCE_PATH.read_bytes() != rendered:
            raise ValueError("committed float evidence is stale")
    except (OSError, ValueError, AssertionError) as error:
        print(f"float canonicality evidence: {error}", file=sys.stderr)
        return 1
    print("float canonicality evidence: exact proposal model agrees")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
