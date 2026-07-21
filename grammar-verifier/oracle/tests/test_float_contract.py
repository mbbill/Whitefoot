"""Hostile extraction tests for the exact finite-float successor contract."""

from __future__ import annotations

import unittest

from core import Failure
from extract import extract_document
from support import fixture_grammars, fixture_inputs


class FloatContractExtractionTests(unittest.TestCase):
    def test_current_and_proposal_float_dialects_are_distinct(self) -> None:
        current, proposal = fixture_grammars()
        current_literal = current.lexical_by_name["literal"]
        proposal_literal = proposal.lexical_by_name["literal"]
        self.assertIn(b"float=-?[0-9]+\\.[0-9]+", current_literal.predicate)
        self.assertIn(
            b"float=-?(0|[1-9][0-9]*)\\.[0-9]+",
            proposal_literal.predicate,
        )
        self.assertIn(b"float-value=signed-zero-or-sign*C*10^(E-F)", proposal_literal.predicate)
        self.assertIn(b"e-0=0", proposal_literal.predicate)
        self.assertIn(b"canonical=min-prefix-bytes,ascii-lex", proposal_literal.predicate)

    def test_proposal_float_predicate_preserves_grammar_membership_only(self) -> None:
        _current, proposal = fixture_grammars()
        literal = proposal.lexical_by_name["literal"]
        fixed = frozenset(proposal.expanded_lowerwords)
        accepted = (
            b"0.0_f32",
            b"-0.0e-0_f64",
            b"1.5_f64",
            b"6.022e23_f64",
            b"1.00e-2_f32",
        )
        rejected = (
            b"01.0_f32",
            b"1.0e01_f64",
            b"1.0e+1_f64",
            b"1._f32",
            b".5_f32",
            b"1.0E2_f64",
        )
        for source in accepted:
            with self.subTest(source=source):
                self.assertEqual(literal.match_prefix(source, 0, fixed), len(source))
        for source in rejected:
            with self.subTest(source=source):
                self.assertIsNone(literal.match_prefix(source, 0, fixed))

    def test_every_load_bearing_float_clause_mutation_fails_closed(self) -> None:
        proposal = fixture_inputs().proposal.data
        mutants = (
            (b"`f32` (IEEE 754 binary32)", b"`f32`"),
            (b"`f64` (IEEE 754 binary64)", b"`f64`"),
            (
                b"C be the nonnegative integer formed by concatenating the integer and fraction digits",
                b"C be a host-decimal coefficient",
            ),
            (b"F be the number of fraction digits", b"F be implementation-defined"),
            (
                b"E be the signed integer formed by the exponent digits and their optional `-`",
                b"E be a host exponent",
            ),
            (b"`e-0` also gives E zero", b"`e-0` is implementation-defined"),
            (b"positive exponents carry no sign", b"positive exponents may carry a plus"),
            (b"independently of E", b"except when E is nonzero"),
            (
                b"magnitude is C \xc3\x97 10^(E \xe2\x88\x92 F), with the leading literal sign applied",
                b"magnitude is host-defined",
            ),
            (b"round-to-nearest, ties-to-even", b"round-to-nearest, ties-away"),
            (b"fewest ASCII bytes before `_TYPE`", b"fewest decimal digits"),
            (
                b"lexicographically least unsigned ASCII bytes",
                b"implementation-selected bytes",
            ),
            (b"total, host-independent, and unique", b"usually unique"),
            (
                b"unique canonical spelling selected by [FORM-5]",
                b"implementation canonical spelling",
            ),
            (b"denotes a finite value of its stated TYPE", b"denotes any value"),
        )
        for original, replacement in mutants:
            with self.subTest(original=original):
                self.assertEqual(proposal.count(original), 1)
                mutated = proposal.replace(original, replacement, 1)
                with self.assertRaises(Failure) as raised:
                    extract_document("mutant", mutated, fixture_inputs().limits)
                self.assertEqual(raised.exception.family, "extraction")
                self.assertEqual(raised.exception.code, "float_contract_shape")
