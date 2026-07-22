"""Narrow adapter around the audited independent FORM-2 parser family."""

from __future__ import annotations

import sys
import re
from pathlib import Path

from identities import PARSER_FILES, PARSER_ROOT, verify_identities
from model import LogicalSource, ParsedSource, RouteError


LEFT_ATTACH = frozenset({b"(", b"[", b"<", b"&", b"."})
RIGHT_ATTACH = frozenset({b")", b"]", b">", b",", b";", b".", b":", b"(", b"<"})
PATH_COMPONENT = re.compile(r"[A-Za-z0-9._-]+\Z")


def validate_logical_path(value: str) -> bytes:
    """Enforce the exact portable, normalized PROG-2 transport path shape."""

    if not isinstance(value, str) or not value or not value.isascii():
        raise RouteError("logical path must be nonempty ASCII")
    if value.startswith("/") or value.endswith("/") or "//" in value:
        raise RouteError("logical path must be normalized and relative")
    parts = value.split("/")
    if any(part in {"", ".", ".."} or PATH_COMPONENT.fullmatch(part) is None for part in parts):
        raise RouteError("logical path contains a forbidden component")
    return value.encode("ascii")


def _load_parser() -> tuple[object, object]:
    verify_identities()
    parser_path = str(PARSER_ROOT)
    if parser_path not in sys.path:
        sys.path.insert(0, parser_path)
    from form2_independent_lex import lex_independently
    from form2_independent_parse import parse_independently

    for name in PARSER_FILES:
        module_name = name.removesuffix(".py")
        module = sys.modules.get(module_name)
        module_file = None if module is None else getattr(module, "__file__", None)
        if module_file is None or Path(module_file).resolve() != (PARSER_ROOT / name).resolve():
            raise RouteError(f"audited parser module loaded from the wrong path: {name}")

    return lex_independently, parse_independently


def _canonical_render(tokens: tuple[object, ...], forest: object) -> bytes:
    if not tokens:
        return b"\n"
    item_starts = frozenset(item.start for item in forest.items[1:])
    requires_closes = frozenset(
        node.end - 1
        for node in forest.descendants()
        if node.kind == "requires_block"
    )
    output = bytearray()
    depth = 0
    previous = None
    for index, token in enumerate(tokens):
        if index in item_starts:
            if not output.endswith(b"\n"):
                raise RouteError("independent item did not end at a line boundary")
            output.extend(b"\n")
        if token.raw == b"}":
            depth -= 1
            if depth < 0:
                raise RouteError("canonical render has a closing-brace underflow")
        if not output or output.endswith(b"\n"):
            output.extend(b"  " * depth)
        else:
            if previous is None:
                raise RouteError("canonical render lost its preceding token")
            if previous.raw not in LEFT_ATTACH and token.raw not in RIGHT_ATTACH:
                output.extend(b" ")
        output.extend(token.raw)
        if token.raw == b"{":
            depth += 1
            output.extend(b"\n")
        elif token.raw == b";":
            output.extend(b"\n")
        elif token.raw == b"}" and index not in requires_closes:
            output.extend(b"\n")
        previous = token
    if depth != 0 or not output.endswith(b"\n") or output.endswith(b"\n\n"):
        raise RouteError("independent canonical render is structurally incomplete")
    return bytes(output)


def parse_bundle(sources: tuple[LogicalSource, ...]) -> tuple[ParsedSource, ...]:
    """Parse and independently prove exact FORM-2 bytes for every source."""

    if not sources:
        raise RouteError("SourceBundle must contain at least one source")
    if len({source.logical_path for source in sources}) != len(sources):
        raise RouteError("SourceBundle repeats a logical path")
    lex_independently, parse_independently = _load_parser()
    parsed: list[ParsedSource] = []
    for ordinal, record in enumerate(sources):
        if not isinstance(record.source, bytes):
            raise RouteError("source bytes must be immutable bytes")
        validate_logical_path(record.logical_path)
        try:
            tokens = lex_independently(record.source)
            forest = parse_independently(tokens, len(record.source))
        except Exception as error:
            raise RouteError(
                f"independent parse failed for source {ordinal}: {error}"
            ) from error
        rendered = _canonical_render(tokens, forest)
        if rendered != record.source:
            raise RouteError(f"source {ordinal} is not exact canonical FORM-2")
        parsed.append(
            ParsedSource(
                ordinal,
                record.logical_path,
                record.source,
                tokens,
                forest,
            )
        )
    return tuple(parsed)
