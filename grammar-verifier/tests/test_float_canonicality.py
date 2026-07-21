from __future__ import annotations

import ast
from fractions import Fraction
import json
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

from float_canonical_evidence import (  # noqa: E402
    build_evidence,
    canonical_json,
    extract_contract,
)
from float_exact import (  # noqa: E402
    F32,
    F64,
    canonical_spelling,
    parse_decimal_literal,
    round_rational,
    rounded_literal_bits,
)


CANDIDATE = PROPOSAL / "kernel-spec-successor-candidate.md"
EVIDENCE = ROOT / "evidence" / "float-canonicality.json"

EXPECTED_CANONICAL = {
    ("f32", 0x00000000): "0.0_f32",
    ("f32", 0x80000000): "-0.0_f32",
    ("f32", 0x3F800000): "1.0_f32",
    ("f32", 0xBF800000): "-1.0_f32",
    ("f32", 0x3FC00000): "1.5_f32",
    ("f32", 0x00000001): "0.1e-44_f32",
    ("f32", 0x007FFFFF): "1.1754942e-38_f32",
    ("f32", 0x00800000): "1.1754943e-38_f32",
    ("f32", 0x7F7FFFFF): "3.4028234e38_f32",
    ("f32", 0x447A0000): "0.1e4_f32",
    ("f64", 0x0000000000000000): "0.0_f64",
    ("f64", 0x8000000000000000): "-0.0_f64",
    ("f64", 0x3FF0000000000000): "1.0_f64",
    ("f64", 0xBFF0000000000000): "-1.0_f64",
    ("f64", 0x3FF8000000000000): "1.5_f64",
    ("f64", 0x0000000000000001): "0.3e-323_f64",
    ("f64", 0x000FFFFFFFFFFFFF): "2.225073858507201e-308_f64",
    ("f64", 0x0010000000000000): "2.2250738585072012e-308_f64",
    ("f64", 0x7FEFFFFFFFFFFFFF): "1.7976931348623157e308_f64",
    ("f64", 0x408F400000000000): "0.1e4_f64",
}


class FloatContractTests(unittest.TestCase):
    def test_exact_candidate_contract_is_bound_to_formats_and_rounding(self) -> None:
        contract = extract_contract(CANDIDATE.read_bytes())
        self.assertEqual(
            contract["formats"],
            {"f32": "IEEE 754 binary32", "f64": "IEEE 754 binary64"},
        )
        self.assertEqual(
            contract["rounding"], "IEEE 754 round-to-nearest-ties-to-even"
        )
        self.assertEqual(
            contract["value_domain"],
            "signed-zero-or-sign-times-C-times-10-to-E-minus-F",
        )
        self.assertEqual(
            contract["zero_exponent"], "absent-and-e-0-both-mean-E-equals-zero"
        )

    def test_each_load_bearing_contract_fragment_fails_closed(self) -> None:
        candidate = CANDIDATE.read_bytes()
        mutations = (
            (b"`f32` (IEEE 754 binary32)", b"`f32`"),
            (b"`f64` (IEEE 754 binary64)", b"`f64`"),
            (b"positive exponents carry no sign", b"positive exponents may use `+`"),
            (
                b"C be the nonnegative integer formed by concatenating the integer and fraction digits",
                b"C be a host coefficient",
            ),
            (b"F be the number of fraction digits", b"F be implementation-defined"),
            (
                b"E be the signed integer formed by the exponent digits and their optional `-`",
                b"E be a host exponent",
            ),
            (b"`e-0` also gives E zero", b"`e-0` is implementation-defined"),
            (b"independently of E", b"except when E is nonzero"),
            (
                b"magnitude is C \xc3\x97 10^(E \xe2\x88\x92 F), with the leading literal sign applied",
                b"magnitude is host-defined",
            ),
            (
                b"IEEE 754 round-to-nearest, ties-to-even",
                b"round-to-nearest, ties-away",
            ),
            (b"fewest ASCII bytes", b"fewest decimal digits"),
            (
                b"lexicographically least unsigned ASCII bytes",
                b"implementation-defined ordering",
            ),
            (b"total, host-independent, and unique", b"host-selected"),
            (
                b"unique canonical spelling selected by [FORM-5]",
                b"any FORM-5 spelling",
            ),
            (b"denotes a finite value of its stated TYPE", b"denotes a value"),
        )
        for old, new in mutations:
            with self.subTest(old=old):
                self.assertEqual(candidate.count(old), 1)
                with self.assertRaises(ValueError):
                    extract_contract(candidate.replace(old, new, 1))


class ExactRoundingTests(unittest.TestCase):
    def test_parser_denotes_exact_rationals_without_host_floats(self) -> None:
        parsed = parse_decimal_literal("6.022e23_f64")
        self.assertEqual(parsed.magnitude, Fraction(6022 * 10**20))
        self.assertFalse(parsed.negative)
        self.assertEqual(
            parse_decimal_literal("1.25e-2_f64").magnitude,
            Fraction(1, 80),
        )
        self.assertEqual(
            parse_decimal_literal("1.25e-0_f64").magnitude,
            Fraction(5, 4),
        )
        for source in (
            "0.0_f32",
            "-0.0e-0_f64",
            "1.00e0_f32",
            "6.022e23_f64",
        ):
            with self.subTest(source=source):
                parse_decimal_literal(source)
        for source in (
            "00.0_f32",
            ".0_f32",
            "0._f32",
            "1.0e+1_f32",
            "1.0e01_f64",
            "1.0E1_f64",
            "1.0",
            "nan_f32",
        ):
            with self.subTest(source=source):
                with self.assertRaises(ValueError):
                    parse_decimal_literal(source)

    def test_all_zero_significands_preserve_sign_independent_of_exponent(self) -> None:
        self.assertEqual(rounded_literal_bits("0.000e-999_f32"), 0x00000000)
        self.assertEqual(rounded_literal_bits("-0.000e999_f32"), 0x80000000)
        self.assertEqual(
            rounded_literal_bits("-0.000e-999_f64"), 0x8000000000000000
        )

    def test_exact_midpoints_select_the_even_significand(self) -> None:
        f32_lower_tie = Fraction((1 << 24) + 1, 1 << 24)
        f32_upper_tie = Fraction((1 << 24) + 3, 1 << 24)
        self.assertEqual(round_rational(F32, f32_lower_tie), 0x3F800000)
        self.assertEqual(round_rational(F32, f32_upper_tie), 0x3F800002)
        self.assertEqual(
            round_rational(F32, f32_lower_tie, negative=True), 0xBF800000
        )

        f64_lower_tie = Fraction((1 << 53) + 1, 1 << 53)
        f64_upper_tie = Fraction((1 << 53) + 3, 1 << 53)
        self.assertEqual(
            round_rational(F64, f64_lower_tie), 0x3FF0000000000000
        )
        self.assertEqual(
            round_rational(F64, f64_upper_tie), 0x3FF0000000000002
        )

    def test_underflow_and_overflow_ties_are_exact(self) -> None:
        self.assertEqual(round_rational(F32, Fraction(1, 1 << 150)), 0)
        self.assertEqual(
            round_rational(F32, Fraction(1, 1 << 150), negative=True),
            0x80000000,
        )
        self.assertEqual(round_rational(F32, Fraction(3, 1 << 151)), 1)
        self.assertEqual(round_rational(F64, Fraction(1, 1 << 1075)), 0)
        self.assertEqual(round_rational(F64, Fraction(3, 1 << 1076)), 1)

        f32_overflow_tie = Fraction((1 << 128) - (1 << 103))
        f64_overflow_tie = Fraction((1 << 1024) - (1 << 970))
        self.assertEqual(round_rational(F32, f32_overflow_tie - 1), 0x7F7FFFFF)
        self.assertEqual(round_rational(F32, f32_overflow_tie), 0x7F800000)
        self.assertEqual(
            round_rational(F64, f64_overflow_tie - 1), 0x7FEFFFFFFFFFFFFF
        )
        self.assertEqual(
            round_rational(F64, f64_overflow_tie), 0x7FF0000000000000
        )


class CanonicalSelectionTests(unittest.TestCase):
    def test_representative_f32_and_f64_spellings_are_pinned(self) -> None:
        formats = {"f32": F32, "f64": F64}
        for (format_name, bits), expected in EXPECTED_CANONICAL.items():
            with self.subTest(format=format_name, bits=hex(bits)):
                actual = canonical_spelling(formats[format_name], bits)
                self.assertEqual(actual, expected)
                self.assertEqual(rounded_literal_bits(actual), bits)

    def test_fixed_vs_exponent_tie_uses_unsigned_ascii_order(self) -> None:
        for suffix, expected_bits in (("f32", 0x447A0000), ("f64", 0x408F400000000000)):
            alternatives = [f"0.1e4_{suffix}", f"1.0e3_{suffix}", f"1000.0_{suffix}"]
            with self.subTest(suffix=suffix):
                self.assertTrue(
                    all(rounded_literal_bits(item) == expected_bits for item in alternatives)
                )
                shortest = min(
                    alternatives,
                    key=lambda item: (len(item.split("_", 1)[0]), item.encode("ascii")),
                )
                self.assertEqual(shortest, f"0.1e4_{suffix}")

    def test_maximum_f32_lexicographic_near_miss_is_noncanonical(self) -> None:
        self.assertEqual(rounded_literal_bits("3.4028234e38_f32"), 0x7F7FFFFF)
        self.assertEqual(rounded_literal_bits("3.4028235e38_f32"), 0x7F7FFFFF)
        self.assertEqual(canonical_spelling(F32, 0x7F7FFFFF), "3.4028234e38_f32")

    def test_authored_examples_are_canonical(self) -> None:
        self.assertEqual(canonical_spelling(F64, rounded_literal_bits("1.5_f64")), "1.5_f64")
        self.assertEqual(
            canonical_spelling(F64, rounded_literal_bits("6.022e23_f64")),
            "6.022e23_f64",
        )


class EvidenceIntegrityTests(unittest.TestCase):
    def test_committed_evidence_is_exactly_regenerated(self) -> None:
        expected = canonical_json(build_evidence(CANDIDATE.read_bytes()))
        self.assertEqual(EVIDENCE.read_bytes(), expected)
        parsed = json.loads(expected)
        self.assertEqual(parsed["schema"], "whitefoot.float-canonicality-evidence.v1")
        self.assertIn("not language or compiler authority", parsed["authority"])

    def test_source_manifest_is_closed_sorted_and_complete(self) -> None:
        declared = (PROPOSAL / "FLOAT_CANONICAL_SOURCES").read_text(
            encoding="ascii"
        ).splitlines()
        expected = [
            "proposal/float_canonical_evidence.py",
            "proposal/float_exact.py",
            "tests/test_float_canonicality.py",
        ]
        self.assertEqual(declared, sorted(declared))
        self.assertEqual(declared, expected)
        self.assertTrue(all((ROOT / path).is_file() for path in declared))

    def test_exact_model_has_no_host_float_authority(self) -> None:
        path = PROPOSAL / "float_exact.py"
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        forbidden_modules = {"decimal", "math", "numpy", "struct"}
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = {alias.name.split(".", 1)[0] for alias in node.names}
            elif isinstance(node, ast.ImportFrom):
                names = {(node.module or "").split(".", 1)[0]}
            else:
                names = set()
            self.assertTrue(names.isdisjoint(forbidden_modules))
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                self.assertNotEqual(node.func.id, "float")


if __name__ == "__main__":
    unittest.main()
