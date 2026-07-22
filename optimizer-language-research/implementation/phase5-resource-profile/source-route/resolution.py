"""Independent inventory and lexical-resolution diagnostic selection."""

from __future__ import annotations

from dataclasses import dataclass

from model import (
    DiagnosticOrigin,
    ParsedSource,
    Role,
    RouteError,
    SelectedDiagnostic,
)
from roles import PRELUDE_RECORDS, operation_spellings
from topology import ProjectionContext


@dataclass(frozen=True)
class Declaration:
    role: Role | None
    domain: str
    declaration_class: str
    spelling: bytes
    partition: str
    scope_id: int
    visible_source: int | None
    visible_byte: int | None
    prelude_ordinal: int | None = None


ROLE_DECLARATIONS = {
    "D01": (("lexical-ident", "function"),),
    "D02": (
        ("nominal-type", "nominal-type"),
        ("constructor", "struct-constructor"),
    ),
    "D03": (("nominal-type", "nominal-type"),),
    "D04": (("constructor", "enum-variant"),),
    "D05": (("contract", "contract"),),
    "D06": (("lexical-ident", "named-const"),),
    "D07": (("nominal-type", "generic-type"),),
    "D08": (("lexical-ident", "const-generic"),),
    "D09": (("region", "region"),),
    "D10": (("lexical-ident", "value"),),
    "D11": (("lexical-ident", "value"),),
    "D12": (("label", "label"),),
    "D13": (("region", "region"),),
    "D14": (("lexical-ident", "value"),),
}

USE_TARGETS = {
    "U01": (("nominal-type", frozenset({"generic-type", "nominal-type"})),),
    "U02": (("contract", frozenset({"contract"})),),
    "U03": (("contract", frozenset({"contract"})),),
    "U04": (
        ("constructor", frozenset({"struct-constructor", "enum-variant"})),
    ),
    "U05": (("constructor", frozenset({"enum-variant"})),),
    "U06": (("region", frozenset({"region"})),),
    "U07": (("region", frozenset({"region"})),),
    "U08": (("region", frozenset({"region"})),),
    "U09": (("region", frozenset({"region"})),),
    "U10": (("region", frozenset({"region"})),),
    "U11": (("label", frozenset({"label"})),),
    "U12": (
        ("lexical-ident", frozenset({"const-generic", "named-const"})),
    ),
    "U13": (("lexical-ident", frozenset({"named-const"})),),
    "U14": (("lexical-ident", frozenset({"value", "named-const"})),),
    "U17": (("lexical-ident", frozenset({"function"})),),
    "U18": (("nominal-type", frozenset({"generic-type"})),),
}

RESERVATION_ROLES = frozenset(
    {"D01", "D06", "D09", "D10", "D11", "D13", "D14", "X01", "X02"}
)
DOMAIN_ORDER = {
    "lexical-ident": 0,
    "nominal-type": 1,
    "constructor": 2,
    "contract": 3,
    "region": 4,
    "label": 5,
}


def _site_by_path(
    context: ProjectionContext, source_ordinal: int, path: tuple[int, ...]
):
    return next(
        site
        for site in context.sites.values()
        if site.source_ordinal == source_ordinal and site.path == path
    )


def _ancestor_site(
    context: ProjectionContext,
    role: Role,
    kind: str,
):
    path = role.node_path
    while path:
        site = _site_by_path(context, role.source_ordinal, path)
        if site.kind == kind:
            return site
        path = path[:-1]
    return None


def _visibility_start(role: Role, context: ProjectionContext) -> tuple[int | None, int | None]:
    if role.role_id == "D01":
        return None, None
    if role.role_id in {"D02", "D03", "D04", "D05", "D07", "D09", "D12", "D13"}:
        return role.source_ordinal, role.byte_end
    owner_kind = {
        "D06": "const_decl",
        "D08": "gparam",
        "D10": "param",
        "D11": "let_stmt",
        "D14": "fieldbind_list",
    }.get(role.role_id)
    if owner_kind is None:
        return role.source_ordinal, role.byte_end
    site = _ancestor_site(context, role, owner_kind)
    if site is None:
        if role.role_id == "D14":
            return role.source_ordinal, role.byte_end
        raise RouteError(f"cannot locate visibility owner for {role.role_id}")
    return role.source_ordinal, site.byte_end


def _prelude_declarations() -> tuple[Declaration, ...]:
    result: list[Declaration] = []
    for ordinal, (name, declaration_class) in enumerate(PRELUDE_RECORDS):
        domain = {
            "nominal-type": "nominal-type",
            "enum-variant": "constructor",
            "contract": "contract",
        }.get(declaration_class)
        if domain is not None:
            result.append(
                Declaration(
                    None,
                    domain,
                    declaration_class,
                    name.encode("ascii"),
                    "root",
                    0,
                    None,
                    None,
                    ordinal,
                )
            )
    if len(result) != 18:
        raise RouteError("PRE-1 source lookup projection does not contain 18 entries")
    return tuple(result)


def build_declarations(
    roles: tuple[Role, ...], context: ProjectionContext
) -> tuple[Declaration, ...]:
    """Build the source and PRE-1 declaration facts without resolving uses."""

    result = list(_prelude_declarations())
    for role in roles:
        for domain, declaration_class in ROLE_DECLARATIONS.get(role.role_id, ()):
            partition = "root" if role.role_id in {f"D{x:02d}" for x in range(1, 7)} else role.partition_chain[-1]
            visible_source, visible_byte = _visibility_start(role, context)
            scope_id = 0 if role.role_id in {f"D{x:02d}" for x in range(1, 7)} else role.scope_id
            result.append(
                Declaration(
                    role,
                    domain,
                    declaration_class,
                    role.spelling,
                    partition,
                    scope_id,
                    visible_source,
                    visible_byte,
                )
            )
    return tuple(result)


def _source_origin(role: Role, label: str) -> DiagnosticOrigin:
    return DiagnosticOrigin(
        "source",
        label,
        role.source_ordinal,
        role.node_path,
        role.byte_start,
        role.byte_end,
        role.role_ordinal,
        role.subtoken_ordinal,
    )


def _origin(declaration: Declaration) -> DiagnosticOrigin:
    if declaration.role is None:
        return DiagnosticOrigin(
            "prelude",
            declaration.declaration_class,
            prelude_ordinal=declaration.prelude_ordinal,
        )
    return _source_origin(declaration.role, declaration.declaration_class)


def _issue(
    stage: str,
    rule: str,
    reason: str,
    role: Role,
    payload: dict[str, object],
    origins: tuple[DiagnosticOrigin, ...] = (),
) -> SelectedDiagnostic:
    return SelectedDiagnostic(
        stage,
        rule,
        reason,
        role.source_ordinal,
        role.node_path,
        role.byte_start,
        role.byte_end,
        payload,
        origins,
    )


def _scope_contains(context: ProjectionContext, ancestor: int, descendant: int) -> bool:
    current: int | None = descendant
    while current is not None:
        if current == ancestor:
            return True
        current = context.scopes[current].parent
    return False


def _partition_applies(declaration: Declaration, role: Role) -> bool:
    return declaration.partition == "root" or declaration.partition in role.partition_chain


def _started(declaration: Declaration, role: Role) -> bool:
    if declaration.visible_source is None:
        return True
    return (role.source_ordinal, role.byte_start) >= (
        declaration.visible_source,
        declaration.visible_byte,
    )


def _visible(
    declaration: Declaration, role: Role, context: ProjectionContext
) -> bool:
    return (
        _partition_applies(declaration, role)
        and _started(declaration, role)
        and _scope_contains(context, declaration.scope_id, role.scope_id)
    )


def _reservation(role: Role) -> tuple[str, int] | None:
    if role.role_id not in RESERVATION_ROLES:
        return None
    spelling = role.spelling[1:] if role.role_id in {"D09", "D13"} else role.spelling
    operations = operation_spellings()
    if spelling in operations and b"." not in spelling:
        return "dotless-operation", operations.index(spelling)
    modes = (b"wrap", b"trap", b"checked", b"sat", b"strict")
    if spelling in modes:
        return "mode-word", modes.index(spelling)
    return None
