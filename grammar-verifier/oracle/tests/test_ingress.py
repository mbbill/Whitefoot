"""Exact framing and closed manifest-input tests."""

from __future__ import annotations

from io import BytesIO
import struct
import unittest

from core import Failure
from ingress import parse_cases, parse_domains, parse_limits, read_frame
from support import fixture_sections, frame_bytes, limits_with


def assert_failure(test: unittest.TestCase, raw: bytes, family: str, code: str) -> None:
    with test.assertRaises(Failure) as raised:
        read_frame(BytesIO(raw))
    test.assertEqual((raised.exception.family, raised.exception.code), (family, code))


class IngressTests(unittest.TestCase):
    def test_exact_frame_round_trips_all_five_sections(self) -> None:
        sections = fixture_sections()
        parsed = read_frame(BytesIO(frame_bytes(sections)))
        self.assertEqual(tuple(item.data for item in parsed.bound_sections()), sections)
        self.assertEqual(
            [case.identifier for case in parsed.cases],
            [
                "deref-p",
                "deref-x",
                "law-unknown-name",
                "law-wrong-function-role",
                "law-wrong-identity-roles",
                "law-zero-arity",
                "ordinary-let",
                "requires-control",
                "requires-doc-only",
                "requires-no-check",
                "requires-nonfinal-check",
                "requires-set",
                "requires-value-match",
                "statement-match",
                "try-let",
                "value-match",
            ],
        )
        self.assertEqual([domain.identifier for domain in parsed.domains], ["fixed-lowerword-calls"])

    def test_header_lengths_eof_and_current_identity_fail_closed(self) -> None:
        exact = frame_bytes()
        assert_failure(self, b"BADGRAM!" + exact[8:], "input", "frame_header")
        assert_failure(self, exact + b"x", "input", "frame_length")
        huge = exact[:8] + struct.pack(">Q", 8_193) + exact[16:]
        assert_failure(self, huge, "input", "section_outer_limit")
        sections = list(fixture_sections())
        sections[1] = b"X" + sections[1][1:]
        assert_failure(self, frame_bytes(tuple(sections)), "input", "current_document_hash")

    def test_limits_and_manifests_are_closed_and_sorted(self) -> None:
        sections = list(fixture_sections())
        limit_lines = sections[0].splitlines(keepends=True)
        sections[0] = limit_lines[1] + limit_lines[0] + b"".join(limit_lines[2:])
        assert_failure(self, frame_bytes(tuple(sections)), "input", "limits_order")

        sections = list(fixture_sections())
        sections[0] = sections[0].replace(b"cpu_timeout_seconds=60", b"cpu_timeout_seconds=61", 1)
        assert_failure(self, frame_bytes(tuple(sections)), "input", "limits_value")

        sections = list(fixture_sections())
        sections[3] = sections[3].replace(b"6465726566287029", b"646572656628702A", 1)
        assert_failure(self, frame_bytes(tuple(sections)), "input", "case")

        sections = list(fixture_sections())
        lines = sections[4].splitlines(keepends=True)
        sections[4] = lines[0] + lines[1] + lines[1]
        assert_failure(self, frame_bytes(tuple(sections)), "input", "domain_order")

        sections = list(fixture_sections())
        sections[3] = sections[3].replace(b"case\tderef-p", b"case\t1deref-p", 1)
        assert_failure(self, frame_bytes(tuple(sections)), "input", "case_identifier")

    def test_oversized_decimal_limit_is_rejected_before_integer_conversion(self) -> None:
        limits = fixture_sections()[0].replace(
            b"cpu_timeout_seconds=60",
            b"cpu_timeout_seconds=" + b"9" * 5_000,
            1,
        )
        self.assertLessEqual(len(limits), 8_192)
        with self.assertRaises(Failure) as raised:
            parse_limits(limits)
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("input", "limits_value"),
        )

    def test_symbol_limit_applies_to_manifest_ids_and_starts(self) -> None:
        cases = fixture_sections()[3]
        domains = fixture_sections()[4]
        parsed_cases = parse_cases(cases, limits_with(max_symbol_bytes=64))
        maximum_case_symbol = max(
            len(value.encode("ascii"))
            for case in parsed_cases
            for value in (case.identifier, case.start)
        )
        self.assertEqual(
            len(parse_cases(cases, limits_with(max_symbol_bytes=maximum_case_symbol))),
            len(parsed_cases),
        )
        with self.assertRaises(Failure) as raised:
            parse_cases(cases, limits_with(max_symbol_bytes=maximum_case_symbol - 1))
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("input", "case_identifier"),
        )
        self.assertEqual(len(parse_domains(domains, limits_with(max_symbol_bytes=21))), 1)

        exact_cases = b"whitefoot.grammar-cases.v1\ncase\taaa\texpr\t78\n"
        exact_domains = (
            b"whitefoot.grammar-domains.v1\n"
            b"domain\taaa\tfixed-lowerword-call\texpr\t78\n"
        )
        self.assertEqual(len(parse_cases(exact_cases, limits_with(max_symbol_bytes=4))), 1)
        self.assertEqual(len(parse_domains(exact_domains, limits_with(max_symbol_bytes=4))), 1)
        for parser, raw, code in (
            (parse_cases, exact_cases.replace(b"\taaa\t", b"\taaaaa\t"), "case_identifier"),
            (parse_cases, exact_cases.replace(b"\texpr\t", b"\texprx\t"), "case_start"),
            (
                parse_domains,
                exact_domains.replace(b"\taaa\t", b"\taaaaa\t"),
                "domain_identifier",
            ),
            (parse_domains, exact_domains.replace(b"\texpr\t", b"\texprx\t"), "domain_start"),
        ):
            with self.subTest(code=code):
                with self.assertRaises(Failure) as raised:
                    parser(raw, limits_with(max_symbol_bytes=4))
                self.assertEqual((raised.exception.family, raised.exception.code), ("input", code))


if __name__ == "__main__":
    unittest.main()
