from dataclasses import replace
from pathlib import Path
import sys
import unittest


sys.path.insert(0, str(Path(__file__).parent))

from manifest import FAMILY_CODEC, FAMILY_COMPILER, ManifestError  # noqa: E402
from measure import (  # noqa: E402
    FIELD_NAMES,
    MeasurementError,
    TRACE_GAP_FIELDS,
    measure,
    require_complete,
    validate_shape,
)
from relation import build_relation  # noqa: E402
from test_manifest import fixture  # noqa: E402


class MeasureTests(unittest.TestCase):
    def test_compiler_family_derives_every_actual_and_receipt(self) -> None:
        manifest, _ = fixture(FAMILY_COMPILER, 1)
        result = measure(manifest)
        actual = result.by_name()
        self.assertEqual(tuple(actual), FIELD_NAMES)
        self.assertEqual(len(result.actuals), 33)
        self.assertEqual(actual["max_sources"], 1)
        self.assertEqual(actual["max_source_bytes"], 593)
        self.assertEqual(actual["max_tokens"], 130)
        self.assertEqual(actual["max_lexemes"], 228)
        self.assertIsNone(actual["max_lexical_scan_work"])
        self.assertEqual(actual["max_production_nodes"], 117)
        self.assertEqual(actual["max_mixed_elements"], 246)
        self.assertEqual(actual["max_tree_depth"], 12)
        self.assertEqual(actual["max_tree_bytes"], 38904)
        self.assertEqual(actual["max_declarations"], 38)
        self.assertEqual(actual["max_declaration_events"], 14)
        self.assertEqual(actual["max_lexical_uses"], 11)
        self.assertEqual(actual["max_deferred_uses"], 2)
        self.assertEqual(actual["max_lookup_entries"], 115)
        self.assertEqual(actual["max_coverage_records"], 144)
        self.assertEqual(result.expected_diagnostic, ("Complete",))
        self.assertEqual(dict(result.derived)["diagnostic_issue_elements"], 0)
        self.assertEqual(tuple(name for name, _ in result.trace_gaps), TRACE_GAP_FIELDS)
        self.assertRaises(MeasurementError, require_complete, result)

    def test_codec_family_derives_independently(self) -> None:
        manifest, _ = fixture(FAMILY_CODEC, 2)
        result = measure(manifest)
        actual = result.by_name()
        self.assertEqual(actual["max_source_bytes"], 1438)
        self.assertEqual(actual["max_tokens"], 287)
        self.assertEqual(actual["max_lexemes"], 496)
        self.assertEqual(actual["max_production_nodes"], 235)
        self.assertEqual(actual["max_mixed_elements"], 521)
        self.assertEqual(actual["max_tree_depth"], 13)
        self.assertEqual(actual["max_node_path_depth"], 13)
        self.assertEqual(actual["max_scopes"], 19)
        self.assertEqual(actual["max_scope_depth"], 4)
        self.assertEqual(actual["max_declaration_events"], 31)
        self.assertEqual(actual["max_lexical_uses"], 30)
        self.assertEqual(actual["max_deferred_uses"], 6)
        self.assertEqual(actual["max_lookup_entries"], 126)
        self.assertEqual(actual["max_coverage_records"], 302)
        self.assertEqual(actual["max_spelling_bytes"], 1662)

    def test_checked_cross_field_relations_hold_at_multiple_scales(self) -> None:
        for family in (FAMILY_COMPILER, FAMILY_CODEC):
            for units in (1, 2, 17):
                manifest, _ = fixture(family, units)
                result = measure(manifest)
                actual = result.by_name()
                derived = dict(result.derived)
                self.assertEqual(actual["max_classified_tokens"], actual["max_tokens"])
                self.assertEqual(
                    derived["private_derivation_elements"],
                    actual["max_production_nodes"] + actual["max_tokens"],
                )
                self.assertEqual(
                    actual["max_mixed_elements"],
                    actual["max_production_nodes"] - 1 + actual["max_tokens"],
                )
                self.assertEqual(
                    actual["max_declarations"], actual["max_declaration_events"] + 24
                )
                self.assertEqual(actual["max_ancestry_steps"], actual["max_scopes"] - 1)
                self.assertEqual(
                    actual["max_coverage_records"],
                    actual["max_production_nodes"]
                    + actual["max_declaration_events"]
                    + actual["max_lexical_uses"]
                    + actual["max_deferred_uses"],
                )

    def test_impossible_generator_path_and_length_fail_before_measurement(self) -> None:
        manifest, _ = fixture()
        bad_source = replace(manifest.sources[0], byte_length=594)
        self.assertRaises(ManifestError, measure, replace(manifest, sources=(bad_source,)))
        bad_path = replace(manifest.sources[0], logical_path="demand/other.wf")
        self.assertRaises(ManifestError, build_relation, replace(manifest, sources=(bad_path,)))

    def test_measurement_shape_rejects_missing_actual_and_diagnostic_mutations(self) -> None:
        manifest, _ = fixture()
        result = measure(manifest)
        self.assertRaises(
            MeasurementError,
            validate_shape,
            replace(result, actuals=result.actuals[:-1]),
        )
        self.assertRaises(
            MeasurementError,
            validate_shape,
            replace(result, expected_diagnostic=("SourceIssue", "")),
        )
        self.assertRaises(
            MeasurementError,
            validate_shape,
            replace(result, trace_gaps=result.trace_gaps[:-1]),
        )

    def test_per_family_role_occurrences_are_closed(self) -> None:
        compiler, _ = fixture(FAMILY_COMPILER, 2)
        codec, _ = fixture(FAMILY_CODEC, 2)
        compiler_roles = build_relation(compiler).roles
        codec_roles = build_relation(codec).roles
        self.assertEqual(
            sum(role.category == "lexical" for role in compiler_roles), 21
        )
        self.assertEqual(
            sum(role.category == "lexical" for role in codec_roles), 30
        )
        self.assertEqual(
            sum(
                role.category == "lexical" and role.spelling == "CompilerRecord000000"
                for role in compiler_roles
            ),
            2,
        )


if __name__ == "__main__":
    unittest.main()
