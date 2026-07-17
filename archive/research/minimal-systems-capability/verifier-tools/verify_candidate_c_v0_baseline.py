#!/usr/bin/env python3
"""Verify the frozen Candidate C v0 audit baseline."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASELINE = (
    ROOT
    / "optimizer-language-research"
    / "implementation"
    / "minimal-systems-capability"
    / "CANDIDATE-C-V0-AUDIT-BASELINE.md"
)


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    text = BASELINE.read_text(encoding="utf-8")
    c0_ids = set(re.findall(r"^\| (C0-\d+) \|", text, re.MULTILINE))
    c_ids = set(re.findall(r"^\| (C-\d+) \|", text, re.MULTILINE))
    expected_c0 = {f"C0-{index}" for index in range(1, 12)}
    expected_c = {f"C-{index}" for index in range(1, 13)}
    if c0_ids != expected_c0:
        fail(f"C0 inventory mismatch: {sorted(c0_ids)}")
    if c_ids != expected_c:
        fail(f"Candidate C inventory mismatch: {sorted(c_ids)}")

    required = (
        "STAGE-0-AUDIT-READY",
        "One demand can expose a gap but cannot by itself admit a family.",
        "Explicit unresolved definitions",
        "Stage 1 must score these absences.",
        "Vacant payload is never read",
        "Unused families add no representation field",
        "`UNKNOWN`",
        "`COMPOSITION-GAP`",
        "`C0-GAP`",
        "`OPTIMIZER-GAP`",
        "C0 table forms are frozen but their platform row inventories remain",
    )
    for phrase in required:
        if phrase not in text:
            fail(f"baseline omits {phrase!r}")

    unresolved = re.findall(r"^\d+\. (?:C-|C0-)", text, re.MULTILINE)
    if len(unresolved) != 6:
        fail(f"expected 6 explicit unresolved definitions, found {len(unresolved)}")

    print("Candidate C v0 baseline: 11 C0 rows, 12 C rows, 6 unresolved definitions")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"Candidate C v0 baseline verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
