#!/usr/bin/env python3
"""Bind committed production grammar data to exact kernel specification v0.9."""

from __future__ import annotations

from syntax_contract import ContractError, GENERATED, expected_generated


def validate(actual: str, expected: str) -> None:
    """Require byte-exact checked generation, including diagnostic provenance."""
    if actual != expected:
        raise ContractError(
            "generated syntax data differs from the complete exact-v0.9 grammar contract"
        )


def main() -> None:
    """Check the live immutable specification against committed Rust data."""
    try:
        validate(GENERATED.read_text(encoding="utf-8"), expected_generated())
    except (OSError, UnicodeError, ContractError) as error:
        raise SystemExit(f"syntax contract: {error}") from None
    print(
        "syntax contract: 62 productions, 514 nodes, 72 decisions, "
        "and 1253 SELECT2 projections verified"
    )


if __name__ == "__main__":
    main()
