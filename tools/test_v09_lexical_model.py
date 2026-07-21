#!/usr/bin/env python3
"""Tests for the compiler-independent v0.9 lexical reference model."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path

import v09_lexical_model as model


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "frontend-corpus" / "v0.9" / "lexical-fixtures.json"
LIMIT_FIELDS = {
    "max_sources",
    "max_source_bytes",
    "max_total_source_bytes",
    "max_token_bytes",
    "max_tokens",
    "max_lexemes",
}


def exact_hex(value: object) -> bytes:
    """Decode one strict lowercase, whitespace-free hexadecimal payload."""

    if not isinstance(value, str) or re.fullmatch(r"(?:[0-9a-f]{2})*", value) is None:
        raise ValueError("fixture payload is not exact lowercase hexadecimal")
    return bytes.fromhex(value)


def render(outcome: model.LexOutcome) -> dict:
    """Project a model result into the neutral checked fixture schema."""

    if isinstance(outcome, model.Complete):
        return {
            "outcome": "complete",
            "token_count": outcome.token_count,
            "source_ranges": [list(pair) for pair in outcome.source_ranges],
            "pieces": [
                [
                    piece.source_ordinal,
                    piece.start,
                    piece.end,
                    piece.kind,
                    piece.exact.hex(),
                ]
                for piece in outcome.pieces
            ],
        }
    if isinstance(outcome, model.SourceIssue):
        return {
            "outcome": "source_issue",
            "source_ordinal": outcome.source_ordinal,
            "start": outcome.start,
            "end": outcome.end,
            "kind": outcome.kind,
            "hex": outcome.exact.hex(),
        }
    if isinstance(outcome, model.ResourceFailure):
        return {
            "outcome": "resource_failure",
            "limit": outcome.limit,
            "maximum": outcome.maximum,
            "actual": outcome.actual,
        }
    raise TypeError(f"unknown lexical outcome {type(outcome)!r}")


def load_fixtures() -> dict:
    """Load the small authored corpus with a closed outer schema."""

    raw = json.loads(FIXTURES.read_text(encoding="utf-8"))
    if set(raw) != {
        "schema",
        "kind",
        "spec_sha256",
        "static_catalog_sha256",
        "default_limits",
        "cases",
    }:
        raise ValueError("lexical fixture top-level schema drifted")
    if raw["schema"] != 1 or raw["kind"] != "whitefoot-v0.9-lexical-fixtures":
        raise ValueError("lexical fixture identity drifted")
    if set(raw["default_limits"]) != LIMIT_FIELDS:
        raise ValueError("lexical fixture default limits drifted")
    return raw


class FixtureTests(unittest.TestCase):
    def test_authority_bindings_are_exact(self) -> None:
        fixtures = load_fixtures()
        specification = hashlib.sha256(
            (ROOT / "spec" / "kernel-spec-v0.9.md").read_bytes()
        ).hexdigest()
        catalog_lock = (ROOT / "facets" / "v0.9" / "static-catalog.sha256").read_bytes()
        self.assertEqual(specification, model.SPEC_SHA256)
        self.assertEqual(catalog_lock, f"{model.STATIC_CATALOG_SHA256}\n".encode("ascii"))
        self.assertEqual(fixtures["spec_sha256"], model.SPEC_SHA256)
        self.assertEqual(
            fixtures["static_catalog_sha256"], model.STATIC_CATALOG_SHA256
        )

    def test_every_authored_fixture_matches_the_independent_model(self) -> None:
        fixtures = load_fixtures()
        defaults = fixtures["default_limits"]
        identifiers: set[str] = set()
        for case in fixtures["cases"]:
            with self.subTest(case=case.get("id")):
                self.assertEqual(
                    set(case),
                    {"id", "sources_hex", "expect"}
                    | ({"limits"} if "limits" in case else set()),
                )
                identifier = case["id"]
                self.assertIsInstance(identifier, str)
                self.assertNotIn(identifier, identifiers)
                identifiers.add(identifier)
                overrides = case.get("limits", {})
                self.assertTrue(set(overrides) <= LIMIT_FIELDS)
                limits = model.LexLimits(**(defaults | overrides))
                sources = tuple(exact_hex(value) for value in case["sources_hex"])
                self.assertEqual(render(model.lex_v0_9(sources, limits)), case["expect"])


class ModelContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.limits = model.LexLimits(
            max_sources=1_000,
            max_source_bytes=1_000_000,
            max_total_source_bytes=2_000_000,
            max_token_bytes=1_000_000,
            max_tokens=1_000_000,
            max_lexemes=2_000_000,
        )

    def complete(self, sources: tuple[bytes, ...]) -> model.Complete:
        outcome = model.lex_v0_9(sources, self.limits)
        self.assertIsInstance(outcome, model.Complete)
        return outcome

    def limited(self, **changes: int) -> model.LexLimits:
        values = vars(self.limits) | changes
        return model.LexLimits(**values)

    def test_complete_partitions_are_contiguous_nonempty_and_exact(self) -> None:
        sources = (b"", b"fn  name\n\n", b'"a\\\\b" -> @label')
        outcome = self.complete(sources)
        for source_ordinal, source in enumerate(sources):
            cursor = 0
            rebuilt = bytearray()
            pieces = outcome.source_pieces(source_ordinal)
            self.assertIsNotNone(pieces)
            for piece in pieces or ():
                self.assertEqual(piece.source_ordinal, source_ordinal)
                self.assertEqual(piece.start, cursor)
                self.assertGreater(piece.end, piece.start)
                self.assertEqual(piece.exact, source[piece.start : piece.end])
                rebuilt.extend(piece.exact)
                cursor = piece.end
            self.assertEqual(cursor, len(source))
            self.assertEqual(bytes(rebuilt), source)

    def test_bundle_scan_equals_independent_source_scans(self) -> None:
        sources = (b"foo.", b"wrap", b"-1_i64 @again\n")
        combined = self.complete(sources)
        for source_ordinal, source in enumerate(sources):
            separate = self.complete((source,))
            combined_shape = [
                (piece.start, piece.end, piece.kind, piece.exact)
                for piece in combined.source_pieces(source_ordinal) or ()
            ]
            separate_shape = [
                (piece.start, piece.end, piece.kind, piece.exact)
                for piece in separate.source_pieces(0) or ()
            ]
            self.assertEqual(combined_shape, separate_shape)

    def test_every_single_top_level_byte_has_the_exact_outcome_class(self) -> None:
        punctuation = b"(){}[]<>,:;.=&"
        for byte in range(256):
            expected_complete = (
                (byte < 0x80 and chr(byte).isalnum())
                or byte in punctuation
                or byte in (0x20, 0x0A)
            )
            outcome = model.lex_v0_9((bytes((byte,)),), self.limits)
            self.assertEqual(
                isinstance(outcome, model.Complete),
                expected_complete,
                f"byte=0x{byte:02x}",
            )
            self.assertNotIsInstance(outcome, model.ResourceFailure)

    def test_every_single_string_byte_has_the_exact_outcome_class(self) -> None:
        for byte in range(256):
            expected_complete = 0x20 <= byte <= 0x7E and byte not in (0x22, 0x5C)
            outcome = model.lex_v0_9((bytes((0x22, byte, 0x22)),), self.limits)
            self.assertEqual(
                isinstance(outcome, model.Complete),
                expected_complete,
                f"byte=0x{byte:02x}",
            )

    def test_number_candidates_remain_opaque_and_operation_suffixes_stay_closed(self) -> None:
        numbers = (
            b"-2147483648_i32",
            b"-0_i32",
            b"01_i32",
            b"1.0E2_f64",
            b"1._f64",
            b"1.0e_f64",
            b"1.0e+2_f64",
        )
        for source in numbers:
            pieces = self.complete((source,)).pieces
            self.assertEqual(
                [(piece.kind, piece.exact) for piece in pieces],
                [("number_form", source)],
            )
        for suffix in (b"wrap", b"trap", b"checked", b"sat", b"strict"):
            exact = b"base." + suffix
            self.assertEqual(self.complete((exact,)).pieces[0].kind, "operation_name_form")
            near = exact + b"x"
            self.assertEqual(
                [piece.kind for piece in self.complete((near,)).pieces],
                ["lower_word_form", "dot", "lower_word_form"],
            )

    def test_exact_limits_are_neutral_resource_failures(self) -> None:
        source = (b"a b\n",)
        exact = model.LexLimits(1, 4, 4, 1, 2, 4)
        self.assertIsInstance(model.lex_v0_9(source, exact), model.Complete)
        one_less = model.LexLimits(1, 4, 4, 1, 1, 4)
        outcome = model.lex_v0_9(source, one_less)
        self.assertEqual(
            render(outcome),
            {
                "outcome": "resource_failure",
                "limit": "tokens",
                "maximum": 1,
                "actual": 2,
            },
        )

    def test_every_resource_ceiling_names_its_first_excess(self) -> None:
        cases = (
            ((b"", b""), self.limited(max_sources=1), "sources", 2),
            ((b"ab",), self.limited(max_source_bytes=1), "source_bytes", 2),
            (
                (b"a", b"b"),
                self.limited(max_total_source_bytes=1),
                "total_source_bytes",
                2,
            ),
            ((b"ab",), self.limited(max_token_bytes=1), "token_bytes", 2),
            ((b"a b",), self.limited(max_tokens=1), "tokens", 2),
            ((b"a b",), self.limited(max_lexemes=2), "lexemes", 3),
        )
        for sources, limits, name, actual in cases:
            with self.subTest(limit=name):
                outcome = model.lex_v0_9(sources, limits)
                self.assertIsInstance(outcome, model.ResourceFailure)
                self.assertEqual(outcome.limit, name)
                self.assertEqual(outcome.actual, actual)

    def test_repeated_runs_are_structurally_identical(self) -> None:
        sources = (b"@label p.field foo.checked -1_i64\n",)
        self.assertEqual(
            render(model.lex_v0_9(sources, self.limits)),
            render(model.lex_v0_9(sources, self.limits)),
        )

    def test_non_bytes_and_negative_limits_fail_at_the_model_boundary(self) -> None:
        with self.assertRaises(TypeError):
            model.lex_v0_9((bytearray(b"x"),), self.limits)  # type: ignore[arg-type]
        with self.assertRaises(ValueError):
            model.LexLimits(-1, 0, 0, 0, 0, 0)

    def test_limit_values_match_the_rust_wire_domain_exactly(self) -> None:
        maximum = model.LexLimits(
            model.U32_MAX,
            model.U64_MAX,
            model.U64_MAX,
            model.U64_MAX,
            model.U64_MAX,
            model.U64_MAX,
        )
        self.assertEqual(maximum.max_sources, model.U32_MAX)
        self.assertEqual(maximum.max_lexemes, model.U64_MAX)

        fields = tuple(vars(maximum))
        for field in fields:
            with self.subTest(field=field, value="bool"):
                values = vars(maximum) | {field: True}
                with self.assertRaises(ValueError):
                    model.LexLimits(**values)
            with self.subTest(field=field, value="negative"):
                values = vars(maximum) | {field: -1}
                with self.assertRaises(ValueError):
                    model.LexLimits(**values)
            with self.subTest(field=field, value="overflow"):
                field_maximum = (
                    model.U32_MAX if field == "max_sources" else model.U64_MAX
                )
                values = vars(maximum) | {field: field_maximum + 1}
                with self.assertRaises(ValueError):
                    model.LexLimits(**values)


if __name__ == "__main__":
    unittest.main()
