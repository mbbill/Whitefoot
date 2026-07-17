#!/usr/bin/env python3
"""Verify the parked D15 systems-performance-coverage research status.

The design package is complete and PARKED (D19). This checks the live status
layer is coherent and points at the parked package; it does NOT read the
archive tree (per the archive charter, no tool reads from it).
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    raise ValueError(message)


def require(path: Path, phrases: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for phrase in phrases:
        if phrase not in text:
            fail(f"{path.relative_to(ROOT)} omits {phrase!r}")


COVERAGE = (
    ROOT / "optimizer-language-research" / "implementation" / "systems-performance-coverage"
)


def main() -> int:
    # Core invariant: agent instructions stay byte-identical.
    if (ROOT / "AGENTS.md").read_bytes() != (ROOT / "CLAUDE.md").read_bytes():
        fail("AGENTS.md and CLAUDE.md are not byte-identical")

    # Live status layer: the package is parked and points at itself + the archive.
    require(
        ROOT / "CLAUDE.md",
        (
            "The D15 systems-performance-coverage capability research is COMPLETE and\n"
            "  PARKED at the owner's direction",
            "systems-performance-coverage/",
            "archive/research/minimal-systems-capability/",
            "NOTHING is authorized for production",
        ),
    )
    require(
        ROOT / "THE-PLAN.md",
        (
            "STATUS (2026-07-17, PARKED)",
            "COMPLETE and PARKED at the owner's direction",
            "archive/research/minimal-systems-capability/",
        ),
    )
    require(
        ROOT / "HANDOVER.md",
        (
            "**PARKED (2026-07-17, D19).**",
            "archive/research/minimal-systems-capability/",
        ),
    )
    require(
        ROOT / "optimizer-language-research" / "notes" / "user-directives.md",
        (
            "## D15 (2026-07-16): Fresh autonomous derivation of the systems-performance capability set",
            "## D19 (2026-07-17): The five concurrency-delta decisions",
        ),
    )

    # The parked package is present and complete.
    for rel in (
        "DESIGN-DOSSIER.md",
        "DESIGN-COMPARISON-AND-RECOMMENDATION.md",
        "SCENARIO-DEMAND-MAP.md",
        "CATALOG-V1-RECUT.md",
        "FOLLOW-UPS.md",
        "m1-loan-judgment/RULES-RATIFIED.md",
        "m2-spec-mass/conc-normative.md",
    ):
        if not (COVERAGE / rel).is_file():
            fail(f"parked package missing {rel!r}")
    require(
        COVERAGE / "DESIGN-DOSSIER.md",
        ("RULED 2026-07-17 (D19).", "## 5. The five owner decisions"),
    )

    print(
        "performance research status: D15 systems-performance-coverage package "
        "PARKED (D19); design complete, landing owner-gated"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance research status verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
