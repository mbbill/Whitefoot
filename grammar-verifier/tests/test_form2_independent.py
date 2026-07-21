from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

import form2_independent_inputs  # noqa: E402
import form2_independent_compare  # noqa: E402
from form2_independent_inputs import (  # noqa: E402
    IndependentInputError,
)
from form2_independent_lex import (  # noqa: E402
    IndependentLexError,
    lex_independently,
    terminal_projection,
)
from form2_independent_parse import parse_independently  # noqa: E402
from form2_independent_render import (  # noqa: E402
    IndependentRenderError,
    horizontal_boundary,
    render_for_migration,
)
from form2_independent_repairs import (  # noqa: E402
    ExactEdit,
    IndependentRepairError,
    apply_exact_edits,
)
from form2_independent_report import (  # noqa: E402
    COMPLETE_STAGE,
    IndependentReportError,
    REPAIR_COMPLETE_STAGE,
    RENDERED_STAGES,
    TAB_RECOVERY_STAGE,
    build_independent_artifacts,
)
from form2_independent_syntax import (  # noqa: E402
    IndependentParseError,
    PRODUCTION_KINDS,
    source_forest_projection,
)
from support_form2_installation import reconstruct_form2_inputs  # noqa: E402


def derive(raw: bytes) -> tuple:
    tokens = lex_independently(raw)
    return tokens, parse_independently(tokens, len(raw))


class IndependentLexicalTests(unittest.TestCase):
    def test_operation_name_and_place_dot_are_distinct(self) -> None:
        tokens = lex_independently(b"iadd.wrap<i32>(p.field, 1_i32)")
        self.assertEqual(tokens[0].kind, "OPNAME")
        self.assertEqual(tokens[0].raw, b"iadd.wrap")
        dot = next(index for index, token in enumerate(tokens) if token.raw == b".")
        self.assertEqual(tokens[dot - 1].raw, b"p")
        self.assertEqual(tokens[dot + 1].raw, b"field")

    def test_strings_do_not_create_formatting_tokens(self) -> None:
        tokens = lex_independently(b'doc "fn hidden () { x; }";')
        self.assertEqual([token.raw for token in tokens], [b"doc", b'"fn hidden () { x; }"', b";"])

    def test_reserved_word_is_not_an_identifier(self) -> None:
        tokens = lex_independently(b"requires ordinary_name")
        self.assertEqual([token.kind for token in tokens], ["FIXED", "IDENT"])

    def test_law_words_remain_ordinary_identifiers(self) -> None:
        tokens = lex_independently(b"identity associative distributive")
        self.assertEqual([token.kind for token in tokens], ["IDENT", "IDENT", "IDENT"])

    def test_tab_and_carriage_return_fail_raw_lexical_formation(self) -> None:
        with self.assertRaisesRegex(IndependentLexError, "source-format"):
            lex_independently(b"\treturn unit;\n")
        with self.assertRaisesRegex(IndependentLexError, "source-format"):
            lex_independently(b"return unit;\r\n")

    def test_name_shapes_do_not_absorb_forbidden_tail_bytes(self) -> None:
        tokens = lex_independently(b"lower Upper9 'region @label")
        self.assertEqual(
            [token.kind for token in tokens],
            ["IDENT", "TYPEID", "REGIONID", "LABEL"],
        )
        with self.assertRaises(IndependentLexError):
            lex_independently(b"Bad_Name")

    def test_non_language_byte_and_bad_escape_fail_closed(self) -> None:
        with self.assertRaises(IndependentLexError):
            lex_independently(b"// comment\n")
        with self.assertRaises(IndependentLexError):
            lex_independently(b'doc "bad\\t";')


class IndependentBoundaryTests(unittest.TestCase):
    def test_boundary_function_is_total_on_representative_pairs(self) -> None:
        cases = {
            (b"f", b"("): b"",
            (b"f", b"<"): b"",
            (b"[", b"'r"): b"",
            (b"'r", b"]"): b"",
            (b"]", b"("): b"",
            (b"x", b":"): b"",
            (b":", b"own"): b" ",
            (b"x", b","): b"",
            (b",", b"y"): b" ",
            (b"match", b"x"): b" ",
            (b"x", b"{"): b" ",
        }
        for pair, expected in cases.items():
            with self.subTest(pair=pair):
                self.assertEqual(horizontal_boundary(*pair), expected)

    def test_headers_and_square_interiors_render_exactly(self) -> None:
        raw = (
            b"fn f <T> [ 'r ] () -> own unit pure { return unit; }\n"
            b"fn main () -> own unit pure { return unit; }\n"
        )
        tokens, root = derive(raw)
        rendered = render_for_migration(tokens, root, None).canonical
        self.assertIn(b"fn f<T> ['r]() -> own unit pure {\n", rendered)
        self.assertIn(b"fn main() -> own unit pure {\n", rendered)
        self.assertTrue(rendered.endswith(b"}\n"))


class IndependentStructuralTests(unittest.TestCase):
    def test_law_head_and_arguments_follow_the_broad_syntax(self) -> None:
        raw = (
            b"contract Rules {"
            b" law arbitrary();"
            b" law identity(identity, 0_u64);"
            b" }\n"
        )
        _, root = derive(raw)
        kinds = [node.kind for node in root.descendants()]
        self.assertEqual(kinds.count("law"), 2)
        self.assertEqual(kinds.count("law_arg"), 2)

    def test_requires_entries_derive_before_fn8_restriction(self) -> None:
        raw = (
            b"fn f (x: own i32) -> own i32 traps requires {"
            b' doc "semantic rejection";'
            b" set x = x;"
            b" return x;"
            b" } { return x; }\n"
        )
        _, root = derive(raw)
        entries = [node for node in root.descendants() if node.kind == "requires_entry"]
        self.assertEqual(len(entries), 3)
        self.assertEqual([entry.children[0].kind for entry in entries], ["doc", "stmt", "stmt"])

    def test_requires_value_match_is_a_real_syntax_tree(self) -> None:
        raw = (
            b"enum Choice { Yes(); }\n"
            b"fn f (x: own Choice) -> own i32 traps requires {"
            b" let result: own i32 = match x { Yes() => { give 1_i32; } }"
            b" } { return 0_i32; }\n"
        )
        _, root = derive(raw)
        kinds = {node.kind for node in root.descendants()}
        self.assertIn("requires_entry", kinds)
        self.assertIn("value_match", kinds)

    def test_inline_declarations_and_arms_expand_structurally(self) -> None:
        raw = (
            b"enum Sign { Neg(); Pos(); }\n"
            b"fn main () -> own unit traps { match Neg() { Neg() => { return unit; } } }\n"
        )
        expected = (
            b"enum Sign {\n"
            b"  Neg();\n"
            b"  Pos();\n"
            b"}\n"
            b"\n"
            b"fn main() -> own unit traps {\n"
            b"  match Neg() {\n"
            b"    Neg() => {\n"
            b"      return unit;\n"
            b"    }\n"
            b"  }\n"
            b"}\n"
        )
        tokens, root = derive(raw)
        self.assertEqual(render_for_migration(tokens, root, None).canonical, expected)

    def test_requires_close_and_body_open_share_one_line(self) -> None:
        raw = (
            b"fn f (x: own i32) -> own i32 traps requires {"
            b" let ok: own Bool = ieq<i32>(x, x);"
            b" check ok else trap \"bad\";"
            b" } { return x; }\n"
        )
        tokens, root = derive(raw)
        rendered = render_for_migration(tokens, root, None).canonical
        self.assertIn(b"\n} {\n", rendered)
        self.assertNotIn(b"\n}\n{\n", rendered)

    def test_render_round_trip_preserves_terminals_and_tree(self) -> None:
        raw = b"fn main () -> own unit pure { return unit; }\n"
        tokens, root = derive(raw)
        canonical = render_for_migration(tokens, root, None).canonical
        second_tokens, second_root = derive(canonical)
        self.assertEqual(terminal_projection(tokens), terminal_projection(second_tokens))
        self.assertEqual(
            source_forest_projection(root),
            source_forest_projection(second_root),
        )
        self.assertEqual(
            render_for_migration(second_tokens, second_root, None).canonical,
            canonical,
        )

    def test_tree_coverage_and_brace_underflow_fail_closed(self) -> None:
        tokens, root = derive(b"fn main() -> own unit pure { return unit; }\n")
        with self.assertRaisesRegex(IndependentRenderError, "complete terminal stream"):
            render_for_migration(tokens[:-1], root, None)

    def test_portable_tree_owns_each_terminal_exactly_once(self) -> None:
        tokens, root = derive(b"fn main() -> own unit pure { return unit; }\n")
        owners = [
            token_index
            for node in root.descendants()
            for token_index in node.direct_token_indices()
        ]
        self.assertEqual(sorted(owners), list(range(len(tokens))))
        self.assertEqual(len(owners), len(set(owners)))

class IndependentProtectedCorpusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, cls.inputs = reconstruct_form2_inputs()
        cls.artifacts = build_independent_artifacts(
            compare_primary=False,
            inputs=cls.inputs,
        )
        cls.report = json.loads(cls.artifacts["form2-independent-report.json"])

    def test_inventory_and_closed_outcomes_are_exact(self) -> None:
        self.assertEqual(len(self.inputs.sources), 293)
        self.assertEqual(
            self.report["summary"]["stage_counts"],
            {
                "complete-derivation": 270,
                "isolated-form2-tab-recovery": 1,
                "no-complete-derivation": 17,
                "no-lexical-formation": 2,
                "syntax-repair-then-complete-derivation": 3,
            },
        )
        self.assertEqual(self.report["summary"]["rendered_source_count"], 274)
        self.assertEqual(self.report["summary"]["raw_failure_source_count"], 23)
        self.assertEqual(
            self.report["summary"]["remaining_intentional_negative_count"], 19
        )
        self.assertFalse(self.report["summary"]["complete_protected_migration_proven"])
        self.assertFalse(self.report["summary"]["compiler_verdict_preservation_proven"])

    def test_every_rendered_derivation_round_trips(self) -> None:
        rows = {
            row["path"]: row
            for row in self.report["files"]
            if row["stage"] in RENDERED_STAGES
        }
        self.assertEqual(len(rows), 274)
        self.assertTrue(all(row["terminal_count"] > 0 for row in rows.values()))
        self.assertTrue(
            all(row["source_forest_node_count"] > 0 for row in rows.values())
        )

    def test_no_derivation_is_not_represented_as_a_tree(self) -> None:
        rows = [row for row in self.report["files"] if row["stage"] not in RENDERED_STAGES]
        self.assertEqual(len(rows), 19)
        for row in rows:
            self.assertEqual(row["expect"]["kind"], "reject")
            self.assertNotIn("source_forest_projection_sha256", row)
            self.assertNotIn("canonical_sha256", row)

    def test_repairs_are_exact_and_expected_verdict_projection_is_unchanged(self) -> None:
        layer = self.report["protected_syntax_repair_layer"]
        self.assertEqual(
            layer["patch_changed_paths"],
            [
                "conformance/cases/const1-neg-noninteger.wf",
                "conformance/cases/pending-const2-item.wf",
                "conformance/cases/type7-neg-match-borrow-expression.wf",
            ],
        )
        self.assertEqual(layer["source_count"], 3)
        self.assertFalse(layer["expected_verdict_projection_changed"])
        self.assertTrue(self.report["manifest_expectation_projection_preserved"])
        by_path = {row["path"]: row for row in layer["proposals"]}
        self.assertEqual(
            len(by_path["conformance/cases/const1-neg-noninteger.wf"]["edits"]),
            3,
        )
        self.assertEqual(
            len(by_path["conformance/cases/pending-const2-item.wf"]["edits"]),
            1,
        )
        self.assertEqual(
            len(
                by_path[
                    "conformance/cases/type7-neg-match-borrow-expression.wf"
                ]["edits"]
            ),
            1,
        )

    def test_all_raw_failures_have_independent_complete_controls(self) -> None:
        controls = self.report["failure_controls"]
        self.assertEqual(len(controls), 23)
        self.assertTrue(
            all(row["predictive_control_parse_complete"] for row in controls)
        )
        dispositions = [row["disposition"] for row in controls]
        self.assertEqual(dispositions.count("protected-source-syntax-mismatch"), 3)
        self.assertEqual(dispositions.count("isolated-form2-format-negative"), 1)
        self.assertEqual(dispositions.count("expected-rule-lexical-boundary"), 2)
        self.assertEqual(dispositions.count("expected-rule-derivation-boundary"), 17)

    def test_intended_form2_cases_have_one_exact_defect(self) -> None:
        rows = {row["id"]: row for row in self.report["files"]}
        for identifier in ("form2-neg-noncanonical-ws", "x-form-form2-tab-indent"):
            row = rows[identifier]
            expected_stage = (
                TAB_RECOVERY_STAGE
                if identifier == "x-form-form2-tab-indent"
                else COMPLETE_STAGE
            )
            self.assertEqual(row["stage"], expected_stage)
            self.assertIsNotNone(row["intentional_defect"])
            source = next(
                source
                for source in self.inputs.sources
                if source.identifier == identifier
            )
            raw = source.raw
            if identifier == "x-form-form2-tab-indent":
                with self.assertRaises(IndependentLexError):
                    derive(raw)
                raw = raw.replace(b"\t", b"  ")
            tokens, root = derive(raw)
            rendering = render_for_migration(tokens, root, identifier)
            start = rendering.intentional_defect["byte_start"]
            canonical = rendering.canonical
            migrated = rendering.migration
            if identifier == "form2-neg-noncanonical-ws":
                self.assertEqual(migrated[start : start + 4], b"    ")
                self.assertEqual(canonical[start : start + 2], b"  ")
            else:
                self.assertEqual(migrated[start : start + 1], b"\t")
                self.assertEqual(canonical[start : start + 2], b"  ")

    def test_every_candidate_production_is_exercised(self) -> None:
        observed: set[str] = set()
        for source in self.inputs.sources:
            raw = source.raw
            if source.identifier == "x-form-form2-tab-indent":
                raw = raw.replace(b"\t", b"  ")
            try:
                _, root = derive(raw)
            except (IndependentLexError, IndependentParseError):
                continue
            observed.update(node.kind for node in root.descendants())
        self.assertEqual(observed, PRODUCTION_KINDS)

    def test_report_is_deterministic_and_patch_stays_in_memory(self) -> None:
        self.assertEqual(
            self.artifacts,
            build_independent_artifacts(compare_primary=False, inputs=self.inputs),
        )
        self.assertEqual(
            set(self.artifacts),
            {"form2-independent-report.json", "form2-independent-evidence.sha256"},
        )
        compared = json.loads(
            build_independent_artifacts(compare_primary=True, inputs=self.inputs)[
                "form2-independent-report.json"
            ]
        )["primary_comparison"]
        self.assertTrue(compared["migration_patch_bytes_equal"])
        self.assertTrue(compared["syntax_repair_patch_bytes_equal"])
        self.assertEqual(compared["control_count"], 23)
        self.assertEqual(compared["remaining_negative_count"], 19)
        self.assertEqual(
            compared["independent_migration_patch_sha256"],
            compared["primary_migration_patch_sha256"],
        )
        self.assertEqual(
            compared["independent_syntax_repair_patch_sha256"],
            compared["primary_syntax_repair_patch_sha256"],
        )

    def test_exact_edit_application_rejects_drift_and_overlap(self) -> None:
        self.assertEqual(
            apply_exact_edits(b"one two one", (ExactEdit(b"one", b"x", 2),)),
            b"x two x",
        )
        with self.assertRaisesRegex(IndependentRepairError, "occurrence count"):
            apply_exact_edits(b"one", (ExactEdit(b"one", b"x", 2),))
        with self.assertRaisesRegex(IndependentRepairError, "overlap"):
            apply_exact_edits(
                b"abcd",
                (ExactEdit(b"abc", b"x"), ExactEdit(b"bcd", b"y")),
            )

    def test_independent_modules_do_not_import_primary_renderer(self) -> None:
        forbidden = (
            "from form2_inputs",
            "from form2_lex",
            "from form2_model",
            "from form2_patch",
            "from form2_render",
            "from form2_report",
            "from form2_structural_report",
            "from form2_tree",
            "from form2_zero_audit",
        )
        for path in PROPOSAL.glob("form2_independent_*.py"):
            text = path.read_text(encoding="utf-8")
            for statement in forbidden:
                self.assertNotIn(statement, text)

    def test_primary_comparison_fails_closed_when_absent(self) -> None:
        previous = form2_independent_compare.PRIMARY_REPORT
        with tempfile.TemporaryDirectory() as directory:
            form2_independent_compare.PRIMARY_REPORT = Path(directory) / "absent.json"
            try:
                with self.assertRaisesRegex(IndependentReportError, "absent"):
                    build_independent_artifacts(
                        compare_primary=True,
                        inputs=self.inputs,
                    )
            finally:
                form2_independent_compare.PRIMARY_REPORT = previous

    def test_primary_comparison_rejects_an_authored_owner_path_digest(self) -> None:
        independent = next(
            row
            for row in self.report["files"]
            if row["stage"] == COMPLETE_STAGE
        )
        primary = {
            "after_sha256": independent["migration_sha256"],
            "before_sha256": independent["source_sha256"],
            "bundle_topology_projection_sha256": independent[
                "bundle_topology_projection_sha256"
            ],
            "canonical_sha256": independent["canonical_sha256"],
            "exact_parse": {"classification": "one"},
            "separator_owner_projection_sha256": "0" * 64,
            "source_forest_projection_sha256": independent[
                "source_forest_projection_sha256"
            ],
            "syntax_repaired_sha256": independent["syntax_repaired_sha256"],
            "terminal_projection_sha256_after": independent[
                "terminal_projection_sha256_after"
            ],
            "terminal_projection_sha256_before": independent[
                "terminal_projection_sha256_before"
            ],
            "terminal_projection_sha256_render_input": independent[
                "terminal_projection_sha256_render_input"
            ],
        }
        with self.assertRaisesRegex(
            form2_independent_compare.IndependentComparisonError,
            "separator_owner_projection_sha256 mismatch",
        ):
            form2_independent_compare._compare_file_rows(
                [independent], {independent["path"]: primary}
            )

    def test_reader_rejects_symlink(self) -> None:
        previous = form2_independent_inputs.REPOSITORY_ROOT
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "regular").write_bytes(b"input\n")
            (root / "link").symlink_to("regular")
            form2_independent_inputs.REPOSITORY_ROOT = root
            try:
                with self.assertRaisesRegex(IndependentInputError, "non-symlink"):
                    form2_independent_inputs._read_regular("link", 64)
            finally:
                form2_independent_inputs.REPOSITORY_ROOT = previous


if __name__ == "__main__":
    unittest.main()
