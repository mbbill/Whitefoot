"""Exact and one-over checks for preallocation resource boundaries."""

from __future__ import annotations

import unittest

from core import Failure, LogicalBudget
from ingress import parse_cases, parse_domains
from source import _split_lines, scan_source
from support import limits_with


class ResourcePreflightTests(unittest.TestCase):
    def test_document_lines_check_count_and_width_before_append(self) -> None:
        raw = b"a\nbb\n"
        exact = _split_lines(
            raw,
            LogicalBudget(limits_with(max_lines=2, max_line_bytes=2)),
        )
        self.assertEqual(tuple(line.content for line in exact), (b"a", b"bb"))

        for name in ("max_lines", "max_line_bytes"):
            with self.subTest(limit=name):
                with self.assertRaises(Failure) as raised:
                    _split_lines(
                        raw,
                        LogicalBudget(limits_with(**{name: 1})),
                    )
                self.assertEqual(
                    (raised.exception.family, raised.exception.code),
                    ("resource", f"limit_{name}"),
                )

    def test_case_records_check_exact_and_one_over_before_append(self) -> None:
        raw = (
            b"whitefoot.grammar-cases.v1\n"
            b"case\ta\texpr\t61\n"
            b"case\tb\texpr\t62\n"
        )
        self.assertEqual(len(parse_cases(raw, limits_with(max_cases=2))), 2)
        with self.assertRaises(Failure) as raised:
            parse_cases(raw, limits_with(max_cases=1))
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("resource", "limit_max_cases"),
        )

    def test_assignment_census_checks_exact_and_one_over_before_append(self) -> None:
        raw = (
            b"[GRAM-1] Resource grammar.\n\n"
            b"```\n"
            b"program := branch\n"
            b'branch := "x"\n'
            b"```\n"
        )
        exact = scan_source(
            raw,
            LogicalBudget(limits_with(max_definitions=2)),
        )
        self.assertEqual(len(exact.assignment_offsets), 2)
        with self.assertRaises(Failure) as raised:
            scan_source(
                raw,
                LogicalBudget(limits_with(max_definitions=1)),
            )
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("resource", "limit_max_definitions"),
        )

    def test_domain_records_check_exact_and_one_over_before_append(self) -> None:
        raw = (
            b"whitefoot.grammar-domains.v1\n"
            b"domain\ta\tfixed-lowerword-call\texpr\t61\n"
            b"domain\tb\tfixed-lowerword-call\texpr\t62\n"
        )
        self.assertEqual(len(parse_domains(raw, limits_with(max_domains=2))), 2)
        with self.assertRaises(Failure) as raised:
            parse_domains(raw, limits_with(max_domains=1))
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("resource", "limit_max_domains"),
        )


if __name__ == "__main__":
    unittest.main()
