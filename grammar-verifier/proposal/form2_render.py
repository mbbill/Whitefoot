from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from typing import Any

from form2_topology import (
    GlobalProgramTopology,
    SourceForestTopology,
    bundle_topology_projection_sha256,
    derive_separator_owners,
    separator_owner_projection_sha256,
)
from form2_tree import Derivation, ProductionNode, Terminal


LEFT_ATTACH = frozenset((b"(", b"[", b"<", b"&", b"."))
RIGHT_ATTACH = frozenset((b")", b"]", b">", b",", b";", b".", b":", b"(", b"<"))

BLOCK_PRODUCTIONS = frozenset(
    (
        "arm",
        "conform_decl",
        "contract_decl",
        "enum_decl",
        "fn_decl",
        "loop_stmt",
        "match_stmt",
        "region_stmt",
        "requires_block",
        "struct_decl",
        "value_match",
    )
)

SEMICOLON_PRODUCTIONS = frozenset(
    (
        "break_stmt",
        "check_stmt",
        "const_decl",
        "doc",
        "expr_stmt",
        "field",
        "fn_bind",
        "fn_sig",
        "give_stmt",
        "law",
        "ordinary_let_rhs",
        "return_stmt",
        "set_stmt",
        "try_let_rhs",
        "variant",
    )
)


class RenderError(RuntimeError):
    pass


@dataclass(frozen=True)
class BraceRole:
    production: str
    role: str


@dataclass(frozen=True)
class RenderResult:
    raw: bytes
    structural_counts: dict[str, Any]
    bundle_topology_projection_sha256: str
    separator_owner_projection_sha256: str


def _same_line_gap(left: Terminal, right: Terminal) -> bytes:
    if left.raw in LEFT_ATTACH or right.raw in RIGHT_ATTACH:
        return b""
    return b" "


def _production_index(
    items: tuple[ProductionNode, ...],
) -> tuple[ProductionNode, ...]:
    return tuple(node for item in items for node in item.walk())


def _brace_roles(
    nodes: tuple[ProductionNode, ...], terminals: tuple[Terminal, ...]
) -> dict[int, BraceRole]:
    roles: dict[int, BraceRole] = {}
    for node in nodes:
        if node.name not in BLOCK_PRODUCTIONS:
            continue
        braces = [
            index
            for index in node.direct_tokens
            if terminals[index].raw in (b"{", b"}")
        ]
        if len(braces) != 2:
            raise RenderError(
                f"block production {node.name} has {len(braces)} direct braces"
            )
        opening, closing = braces
        if terminals[opening].raw != b"{" or terminals[closing].raw != b"}":
            raise RenderError(f"block production {node.name} has reversed braces")
        if opening >= closing:
            raise RenderError(f"block production {node.name} has an empty token span")
        for index, role in ((opening, "open"), (closing, "close")):
            if index in roles:
                raise RenderError("one brace belongs directly to two block productions")
            roles[index] = BraceRole(node.name, role)
    observed = {
        terminal.index for terminal in terminals if terminal.raw in (b"{", b"}")
    }
    if set(roles) != observed:
        raise RenderError("not every brace belongs to a listed block production")
    return roles


def _semicolon_owners(
    nodes: tuple[ProductionNode, ...], terminals: tuple[Terminal, ...]
) -> dict[int, str]:
    owners: dict[int, str] = {}
    for node in nodes:
        for index in node.direct_tokens:
            if terminals[index].raw != b";":
                continue
            if node.name not in SEMICOLON_PRODUCTIONS:
                raise RenderError(
                    f"semicolon belongs to unlisted production {node.name}"
                )
            if index in owners:
                raise RenderError("one semicolon belongs directly to two productions")
            owners[index] = node.name
    observed = {terminal.index for terminal in terminals if terminal.raw == b";"}
    if set(owners) != observed:
        raise RenderError("not every semicolon belongs to a line-bearing production")
    return owners


def _top_level_boundaries(items: tuple[ProductionNode, ...]) -> set[int]:
    boundaries: set[int] = set()
    for left, right in zip(items, items[1:]):
        if left.last_token + 1 != right.first_token:
            raise RenderError("top-level items do not cover adjacent token ranges")
        boundaries.add(left.last_token)
    return boundaries


def _requires_body_boundaries(
    nodes: tuple[ProductionNode, ...],
    braces: dict[int, BraceRole],
) -> set[int]:
    boundaries: set[int] = set()
    for node in nodes:
        if node.name != "fn_decl":
            continue
        requires = [child for child in node.children if child.name == "requires_block"]
        if not requires:
            continue
        if len(requires) != 1:
            raise RenderError("function contains more than one requires block")
        closes = [
            index
            for index in range(requires[0].first_token, requires[0].last_token + 1)
            if braces.get(index) == BraceRole("requires_block", "close")
        ]
        body_opens = [
            index
            for index in node.direct_tokens
            if braces.get(index) == BraceRole("fn_decl", "open")
        ]
        if len(closes) != 1 or len(body_opens) != 1 or closes[0] + 1 != body_opens[0]:
            raise RenderError("requires close is not adjacent to the function body open")
        boundaries.add(closes[0])
    return boundaries


def _boundary_gaps(
    derivation: Derivation,
    braces: dict[int, BraceRole],
    semicolons: dict[int, str],
    top_level: set[int],
    requires_body: set[int],
) -> tuple[bytes, ...]:
    gaps: list[bytes] = []
    for index, (left, right) in enumerate(
        zip(derivation.terminals, derivation.terminals[1:])
    ):
        if index in top_level:
            gap = b"\n\n"
        elif index in requires_body:
            gap = b" "
        elif (
            index in semicolons
            or braces.get(index, BraceRole("", "")).role in {"open", "close"}
            or braces.get(index + 1, BraceRole("", "")).role == "close"
        ):
            gap = b"\n"
        else:
            gap = _same_line_gap(left, right)
        gaps.append(gap)
    return tuple(gaps)


def render_derivation(derivation: Derivation) -> RenderResult:
    terminals = derivation.terminals
    source = SourceForestTopology(
        source_ordinal=0,
        source_length=derivation.source_length,
        terminal_count=len(terminals),
        items=derivation.items,
    )
    topology = GlobalProgramTopology((source,))
    separator_owners = derive_separator_owners(topology, 0)
    if len(separator_owners) != len(terminals) + 1:
        raise RenderError("separator ownership does not cover every boundary")
    topology_sha256 = bundle_topology_projection_sha256(topology)
    owner_sha256 = separator_owner_projection_sha256(topology, 0)
    nodes = _production_index(derivation.items)
    production_counts = Counter(node.name for node in nodes)
    if not terminals:
        if derivation.items:
            raise RenderError("tokenless source forest contains an item")
        return RenderResult(
            b"\n",
            {
                "block_production_counts": {},
                "empty_block_count": 0,
                "line_bearing_production_counts": {},
                "physical_line_count": 1,
                "production_counts": dict(sorted(production_counts.items())),
                "requires_body_transition_count": 0,
                "top_level_item_count": 0,
            },
            topology_sha256,
            owner_sha256,
        )

    braces = _brace_roles(nodes, terminals)
    semicolons = _semicolon_owners(nodes, terminals)
    top_level = _top_level_boundaries(derivation.items)
    requires_body = _requires_body_boundaries(nodes, braces)
    gaps = _boundary_gaps(
        derivation, braces, semicolons, top_level, requires_body
    )

    output = bytearray()
    depth = 0
    line_start = True
    for index, terminal in enumerate(terminals):
        role = braces.get(index)
        if line_start:
            if role is not None and role.role == "close":
                depth -= 1
                if depth < 0:
                    raise RenderError("closing brace reduces indentation below zero")
            output.extend(b"  " * depth)
            line_start = False
        elif role is not None and role.role == "close":
            raise RenderError("a closing brace is not at the start of its line")

        output.extend(terminal.raw)
        if role is not None and role.role == "open":
            depth += 1

        if index < len(gaps):
            gap = gaps[index]
            output.extend(gap)
            line_start = gap.endswith(b"\n")

    if depth != 0:
        raise RenderError("rendering ends with unclosed block indentation")
    output.extend(b"\n")
    raw = bytes(output)
    if b"\r" in raw or b"\t" in raw or any(
        line.endswith(b" ") for line in raw.split(b"\n")
    ):
        raise RenderError("renderer emitted forbidden horizontal trivia")

    block_counts = Counter(role.production for role in braces.values() if role.role == "open")
    line_counts = Counter(semicolons.values())
    empty_blocks = sum(
        1
        for index, role in braces.items()
        if role.role == "open"
        and braces.get(index + 1) is not None
        and braces[index + 1].role == "close"
    )
    return RenderResult(
        raw,
        {
            "block_production_counts": dict(sorted(block_counts.items())),
            "empty_block_count": empty_blocks,
            "line_bearing_production_counts": dict(sorted(line_counts.items())),
            "physical_line_count": raw.count(b"\n"),
            "production_counts": dict(sorted(production_counts.items())),
            "requires_body_transition_count": len(requires_body),
            "top_level_item_count": len(derivation.items),
        },
        topology_sha256,
        owner_sha256,
    )


def inject_isolated_fixture_defect(
    identifier: str,
    canonical: bytes,
) -> tuple[bytes, dict[str, Any]]:
    marker = b"\n  let "
    location = canonical.find(marker)
    if location < 0 or canonical.find(marker, location + 1) >= 0:
        raise RenderError(f"FORM-2 fixture {identifier} lacks one unique let line")
    indentation_start = location + 1
    if identifier == "form2-neg-noncanonical-ws":
        replacement = b"    "
        kind = "four-space-indentation-at-depth-one"
    elif identifier == "x-form-form2-tab-indent":
        replacement = b"\t"
        kind = "tab-indentation-at-depth-one"
    else:
        raise RenderError(f"unknown FORM-2 fixture: {identifier}")
    migrated = canonical[:indentation_start] + replacement + canonical[indentation_start + 2 :]
    return migrated, {
        "actual_hex": replacement.hex(),
        "byte_end": indentation_start + len(replacement),
        "byte_start": indentation_start,
        "canonical_hex": b"  ".hex(),
        "kind": kind,
        "line": canonical[:indentation_start].count(b"\n") + 1,
    }
