"""Independent token membership and source-tree derivation tests."""

from __future__ import annotations

import hashlib
import struct
import unittest

from core import Failure
from extract import extract_document
from generalized import compile_grammar, parse_source, tokenize_source
from report import analyze, render_success
from support import fixture_evidence, fixture_grammars, fixture_inputs, limits_with


def decode_tree(raw: bytes, cursor: int = 0) -> tuple[tuple[str, tuple[object, ...]], int]:
    if raw[cursor : cursor + 1] != b"T":
        raise ValueError("tree marker")
    cursor += 1
    label_size = struct.unpack(">I", raw[cursor : cursor + 4])[0]
    cursor += 4
    label = raw[cursor : cursor + label_size].decode("ascii")
    cursor += label_size
    child_count = struct.unpack(">I", raw[cursor : cursor + 4])[0]
    cursor += 4
    children: list[object] = []
    for _ in range(child_count):
        child_size = struct.unpack(">Q", raw[cursor : cursor + 8])[0]
        cursor += 8
        child, child_end = decode_tree(raw[cursor : cursor + child_size])
        if child_end != child_size:
            raise ValueError("child length")
        children.append(child)
        cursor += child_size
    return (label, tuple(children)), cursor


def tree_labels(tree: tuple[str, tuple[object, ...]]) -> tuple[str, ...]:
    label, children = tree
    return (label,) + tuple(
        nested
        for child in children
        for nested in tree_labels(child)  # type: ignore[arg-type]
    )


def tree_has_label_path(
    tree: tuple[str, tuple[object, ...]],
    path: tuple[str, ...],
) -> bool:
    label, children = tree
    if not path or label != path[0]:
        return False
    if len(path) == 1:
        return True
    return any(
        tree_has_label_path(child, path[1:])  # type: ignore[arg-type]
        for child in children
    )


class GeneralizedParserTests(unittest.TestCase):
    def test_zero_one_and_many_are_complete_source_tree_counts(self) -> None:
        current, proposal = fixture_grammars()
        for grammar, source, expected in (
            (current, b"x x", "zero"),
            (current, b"x", "one"),
            (current, b"deref(x)", "many"),
            (proposal, b"deref(x)", "one"),
        ):
            with self.subTest(document=grammar.name, source=source):
                result = parse_source(
                    grammar,
                    compile_grammar(grammar),
                    "expr",
                    source,
                    fixture_inputs().limits,
                )
                self.assertEqual(result.classification, expected)
                self.assertEqual(len(result.traces), {"zero": 0, "one": 1, "many": 2}[expected])
                for trace in result.traces:
                    _tree, end = decode_tree(trace)
                    self.assertEqual(end, len(trace))

    def test_fixed_and_lexical_token_alternatives_are_independent(self) -> None:
        current, proposal = fixture_grammars()
        current_tokens = tokenize_source(current, b"deref(x)", fixture_inputs().limits)
        proposal_tokens = tokenize_source(proposal, b"deref(x)", fixture_inputs().limits)
        self.assertEqual(
            current_tokens.slots[0].labels,
            ("fixed:6465726566", "lexical:4944454e54"),
        )
        self.assertEqual(proposal_tokens.slots[0].labels, ("fixed:6465726566",))

    def test_law_syntax_leaves_name_and_arity_to_the_checker(self) -> None:
        current, proposal = fixture_grammars()
        compiled_current = compile_grammar(current)
        compiled_proposal = compile_grammar(proposal)
        limits = fixture_inputs().limits
        for source in (
            b"law associative(combine);",
            b"law commutative(combine);",
            b"law identity(combine, zero);",
        ):
            with self.subTest(source=source):
                self.assertEqual(
                    parse_source(current, compiled_current, "law", source, limits).classification,
                    "one",
                )
                self.assertEqual(
                    parse_source(proposal, compiled_proposal, "law", source, limits).classification,
                    "one",
                )

        identity_literal = b"law identity(combine, 0_u64);"
        self.assertEqual(
            parse_source(
                current,
                compiled_current,
                "law",
                identity_literal,
                limits,
            ).classification,
            "zero",
        )
        proposal_result = parse_source(
            proposal,
            compiled_proposal,
            "law",
            identity_literal,
            limits,
        )
        self.assertEqual(proposal_result.classification, "one")
        proposal_tree, end = decode_tree(proposal_result.traces[0])
        self.assertEqual(end, len(proposal_result.traces[0]))
        self.assertIn("production:6c61775f617267", tree_labels(proposal_tree))

        for semantic_rejection in (
            b"law distributive(combine, combine);",
            b"law associative();",
            b"law associative(0_u64);",
            b"law identity(0_u64, combine);",
        ):
            with self.subTest(semantic_rejection=semantic_rejection):
                self.assertEqual(
                    parse_source(
                        proposal,
                        compiled_proposal,
                        "law",
                        semantic_rejection,
                        limits,
                    ).classification,
                    "one",
                )

        function_source = b"fn identity() -> own unit pure { return unit; }"
        function_result = parse_source(
            proposal,
            compiled_proposal,
            "fn_decl",
            function_source,
            limits,
        )
        self.assertEqual(function_result.classification, "one")
        tokens = tokenize_source(proposal, function_source, limits)
        identity_slot = next(
            slot for slot in tokens.slots if function_source[slot.start : slot.end] == b"identity"
        )
        self.assertEqual(
            identity_slot.labels,
            ("lexical:4944454e54",),
        )

    def test_deref_cases_retain_one_tree_and_remove_one(self) -> None:
        evidence = fixture_evidence()
        for identifier in ("deref-p", "deref-x"):
            current = next(
                item
                for item in evidence.cases
                if item.document == "current" and item.identifier == identifier
            )
            proposal = next(
                item
                for item in evidence.cases
                if item.document == "proposal" and item.identifier == identifier
            )
            deltas = [item for item in evidence.deltas if item.identifier == identifier]
            self.assertEqual(current.result.classification, "many")
            self.assertEqual(proposal.result.classification, "one")
            self.assertEqual([item.status for item in deltas], ["retained", "removed"])
            retained = next(item.trace for item in deltas if item.status == "retained")
            removed = next(item.trace for item in deltas if item.status == "removed")
            self.assertEqual(retained, proposal.result.traces[0])
            self.assertIn(retained, current.result.traces)
            self.assertIn(removed, current.result.traces)

            retained_tree, retained_end = decode_tree(retained)
            removed_tree, removed_end = decode_tree(removed)
            self.assertEqual(retained_end, len(retained))
            self.assertEqual(removed_end, len(removed))
            retained_labels = tree_labels(retained_tree)
            removed_labels = tree_labels(removed_tree)
            self.assertTrue(
                tree_has_label_path(
                    retained_tree,
                    (
                        "production:65787072",
                        "node:65787072:0:choice:0",
                        "node:65787072:0.0:ref:61746f6d",
                        "production:61746f6d",
                        "node:61746f6d:0:choice:2",
                        "node:61746f6d:0.2:ref:706c616365",
                        "production:706c616365",
                        "node:706c616365:0:sequence:-",
                        "node:706c616365:0.0:ref:7062617365",
                        "production:7062617365",
                        "node:7062617365:0:choice:1",
                        "node:7062617365:0.1:sequence:-",
                        "node:7062617365:0.1.0:fixed:6465726566",
                        "token:fixed:6465726566:0:5",
                    ),
                )
            )
            self.assertTrue(
                tree_has_label_path(
                    removed_tree,
                    (
                        "production:65787072",
                        "node:65787072:0:choice:1",
                        "node:65787072:0.1:ref:63616c6c",
                        "production:63616c6c",
                        "node:63616c6c:0:sequence:-",
                        "node:63616c6c:0.0:ref:63616c6c6565",
                        "production:63616c6c6565",
                        "node:63616c6c6565:0:choice:0",
                        "node:63616c6c6565:0.0:ref:4944454e54",
                        "token:lexical:4944454e54:0:5",
                    ),
                )
            )
            self.assertEqual(
                tuple(label for label in retained_labels if label.startswith("token:")),
                (
                    "token:fixed:6465726566:0:5",
                    "token:fixed:28:5:6",
                    "token:lexical:4944454e54:6:7",
                    "token:fixed:29:7:8",
                ),
            )
            self.assertEqual(
                tuple(label for label in removed_labels if label.startswith("token:")),
                (
                    "token:lexical:4944454e54:0:5",
                    "token:fixed:28:5:6",
                    "token:lexical:4944454e54:6:7",
                    "token:fixed:29:7:8",
                ),
            )
            self.assertFalse(any("helper:" in label for label in retained_labels))
            self.assertFalse(any("helper:" in label for label in removed_labels))

    def test_authored_statement_families_select_distinct_proposal_nodes(self) -> None:
        evidence = fixture_evidence()
        expected_nodes = {
            "ordinary-let": {"ordinary_let_rhs"},
            "requires-value-match": {"requires_entry", "let_stmt", "value_match"},
            "statement-match": {"match_stmt"},
            "try-let": {"try_let_rhs"},
            "value-match": {"value_match"},
        }
        forbidden_nodes = {
            "ordinary-let": {"try_let_rhs", "value_match", "match_stmt"},
            "requires-value-match": {"try_let_rhs", "match_stmt"},
            "statement-match": {"value_match"},
            "try-let": {"ordinary_let_rhs", "value_match", "match_stmt"},
            "value-match": {"match_stmt"},
        }
        for identifier, required in expected_nodes.items():
            with self.subTest(identifier=identifier):
                result = next(
                    item.result
                    for item in evidence.cases
                    if item.document == "proposal" and item.identifier == identifier
                )
                self.assertEqual(result.classification, "one")
                tree, end = decode_tree(result.traces[0])
                self.assertEqual(end, len(result.traces[0]))
                labels = set(tree_labels(tree))
                for production in required:
                    self.assertIn(
                        f"production:{production.encode('ascii').hex()}",
                        labels,
                    )
                for production in forbidden_nodes[identifier]:
                    self.assertNotIn(
                        f"production:{production.encode('ascii').hex()}",
                        labels,
                    )

    def test_adjacent_optional_ambiguity_retains_two_tree_shapes(self) -> None:
        inputs = fixture_inputs()
        mutated = inputs.current.data.replace(
            b"expr           := atom | call | construct",
            b"expr           := atom? atom? | call | construct",
            1,
        )
        grammar = extract_document("mutant", mutated, inputs.limits)
        result = parse_source(grammar, compile_grammar(grammar), "expr", b"x", inputs.limits)
        self.assertEqual(result.classification, "many")
        self.assertEqual(len(set(result.traces)), 2)
        decoded = []
        for trace in result.traces:
            tree, end = decode_tree(trace)
            self.assertEqual(end, len(trace))
            decoded.append(tree_labels(tree))
        for labels in decoded:
            optional = tuple(label for label in labels if ":optional:" in label)
            self.assertEqual(len(optional), 2)
            self.assertEqual(sum(label.endswith(":empty") for label in optional), 1)
            self.assertEqual(sum(label.endswith(":present") for label in optional), 1)
        self.assertNotEqual(decoded[0], decoded[1])

    def test_named_logical_limits_are_inconclusive_not_zero(self) -> None:
        current, _proposal = fixture_grammars()
        compiled = compile_grammar(current)
        for name, value, source in (
            ("oracle_max_source_tokens", 1, b"x x"),
            ("oracle_max_chart_items", 1, b"x"),
            ("oracle_max_packed_edges", 1, b"x"),
            ("oracle_max_proof_nodes", 1, b"x"),
            ("max_engine_output_bytes", 20, b"x"),
        ):
            with self.subTest(limit=name):
                with self.assertRaises(Failure) as raised:
                    parse_source(current, compiled, "expr", source, limits_with(**{name: value}))
                self.assertEqual(raised.exception.family, "resource")
                self.assertEqual(raised.exception.code, f"limit_{name}")

    def test_domain_is_independently_generated_and_length_bound(self) -> None:
        inputs = fixture_inputs()
        evidence = fixture_evidence()
        for domain in evidence.domains:
            with self.subTest(document=domain.document):
                self.assertEqual(len(domain.streams), 48)
                digest = hashlib.sha256()
                for stream in domain.streams:
                    digest.update(struct.pack(">Q", len(stream.source)))
                    digest.update(stream.source)
                self.assertEqual(digest.hexdigest(), domain.digest)
                self.assertEqual(
                    domain.digest,
                    "f3e54408ce7c4234bb3b61e27f2decd6c84ffcc4d7fb1b201c9583dd0190480c",
                )
        limited_inputs = type(inputs)(
            inputs.limits_bytes,
            inputs.current,
            inputs.proposal,
            inputs.cases_bytes,
            inputs.domains_bytes,
            limits_with(max_generated_streams=47),
            inputs.cases,
            inputs.domains,
        )
        current, proposal = fixture_grammars()
        with self.assertRaises(Failure) as raised:
            analyze(limited_inputs, current, proposal)
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("resource", "limit_max_generated_streams"),
        )

    def test_report_is_deterministic_and_output_bounded(self) -> None:
        inputs = fixture_inputs()
        current, proposal = fixture_grammars()
        evidence = fixture_evidence()
        first = render_success(inputs, current, proposal, evidence)
        second = render_success(inputs, current, proposal, evidence)
        self.assertEqual(first, second)
        self.assertTrue(first.endswith(b"ORACLE-END\nEND\n"))
        limited_inputs = type(inputs)(
            inputs.limits_bytes,
            inputs.current,
            inputs.proposal,
            inputs.cases_bytes,
            inputs.domains_bytes,
            limits_with(max_engine_output_bytes=100),
            inputs.cases,
            inputs.domains,
        )
        with self.assertRaises(Failure) as raised:
            render_success(limited_inputs, current, proposal, evidence)
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("resource", "limit_max_engine_output_bytes"),
        )


if __name__ == "__main__":
    unittest.main()
