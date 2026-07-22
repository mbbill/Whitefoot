#!/usr/bin/env python3
"""Regenerate the committed exact-v0.9 production grammar data."""

from __future__ import annotations

from syntax_contract import GENERATED, expected_generated


def main() -> None:
    """Write the mechanically derived Rust data in one replacement."""
    GENERATED.write_text(expected_generated(), encoding="utf-8")
    print(f"syntax contract generated: {GENERATED.relative_to(GENERATED.parents[3])}")


if __name__ == "__main__":
    main()
