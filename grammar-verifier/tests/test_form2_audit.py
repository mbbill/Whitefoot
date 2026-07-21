from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROPOSAL = ROOT / "proposal"
if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

import form2_inputs  # noqa: E402
from form2_inputs import InputError, sha256  # noqa: E402
from form2_lex import tokenize  # noqa: E402
from form2_model import audit_current, audit_proposed  # noqa: E402
from support_form2_installation import reconstruct_form2_inputs  # noqa: E402


def clauses(raw: bytes, *, proposed: bool = False) -> dict[str, int]:
    audit = audit_proposed(raw) if proposed else audit_current(raw)
    return audit.report["violation_counts_by_clause"]


class Form2BoundaryTests(unittest.TestCase):
    def test_current_attached_paren_and_angle_spelling(self) -> None:
        raw = b"fn f<T>() -> own unit pure {\n  return unit;\n}\n"
        audit = audit_current(raw).report
        self.assertEqual(audit["violation_counts_by_clause"], {})
        self.assertEqual(
            audit["clauses"]["token_spacing"][
                "indeterminate_line_boundary_count"
            ],
            2,
        )

    def test_proposal_compacts_square_interiors_but_retains_preceding_space(self) -> None:
        current = b"fn f [ 'r ]() -> own unit pure {\n  return unit;\n}\n"
        proposed = b"fn f ['r]() -> own unit pure {\n  return unit;\n}\n"
        self.assertEqual(clauses(current), {})
        self.assertEqual(clauses(proposed, proposed=True), {})
        self.assertGreater(clauses(proposed).get("token-spacing", 0), 0)
        self.assertGreater(clauses(current, proposed=True).get("token-spacing", 0), 0)

    def test_overlapping_punctuation_obligations_are_not_given_precedence(self) -> None:
        raw = b"fn f() -> own unit pure {\n  let x: own T = K(a,);\n  return unit;\n}\n"
        audit = audit_current(raw).report
        conflicts = [
            violation
            for violation in audit["violations"]
            if violation["clause"] == "token-spacing-policy-conflict"
        ]
        self.assertEqual(len(conflicts), 1)
        self.assertEqual(
            conflicts[0]["requirements"],
            ["no-space-before-)", "one-space-after-comma"],
        )

    def test_non_token_comment_bytes_are_reported_without_invented_spacing(self) -> None:
        raw = b"fn f() -> own unit pure {\n  // not a language token\n  return unit;\n}\n"
        audit = audit_current(raw).report
        self.assertGreater(len(audit["lexical_observation_issues"]), 0)
        self.assertGreater(
            audit["clauses"]["token_spacing"][
                "indeterminate_non_token_boundary_count"
            ],
            0,
        )
        self.assertNotIn("token-spacing", audit["violation_counts_by_clause"])

    def test_fat_arrow_is_one_terminal(self) -> None:
        lexical = tokenize(b"True() => {\n}\n")
        self.assertIn(b"=>", [token.raw for token in lexical.tokens])
        self.assertNotIn(b"=", [token.raw for token in lexical.tokens])

    def test_opname_is_one_terminal_but_place_dots_are_separate(self) -> None:
        lexical = tokenize(b"iadd.checked<i32>(x, y); outer.inner.value;")
        raw = [token.raw for token in lexical.tokens]
        self.assertIn(b"iadd.checked", raw)
        self.assertNotIn(b"outer.inner", raw)
        self.assertEqual(raw[-6:], [b"outer", b".", b"inner", b".", b"value", b";"])


class Form2ClauseTests(unittest.TestCase):
    BASE = b"fn f() -> own unit pure {\n  return unit;\n}\n"

    def test_utf8_lf_eof_trailing_and_indentation_mutants(self) -> None:
        cases = {
            "utf8": self.BASE.replace(b"unit", b"\xffnit", 1),
            "lf-line-endings": self.BASE.replace(b"\n", b"\r\n", 1),
            "exactly-one-final-lf": self.BASE[:-1],
            "no-trailing-whitespace": self.BASE.replace(b"pure {\n", b"pure { \n"),
            "indentation-two-spaces-per-brace-level": self.BASE.replace(
                b"  return", b"\treturn"
            ),
        }
        for expected, raw in cases.items():
            with self.subTest(expected=expected):
                self.assertGreater(clauses(raw).get(expected, 0), 0)

    def test_statement_wrap_and_declaration_separator_mutants(self) -> None:
        wrapped = b"fn f() -> own unit pure {\n  return\n    unit;\n}\n"
        adjacent = self.BASE + self.BASE
        self.assertEqual(clauses(wrapped).get("statement-is-one-line"), 1)
        self.assertEqual(
            clauses(adjacent).get(
                "one-blank-line-between-top-level-declarations"
            ),
            1,
        )


class Form2ProtectedSurfaceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.inputs, _ = reconstruct_form2_inputs()

    def test_inventory_is_exact_and_hash_bound(self) -> None:
        self.assertEqual(len(self.inputs.sources), 293)
        self.assertEqual(
            sum(source.manifest is not None for source in self.inputs.sources), 292
        )
        self.assertEqual(
            sum(source.manifest is None for source in self.inputs.sources), 1
        )
        for source in self.inputs.sources:
            self.assertEqual(sha256(source.raw), source.sha256)

    def test_intended_form2_fixture_inventory_is_exact(self) -> None:
        identifiers = [
            source.identifier
            for source in self.inputs.sources
            if source.manifest is not None
            and source.manifest["expect"]
            == {"kind": "reject", "rule": "FORM-2"}
        ]
        self.assertEqual(
            identifiers,
            ["form2-neg-noncanonical-ws", "x-form-form2-tab-indent"],
        )

    def test_input_reader_rejects_a_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "target").write_bytes(b"protected\n")
            (root / "link").symlink_to("target")
            previous = form2_inputs.REPOSITORY
            form2_inputs.REPOSITORY = root
            try:
                with self.assertRaisesRegex(InputError, "non-symlink"):
                    form2_inputs._read_bounded("link", 64)
            finally:
                form2_inputs.REPOSITORY = previous


if __name__ == "__main__":
    unittest.main()
