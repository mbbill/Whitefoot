#!/usr/bin/env python3
"""Hostile tests for the exact-v0.9 normative source index."""

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from bisect import bisect_right
from collections import Counter
from pathlib import Path
from unittest import mock

import facets


SPECIFICATION = facets.SPEC_PATH.read_bytes()
EXPECTED_TABLE_ONLY_DOTLESS = sorted(
    [
        "fgt",
        "fge",
        "fne",
        "array_new",
        "buffer_new",
        "iand",
        "ior",
        "ixor",
        "inot",
        "irotl",
        "irotr",
        "ipopcount",
        "iclz",
        "ictz",
        "ibswap",
        "imulhi",
        "imin",
        "imax",
        "reinterpret",
        "fneg",
        "fabs",
        "fcopysign",
        "fmin",
        "fmax",
        "ffloor",
        "fceil",
        "ftrunc",
        "froundeven",
        "frem",
        "finf",
        "fnan",
    ]
)
EXPECTED_LISTED_DOTLESS = [
    "ieq",
    "ine",
    "ilt",
    "ile",
    "igt",
    "ige",
    "eeq",
    "ene",
    "feq",
    "flt",
    "fle",
    "band",
    "bor",
    "bxor",
    "bnot",
    "cvt",
    "len",
    "slice_of",
    "box_new",
    "arena_new",
]


def replace_once(source: bytes, old: bytes, new: bytes) -> bytes:
    """Replace one exact source fragment and guard the mutation itself."""
    count = source.count(old)
    if count != 1:
        raise AssertionError(f"mutation target occurred {count} times: {old!r}")
    return source.replace(old, new, 1)


class FacetCatalogPinnedFactsTests(unittest.TestCase):
    """Pin source facts without turning them into completeness claims."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = facets.extract_source_index(SPECIFICATION)

    def test_exact_specification_identity_and_shape(self) -> None:
        self.assertEqual(len(SPECIFICATION), 98_044)
        self.assertEqual(SPECIFICATION.count(b"\n"), 527)
        self.assertEqual(facets.sha256(SPECIFICATION), facets.SPEC_SHA256)
        self.assertEqual(
            self.catalog["counts"],
            {
                "byte_exact_fences": 2,
                "core_grammar_productions": 58,
                "inline_grammar_productions": 4,
                "operation_name_occurrences": 84,
                "operation_names": 83,
                "operation_rows": 44,
                "report_rows": 4,
                "rules": 92,
                "sections": 17,
                "syntax_productions": 62,
            },
        )

    def test_all_syntax_productions_remain_distinct_source_atoms(self) -> None:
        productions = self.catalog["syntax_productions"]
        self.assertEqual(len(productions), 62)
        self.assertEqual(len({record["id"] for record in productions}), 62)
        self.assertEqual(
            Counter(record["source_form"] for record in productions),
            Counter({"fenced-core": 58, "inline-rule": 4}),
        )
        self.assertEqual(
            Counter(
                record["owner_rule"]
                for record in productions
                if record["source_form"] == "fenced-core"
            ),
            Counter({"GRAM-2": 24, "GRAM-3": 5, "GRAM-4": 17, "GRAM-5": 12}),
        )
        inline = [
            record
            for record in productions
            if record["source_form"] == "inline-rule"
        ]
        self.assertEqual(
            [(record["id"], record["lhs"], record["owner_rule"]) for record in inline],
            [
                ("production:CONST-1:const", "const", "CONST-1"),
                ("production:CONST-2:cvalue", "cvalue", "CONST-2"),
                ("production:EFF-1:effects", "effects", "EFF-1"),
                ("production:EFF-1:effect", "effect", "EFF-1"),
            ],
        )

    def test_every_source_atom_has_an_exact_in_range_span(self) -> None:
        lines = SPECIFICATION.splitlines(keepends=True)
        offsets = [0]
        for line in lines:
            offsets.append(offsets[-1] + len(line))
        atoms = [
            *self.catalog["rules"],
            *self.catalog["syntax_productions"],
            *self.catalog["operation_rows"],
            *self.catalog["report_rows"],
            *self.catalog["byte_exact_fences"],
        ]
        self.assertEqual(len(atoms), 92 + 62 + 44 + 4 + 2)
        self.assertEqual(len({atom["id"] for atom in atoms}), len(atoms))
        sources = [atom["source"] for atom in atoms]
        name_sets = self.catalog["operation_name_sets"]
        sources.extend([name_sets["listed_source"], name_sets["table_source"]])
        for source in sources:
            with self.subTest(source=source):
                self.assertEqual(
                    set(source),
                    {"byte_end", "byte_start", "line_end", "line_start", "sha256"},
                )
                byte_start = source["byte_start"]
                byte_end = source["byte_end"]
                self.assertTrue(0 <= byte_start < byte_end <= len(SPECIFICATION))
                self.assertEqual(
                    source["line_start"], bisect_right(offsets, byte_start)
                )
                self.assertEqual(
                    source["line_end"], bisect_right(offsets, byte_end - 1)
                )
                self.assertTrue(1 <= source["line_start"] <= source["line_end"] <= 527)
                self.assertEqual(
                    source["sha256"],
                    hashlib.sha256(SPECIFICATION[byte_start:byte_end]).hexdigest(),
                )

    def test_byte_exact_fences_have_pinned_content_identities(self) -> None:
        fences = {
            record["owner_rule"]: record
            for record in self.catalog["byte_exact_fences"]
        }
        self.assertEqual(
            {
                owner: (
                    record["source"]["line_start"],
                    record["source"]["line_end"],
                    record["byte_length"],
                    record["source"]["sha256"],
                )
                for owner, record in fences.items()
            },
            {
                "PRE-1": (
                    436,
                    468,
                    303,
                    "547eedebc7d9f262580c824045acf6b4643b10e42e388ce399479f901240c469",
                ),
                "EX-1": (
                    476,
                    517,
                    863,
                    "490b202c156669e29030a4e6c2b2a86434da0aa7d33005f3db5079d830cbec71",
                ),
            },
        )

    def test_operation_and_report_tables_have_exact_structural_counts(self) -> None:
        operation_rows = self.catalog["operation_rows"]
        operation_names = [
            name
            for row in operation_rows
            for name in row["names"]
        ]
        self.assertEqual(len(operation_rows), 44)
        self.assertEqual(len(operation_names), 84)
        self.assertEqual(len(set(operation_names)), 83)
        self.assertEqual(operation_names.count("cvt"), 2)
        self.assertEqual(
            [row["report"] for row in self.catalog["report_rows"]],
            ["trap", "check", "lifetime", "check-density"],
        )

    def test_operation_name_sets_are_preserved_without_interpretation(self) -> None:
        name_sets = self.catalog["operation_name_sets"]
        table_names = name_sets["table_dotless_identifiers"]
        listed_names = name_sets["listed_dotless_identifiers"]
        self.assertEqual(len(table_names), 51)
        self.assertEqual(listed_names, EXPECTED_LISTED_DOTLESS)
        self.assertEqual(
            name_sets["table_only_identifiers"], EXPECTED_TABLE_ONLY_DOTLESS
        )
        self.assertEqual(name_sets["listed_only_identifiers"], [])
        self.assertEqual(
            sorted(set(table_names) - set(listed_names)),
            sorted(EXPECTED_TABLE_ONLY_DOTLESS),
        )
        self.assertNotEqual(table_names, listed_names)

    def test_canonical_output_is_ascii_deterministic_json(self) -> None:
        first = facets.canonical_bytes(self.catalog)
        second = facets.canonical_bytes(
            facets.extract_source_index(SPECIFICATION)
        )
        self.assertEqual(first, second)
        self.assertTrue(first.endswith(b"\n"))
        self.assertEqual(json.loads(first.decode("ascii")), self.catalog)

    def test_checked_in_output_is_fresh_and_stale_bytes_are_rejected(self) -> None:
        expected = facets.generated_bytes()
        self.assertEqual(facets.OUTPUT_PATH.read_bytes(), expected)
        with tempfile.TemporaryDirectory() as directory:
            stale_path = Path(directory) / "source.json"
            stale_path.write_bytes(expected + b" ")
            with mock.patch.object(facets, "OUTPUT_PATH", stale_path):
                with self.assertRaisesRegex(facets.CatalogError, "stale"):
                    facets.check()


class FacetCatalogFailClosedTests(unittest.TestCase):
    """Exercise exact identity and structural drift independently."""

    def assert_structural_error(self, source: bytes) -> None:
        with self.assertRaises(facets.CatalogError):
            facets.extract_source_index(source, expected_spec_sha256=None)

    def test_any_specification_byte_drift_fails_exact_identity(self) -> None:
        changed = replace_once(SPECIFICATION, b"enum Bool", b"enum Bxol")
        with self.assertRaisesRegex(facets.CatalogError, "specification hash"):
            facets.extract_source_index(changed)

    def test_crlf_and_missing_terminal_lf_fail(self) -> None:
        self.assert_structural_error(SPECIFICATION.replace(b"\n", b"\r\n"))
        self.assert_structural_error(SPECIFICATION[:-1])

    def test_duplicate_rule_identifier_fails_before_catalog_creation(self) -> None:
        changed = replace_once(SPECIFICATION, b"\n[GRAM-1] ", b"\n[GRAM-2] ")
        self.assert_structural_error(changed)

    def test_reordered_numbered_sections_fail(self) -> None:
        changed = replace_once(
            SPECIFICATION,
            b"## 1. Scope and conformance\n",
            b"## 2. Scope and conformance\n",
        )
        changed = replace_once(
            changed,
            b"## 2. Canonical form\n",
            b"## 1. Canonical form\n",
        )
        self.assert_structural_error(changed)

    def test_missing_core_grammar_production_fails(self) -> None:
        changed = replace_once(SPECIFICATION, b"program      :=", b"program       =")
        self.assert_structural_error(changed)

    def test_missing_inline_effect_grammar_production_fails(self) -> None:
        for old, new in (
            (b"`effects :=", b"`effects  ="),
            (b"with `effect :=", b"with `effect  ="),
        ):
            with self.subTest(old=old):
                self.assert_structural_error(replace_once(SPECIFICATION, old, new))

    def test_orphan_core_grammar_continuation_fails(self) -> None:
        changed = replace_once(
            SPECIFICATION,
            b'\n                "->" rtype effects',
            b'\n"->" rtype effects',
        )
        self.assert_structural_error(changed)

    def test_unexpected_grammar_definition_operator_fails(self) -> None:
        changed = replace_once(
            SPECIFICATION,
            b"[SCOPE-1] This document",
            b"[SCOPE-1] := This document",
        )
        self.assert_structural_error(changed)

    def test_operation_table_header_separator_and_row_drift_fail(self) -> None:
        mutations = [
            (
                b"| op | domain | signature | effects |",
                b"| operation | domain | signature | effects |",
            ),
            (
                b"| op | domain | signature | effects |\n|---|---|---|---|",
                b"| op | domain | signature | effects |\n|---|---|---|:---|",
            ),
            (
                b"| `finf` `fnan` | f32 f64 | `() -> own T` | pure |",
                b"  `finf` `fnan` | f32 f64 | `() -> own T` | pure |",
            ),
        ]
        for old, new in mutations:
            with self.subTest(old=old):
                self.assert_structural_error(replace_once(SPECIFICATION, old, new))

    def test_diagnostic_table_header_separator_and_row_drift_fail(self) -> None:
        mutations = [
            (
                b"| report | fields (all required) |",
                b"| report kind | fields (all required) |",
            ),
            (b"|---|---|\n| trap |", b"|---|:---|\n| trap |"),
            (
                b"| check-density | per function:",
                b"  check-density | per function:",
            ),
        ]
        for old, new in mutations:
            with self.subTest(old=old):
                self.assert_structural_error(replace_once(SPECIFICATION, old, new))

    def test_duplicate_listed_dotless_operation_name_fails(self) -> None:
        changed = replace_once(
            SPECIFICATION,
            b"or a dotless IDENT (`ieq ine ilt",
            b"or a dotless IDENT (`ieq ieq ilt",
        )
        self.assert_structural_error(changed)

    def test_prelude_and_example_fence_marker_drift_fail(self) -> None:
        for owner_text in (
            b"[PRE-1] The prelude is exactly:\n\n```\n",
            (
                b"[EX-1] The following complete program is byte-exact "
                b"canonical form:\n\n```\n"
            ),
        ):
            with self.subTest(owner=owner_text[:7]):
                changed = replace_once(
                    SPECIFICATION,
                    owner_text,
                    owner_text.replace(b"```", b"``wf", 1),
                )
                self.assert_structural_error(changed)

    def test_prelude_and_example_payload_drift_fail_exact_digest(self) -> None:
        for old, new in (
            (b"enum Bool {", b"enum Bxol {"),
            (b"enum Sign {", b"enum Sxgn {"),
        ):
            with self.subTest(old=old):
                self.assert_structural_error(replace_once(SPECIFICATION, old, new))


if __name__ == "__main__":
    unittest.main(verbosity=2)
