#!/usr/bin/env python3
"""Bind the production fixed-terminal table to the exact v0.9 grammar."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT.parent / "spec" / "kernel-spec-v0.9.md"
RUST = ROOT / "crates" / "whitefoot-language-data" / "src" / "terminal.rs"
SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
FIXED_ATOM = re.compile(rb'"([^"\n]+)"')
ALL_FIXED_BODY = re.compile(
    r"pub const ALL_FIXED_TERMINALS_V0_9:[^=]+=\s*\[(.*?)\];",
    re.DOTALL,
)
ALL_PREDICATES_BODY = re.compile(
    r"pub const ALL_TERMINAL_PREDICATES_V0_9:[^=]+=\s*\[(.*?)\];",
    re.DOTALL,
)
FIXED_VARIANT = re.compile(r"FixedTerminalV0_9::([A-Za-z][A-Za-z0-9_]*)")
SPELLING_ARM = re.compile(r'Self::([A-Za-z][A-Za-z0-9_]*)\s*=>\s*b"([^"\\]*)"')
EXTERNAL_VARIANTS = (
    "Identifier",
    "TypeIdentifier",
    "RegionIdentifier",
    "Label",
    "OperationName",
    "Literal",
    "String",
    "Digits",
)


class ContractError(ValueError):
    """A controlled fixed-terminal contract mismatch."""


def expand_fixed_atom(atom: bytes) -> tuple[bytes, ...]:
    """Lex one quoted grammar atom into its raw fixed-token spellings."""
    if atom == b"[0-9]+":
        return ()
    output: list[bytes] = []
    cursor = 0
    while cursor < len(atom):
        if atom[cursor : cursor + 2] in (b"->", b"=>"):
            output.append(atom[cursor : cursor + 2])
            cursor += 2
            continue
        byte = atom[cursor]
        if 0x61 <= byte <= 0x7A:
            end = cursor + 1
            while end < len(atom) and (
                0x61 <= atom[end] <= 0x7A
                or 0x30 <= atom[end] <= 0x39
                or atom[end] == 0x5F
            ):
                end += 1
            output.append(atom[cursor:end])
            cursor = end
            continue
        if byte in b"(){}[]<>,:;.=&#":
            output.append(atom[cursor : cursor + 1])
            cursor += 1
            continue
        raise ContractError(f"quoted grammar atom has an unknown raw shape: {atom!r}")
    return tuple(output)


def fixed_from_spec(specification: bytes) -> tuple[bytes, ...]:
    """Derive unique raw fixed predicates in first grammar occurrence order."""
    if b"## 3. Grammar" not in specification:
        raise ContractError("cannot locate the grammar section")
    seen: set[bytes] = set()
    output: list[bytes] = []
    events: list[tuple[int, bytes]] = []
    fences = list(
        re.finditer(
            rb"\x60\x60\x60[^\n]*\n(.*?)\x60\x60\x60",
            specification,
            re.DOTALL,
        )
    )
    for fence in fences:
        content = fence.group(1)
        if b":=" not in content:
            continue
        for atom in FIXED_ATOM.finditer(content):
            events.append((fence.start(1) + atom.start(), atom.group(1)))
    for line in re.finditer(rb"^.*:=.*$", specification, re.MULTILINE):
        if any(fence.start() <= line.start() < fence.end() for fence in fences):
            continue
        for atom in FIXED_ATOM.finditer(line.group(0)):
            events.append((line.start() + atom.start(), atom.group(1)))
    for _, atom in sorted(events):
        for spelling in expand_fixed_atom(atom):
            if spelling not in seen:
                seen.add(spelling)
                output.append(spelling)
    if not output:
        raise ContractError("grammar contains no fixed predicates")
    return tuple(output)


def one_body(pattern: re.Pattern[str], rust: str, label: str) -> str:
    """Extract one closed Rust constant body."""
    matches = pattern.findall(rust)
    if len(matches) != 1:
        raise ContractError(f"Rust source must contain exactly one {label}")
    return matches[0]


def fixed_from_rust(rust: str) -> tuple[bytes, ...]:
    """Recover the checked Rust table through its order and spelling arms."""
    body = one_body(ALL_FIXED_BODY, rust, "fixed-terminal table")
    order = FIXED_VARIANT.findall(body)
    arms = SPELLING_ARM.findall(rust)
    spelling_by_variant: dict[str, bytes] = {}
    for variant, spelling in arms:
        if variant in spelling_by_variant:
            raise ContractError(f"duplicate spelling arm for {variant}")
        spelling_by_variant[variant] = spelling.encode("ascii")
    if set(order) != set(spelling_by_variant):
        raise ContractError("fixed table variants and spelling arms differ")
    if len(order) != len(set(order)):
        raise ContractError("fixed table repeats a variant")
    return tuple(spelling_by_variant[variant] for variant in order)


def validate(specification: bytes, rust: str) -> None:
    """Require exact grammar/table agreement and one complete predicate universe."""
    expected = fixed_from_spec(specification)
    actual = fixed_from_rust(rust)
    if actual != expected:
        raise ContractError(
            f"fixed-terminal table differs from grammar: expected {expected!r}, got {actual!r}"
        )
    body = one_body(ALL_PREDICATES_BODY, rust, "terminal-predicate table")
    fixed_order = FIXED_VARIANT.findall(body)
    fixed_body = one_body(ALL_FIXED_BODY, rust, "fixed-terminal table")
    if tuple(fixed_order) != tuple(FIXED_VARIANT.findall(fixed_body)):
        raise ContractError("predicate table does not retain the fixed-table order")
    for variant in EXTERNAL_VARIANTS:
        occurrence = f"TerminalPredicateV0_9::{variant},"
        if body.count(occurrence) != 1:
            raise ContractError(f"predicate table must contain exactly one {variant}")
    if len(fixed_order) + len(EXTERNAL_VARIANTS) != 72:
        raise ContractError("v0.9 terminal-predicate count is not 72")


def main() -> None:
    """Check the live exact specification and production table."""
    specification = SPEC.read_bytes()
    digest = hashlib.sha256(specification).hexdigest()
    if digest != SPEC_SHA256:
        raise SystemExit(f"terminal contract: v0.9 identity drifted to {digest}")
    try:
        validate(specification, RUST.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, ContractError) as error:
        raise SystemExit(f"terminal contract: {error}") from None
    print("terminal contract: 64 fixed and 72 total v0.9 predicates verified")


if __name__ == "__main__":
    main()
