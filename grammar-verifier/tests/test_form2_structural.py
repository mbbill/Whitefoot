from __future__ import annotations

import json
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

from form2_patch import unified_patch  # noqa: E402
from form2_render import (  # noqa: E402
    BLOCK_PRODUCTIONS,
    SEMICOLON_PRODUCTIONS,
    inject_isolated_fixture_defect,
    render_derivation,
)
from form2_structural_report import build_structural_artifacts  # noqa: E402
from form2_tree import (  # noqa: E402
    MAX_TRACE_BYTES,
    TreeError,
    decode_trace,
    load_candidate_parser,
    parse_one,
)
from support_form2_installation import reconstruct_form2_inputs  # noqa: E402


class StructuralRendererTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.parser = load_candidate_parser()

    def render(self, raw: bytes) -> bytes:
        attempt = parse_one(self.parser, raw)
        self.assertEqual(attempt.classification, "one")
        self.assertIsNotNone(attempt.derivation)
        return render_derivation(attempt.derivation).raw

    def test_empty_program_is_exactly_one_lf(self) -> None:
        self.assertEqual(self.render(b""), b"\n")
        self.assertEqual(self.render(b"\n"), b"\n")

    def test_attached_punctuation_and_region_parameter_spacing(self) -> None:
        source = (
            b"fn f < T > [ 'r , 's ] ( x : own Result < i32 , Overflow > ) "
            b"-> own unit pure { return unit ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"fn f<T> ['r, 's](x: own Result<i32, Overflow>) -> own unit pure {\n"
            b"  return unit;\n"
            b"}\n",
        )

    def test_inline_and_empty_blocks_receive_structural_lines(self) -> None:
        source = (
            b"struct Empty { } enum Flag { On ( ) ; Off ( ) ; } "
            b"fn main ( ) -> own unit pure { return unit ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"struct Empty {\n"
            b"}\n\n"
            b"enum Flag {\n"
            b"  On();\n"
            b"  Off();\n"
            b"}\n\n"
            b"fn main() -> own unit pure {\n"
            b"  return unit;\n"
            b"}\n",
        )

    def test_requires_close_and_body_open_share_one_line(self) -> None:
        source = (
            b"fn f ( x : own i32 ) -> own i32 traps requires { "
            b"check x else trap \"bad\" ; } { return x ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"fn f(x: own i32) -> own i32 traps requires {\n"
            b"  check x else trap \"bad\";\n"
            b"} {\n"
            b"  return x;\n"
            b"}\n",
        )

    def test_broad_syntax_renders_before_fn4_and_fn8_semantic_checks(self) -> None:
        source = (
            b"contract Bad { fn combine ( x : own i32 ) -> own i32 pure ; "
            b"law distributive ( combine , combine ) ; } "
            b"fn f ( x : own i32 ) -> own i32 traps requires { "
            b"doc \"semantic FN-8 rejection\" ; set x = x ; return x ; } "
            b"{ return x ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"contract Bad {\n"
            b"  fn combine(x: own i32) -> own i32 pure;\n"
            b"  law distributive(combine, combine);\n"
            b"}\n\n"
            b"fn f(x: own i32) -> own i32 traps requires {\n"
            b"  doc \"semantic FN-8 rejection\";\n"
            b"  set x = x;\n"
            b"  return x;\n"
            b"} {\n"
            b"  return x;\n"
            b"}\n",
        )

    def test_match_arms_and_value_match_nest_by_tree_depth(self) -> None:
        source = (
            b"fn f ( x : own Bool ) -> own i32 pure { "
            b"let y : own i32 = match x { True ( ) => { give 1_i32 ; } "
            b"False ( ) => { give 0_i32 ; } } return y ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"fn f(x: own Bool) -> own i32 pure {\n"
            b"  let y: own i32 = match x {\n"
            b"    True() => {\n"
            b"      give 1_i32;\n"
            b"    }\n"
            b"    False() => {\n"
            b"      give 0_i32;\n"
            b"    }\n"
            b"  }\n"
            b"  return y;\n"
            b"}\n",
        )

    def test_contract_conform_region_loop_and_statement_match_layout(self) -> None:
        source = (
            b"contract C { fn f ( x : own i32 ) -> own i32 pure ; "
            b"law associative ( f ) ; } "
            b"fn impl ( x : own i32 ) -> own i32 pure { return x ; } "
            b"conform i32 : C { f = impl ; } "
            b"fn main ( ) -> own unit traps { region 'r { loop @again { "
            b"match True ( ) { True ( ) => { break @again ; } "
            b"False ( ) => { check True ( ) else trap \"x\" ; } } } } "
            b"return unit ; }\n"
        )
        self.assertEqual(
            self.render(source),
            b"contract C {\n"
            b"  fn f(x: own i32) -> own i32 pure;\n"
            b"  law associative(f);\n"
            b"}\n\n"
            b"fn impl(x: own i32) -> own i32 pure {\n"
            b"  return x;\n"
            b"}\n\n"
            b"conform i32: C {\n"
            b"  f = impl;\n"
            b"}\n\n"
            b"fn main() -> own unit traps {\n"
            b"  region 'r {\n"
            b"    loop @again {\n"
            b"      match True() {\n"
            b"        True() => {\n"
            b"          break @again;\n"
            b"        }\n"
            b"        False() => {\n"
            b"          check True() else trap \"x\";\n"
            b"        }\n"
            b"      }\n"
            b"    }\n"
            b"  }\n"
            b"  return unit;\n"
            b"}\n",
        )

    def test_render_parse_render_preserves_tree_and_terminals(self) -> None:
        source = b"fn main ( ) -> own unit pure { return unit ; }\n"
        before = parse_one(self.parser, source)
        canonical = render_derivation(before.derivation).raw
        after = parse_one(self.parser, canonical)
        self.assertEqual(after.classification, "one")
        self.assertEqual(
            before.derivation.source_forest_projection_sha256,
            after.derivation.source_forest_projection_sha256,
        )
        self.assertEqual(
            before.derivation.terminal_projection_sha256,
            after.derivation.terminal_projection_sha256,
        )
        self.assertEqual(render_derivation(after.derivation).raw, canonical)

    def test_tabs_comments_and_unknown_tokens_do_not_gain_a_tree(self) -> None:
        malformed = (
            b"fn main() -> own unit pure {\n\treturn unit;\n}\n",
            b"fn main() -> own unit pure {\n  // no comments\n}\n",
            b"fn main() -> own unit pure {\n  mystery!;\n}\n",
        )
        for source in malformed:
            with self.subTest(source=source):
                self.assertEqual(parse_one(self.parser, source).classification, "zero")

    def test_resource_exhaustion_is_closed(self) -> None:
        source = b" ".join([b"fn"] * 4_097)
        attempt = parse_one(self.parser, source)
        self.assertEqual(attempt.classification, "resource-failure")
        self.assertEqual(attempt.failure, "resource:limit_oracle_max_source_tokens")

    def test_trace_decoder_rejects_bad_markers_lengths_and_trailing_bytes(self) -> None:
        malformed = (b"", b"X", b"T\0\0", b"T\0\0\0\0\0\0\0\0x")
        for raw in malformed:
            with self.subTest(raw=raw), self.assertRaises(TreeError):
                decode_trace(raw)
        with self.assertRaisesRegex(TreeError, "byte limit"):
            decode_trace(b"X" * (MAX_TRACE_BYTES + 1))

    def test_fixture_mutations_are_one_exact_splice(self) -> None:
        canonical = b"fn main() -> own unit pure {\n  let a: own i32 = 1_i32;\n}\n"
        expected = {
            "form2-neg-noncanonical-ws": b"\n    let ",
            "x-form-form2-tab-indent": b"\n\tlet ",
        }
        for identifier, marker in expected.items():
            with self.subTest(identifier=identifier):
                migrated, defect = inject_isolated_fixture_defect(
                    identifier, canonical
                )
                self.assertIn(marker, migrated)
                start = defect["byte_start"]
                actual = bytes.fromhex(defect["actual_hex"])
                self.assertEqual(
                    canonical[:start] + actual + canonical[start + 2 :], migrated
                )

    def test_patch_marks_a_missing_terminal_lf(self) -> None:
        patch = unified_patch([("source.wf", b"return unit;", b"return unit;\n")])
        self.assertIn(b"-return unit;\n\\ No newline at end of file\n", patch)
        self.assertIn(b"+return unit;\n", patch)


class StructuralProtectedSurfaceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.inputs, _ = reconstruct_form2_inputs()
        cls.artifacts = build_structural_artifacts(cls.inputs, {})
        cls.evidence = json.loads(
            cls.artifacts["form2-structural-layout-evidence.json"]
        )
        cls.migration = json.loads(
            cls.artifacts["form2-structural-migration.json"]
        )

    def test_complete_inventory_and_source_only_patch(self) -> None:
        self.assertEqual(len(self.evidence["files"]), 293)
        self.assertEqual(
            len(self.evidence["protected_inventory"]["inventory"]), 293
        )
        patch = self.artifacts["form2-structural-migration.patch"]
        self.assertNotIn(b"conformance/manifest.jsonl", patch)
        self.assertNotIn(b"spec/", patch)
        self.assertNotIn(b"governance/", patch)
        patch_paths = [
            line.removeprefix(b"+++ b/").decode("utf-8")
            for line in patch.splitlines()
            if line.startswith(b"+++ b/")
        ]
        changed_paths = [
            row["path"]
            for row in self.migration["files"]
            if row["after_sha256"] != row["before_sha256"]
        ]
        self.assertEqual(patch_paths, changed_paths)
        self.assertEqual(
            len(patch_paths), self.evidence["summary"]["changed_source_count"]
        )

    def test_every_declared_layout_family_is_exercised(self) -> None:
        counts = self.evidence["structural_counts"]
        self.assertEqual(counts["missing_block_family_coverage"], [])
        self.assertEqual(counts["missing_line_family_coverage"], [])
        self.assertEqual(counts["missing_production_coverage"], [])
        self.assertEqual(set(counts["block_production_counts"]), BLOCK_PRODUCTIONS)
        self.assertEqual(
            set(counts["line_bearing_production_counts"]),
            SEMICOLON_PRODUCTIONS,
        )
        self.assertGreater(counts["empty_block_count"], 0)
        self.assertGreater(counts["requires_body_transition_count"], 0)

    def test_non_form2_tree_sources_migrate_to_their_canonical_render(self) -> None:
        for row in self.evidence["files"]:
            if (
                not row["intended_form2_fixture"]
                and row["canonical_sha256"] is not None
            ):
                self.assertEqual(row["after_sha256"], row["canonical_sha256"])

    def test_two_form2_fixtures_have_one_isolated_defect_each(self) -> None:
        defects = self.migration["isolated_form2_fixture_defects"]
        self.assertEqual(
            [row["id"] for row in defects],
            ["form2-neg-noncanonical-ws", "x-form-form2-tab-indent"],
        )
        self.assertTrue(all(row["defect"]["kind"] for row in defects))
        self.assertTrue(
            all(row["canonical_sha256"] != row["migrated_sha256"] for row in defects)
        )

    def test_manifest_projection_and_terminal_lexemes_are_bound(self) -> None:
        projection = self.migration["manifest_projection"]
        self.assertEqual(
            projection["expect_and_status_sha256_before"],
            projection["expect_and_status_sha256_after"],
        )
        for row in self.migration["files"]:
            if row["canonical_sha256"] is not None:
                self.assertEqual(
                    row["terminal_projection_sha256_render_input"],
                    row["terminal_projection_sha256_after"],
                )
            if row["protected_syntax_repair"] is None:
                self.assertEqual(
                    row["terminal_projection_sha256_before"],
                    row["terminal_projection_sha256_render_input"],
                )

    def test_all_exact_zero_derivations_have_one_isolated_control(self) -> None:
        audit = self.evidence["zero_derivation_audit"]
        self.assertTrue(audit["source_inventory_complete"])
        self.assertTrue(audit["all_controls_have_one_complete_derivation"])
        self.assertEqual(audit["exact_zero_derivation_source_count"], 23)
        records = audit["records"]
        self.assertEqual(len(records), 23)
        self.assertTrue(
            all(row["control"]["unique_complete_derivation"] for row in records)
        )
        dispositions = [row["disposition"] for row in records]
        self.assertEqual(dispositions.count("protected-source-syntax-mismatch"), 3)
        self.assertEqual(dispositions.count("isolated-form2-format-negative"), 1)
        self.assertEqual(dispositions.count("expected-rule-lexical-boundary"), 2)
        self.assertEqual(dispositions.count("expected-rule-derivation-boundary"), 17)

    def test_protected_syntax_repairs_are_exact_and_separate(self) -> None:
        layer = self.evidence["protected_syntax_repair_layer"]
        expected_paths = [
            "conformance/cases/const1-neg-noninteger.wf",
            "conformance/cases/pending-const2-item.wf",
            "conformance/cases/type7-neg-match-borrow-expression.wf",
        ]
        self.assertTrue(layer["owner_approval_required"])
        self.assertFalse(layer["expected_verdict_projection_changed"])
        self.assertEqual(layer["patch_changed_paths"], expected_paths)
        self.assertEqual(layer["source_count"], 3)
        patch = self.artifacts["form2-protected-syntax-repairs.patch"]
        patch_paths = [
            line.removeprefix(b"+++ b/").decode("utf-8")
            for line in patch.splitlines()
            if line.startswith(b"+++ b/")
        ]
        self.assertEqual(patch_paths, expected_paths)
        self.assertNotIn(b"conformance/manifest.jsonl", patch)
        repair_records = {
            row["path"]: row for row in layer["proposals"]
        }
        self.assertEqual(
            len(repair_records[expected_paths[0]]["edits"]), 3
        )
        self.assertEqual(
            len(repair_records[expected_paths[1]]["edits"]), 1
        )
        self.assertEqual(
            len(repair_records[expected_paths[2]]["edits"]), 1
        )
        self.assertTrue(
            all(
                row["owner_approval_required"]
                and not row["intended_verdict_change_proposed"]
                and not row["runtime_behavior_change_proposed"]
                for row in repair_records.values()
            )
        )

    def test_repaired_sources_then_enter_the_form2_layer(self) -> None:
        summary = self.evidence["summary"]
        self.assertEqual(summary["syntax_repair_source_count"], 3)
        self.assertEqual(summary["zero_derivation_audit_source_count"], 23)
        self.assertEqual(summary["no_complete_derivation_source_count"], 19)
        self.assertEqual(summary["rendered_source_count_including_tab_recovery"], 274)
        self.assertEqual(summary["canonical_migrated_source_count"], 272)
        self.assertEqual(summary["patch_changed_path_count"], 274)
        self.assertEqual(
            self.migration["form2_layer_patch"]["changed_path_count"], 274
        )

    def test_no_derivation_sources_are_explicit_not_fabricated(self) -> None:
        blockers = self.evidence["blockers"]
        self.assertGreater(len(blockers), 0)
        self.assertTrue(
            all(row["parse"]["classification"] != "one" for row in blockers)
        )
        accepted = [
            row
            for row in blockers
            if row["classification"]
            == "candidate-grammar-contradicts-expected-accept"
        ]
        self.assertEqual(
            len(accepted),
            self.evidence["summary"][
                "candidate_expected_accept_contradiction_count"
            ],
        )
        self.assertTrue(self.evidence["summary"]["primary_evidence_complete"])
        self.assertFalse(
            self.evidence["summary"]["evidence_ready_for_owner_review"]
        )
        self.assertFalse(self.evidence["summary"]["migration_authorized"])
        self.assertFalse(self.evidence["summary"]["compiler_validated"])


if __name__ == "__main__":
    unittest.main()
