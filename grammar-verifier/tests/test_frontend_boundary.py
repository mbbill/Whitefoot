from __future__ import annotations

import ast
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

import frontend_boundary_evidence as evidence  # noqa: E402
import frontend_boundary_independent as independent  # noqa: E402
import frontend_boundary_primary as primary  # noqa: E402


def cloned_descriptor() -> dict[str, object]:
    descriptor, _ = primary.load_descriptor()
    return deepcopy(descriptor)


def case(descriptor: dict[str, object], identifier: str) -> dict[str, object]:
    matches = [row for row in descriptor["cases"] if row["id"] == identifier]
    if len(matches) != 1:
        raise AssertionError(f"case {identifier} occurs {len(matches)} times")
    return matches[0]


class FrontendBoundaryAgreementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.report = evidence.build_evidence()
        cls.outcomes = {
            row["id"]: row["outcome"] for row in cls.report["cases"]
        }

    def test_models_bind_identical_candidate_descriptor_and_outcomes(self) -> None:
        left = primary.build_primary()
        right = independent.build_independent()
        self.assertEqual(left["candidate"], right["candidate"])
        self.assertEqual(left["descriptor"], right["descriptor"])
        self.assertEqual(left["evaluation"]["cases"], right["evaluation"]["cases"])
        self.assertEqual(
            left["evaluation"]["requirements"],
            right["evaluation"]["requirements"],
        )
        self.assertEqual(self.report["agreement"]["case_count"], 134)
        self.assertEqual(self.report["agreement"]["structured_case_count"], 100)
        self.assertEqual(self.report["agreement"]["raw_case_count"], 34)

    def test_all_ten_boundary_requirements_have_explicit_cases(self) -> None:
        requirements = self.report["requirements"]
        self.assertEqual(set(requirements), {f"B{number:02d}" for number in range(1, 11)})
        self.assertTrue(all(names == sorted(set(names)) and names for names in requirements.values()))
        covered = {name for names in requirements.values() for name in names}
        self.assertEqual(covered, set(self.outcomes))

    def test_candidate_and_descriptor_are_full_byte_bound(self) -> None:
        candidate = (PROPOSAL / "kernel-spec-successor-candidate.md").read_bytes()
        descriptor = (PROPOSAL / "frontend-boundary-cases.json").read_bytes()
        self.assertEqual(self.report["candidate"]["sha256"], hashlib.sha256(candidate).hexdigest())
        self.assertEqual(self.report["candidate"]["byte_length"], len(candidate))
        self.assertEqual(self.report["descriptor"]["sha256"], hashlib.sha256(descriptor).hexdigest())
        self.assertEqual(self.report["descriptor"]["byte_length"], len(descriptor))

    def test_source_manifest_is_closed_and_exact(self) -> None:
        declared = (PROPOSAL / "FRONTEND_BOUNDARY_SOURCES").read_text(encoding="ascii").splitlines()
        self.assertEqual(declared, list(evidence.SOURCE_PATHS))
        self.assertEqual(len(declared), len(set(declared)))
        self.assertEqual(declared, sorted(declared))
        self.assertEqual(
            set(self.report["source_revisions"]),
            {"grammar-verifier/" + path for path in declared},
        )

    def test_committed_report_and_checksum_are_exactly_regenerated(self) -> None:
        rendered = evidence.rendered_evidence()
        self.assertEqual(evidence.EVIDENCE_PATH.read_bytes(), rendered)
        self.assertEqual(
            evidence.CHECKSUM_PATH.read_bytes(),
            evidence.rendered_checksum(rendered),
        )

    def test_primary_and_independent_models_do_not_import_each_other(self) -> None:
        checks = [
            (path, "frontend_boundary_independent")
            for path in PROPOSAL.glob("frontend_boundary_*primary*.py")
        ] + [
            (path, "frontend_boundary_primary")
            for path in PROPOSAL.glob("frontend_boundary_*independent*.py")
        ]
        for path, forbidden in checks:
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            imported = set()
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    imported.update(alias.name for alias in node.names)
                elif isinstance(node, ast.ImportFrom) and node.module:
                    imported.add(node.module)
            with self.subTest(path=path.name):
                self.assertNotIn(forbidden, imported)


class FrontendBoundaryContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.report = evidence.build_evidence()
        cls.outcomes = {
            row["id"]: row["outcome"] for row in cls.report["cases"]
        }

    def test_envelope_and_tool_failures_have_no_language_authority(self) -> None:
        for identifier in (
            "envelope-zero-records",
            "envelope-empty-path",
            "envelope-parent-path",
            "envelope-double-separator",
            "envelope-absolute-path",
            "envelope-duplicate-path",
            "resource-record-limit",
            "resource-record-byte-limit",
        ):
            outcome = self.outcomes[identifier]
            self.assertNotIn("rule", outcome)
            self.assertNotIn("location", outcome)
            self.assertNotIn("expected_terminals", outcome)

    def test_zero_bytes_and_one_lf_are_distinct(self) -> None:
        zero = self.outcomes["form2-zero-byte-source"]
        one_lf = self.outcomes["form2-one-lf-source"]
        self.assertEqual(zero["rule"], "FORM-2")
        self.assertEqual(zero["location"]["coordinate"], [0, 0, 0])
        self.assertEqual(one_lf, {"family": "frontend-complete"})

    def test_bundle_identity_retains_order_paths_partitions_and_empty_records(self) -> None:
        for identifier in (
            "identity-source-order",
            "identity-path-binding",
            "identity-record-repartition",
            "identity-empty-record-retention",
        ):
            self.assertFalse(self.outcomes[identifier]["same"])

    def test_function_visibility_is_global_but_lexical_bindings_are_ordered(self) -> None:
        self.assertEqual(
            self.outcomes["visibility-forward-function"],
            {"family": "frontend-complete"},
        )
        self.assertEqual(
            self.outcomes["visibility-earlier-constant-accepted"],
            {"family": "frontend-complete"},
        )
        for identifier in (
            "visibility-forward-constant-rejected",
            "visibility-forward-local-rejected",
            "visibility-forward-region-rejected",
            "visibility-forward-label-rejected",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "TYPE-6")

    def test_no_fragment_may_cross_a_record_boundary(self) -> None:
        self.assertEqual(
            self.outcomes["boundary-all-fragments-source-local"],
            {"family": "frontend-complete"},
        )
        for identifier in (
            "boundary-token-crossing-rejected",
            "boundary-trivia-crossing-rejected",
            "boundary-production-crossing-rejected",
            "boundary-source-span-crossing-rejected",
        ):
            self.assertEqual(
                self.outcomes[identifier]["family"], "compiler-invariant-failure"
            )
            self.assertNotIn("rule", self.outcomes[identifier])

    def test_root_order_uses_source_ordinal_and_retains_empty_extents(self) -> None:
        outcome = self.outcomes["root-flatten-order-and-extents"]
        self.assertEqual([child["id"] for child in outcome["children"]], ["z0", "z1", "m0"])
        self.assertEqual(outcome["extent"], [[0, 0, 3], [1, 0, 1], [2, 0, 2]])
        self.assertEqual(outcome["node_path"], [])
        self.assertEqual(outcome["program_root_count"], 1)
        self.assertEqual(
            outcome["source_forests"],
            [
                {"item_node_paths": [[0], [1]], "source_ordinal": 0},
                {"item_node_paths": [], "source_ordinal": 1},
                {"item_node_paths": [[2]], "source_ordinal": 2},
            ],
        )

    def test_every_raw_lexical_class_has_exact_rule_and_coordinate(self) -> None:
        expected = {
            "raw-invalid-utf8": "FORM-2",
            "raw-cr": "FORM-2",
            "raw-tab": "FORM-2",
            "raw-other-whitespace": "FORM-2",
            "raw-line-comment-prefix": "FORM-4",
            "raw-block-comment-prefix": "FORM-4",
            "raw-malformed-string": "FORM-5",
            "raw-malformed-region": "FORM-3",
            "raw-malformed-label": "FORM-3",
            "raw-unrecognized-token-start": "FORM-1",
        }
        for identifier, rule in expected.items():
            with self.subTest(identifier=identifier):
                self.assertEqual(self.outcomes[identifier]["rule"], rule)
                self.assertEqual(self.outcomes[identifier]["location"]["kind"], "SourceBytes")
        self.assertEqual(self.outcomes["raw-stage-before-earlier-grammar"]["rule"], "FORM-2")
        self.assertEqual(
            self.outcomes["raw-source-order-before-byte-offset"]["location"]["coordinate"],
            [0, 2, 3],
        )

    def test_form2_locations_distinguish_owned_and_source_boundaries(self) -> None:
        for identifier in (
            "form2-production-missing-separator",
            "form2-production-wrong-separator",
        ):
            self.assertEqual(self.outcomes[identifier]["location"]["kind"], "SourceNode")
        for identifier in (
            "form2-source-leading-excess",
            "form2-source-final-missing",
            "form2-interitem-excess",
        ):
            self.assertEqual(self.outcomes[identifier]["location"]["kind"], "SourceBytes")

    def test_form2_owner_is_terminal_lca_and_coordinate_is_a_subinterval(self) -> None:
        missing = self.outcomes["form2-production-missing-separator"]["location"]
        wrong = self.outcomes["form2-production-wrong-separator"]["location"]
        self.assertEqual(missing["node_path"], [0, 2])
        self.assertEqual(wrong["node_path"], [0, 2])
        self.assertEqual(missing["coordinate"], [0, 1, 1])
        self.assertEqual(wrong["coordinate"], [0, 1, 3])

    def test_whole_unit_missing_and_duplicate_declarations_use_distinct_locations(self) -> None:
        missing = self.outcomes["whole-missing-main"]
        duplicate = self.outcomes["whole-duplicate-declaration"]
        self.assertEqual((missing["rule"], missing["location"]["kind"]), ("FN-7", "BundleRoot"))
        self.assertEqual(missing["location"]["extent"], [[0, 0, 1]])
        self.assertEqual((duplicate["rule"], duplicate["location"]["kind"]), ("TYPE-6", "SourceNode"))
        self.assertEqual(duplicate["location"]["coordinate"], [0, 4, 7])


class DiagnosticMachineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.report = evidence.build_evidence()
        cls.outcomes = {
            row["id"]: row["outcome"] for row in cls.report["cases"]
        }

    def test_wrong_name_forms_and_reserved_wrappers_cite_form3(self) -> None:
        for identifier in (
            "diag-fn-main",
            "diag-enum-sign",
            "diag-region-scope",
            "diag-wrapper-requires",
            "diag-wrapper-trap",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "FORM-3")
        self.assertEqual(self.outcomes["diag-numeric-membership"]["rule"], "FORM-5")

    def test_unknown_construct_attribution_ignores_symbol_lookup(self) -> None:
        declared = self.outcomes["diag-gimble-declared"]
        undeclared = self.outcomes["diag-gimble-undeclared"]
        self.assertEqual(declared, undeclared)
        self.assertEqual(declared["rule"], "FORM-1")
        self.assertEqual(declared["location"]["coordinate"], [0, 0, 6])
        self.assertEqual(self.outcomes["diag-unknown-item-entry"]["rule"], "FORM-1")
        self.assertEqual(self.outcomes["diag-unknown-requires-entry"]["rule"], "FORM-1")

    def test_effect_failures_stop_at_the_eff1_frontier(self) -> None:
        self.assertEqual(
            self.outcomes["diag-eff-pure-continuation"]["expected_terminals"], ["{"]
        )
        self.assertEqual(
            self.outcomes["diag-eff-writes-missing-paren"]["expected_terminals"], ["("]
        )
        self.assertEqual(
            self.outcomes["diag-eff-trailing-comma"]["expected_terminals"],
            ["reads", "writes", "allocates", "traps"],
        )
        for identifier in (
            "diag-eff-pure-continuation",
            "diag-eff-writes-missing-paren",
            "diag-eff-trailing-comma",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "EFF-1")

    def test_dotted_call_attribution_is_lookup_free_and_precedes_other_overrides(self) -> None:
        for identifier in (
            "diag-dotted-iadd-bogus",
            "diag-dotted-alpha-omega",
            "diag-dotted-priority",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "FORM-3")
        self.assertEqual(
            self.outcomes["diag-dotted-priority"]["location"]["coordinate"], [0, 0, 5]
        )

    def test_dotted_window_covers_every_boundary_member_for_both_openers(self) -> None:
        for opener in ("call", "targs"):
            for index, label in enumerate(("first-ident", "dot", "second-ident")):
                identifier = f"diag-dotted-{opener}-boundary-{label}"
                outcome = self.outcomes[identifier]
                self.assertEqual(outcome["boundary_index"], index)
                self.assertEqual(outcome["attribution"]["rule"], "FORM-3")
        self.assertEqual(self.outcomes["diag-dotted-iadd-bogus"]["rule"], "FORM-3")
        self.assertEqual(self.outcomes["diag-dotted-alpha-omega"]["rule"], "FORM-3")

    def test_transparent_nullable_name_paths_use_full_source_topology(self) -> None:
        positives = {
            "diag-name-nested-doc-field-star": ["doc", "IDENT", "}"],
            "diag-name-nested-doc-variant-star": ["doc", "TYPEID", "}"],
            "diag-name-param-list-optional": ["IDENT", ")"],
            "diag-name-vfield-list-optional": ["IDENT", ")"],
        }
        for identifier, expected in positives.items():
            with self.subTest(identifier=identifier):
                outcome = self.outcomes[identifier]
                self.assertEqual(outcome["rule"], "FORM-3")
                self.assertEqual(outcome["expected_terminals"], expected)
        contract = self.outcomes["diag-name-nested-doc-contract-negative"]
        self.assertEqual(contract["rule"], "GRAM-2")
        self.assertEqual(contract["expected_terminals"], ["doc", "fn", "law", "}"])

    def test_every_forbidden_start_is_gram9_in_every_atom_slot(self) -> None:
        identifiers = [
            identifier
            for identifier in self.outcomes
            if identifier.startswith("diag-gram9-")
            and not identifier.endswith("legal-place-near-miss")
        ]
        self.assertEqual(len(identifiers), 18)
        for identifier in identifiers:
            with self.subTest(identifier=identifier):
                outcome = self.outcomes[identifier]
                self.assertEqual(outcome["rule"], "GRAM-9")
                coordinate = outcome["location"]["coordinate"]
                self.assertEqual(coordinate[1], 0)
                self.assertGreater(coordinate[2], coordinate[1])

    def test_legal_place_prefixes_are_not_gram9_starts(self) -> None:
        for slot in ("atom_list", "fieldinit", "index-offset"):
            outcome = self.outcomes[f"diag-gram9-{slot}-legal-place-near-miss"]
            self.assertFalse(outcome["forbidden_call_or_construct_start"])
            self.assertEqual(outcome["slot"], slot)

    def test_mixed_name_at_structural_choice_is_not_form3(self) -> None:
        outcome = self.outcomes["diag-foo-structural-choice"]
        self.assertEqual(outcome["rule"], "GRAM-4")
        self.assertEqual(outcome["expected_terminals"], ["let", "IDENT"])

    def test_ties_second_token_end_and_terminal_order_are_exact(self) -> None:
        self.assertEqual(self.outcomes["diag-repeat-tie"]["rule"], "GRAM-4")
        self.assertEqual(
            self.outcomes["diag-synthetic-second-token"]["location"]["coordinate"],
            [0, 1, 2],
        )
        self.assertEqual(
            self.outcomes["diag-synthetic-end-token"]["location"]["coordinate"],
            [0, 1, 1],
        )
        self.assertEqual(
            self.outcomes["diag-grammar-rank-not-lexical-sort"]["expected_terminals"],
            ["z", "a"],
        )
        self.assertEqual(
            self.outcomes["diag-written-before-source-end"]["expected_terminals"],
            ["z", "SOURCE_END"],
        )

    def test_residual_program_token_expects_only_source_end(self) -> None:
        outcome = self.outcomes["diag-program-residual-source-end"]
        self.assertEqual(outcome["rule"], "GRAM-2")
        self.assertEqual(outcome["expected_terminals"], ["SOURCE_END"])
        self.assertEqual(outcome["location"]["coordinate"], [0, 0, 3])


class DiagnosticMutationTests(unittest.TestCase):
    def assert_both_reject(self, descriptor: dict[str, object]) -> None:
        with self.assertRaises((primary.BoundaryModelError, AssertionError)):
            primary.evaluate_descriptor(deepcopy(descriptor))
        with self.assertRaises((independent.IndependentBoundaryError, AssertionError)):
            independent.interpret(deepcopy(descriptor))

    def test_override_priority_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        case(descriptor, "diag-dotted-priority")["frontier"]["context"][
            "dotted_window"
        ] = None
        self.assert_both_reject(descriptor)

    def test_wrong_boundary_coordinate_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        case(descriptor, "diag-synthetic-second-token")["expect"]["location"][
            "coordinate"
        ] = [0, 0, 1]
        self.assert_both_reject(descriptor)

    def test_lexical_expected_sort_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        frontier = case(descriptor, "diag-grammar-rank-not-lexical-sort")["frontier"]
        frontier["arms"][0]["rows"][0][1]["rank"] = 30
        frontier["arms"][1]["rows"][0][1]["rank"] = 5
        self.assert_both_reject(descriptor)

    def test_follow_blind_unrestricted_descent_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        case(descriptor, "diag-eff-pure-continuation")["frontier"]["arms"][0][
            "rows"
        ][0][1]["origin"] = "inside"
        self.assert_both_reject(descriptor)

    def test_same_shape_only_name_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        case(descriptor, "diag-enum-sign")["frontier"]["direct"] = False
        self.assert_both_reject(descriptor)

    def test_broad_any_name_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        context = case(descriptor, "diag-foo-structural-choice")["frontier"]["context"]
        for root in context["mandatory_name_roots"]:
            root["topology"] = {"kind": "terminal", "predicate": root["start"]}
        self.assert_both_reject(descriptor)

    def test_no_wrapper_delegation_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        case(descriptor, "diag-region-scope")["frontier"]["direct"] = False
        self.assert_both_reject(descriptor)

    def test_nullable_accepting_direction_blocks_name_path(self) -> None:
        descriptor = cloned_descriptor()
        roots = case(descriptor, "diag-name-nested-doc-field-star")["frontier"][
            "context"
        ]["mandatory_name_roots"]
        field_exit = roots[0]["topology"]["directions"][1]["directions"][1][
            "predicate"
        ]
        field_exit["name"] = "TYPEID"
        roots[1]["topology"] = {
            "directions": [
                {
                    "kind": "terminal",
                    "predicate": {"name": "IDENT", "origin": "inside", "rank": 10},
                },
                {
                    "kind": "terminal",
                    "predicate": {"name": "TYPEID", "origin": "inside", "rank": 12},
                },
            ],
            "kind": "nullable",
            "source": "candidate?",
        }
        self.assert_both_reject(descriptor)

    def test_heterogeneous_transparent_name_paths_do_not_guess(self) -> None:
        descriptor = cloned_descriptor()
        roots = case(descriptor, "diag-name-nested-doc-field-star")["frontier"][
            "context"
        ]["mandatory_name_roots"]
        field_exit = roots[0]["topology"]["directions"][1]["directions"][1][
            "predicate"
        ]
        field_exit["name"] = "REGIONID"
        self.assert_both_reject(descriptor)

    def test_boundary_opener_dependent_dotted_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        row = case(descriptor, "diag-dotted-call-boundary-dot")
        row["boundary_index"] = 3
        self.assert_both_reject(descriptor)

    def test_form2_terminal_lca_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        row = case(descriptor, "form2-production-missing-separator")
        row["mismatch"]["right_terminal_path"] = [0, 3, 0]
        self.assert_both_reject(descriptor)

    def test_form2_node_extent_mutant_is_detected(self) -> None:
        descriptor = cloned_descriptor()
        row = case(descriptor, "form2-production-wrong-separator")
        row["mismatch"]["node_extent"] = [3, 4]
        self.assert_both_reject(descriptor)

    def test_lookup_state_does_not_change_unknown_or_dotted_results(self) -> None:
        descriptor = cloned_descriptor()
        for identifier in ("diag-gimble-declared", "diag-dotted-alpha-omega"):
            row = case(descriptor, identifier)
            original = primary._evaluate_case(descriptor, row)
            row["frontier"]["context"]["lookup_state"] = "undeclared"
            self.assertEqual(primary._evaluate_case(descriptor, row), original)
            self.assertEqual(independent._run_case(descriptor, row), original)

    def test_candidate_byte_mutant_changes_both_full_hash_bindings(self) -> None:
        candidate = (PROPOSAL / "kernel-spec-successor-candidate.md").read_bytes()
        with tempfile.TemporaryDirectory(prefix="whitefoot-boundary-candidate-") as directory:
            path = Path(directory) / "candidate.md"
            path.write_bytes(candidate + b"\n")
            left, _ = primary.candidate_contract(path)
            right, _ = independent.specification_contract(path)
        original = hashlib.sha256(candidate).hexdigest()
        self.assertEqual(left, right)
        self.assertNotEqual(left["sha256"], original)

    def test_missing_requirement_and_changed_expectation_fail_closed(self) -> None:
        descriptor = cloned_descriptor()
        del descriptor["requirements"]["B10"]
        self.assert_both_reject(descriptor)
        descriptor = cloned_descriptor()
        case(descriptor, "whole-missing-main")["expect"]["rule"] = "PROG-1"
        self.assert_both_reject(descriptor)


if __name__ == "__main__":
    unittest.main()
