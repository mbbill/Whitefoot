from __future__ import annotations

from dataclasses import dataclass

from form2_independent_lex import IndependentToken
from form2_independent_syntax import IndependentForest
from form2_independent_topology import (
    IndependentGlobalTopology,
    IndependentSourceTopology,
    bundle_topology_projection_sha256,
    derive_separator_owners,
    separator_owner_projection_sha256,
)


LEFT_ATTACH = frozenset({b"(", b"[", b"<", b"&", b"."})
RIGHT_ATTACH = frozenset({b")", b"]", b">", b",", b";", b".", b":", b"(", b"<"})
FORM2_NEGATIVE = "form2-neg-noncanonical-ws"
FORM2_TAB_NEGATIVE = "x-form-form2-tab-indent"


class IndependentRenderError(RuntimeError):
    pass


@dataclass(frozen=True)
class IndependentRendering:
    canonical: bytes
    migration: bytes
    intentional_defect: dict[str, int | str] | None
    bundle_topology_projection_sha256: str
    separator_owner_projection_sha256: str


def horizontal_boundary(left: bytes, right: bytes) -> bytes:
    if left in LEFT_ATTACH or right in RIGHT_ATTACH:
        return b""
    return b" "


def _structural_indexes(
    forest: IndependentForest,
) -> tuple[frozenset[int], frozenset[int]]:
    item_starts = frozenset(item.start for item in forest.items[1:])
    requires_closes = frozenset(
        node.end - 1
        for node in forest.descendants()
        if node.kind == "requires_block"
    )
    return item_starts, requires_closes


def render_canonical(
    tokens: tuple[IndependentToken, ...], forest: IndependentForest
) -> bytes:
    if forest.terminal_count != len(tokens):
        raise IndependentRenderError(
            "source forest does not cover the complete terminal stream"
        )
    if not tokens:
        return b"\n"

    item_starts, requires_closes = _structural_indexes(forest)
    output = bytearray()
    depth = 0
    previous: IndependentToken | None = None
    for index, token in enumerate(tokens):
        if index in item_starts:
            if not output.endswith(b"\n"):
                raise IndependentRenderError("top-level item did not end at a line boundary")
            output.extend(b"\n")

        if token.raw == b"}":
            if depth == 0:
                raise IndependentRenderError("closing brace underflow")
            depth -= 1

        at_line_start = not output or output.endswith(b"\n")
        if at_line_start:
            output.extend(b"  " * depth)
        else:
            if previous is None:
                raise IndependentRenderError("missing preceding terminal")
            output.extend(horizontal_boundary(previous.raw, token.raw))
        output.extend(token.raw)

        if token.raw == b"{":
            depth += 1
            output.extend(b"\n")
        elif token.raw == b";":
            output.extend(b"\n")
        elif token.raw == b"}" and index not in requires_closes:
            output.extend(b"\n")
        previous = token

    if depth != 0:
        raise IndependentRenderError("render finished with unclosed brace depth")
    if not output.endswith(b"\n") or output.endswith(b"\n\n"):
        raise IndependentRenderError("render did not produce exactly one final LF")
    return bytes(output)


def _body_indent_site(canonical: bytes) -> int:
    marker = b"\n  let "
    if canonical.count(marker) != 1:
        raise IndependentRenderError(
            "FORM-2 negative does not have one unique first-let indentation site"
        )
    return canonical.index(marker) + 1


def apply_intentional_defect(
    canonical: bytes, identifier: str | None
) -> tuple[bytes, dict[str, int | str] | None]:
    if identifier not in (FORM2_NEGATIVE, FORM2_TAB_NEGATIVE):
        return canonical, None
    start = _body_indent_site(canonical)
    if canonical[start : start + 2] != b"  ":
        raise IndependentRenderError("intentional defect does not bind canonical indentation")
    if identifier == FORM2_NEGATIVE:
        replacement = b"    "
        kind = "first-body-line-four-space-indentation"
    else:
        replacement = b"\t"
        kind = "first-body-line-tab-indentation"
    migrated = canonical[:start] + replacement + canonical[start + 2 :]
    return migrated, {
        "byte_end": start + len(replacement),
        "byte_start": start,
        "kind": kind,
    }


def render_for_migration(
    tokens: tuple[IndependentToken, ...],
    forest: IndependentForest,
    identifier: str | None,
) -> IndependentRendering:
    canonical = render_canonical(tokens, forest)
    topology = IndependentGlobalTopology(
        (IndependentSourceTopology(0, forest),)
    )
    owners = derive_separator_owners(topology, 0)
    if len(owners) != len(tokens) + 1:
        raise IndependentRenderError(
            "separator ownership does not cover every boundary"
        )
    migration, defect = apply_intentional_defect(canonical, identifier)
    return IndependentRendering(
        canonical,
        migration,
        defect,
        bundle_topology_projection_sha256(topology),
        separator_owner_projection_sha256(topology, 0),
    )
