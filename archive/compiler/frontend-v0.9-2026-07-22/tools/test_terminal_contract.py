#!/usr/bin/env python3
"""Hostile tests for the independent v0.9 terminal-table check."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_contract.py")
SPEC = importlib.util.spec_from_file_location("check_terminal_contract", MODULE_PATH)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import machinery failure
    raise RuntimeError("cannot load check_terminal_contract.py")
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)


def specification(grammar: bytes) -> bytes:
    """Wrap one test grammar in the section boundary used by the checker."""
    fence = b"\x60\x60\x60"
    return b"# Test\n## 3. Grammar\n" + fence + b"\n" + grammar + fence + b"\n## 4. Next\n"


def rust_source(order: tuple[tuple[str, str], ...]) -> str:
    """Build the minimum Rust surface consumed by the checker."""
    fixed = "\n".join(f"FixedTerminalV0_9::{variant}," for variant, _ in order)
    predicates = "\n".join(
        f"TerminalPredicateV0_9::Fixed(FixedTerminalV0_9::{variant}),"
        for variant, _ in order
    )
    predicates += "\n" + "\n".join(
        f"TerminalPredicateV0_9::{variant},"
        for variant in check.EXTERNAL_VARIANTS
    )
    arms = "\n".join(f'Self::{variant} => b"{spelling}",' for variant, spelling in order)
    return f"""
pub const ALL_FIXED_TERMINALS_V0_9: [FixedTerminalV0_9; 64] = [
{fixed}
];
pub const ALL_TERMINAL_PREDICATES_V0_9: [TerminalPredicateV0_9; 72] = [
{predicates}
];
fn spelling(self) {{
    match self {{
{arms}
    }}
}}
"""


class TerminalContractTests(unittest.TestCase):
    def test_pattern_atom_is_not_a_fixed_predicate(self) -> None:
        source = specification(b'const := "const" "[0-9]+" ";"\n')
        self.assertEqual(check.fixed_from_spec(source), (b"const", b";"))

    def test_compound_atom_expands_to_raw_tokens(self) -> None:
        source = specification(b'mode := "&uniq" IDENT "->" type\n')
        self.assertEqual(check.fixed_from_spec(source), (b"&", b"uniq", b"->"))

    def test_repeated_fixed_spelling_keeps_first_occurrence_only(self) -> None:
        source = specification(b'a := "x" "(" "x" ")"\n')
        self.assertEqual(check.fixed_from_spec(source), (b"x", b"(", b")"))

    def test_continuation_line_atoms_are_not_lost(self) -> None:
        source = specification(b'a := "x"\n     "y" Z\n')
        self.assertEqual(check.fixed_from_spec(source), (b"x", b"y"))

    def test_inline_production_after_grammar_fence_is_included(self) -> None:
        source = specification(b'a := "x"\n') + b'effects := "pure" | "traps"\n'
        self.assertEqual(check.fixed_from_spec(source), (b"x", b"pure", b"traps"))

    def test_missing_spelling_arm_is_rejected(self) -> None:
        rust = rust_source((("X", "x"),)).replace('Self::X => b"x",', "")
        with self.assertRaisesRegex(check.ContractError, "variants and spelling arms differ"):
            check.fixed_from_rust(rust)

    def test_reordered_table_is_rejected(self) -> None:
        source = specification(b'a := "x" "y"\n')
        rust = rust_source((("Y", "y"), ("X", "x")))
        with self.assertRaisesRegex(check.ContractError, "differs from grammar"):
            check.validate(source, rust)

    def test_external_predicate_omission_is_rejected(self) -> None:
        source = specification(b'a := "x"\n')
        rust = rust_source((("X", "x"),)).replace(
            "TerminalPredicateV0_9::Literal,", ""
        )
        with self.assertRaisesRegex(check.ContractError, "exactly one Literal"):
            check.validate(source, rust)


if __name__ == "__main__":
    unittest.main()
