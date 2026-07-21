"""Closed-notation extraction and common-ledger regressions."""

from __future__ import annotations

import unittest

from core import Failure, LogicalBudget
from ebnf import walk_nodes
from extract import extract_document
from source import scan_source
from support import fixture_grammars, fixture_inputs


class ExtractionTests(unittest.TestCase):
    def test_exact_documents_have_complete_expected_census(self) -> None:
        current, proposal = fixture_grammars()
        expected = {
            "current": (91, 59, 8, 169, 176),
            "proposal": (92, 62, 7, 169, 178),
        }
        for grammar in (current, proposal):
            with self.subTest(document=grammar.name):
                (
                    rule_count,
                    production_count,
                    lexical_count,
                    fixed_count,
                    reference_count,
                ) = expected[grammar.name]
                self.assertEqual(len(grammar.rules), rule_count)
                self.assertEqual(len(grammar.productions), production_count)
                self.assertEqual(len(grammar.lexical), lexical_count)
                self.assertEqual(len(grammar.fixed), fixed_count)
                self.assertEqual(len(grammar.references), reference_count)
                expected_coverage = {
                    "current": (production_count, 4, 2, 3, 0),
                    "proposal": (production_count, 4, 4, 2, 0),
                }
                self.assertEqual(
                    (
                        grammar.coverage.assignments,
                        grammar.coverage.fences,
                        grammar.coverage.inline,
                        grammar.coverage.lexical_cues,
                        grammar.coverage.unclassified,
                    ),
                    expected_coverage[grammar.name],
                )
                self.assertEqual(len(grammar.source_lowerwords), 47)
                self.assertEqual(len(grammar.expanded_lowerwords), 48)
                self.assertIn(b"uniq", grammar.expanded_lowerwords)
                self.assertNotIn(b"uniq", grammar.source_lowerwords)

    def test_proposal_productions_retain_their_numbered_rule_owners(self) -> None:
        _current, proposal = fixture_grammars()
        expected = {
            "program": "GRAM-2",
            "requires_block": "GRAM-2",
            "requires_entry": "GRAM-2",
            "law": "GRAM-2",
            "const": "CONST-1",
            "cvalue": "CONST-2",
            "effects": "EFF-1",
            "effect": "EFF-1",
        }
        self.assertEqual(
            {
                name: proposal.production_by_name[name].owner
                for name in expected
            },
            expected,
        )

    def test_rule_heading_inside_fence_is_masked(self) -> None:
        source = (
            b"[GRAM-1] Grammar.\n"
            b"```\n"
            b"root := \"x\"\n"
            b"[FAKE-9] This is fence content, not a rule.\n"
            b"```\n"
            b"[FORM-1] Next rule.\n"
        )
        scan = scan_source(source, LogicalBudget(fixture_inputs().limits))
        self.assertEqual([rule.owner for rule in scan.rules], ["GRAM-1", "FORM-1"])
        self.assertEqual(scan.regions[0].owner, "GRAM-1")

    def test_malformed_fence_masks_duplicate_looking_heading_first(self) -> None:
        sources = (
            (
                b"[GRAM-1] Grammar.\n"
                b"```ebnf\n"
                b"[GRAM-1] Looks duplicate but is still fenced.\n"
                b"```\n",
                "malformed_fence",
            ),
            (
                b"[GRAM-1] Grammar.\n"
                b"```\n"
                b"root := \"x\"\n"
                b"[GRAM-1] Looks duplicate but is still fenced.\n",
                "unterminated_fence",
            ),
        )
        for source, code in sources:
            with self.subTest(code=code):
                with self.assertRaises(Failure) as raised:
                    scan_source(source, LogicalBudget(fixture_inputs().limits))
                self.assertEqual(
                    (raised.exception.family, raised.exception.code),
                    ("extraction", code),
                )

    def test_ident_modifier_changes_exact_semantic_span_and_membership(self) -> None:
        current, proposal = fixture_grammars()
        current_ident = current.lexical_by_name["IDENT"]
        proposal_ident = proposal.lexical_by_name["IDENT"]
        self.assertEqual(current_ident.predicate, b"pattern=[a-z][a-z0-9_]*;exclude=none")
        self.assertEqual(
            proposal_ident.predicate,
            b"pattern=[a-z][a-z0-9_]*;exclude=fixed-lowerwords",
        )
        self.assertEqual(
            current.source[current_ident.start : current_ident.end],
            b"IDENT `[a-z][a-z0-9_]*`",
        )
        self.assertTrue(
            proposal.source[proposal_ident.start : proposal_ident.end].endswith(
                b"complete grammar"
            )
        )

    def test_comment_is_outside_production_and_rhs_spans(self) -> None:
        current, _proposal = fixture_grammars()
        production = current.production_by_name["cvalue"]
        definition = current.source[production.definition_start : production.definition_end]
        rhs = current.source[production.rhs_start : production.rhs_end]
        self.assertNotIn(b"#", definition)
        self.assertNotIn(b"#", rhs)
        self.assertTrue(rhs.endswith(b'"]"'))

    def test_hash_inside_quoted_atom_is_not_a_line_comment(self) -> None:
        inputs = fixture_inputs()
        mutated = inputs.current.data.replace(b'"struct" TYPEID', b'"#" TYPEID', 1)
        grammar = extract_document("mutant", mutated, inputs.limits)
        self.assertIn(b"#", {item.spelling for item in grammar.fixed})

    def test_blank_grammar_line_preserves_the_pending_production(self) -> None:
        inputs = fixture_inputs()
        current, _proposal = fixture_grammars()
        anchor = (
            b'fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"\n'
            b'                "->" rtype effects requires_block? "{" doc? stmt* "}"'
        )
        replacement = anchor.replace(b"\n", b"\n\n", 1)
        self.assertEqual(inputs.current.data.count(anchor), 1)
        mutated = inputs.current.data.replace(anchor, replacement, 1)
        grammar = extract_document("mutant", mutated, inputs.limits)

        baseline = current.production_by_name["fn_decl"]
        observed = grammar.production_by_name["fn_decl"]
        baseline_shape = tuple(
            (path, node.kind, node.value) for path, node in walk_nodes(baseline.root)
        )
        observed_shape = tuple(
            (path, node.kind, node.value) for path, node in walk_nodes(observed.root)
        )
        self.assertEqual(observed_shape, baseline_shape)
        self.assertEqual(len(grammar.productions), len(current.productions))
        self.assertEqual(grammar.coverage, current.coverage)
        self.assertEqual(observed.definition_start, baseline.definition_start)
        self.assertEqual(observed.rhs_start, baseline.rhs_start)
        self.assertEqual(observed.definition_end, baseline.definition_end + 1)
        self.assertIn(
            b'param_list? ")"\n\n                "->"',
            grammar.source[observed.rhs_start : observed.definition_end],
        )

    def test_raw_single_equals_grammar_candidate_is_rejected(self) -> None:
        inputs = fixture_inputs()
        for candidate in (b"shadow = item\n", b'rogue="x"\n'):
            with self.subTest(candidate=candidate):
                mutated = inputs.current.data + candidate
                with self.assertRaises(Failure) as raised:
                    extract_document("mutant", mutated, inputs.limits)
                self.assertEqual(
                    (raised.exception.family, raised.exception.code),
                    ("extraction", "single_equals_grammar"),
                )

    def test_unsupported_or_uncovered_notation_fails_closed(self) -> None:
        inputs = fixture_inputs()
        source = inputs.current.data
        form3 = next(line for line in source.splitlines() if line.startswith(b"[FORM-3] "))
        duplicate_declaration = source + b"[FORM-99] " + form3.split(b"] ", 1)[1] + b"\n"
        mutations = (
            (
                duplicate_declaration,
                "duplicate_lexical_definition",
            ),
            (
                source.replace(
                    b"IDENT `[a-z][a-z0-9_]*`; TYPEID",
                    b"TYPEID `[A-Z][A-Za-z0-9]*`; IDENT",
                    1,
                ),
                "lexical_classes_shape",
            ),
            (source.replace(b"Lexical classes:", b"Token classes:", 1), "unclassified_lexical_cue"),
            (
                source.replace(b"IDENT `[a-z][a-z0-9_]*`", b"IDENT `[a-z](?=x)`", 1),
                "lexical_classes_shape",
            ),
            (source.replace(b"program      := item*", b"program      := (item?)*", 1), "nullable_repetition"),
            (source.replace(b"program      := item*", b"program      := missing*", 1), "undefined_reference"),
            (source.replace(b"program      := item*", b"program       = item*", 1), "single_equals_grammar"),
            (source.replace(b"\n", b" \n", 1), "trailing_whitespace"),
            (b"outside := not grammar.\n" + source, "unowned_assignment"),
            (source + b"Prose := not grammar.\n", "assignment_coverage"),
        )
        for ordinal, (mutated, code) in enumerate(mutations):
            with self.subTest(ordinal=ordinal, code=code):
                with self.assertRaises(Failure) as raised:
                    extract_document("mutant", mutated, inputs.limits)
                self.assertEqual(raised.exception.family, "extraction")
                self.assertEqual(raised.exception.code, code)


if __name__ == "__main__":
    unittest.main()
