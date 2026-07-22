#!/usr/bin/env python3
"""Independently bind the v0.9 FORM-2 policy to the source audit."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT.parent / "spec" / "kernel-spec-v0.9.md"
RUST = (
    ROOT
    / "crates"
    / "whitefoot-syntax"
    / "src"
    / "parser"
    / "finalize"
    / "canonical.rs"
)
FORMAT_RUST = RUST.with_name("canonical") / "format.rs"
SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
PRODUCTION = re.compile(r"ProductionV0_9::([A-Za-z][A-Za-z0-9_]*)")
FIXED = re.compile(r"FixedTerminalV0_9::([A-Za-z][A-Za-z0-9_]*)")
BACKTICK = re.compile(r"`([^`]+)`")
PUNCTUATION_VARIANT = {
    "(": "LeftParen",
    ")": "RightParen",
    "[": "LeftBracket",
    "]": "RightBracket",
    "<": "LeftAngle",
    ">": "RightAngle",
    "&": "Ampersand",
    ".": "Dot",
    ",": "Comma",
    ";": "Semicolon",
    ":": "Colon",
}


class ContractError(ValueError):
    """A controlled specification/audit mismatch."""


def snake_to_variant(name: str) -> str:
    """Map one normative production spelling to its generated Rust variant."""
    if re.fullmatch(r"[a-z][a-z0-9_]*", name) is None:
        raise ContractError(f"invalid production spelling in FORM-2: {name!r}")
    return "".join(part.capitalize() for part in name.split("_"))


def form2_text(specification: str) -> str:
    """Extract the one exact FORM-2 rule body."""
    start = specification.find("[FORM-2]")
    end = specification.find("\n[FORM-3]", start)
    if start < 0 or end < 0:
        raise ContractError("cannot isolate FORM-2")
    return specification[start:end]


def sentence_between(text: str, start: str, end: str) -> str:
    """Extract one uniquely marked FORM-2 sentence fragment."""
    first = text.find(start)
    if first < 0:
        raise ContractError(f"missing FORM-2 marker: {start}")
    last = text.find(end, first + len(start))
    if last < 0:
        raise ContractError(f"missing FORM-2 marker: {end}")
    return text[first + len(start) : last]


def function_body(rust: str, name: str) -> str:
    """Extract one Rust function body with balanced braces."""
    matches = list(re.finditer(rf"\bfn\s+{re.escape(name)}\s*\(", rust))
    if len(matches) != 1:
        raise ContractError(f"Rust source must contain exactly one {name} function")
    opening = rust.find("{", matches[0].end())
    if opening < 0:
        raise ContractError(f"{name} has no body")
    depth = 0
    for index in range(opening, len(rust)):
        if rust[index] == "{":
            depth += 1
        elif rust[index] == "}":
            depth -= 1
            if depth == 0:
                return rust[opening + 1 : index]
    raise ContractError(f"{name} has an unterminated body")


def production_sets(form2: str) -> tuple[set[str], set[str]]:
    """Derive the exact line- and block-bearing production sets."""
    line_text = sentence_between(
        form2,
        "The line-bearing simple productions are ",
        ". Each renders completely on one line",
    )
    block_text = sentence_between(
        form2,
        "The block-bearing productions are ",
        ". Their introducer through",
    )
    simple_text, separator, conditional_text = line_text.partition(", plus a ")
    if not separator:
        raise ContractError("FORM-2 conditional let_stmt clause is missing")
    line_names = set(BACKTICK.findall(simple_text))
    conditional_names = BACKTICK.findall(conditional_text)
    if conditional_names != ["let_stmt", "ordinary_let_rhs", "try_let_rhs"]:
        raise ContractError("FORM-2 conditional let_stmt clause differs")
    line_names.add("let_stmt")
    block_names = set(BACKTICK.findall(block_text))
    if "let_stmt" not in line_names or "fn_decl" not in block_names:
        raise ContractError("FORM-2 conditional line or function-body rule is missing")
    return (
        {snake_to_variant(name) for name in line_names},
        {snake_to_variant(name) for name in block_names},
    )


def attachment_sets(form2: str) -> tuple[set[str], set[str]]:
    """Derive the exact fixed-terminal attachment sets."""
    left_text = sentence_between(
        form2,
        "The left-attachment set contains ",
        ". The right-attachment set contains ",
    )
    right_text = sentence_between(
        form2,
        ". The right-attachment set contains ",
        ". Between two consecutive terminals",
    )
    try:
        left = {PUNCTUATION_VARIANT[value] for value in BACKTICK.findall(left_text)}
        right = {PUNCTUATION_VARIANT[value] for value in BACKTICK.findall(right_text)}
    except KeyError as error:
        raise ContractError(f"unknown FORM-2 attachment token: {error.args[0]!r}") from None
    return left, right


def validate(specification: str, rust: str) -> None:
    """Require exact FORM-2 policy coverage and the streaming audit seams."""
    form2 = form2_text(specification)
    expected_lines, expected_blocks = production_sets(form2)
    line_body = function_body(rust, "is_line_bearing")
    block_body = function_body(rust, "is_block_bearing")

    fixed_line_body = line_body.split("if fixed", 1)[0]
    actual_fixed_lines = set(PRODUCTION.findall(fixed_line_body))
    expected_fixed_lines = expected_lines - {"LetStmt"}
    if actual_fixed_lines != expected_fixed_lines:
        raise ContractError(
            "line-bearing production set differs: "
            f"expected {sorted(expected_fixed_lines)!r}, got {sorted(actual_fixed_lines)!r}"
        )
    required_conditional = {"LetStmt", "OrdinaryLetRhs", "TryLetRhs"}
    if set(PRODUCTION.findall(line_body)) - actual_fixed_lines != required_conditional:
        raise ContractError("conditional let_stmt FORM-2 policy differs")

    actual_blocks = set(PRODUCTION.findall(block_body))
    if actual_blocks != expected_blocks:
        raise ContractError(
            "block-bearing production set differs: "
            f"expected {sorted(expected_blocks)!r}, got {sorted(actual_blocks)!r}"
        )

    expected_left, expected_right = attachment_sets(form2)
    actual_left = set(FIXED.findall(function_body(rust, "left_attaches")))
    actual_right = set(FIXED.findall(function_body(rust, "right_attaches")))
    if actual_left != expected_left:
        raise ContractError("left-attachment set differs from FORM-2")
    if actual_right != expected_right:
        raise ContractError("right-attachment set differs from FORM-2")

    required_fragments = (
        "fixed.spelling()",
        "token.span().bytes()",
        "source_bundle().iter()",
        "core::iter::once(b'\\n')",
        "GapStyle::Blank",
        "lowest_common_ancestor",
        "CanonicalLocation::SourceBytes",
        "CanonicalLocation::SourceNode",
    )
    for fragment in required_fragments:
        if fragment not in rust:
            raise ContractError(f"canonical audit is missing required seam: {fragment}")
    if "Vec<u8>" in rust:
        raise ContractError("canonical audit must not allocate a duplicate rendered byte buffer")


def main() -> None:
    """Check the live exact specification and canonical-source audit."""
    specification_bytes = SPEC.read_bytes()
    digest = hashlib.sha256(specification_bytes).hexdigest()
    if digest != SPEC_SHA256:
        raise SystemExit(f"canonical contract: v0.9 identity drifted to {digest}")
    try:
        specification = specification_bytes.decode("utf-8")
        rust = RUST.read_text(encoding="utf-8") + FORMAT_RUST.read_text(encoding="utf-8")
        validate(specification, rust)
    except (ContractError, OSError, UnicodeError) as error:
        raise SystemExit(f"canonical contract: {error}") from None
    print(
        "canonical contract: exact FORM-2 line/block/attachment policy, "
        "tree locations, and streaming comparison are bound"
    )


if __name__ == "__main__":
    main()
