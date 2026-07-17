#!/usr/bin/env python3
"""Verify the active D15 systems-performance coverage research status."""

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
CAPABILITY = (
    ROOT / "optimizer-language-research" / "implementation" / "minimal-systems-capability"
)


def main() -> int:
    agents = (ROOT / "AGENTS.md").read_bytes()
    claude = (ROOT / "CLAUDE.md").read_bytes()
    if agents != claude:
        fail("AGENTS.md and CLAUDE.md are not byte-identical")

    # Active status layer: D15 fresh-derivation track.
    require(
        ROOT / "AGENTS.md",
        (
            "D15 systems-performance coverage research is the active capability track",
            "at least one blessed way of writing",
            "The D14 B-Strata-only lock is suspended",
            "remain historical evidence and falsifiers",
            "systems-performance-coverage/",
            "Gate #1 passed and is ratified",
            "exactly 15 rules, machine-verified on a",
            "97-program corpus with a 9/9 mutation-caught harness",
            "Owner rulings D16-D18 bind",
            "No production language,",
        ),
    )
    require(
        ROOT / "THE-PLAN.md",
        (
            "D15 FRESH-DERIVATION TRACK\n   ACTIVE",
            "at least one blessed way of writing",
            "The\n   D14 B-Strata-only lock no longer constrains the active derivation",
            "systems-performance-coverage/",
            "three-tier architecture",
            "loan/freeze judgment\n   plus confined borrow-carrying values",
            "M1 (paper falsification of the loan judgment",
            "STATUS (2026-07-17): all six kernel deltas have completed drafting and",
            "adversarial review. Gate #1 (loan/freeze, 15 rules) is passed and ratified",
            "Historical context of the superseded lock",
            "historical evidence, not active authority",
            "suspended under D15, retained as evidence",
        ),
    )
    require(
        ROOT / "HANDOVER.md",
        (
            "Status: active handover updated under D15 on 2026-07-16.",
            "## Current D15 status — read this first",
            "The D14 B-Strata-only lock is suspended",
            "a 9-family/51-scenario demand map, four independent complete designs,",
            "gate #1: a decidable loan/freeze judgment plus confined borrow-carrying\nvalues",
            "M1-M10 is preregistered with frozen pass/fail bands",
            "No production change\nis authorized before those decisions.",
            "## Historical B-Strata-only decision (superseded by D15 as active authority)",
        ),
    )
    require(
        ROOT / "optimizer-language-research" / "notes" / "user-directives.md",
        (
            "## D15 (2026-07-16): Fresh autonomous derivation of the systems-performance capability set",
            "at least one blessed way of writing whose performance reaches or\n  exceeds the best existing implementations",
            "The D14 B-Strata-only lock no longer constrains the active derivation.",
            "readmits compiler-known forms with disciplined\n  trusted internals",
            # Historical rulings must remain intact in the append-only record.
            "D14 B-Strata decisive ruling",
            "D14 B-Strata demand-substitution amendment",
        ),
    )
    require(
        ROOT / "mcts_mem" / "xlang.md",
        (
            "2026-07-16 owner D15 redirection and first fresh-pass finding",
            "the D14 B-Strata-only lock is suspended",
            "decidable per-binding loan/freeze judgment plus confined borrow-carrying values",
            # Historical facts must remain intact.
            "B-Strata is now the sole capability architecture under development",
        ),
    )

    # Active research record.
    require(
        COVERAGE / "DESIGN-COMPARISON-AND-RECOMMENDATION.md",
        (
            "Status: research result under D15.",
            "9-family, 51-scenario demand map",
            "## 3. The decisive cross-cutting finding",
            "Confined borrow-carrying values",
            "Loan/freeze judgment",
            "## 5. Recommended catalog skeleton",
            "## 6. Owner decision points",
            "## 8. Validation plan (preregistered, cheapest-decisive first)",
            "kills the architecture on failure",
        ),
    )
    require(
        COVERAGE / "SCENARIO-DEMAND-MAP.md",
        (
            "Status: research coverage target under D15.",
            "not a normative language document",
        ),
    )
    for name in (
        "scenario-map.json",
        "design-builtin.json",
        "design-core.json",
        "design-hybrid.json",
        "design-schema.json",
        "attacks-builtin.json",
        "attacks-core.json",
        "attacks-hybrid.json",
        "attacks-schema.json",
        "judge.json",
    ):
        if not (COVERAGE / "evidence" / name).is_file():
            fail(f"missing evidence artifact {name!r}")

    # Historical documents keep their frozen dispositions.
    require(
        CAPABILITY / "CANDIDATE-B-STRATA-DECISIVE-PLAN.md",
        (
            "controlling plan for the owner-selected B-Strata-only research track",
            "`K1 ROOTED-PLACE`",
            "`K2 SEALED-STATE`",
            "`K3 LINEAR-STEP`",
            "The work is complete only at `STRATA-YES` or `STRATA-NO`.",
        ),
    )
    require(
        CAPABILITY / "CANDIDATE-B-ELEGANT-DESIGN.md",
        (
            "Candidate B Design Gate disposition: `B-REVISE`.",
            "`B-STRATA` has six closed and eight open rows.",
            "`B-GRAPHS` has six closed and eight open rows.",
        ),
    )
    require(
        CAPABILITY / "CANDIDATE-C-SPARSE-REPAIR-CANDIDATES.md",
        (
            "Sparse Repair Gate disposition: `SPARSE-SELECT: SR-PROFILE`.",
            "Candidate C v0 is unchanged.",
        ),
    )
    require(
        CAPABILITY / "CANDIDATE-C-HASHBROWN-AUDIT.md",
        (
            "Gate 1 disposition: `C-REVISE`.",
            "Stage 2 is not authorized.",
        ),
    )

    print(
        "performance research status: D15 systems-performance coverage track "
        "active; gate #1 ratified, validation ladder in progress"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance research status verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
