from __future__ import annotations

from copy import deepcopy
import random
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

import frontend_boundary_evidence as evidence  # noqa: E402
import frontend_boundary_independent as independent  # noqa: E402
import frontend_boundary_primary as primary  # noqa: E402
from frontend_boundary_raw_independent import scan_sources as independent_scan  # noqa: E402
from frontend_boundary_raw_primary import scan_sources as primary_scan  # noqa: E402


def raw_case(descriptor: dict[str, object], identifier: str) -> dict[str, object]:
    matches = [row for row in descriptor["raw_cases"] if row["id"] == identifier]
    if len(matches) != 1:
        raise AssertionError(f"raw case {identifier} occurs {len(matches)} times")
    return matches[0]


class RawBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        report = evidence.build_evidence()
        cls.outcomes = {row["id"]: row["outcome"] for row in report["cases"]}

    def test_quote_aware_precedence_has_exact_rules_and_spans(self) -> None:
        expected = {
            "raw-exact-invalid-utf8-outside": ("FORM-2", [0, 0, 1]),
            "raw-exact-invalid-utf8-inside-string": ("FORM-2", [0, 2, 3]),
            "raw-exact-invalid-utf8-after-backslash": ("FORM-2", [0, 2, 3]),
            "raw-exact-nonascii-outside": ("FORM-1", [0, 0, 2]),
            "raw-exact-nonascii-inside-string": ("FORM-5", [0, 1, 3]),
            "raw-exact-nonascii-after-backslash": ("FORM-5", [0, 1, 4]),
            "raw-exact-invalid-string-escape": ("FORM-5", [0, 1, 3]),
            "raw-exact-final-string-backslash": ("FORM-5", [0, 4, 5]),
            "raw-exact-unterminated-string": ("FORM-5", [0, 0, 4]),
        }
        for identifier, (rule, coordinate) in expected.items():
            with self.subTest(identifier=identifier):
                outcome = self.outcomes[identifier]
                self.assertEqual(outcome["rule"], rule)
                self.assertEqual(outcome["location"]["coordinate"], coordinate)

    def test_comments_controls_and_sigils_distinguish_inside_from_outside(self) -> None:
        for identifier in (
            "raw-exact-line-comment-outside",
            "raw-exact-block-comment-outside",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "FORM-4")
        for identifier in (
            "raw-exact-line-comment-inside-string",
            "raw-exact-block-comment-inside-string",
        ):
            self.assertEqual(self.outcomes[identifier]["family"], "raw-scan-complete")
        for identifier in (
            "raw-exact-cr-outside",
            "raw-exact-tab-outside",
            "raw-exact-nul-outside",
            "raw-exact-del-outside",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "FORM-2")
        for identifier in (
            "raw-exact-cr-inside-string",
            "raw-exact-tab-inside-string",
            "raw-exact-lf-inside-string",
            "raw-exact-del-inside-string",
        ):
            self.assertEqual(self.outcomes[identifier]["rule"], "FORM-5")
        self.assertEqual(self.outcomes["raw-exact-malformed-region-sigil"]["location"]["coordinate"], [0, 0, 1])
        self.assertEqual(self.outcomes["raw-exact-malformed-label-sigil"]["location"]["coordinate"], [0, 0, 1])

    def test_complete_projections_pin_maximal_munch_and_boundaries(self) -> None:
        numeric = self.outcomes["raw-exact-broad-numeric-maximal-munch"]["sources"][0]
        self.assertEqual([row["kind"] for row in numeric["tokens"]], ["numeric"] * 4)
        suffixes = self.outcomes["raw-exact-operation-suffix-boundaries"]["sources"][0]
        self.assertEqual(suffixes["tokens"][0]["kind"], "opname")
        self.assertEqual([row["kind"] for row in suffixes["tokens"][1:]], [
            "lower-word", "punctuation", "lower-word",
            "lower-word", "punctuation", "lower-word",
            "lower-word", "punctuation", "lower-word",
        ])
        arrows = self.outcomes["raw-exact-punctuation-arrows"]["sources"][0]
        self.assertEqual([row["spelling_hex"] for row in arrows["tokens"][:2]], ["2d3e", "3d3e"])
        segmented = self.outcomes["raw-exact-trivia-segmentation"]["sources"][0]
        self.assertEqual(
            [row["source_coordinate"][1:] for row in segmented["trivia"]],
            [[0, 2], [2, 3], [3, 4], [4, 5]],
        )

    def test_scanners_never_join_records(self) -> None:
        outcome = self.outcomes["raw-exact-no-cross-source-token"]
        self.assertEqual(len(outcome["sources"]), 2)
        self.assertEqual(outcome["sources"][0]["tokens"][0]["kind"], "lower-word")
        self.assertEqual(
            [row["kind"] for row in outcome["sources"][1]["tokens"]],
            ["punctuation", "lower-word"],
        )
        self.assertEqual(
            self.outcomes["raw-exact-source-order"]["location"]["coordinate"],
            [1, 0, 1],
        )

    def test_independent_scanners_agree_on_deterministic_byte_fuzz(self) -> None:
        generator = random.Random(23)
        for sequence in range(5000):
            source = bytes(generator.randrange(256) for _ in range(generator.randrange(18)))
            with self.subTest(sequence=sequence, source=source.hex()):
                self.assertEqual(primary_scan([source]), independent_scan([source]))


class RawBoundaryMutationTests(unittest.TestCase):
    def assert_both_reject(self, descriptor: dict[str, object]) -> None:
        with self.assertRaises(primary.BoundaryModelError):
            primary.evaluate_descriptor(deepcopy(descriptor))
        with self.assertRaises(independent.IndependentBoundaryError):
            independent.interpret(deepcopy(descriptor))

    def test_quote_precedence_span_mutant_is_detected(self) -> None:
        descriptor, _ = primary.load_descriptor()
        row = raw_case(descriptor, "raw-exact-invalid-utf8-after-backslash")
        row["expect"]["coordinate"] = [0, 1, 3]
        self.assert_both_reject(descriptor)

    def test_maximal_munch_token_boundary_mutant_is_detected(self) -> None:
        descriptor, _ = primary.load_descriptor()
        row = raw_case(descriptor, "raw-exact-operation-suffix-boundaries")
        row["expect"]["sources"][0]["tokens"][0][2] = 8
        self.assert_both_reject(descriptor)


if __name__ == "__main__":
    unittest.main()
