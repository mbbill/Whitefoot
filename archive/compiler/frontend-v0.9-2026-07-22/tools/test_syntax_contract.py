#!/usr/bin/env python3
"""Hostile tests for exact-v0.9 source grammar extraction and data binding."""

from __future__ import annotations

import unittest

import syntax_contract as contract
from check_syntax_contract import validate


class SyntaxContractTests(unittest.TestCase):
    def test_live_contract_has_complete_closed_inventory(self) -> None:
        grammar = contract.build_grammar(
            contract.SPEC.read_bytes(), contract.TERMINALS.read_text(encoding="utf-8")
        )
        self.assertEqual(len(grammar.productions), 62)
        self.assertEqual(len(grammar.nodes), 514)
        self.assertEqual(len(grammar.decisions), 72)
        self.assertEqual(len(grammar.diagnostic_order), 72)

    def test_source_definition_order_is_exact(self) -> None:
        definitions = contract.production_texts(contract.SPEC.read_bytes())
        self.assertEqual(tuple(name for name, _ in definitions), contract.PRODUCTION_NAMES)

    def test_inline_productions_are_not_lost(self) -> None:
        definitions = dict(contract.production_texts(contract.SPEC.read_bytes()))
        self.assertIn('"[0-9]+" | IDENT', definitions["const"])
        self.assertIn('"allocates"', definitions["effect"])

    def test_compound_fixed_atom_keeps_one_source_provenance(self) -> None:
        self.assertEqual(contract.expand_fixed("&uniq"), (b"&", b"uniq"))
        self.assertEqual(contract.expand_fixed("->"), (b"->",))

    def test_generated_row_mutation_is_rejected(self) -> None:
        expected = contract.expected_generated()
        actual = expected.replace("SelectRowV0_9::new(0,", "SelectRowV0_9::new(1,", 1)
        with self.assertRaisesRegex(contract.ContractError, "differs"):
            validate(actual, expected)

    def test_generated_provenance_mutation_is_rejected(self) -> None:
        expected = contract.expected_generated()
        actual = expected.replace("Some(GrammarNodeIdV0_9::new(", "None /* changed */ //", 1)
        with self.assertRaisesRegex(contract.ContractError, "differs"):
            validate(actual, expected)

    def test_generated_production_omission_is_rejected(self) -> None:
        expected = contract.expected_generated()
        actual = expected.replace("    ProductionV0_9::Program,\n", "", 1)
        with self.assertRaisesRegex(contract.ContractError, "differs"):
            validate(actual, expected)

    def test_nullable_repetition_is_rejected(self) -> None:
        parser = contract.EbnfParser('(IDENT?)*')
        root = parser.parse()
        production = contract.Production("program", "Gram2", root)
        contract.number_nodes([production])
        nullable_map = {"program": False}
        self.assertTrue(contract.nullable(root.children[0], nullable_map, {"program"}))


if __name__ == "__main__":
    unittest.main()
