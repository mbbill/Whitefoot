#!/usr/bin/env python3
"""Run only the Oracle's standard-library tests under isolated Python 3."""

from __future__ import annotations

from pathlib import Path
import sys
import unittest


TEST_ROOT = Path(__file__).resolve().parent
ORACLE_ROOT = TEST_ROOT.parent
sys.path.insert(0, str(TEST_ROOT))
sys.path.insert(0, str(ORACLE_ROOT))


def main() -> int:
    suite = unittest.defaultTestLoader.discover(str(TEST_ROOT), pattern="test_*.py")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
