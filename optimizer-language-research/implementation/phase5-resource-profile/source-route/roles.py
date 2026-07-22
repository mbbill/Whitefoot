"""Independent projection of the approved closed grammar-role matrix."""

from __future__ import annotations

from dataclasses import dataclass
import re

from identities import PROPOSAL_ROOT
from model import ParsedSource, Role, RouteError
from topology import ProjectionContext


DECLARATION_ROLES = frozenset(
    {f"D{index:02d}" for index in range(1, 15)}
    | {"X01", "X02", "X03"}
)
LEXICAL_USE_ROLES = frozenset(f"U{index:02d}" for index in range(1, 19))
DEFERRED_USE_ROLES = frozenset(f"X{index:02d}" for index in range(4, 10))
ALL_ROLES = DECLARATION_ROLES | LEXICAL_USE_ROLES | DEFERRED_USE_ROLES
ROLE_CLASS_ORDER = {"declaration": 0, "lexical-use": 1, "deferred-use": 2}
GENERIC_LITERAL = re.compile(rb"[01]_([A-Z][A-Za-z0-9]*)\Z")
PARSER_FIXED_COMPATIBILITY = frozenset({b"enum", b"let"})


@dataclass(frozen=True)
class _Candidate:
    role_id: str
    role_class: str
    token_index: int
    byte_start: int
    byte_end: int
    spelling: bytes
    subtoken_ordinal: int = 0


def _direct(node: object) -> tuple[int, ...]:
    return node.direct_token_indices()


def _of_kind(
    source: ParsedSource, node: object, kind: str
) -> tuple[tuple[int, object], ...]:
    return tuple(
        (index, source.tokens[index])
        for index in _direct(node)
        if source.tokens[index].kind == kind
    )


def _add(
    output: list[_Candidate],
    role_id: str,
    role_class: str,
    index: int,
    token: object,
) -> None:
    output.append(
        _Candidate(
            role_id,
            role_class,
            index,
            token.start,
            token.end,
            token.raw,
        )
    )


def _role_candidates(source: ParsedSource, node: object) -> list[_Candidate]:
    result: list[_Candidate] = []
    kind = node.kind

    declarations = {
        "fn_decl": ("D01", "IDENT"),
        "struct_decl": ("D02", "TYPEID"),
        "enum_decl": ("D03", "TYPEID"),
        "variant": ("D04", "TYPEID"),
        "contract_decl": ("D05", "TYPEID"),
        "const_decl": ("D06", "IDENT"),
        "param": ("D10", "IDENT"),
        "let_stmt": ("D11", "IDENT"),
        "loop_stmt": ("D12", "LABEL"),
        "region_stmt": ("D13", "REGIONID"),
        "field": ("X01", "IDENT"),
        "vfield": ("X02", "IDENT"),
        "fn_sig": ("X03", "IDENT"),
    }
    if kind in declarations:
        role_id, token_kind = declarations[kind]
        matches = tuple(
            match
            for match in _of_kind(source, node, token_kind)
            if match[1].raw not in PARSER_FIXED_COMPATIBILITY
        )
        if not matches:
            raise RouteError(f"{kind} lacks its required declaration carrier")
        index, token = matches[0]
        _add(result, role_id, "declaration", index, token)

    if kind == "gparam":
        type_ids = _of_kind(source, node, "TYPEID")
        identifiers = _of_kind(source, node, "IDENT")
        if type_ids:
            index, token = type_ids[0]
            _add(result, "D07", "declaration", index, token)
            if len(type_ids) == 2:
                index, token = type_ids[1]
                _add(result, "U02", "lexical-use", index, token)
            elif len(type_ids) != 1:
                raise RouteError("type gparam has an unexpected TYPEID count")
        elif identifiers:
            if len(identifiers) != 1:
                raise RouteError("const gparam has an unexpected IDENT count")
            index, token = identifiers[0]
            _add(result, "D08", "declaration", index, token)
        else:
            raise RouteError("gparam has no declared name")

    if kind == "region_params":
        for index, token in _of_kind(source, node, "REGIONID"):
            _add(result, "D09", "declaration", index, token)

    if kind == "fieldbind":
        identifiers = _of_kind(source, node, "IDENT")
        if len(identifiers) != 2:
            raise RouteError("fieldbind does not contain exactly two IDENT carriers")
        _add(result, "X05", "deferred-use", *identifiers[0])
        _add(result, "D14", "declaration", *identifiers[1])

    lexical_single = {
        "construct": ("U04", "TYPEID"),
        "arm": ("U05", "TYPEID"),
        "borrow_expr": ("U10", "REGIONID"),
        "break_stmt": ("U11", "LABEL"),
        "const": ("U12", "IDENT"),
        "cvalue": ("U13", "IDENT"),
        "pbase": ("U14", "IDENT"),
        "fieldinit": ("X04", "IDENT"),
        "psuffix": ("X06", "IDENT"),
    }
    if kind in lexical_single:
        role_id, token_kind = lexical_single[kind]
        matches = _of_kind(source, node, token_kind)
        if matches:
            role_class = (
                "lexical-use" if role_id.startswith("U") else "deferred-use"
            )
            _add(result, role_id, role_class, *matches[0])

    if kind == "type":
        for index, token in _of_kind(source, node, "TYPEID"):
            _add(result, "U01", "lexical-use", index, token)
        for index, token in _of_kind(source, node, "REGIONID"):
            _add(result, "U06", "lexical-use", index, token)
    elif kind == "mode":
        for index, token in _of_kind(source, node, "REGIONID"):
            _add(result, "U07", "lexical-use", index, token)
    elif kind == "targ":
        for index, token in _of_kind(source, node, "REGIONID"):
            _add(result, "U08", "lexical-use", index, token)
    elif kind == "effect":
        for index, token in _of_kind(source, node, "REGIONID"):
            _add(result, "U09", "lexical-use", index, token)
    elif kind == "conform_decl":
        for index, token in _of_kind(source, node, "TYPEID"):
            _add(result, "U03", "lexical-use", index, token)
    elif kind == "callee":
        for index, token in _of_kind(source, node, "IDENT"):
            _add(result, "U15", "lexical-use", index, token)
        for index, token in _of_kind(source, node, "OPNAME"):
            _add(result, "U16", "lexical-use", index, token)
    elif kind == "fn_bind":
        identifiers = _of_kind(source, node, "IDENT")
        if len(identifiers) != 2:
            raise RouteError("fn_bind does not contain exactly two IDENT carriers")
        _add(result, "X07", "deferred-use", *identifiers[0])
        _add(result, "U17", "lexical-use", *identifiers[1])
    elif kind == "law":
        identifiers = _of_kind(source, node, "IDENT")
        if not identifiers:
            raise RouteError("law does not contain its name carrier")
        _add(result, "X08", "deferred-use", *identifiers[0])
    elif kind == "law_arg":
        direct = _direct(node)
        if len(direct) != 1:
            raise RouteError("law_arg does not contain one complete carrier")
        index = direct[0]
        _add(result, "X09", "deferred-use", index, source.tokens[index])

    for index in _direct(node):
        token = source.tokens[index]
        match = GENERIC_LITERAL.fullmatch(token.raw)
        if match is None:
            continue
        suffix_start = token.start + 2
        result.append(
            _Candidate(
                "U18",
                "lexical-use",
                index,
                suffix_start,
                token.end,
                match.group(1),
                1 if kind == "law_arg" else 0,
            )
        )
    return result


def project_roles(
    parsed: tuple[ParsedSource, ...], context: ProjectionContext
) -> tuple[Role, ...]:
    """Project every matrix occurrence and fail on any unclassified name carrier."""

    projected: list[Role] = []
    for source in parsed:
        for node in source.forest.descendants():
            candidates = _role_candidates(source, node)
            covered = {candidate.token_index for candidate in candidates}
            for index in _direct(node):
                token = source.tokens[index]
                if token.kind in {"IDENT", "TYPEID", "REGIONID", "LABEL", "OPNAME"}:
                    if index not in covered and token.raw not in PARSER_FIXED_COMPATIBILITY:
                        raise RouteError(
                            f"unclassified name-shaped carrier in {node.kind}: {token.raw!r}"
                        )
            grouped = sorted(
                {candidate.token_index for candidate in candidates},
                key=lambda index: (source.tokens[index].start, source.tokens[index].end),
            )
            ordinals = {index: ordinal for ordinal, index in enumerate(grouped)}
            candidates.sort(
                key=lambda candidate: (
                    source.tokens[candidate.token_index].start,
                    source.tokens[candidate.token_index].end,
                    candidate.subtoken_ordinal,
                    ROLE_CLASS_ORDER[candidate.role_class],
                    candidate.role_id,
                )
            )
            site = context.sites[id(node)]
            for candidate in candidates:
                if candidate.role_id not in ALL_ROLES:
                    raise RouteError(f"projected an unknown role: {candidate.role_id}")
                projected.append(
                    Role(
                        candidate.role_id,
                        candidate.role_class,
                        node.kind,
                        source.ordinal,
                        site.path,
                        ordinals[candidate.token_index],
                        candidate.subtoken_ordinal,
                        candidate.token_index,
                        candidate.byte_start,
                        candidate.byte_end,
                        candidate.spelling,
                        context.node_scope[id(node)],
                        context.node_partitions[id(node)],
                        context.node_functions[id(node)],
                        context.node_arms[id(node)],
                    )
                )
    ordered = tuple(sorted(projected, key=lambda role: role.event_key))
    if len({(role.event_key, role.role_id) for role in ordered}) != len(ordered):
        raise RouteError("role projection duplicated one matrix occurrence")
    return ordered


PRELUDE_RECORDS = (
    ("Bool", "nominal-type"),
    ("True", "enum-variant"),
    ("False", "enum-variant"),
    ("Option", "nominal-type"),
    ("T", "owner-generic-type"),
    ("None", "enum-variant"),
    ("Some", "enum-variant"),
    ("value", "owner-field"),
    ("Result", "nominal-type"),
    ("T", "owner-generic-type"),
    ("E", "owner-generic-type"),
    ("Ok", "enum-variant"),
    ("value", "owner-field"),
    ("Err", "enum-variant"),
    ("error", "owner-field"),
    ("Overflow", "nominal-type"),
    ("Overflow", "enum-variant"),
    ("DivError", "nominal-type"),
    ("DivideByZero", "enum-variant"),
    ("DivOverflow", "enum-variant"),
    ("NarrowError", "nominal-type"),
    ("NarrowError", "enum-variant"),
    ("Int", "contract"),
    ("Float", "contract"),
)


def operation_spellings() -> tuple[bytes, ...]:
    """Independently extract OP-1 families in first table-occurrence order."""

    raw = (PROPOSAL_ROOT / "kernel-spec-v0.10-candidate.md").read_bytes()
    start = raw.index(b"[OP-1] Every computation")
    end = raw.index(b"Let `DotlessOperationNames`", start)
    found: list[bytes] = []
    seen: set[bytes] = set()
    for line in raw[start:end].splitlines():
        if not line.startswith(b"| `"):
            continue
        cell = line.split(b"|", 2)[1]
        for spelling in re.findall(rb"`([a-z][a-z0-9_]*(?:\.[a-z]+)?)`", cell):
            if spelling not in seen:
                seen.add(spelling)
                found.append(spelling)
    if len(found) != 83:
        raise RouteError(f"OP-1 extraction produced {len(found)} rather than 83 families")
    return tuple(found)


def fixed_spelling_charge() -> tuple[int, dict[str, int]]:
    """Return the four exact R-04 built-in spelling components."""

    prelude = sum(len(name.encode("ascii")) for name, _ in PRELUDE_RECORDS)
    operations = operation_spellings()
    operation_bytes = sum(map(len, operations))
    dotless = tuple(spelling for spelling in operations if b"." not in spelling)
    if len(dotless) != 51:
        raise RouteError("derived dotless-operation inventory does not contain 51 records")
    reservation = sum(map(len, dotless))
    mode = sum(map(len, (b"wrap", b"trap", b"checked", b"sat", b"strict")))
    components = {
        "prelude": prelude,
        "operation_families": operation_bytes,
        "dotless_reservations": reservation,
        "mode_words": mode,
    }
    return sum(components.values()), components
