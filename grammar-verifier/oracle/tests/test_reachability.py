"""Focused production-root and reachability regressions."""

from __future__ import annotations

import unittest

from core import Failure
from extract import extract_document
from support import fixture_inputs


PROGRAM = b"program      := item*"
PROBE = (
    b"\n[GRAM-99] Reachability grammar.\n\n"
    b"```\n"
    b'reachability_probe := "reachability_probe"\n'
    b"```\n"
)


class ReachabilityTests(unittest.TestCase):
    def test_program_reaches_every_production(self) -> None:
        current = fixture_inputs().current.data
        self.assertEqual(current.count(PROGRAM), 1)
        source = current.replace(
            PROGRAM,
            b"program      := reachability_probe? item*",
            1,
        )
        source += PROBE
        grammar = extract_document("reachable", source, fixture_inputs().limits)
        self.assertIn("reachability_probe", grammar.production_by_name)
        self.assertEqual(len(grammar.productions), 60)

    def test_missing_program_root_is_rejected(self) -> None:
        current = fixture_inputs().current.data
        self.assertEqual(current.count(PROGRAM), 1)
        with self.assertRaises(Failure) as raised:
            extract_document(
                "missing-root",
                current.replace(PROGRAM, b"root         := item*", 1),
                fixture_inputs().limits,
            )
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("extraction", "program_start_missing"),
        )

    def test_unreachable_production_is_rejected(self) -> None:
        source = fixture_inputs().current.data + PROBE
        with self.assertRaises(Failure) as raised:
            extract_document("unreachable", source, fixture_inputs().limits)
        self.assertEqual(
            (raised.exception.family, raised.exception.code),
            ("extraction", "unreachable_production"),
        )


if __name__ == "__main__":
    unittest.main()
