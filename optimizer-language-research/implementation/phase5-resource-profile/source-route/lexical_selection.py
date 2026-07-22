"""Lexical-use diagnostic selection after a complete inventory."""

from __future__ import annotations

from model import Role, RouteError, SelectedDiagnostic
from roles import operation_spellings
from topology import ProjectionContext
from resolution import (
    USE_TARGETS,
    Declaration,
    _issue,
    _origin,
    _partition_applies,
    _scope_contains,
    _visible,
)

def _use_targets(role: Role) -> tuple[tuple[str, frozenset[str]], ...]:
    if role.role_id == "U15":
        operations = operation_spellings()
        if role.spelling in operations and b"." not in role.spelling:
            return (("operation", frozenset({"operation-family"})),)
        return (("lexical-ident", frozenset({"function"})),)
    if role.role_id == "U16":
        return (("operation", frozenset({"operation-family"})),)
    return USE_TARGETS[role.role_id]


def select_resolution_issue(
    roles: tuple[Role, ...],
    declarations: tuple[Declaration, ...],
    context: ProjectionContext,
) -> SelectedDiagnostic | None:
    operations = operation_spellings()
    for role in (role for role in roles if role.role_class == "lexical-use"):
        targets = _use_targets(role)
        if targets[0][0] == "operation":
            if role.spelling in operations:
                continue
            return _issue(
                "lexical-resolution",
                "OP-1",
                "operation-family-absent",
                role,
                {"spelling_hex": role.spelling.hex(), "available_classes": []},
            )
        universe = [
            declaration
            for declaration in declarations
            if declaration.spelling == role.spelling
            and _partition_applies(declaration, role)
            and any(
                declaration.domain == domain
                and declaration.declaration_class in classes
                for domain, classes in targets
            )
        ]
        visible = [
            declaration
            for declaration in universe
            if _visible(declaration, role, context)
        ]
        if role.role_id == "U11" and visible:
            visible = [
                declaration
                for declaration in visible
                if declaration.role is not None
                and _scope_contains(context, declaration.role.scope_id, role.scope_id)
            ]
        if len(visible) == 1:
            continue
        if len(visible) > 1:
            raise RouteError("inventory admitted multiple visible lexical targets")
        if universe:
            reason = "label-not-enclosing" if role.role_id == "U11" else "exact-candidate-not-visible"
            rank = 2 if role.role_id == "U11" else 1
            return _issue(
                "lexical-resolution",
                "TYPE-6" if role.role_id in {"U05", "U11"} else "OP-1" if role.role_id in {"U15", "U16"} else "TYPE-5",
                reason,
                role,
                {
                    "admissible_classes": sorted(
                        {value for _, classes in targets for value in classes}
                    ),
                    "lookup_rank": rank,
                    "spelling_hex": role.spelling.hex(),
                },
                tuple(
                    _origin(declaration)
                    for declaration in sorted(
                        universe,
                        key=lambda declaration: (
                            declaration.prelude_ordinal is None,
                            declaration.prelude_ordinal
                            if declaration.prelude_ordinal is not None
                            else declaration.role.event_key,
                        ),
                    )
                ),
            )
        available = sorted(
            {
                declaration.declaration_class
                for declaration in declarations
                if declaration.spelling == role.spelling
                and _partition_applies(declaration, role)
                and _visible(declaration, role, context)
            }
        )
        rule = {
            "U02": "FN-3",
            "U03": "FN-3",
            "U04": "TYPE-6",
            "U05": "TYPE-6",
            "U06": "OWN-3",
            "U07": "OWN-3",
            "U08": "OWN-3",
            "U09": "OWN-3",
            "U10": "OWN-3",
            "U11": "TYPE-6",
            "U12": "CONST-1",
            "U13": "CONST-2",
            "U14": "TYPE-5",
            "U15": "OP-1",
            "U17": "FN-4",
            "U18": "FORM-5",
        }.get(role.role_id, "TYPE-5")
        return _issue(
            "lexical-resolution",
            rule,
            "admissible-target-absent",
            role,
            {
                "admissible_classes": sorted(
                    {value for _, classes in targets for value in classes}
                ),
                "available_classes": available,
                "lookup_rank": 3,
                "spelling_hex": role.spelling.hex(),
            },
        )
    return None
