#!/usr/bin/env python3
"""Hostile tests for the independent FORM-2 source-audit check."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_canonical_contract.py")
SPEC = importlib.util.spec_from_file_location("check_canonical_contract", MODULE_PATH)
if SPEC is None or SPEC.loader is None:  # pragma: no cover - import machinery failure
    raise RuntimeError("cannot load check_canonical_contract.py")
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)

SPECIFICATION = check.SPEC.read_text(encoding="utf-8")
RUST = check.RUST.read_text(encoding="utf-8") + check.FORMAT_RUST.read_text(encoding="utf-8")


class CanonicalContractTests(unittest.TestCase):
    def test_live_contract_passes(self) -> None:
        check.validate(SPECIFICATION, RUST)

    def test_missing_line_production_is_rejected(self) -> None:
        mutant = RUST.replace("| ProductionV0_9::GiveStmt", "", 1)
        with self.assertRaisesRegex(check.ContractError, "line-bearing production"):
            check.validate(SPECIFICATION, mutant)

    def test_extra_block_production_is_rejected(self) -> None:
        mutant = RUST.replace(
            "ProductionV0_9::StructDecl",
            "ProductionV0_9::Field | ProductionV0_9::StructDecl",
            1,
        )
        with self.assertRaisesRegex(check.ContractError, "block-bearing production"):
            check.validate(SPECIFICATION, mutant)

    def test_conditional_let_policy_is_rejected_when_weakened(self) -> None:
        mutant = RUST.replace("| ProductionV0_9::TryLetRhs", "", 1)
        with self.assertRaisesRegex(check.ContractError, "conditional let_stmt"):
            check.validate(SPECIFICATION, mutant)

    def test_attachment_drift_is_rejected(self) -> None:
        mutant = RUST.replace("| FixedTerminalV0_9::Ampersand", "", 1)
        with self.assertRaisesRegex(check.ContractError, "left-attachment"):
            check.validate(SPECIFICATION, mutant)

    def test_fixed_spelling_seam_is_required(self) -> None:
        mutant = RUST.replace("fixed.spelling()", "token.span().bytes()", 1)
        with self.assertRaisesRegex(check.ContractError, "fixed.spelling"):
            check.validate(SPECIFICATION, mutant)

    def test_duplicate_render_buffer_is_forbidden(self) -> None:
        mutant = RUST + "\nfn forbidden() { let _: Vec<u8> = Vec::new(); }\n"
        with self.assertRaisesRegex(check.ContractError, "duplicate rendered byte buffer"):
            check.validate(SPECIFICATION, mutant)


if __name__ == "__main__":
    unittest.main()
