"""Inventory-rank diagnostic selection over independent declaration facts."""

from __future__ import annotations

from model import Role, SelectedDiagnostic
from topology import ProjectionContext
from resolution import (
    DOMAIN_ORDER,
    ROLE_DECLARATIONS,
    Declaration,
    _ancestor_site,
    _issue,
    _origin,
    _partition_applies,
    _reservation,
    _scope_contains,
    _visible,
)

def select_inventory_issue(
    roles: tuple[Role, ...],
    declarations: tuple[Declaration, ...],
    context: ProjectionContext,
) -> SelectedDiagnostic | None:
    source_declarations = [role for role in roles if role.role_id in ROLE_DECLARATIONS or role.role_id in {"X01", "X02", "X03"}]
    source_facts = [declaration for declaration in declarations if declaration.role is not None]
    prelude = [declaration for declaration in declarations if declaration.role is None]
    for role in source_declarations:
        reserved = _reservation(role)
        if reserved is not None:
            reserved_class, ordinal = reserved
            normalized = role.spelling[1:] if role.role_id in {"D09", "D13"} else role.spelling
            return _issue(
                "inventory",
                "FORM-3",
                "reserved-declaration-name",
                role,
                {
                    "declaration_role": role.role_id,
                    "inventory_ordinal": ordinal,
                    "reserved_class": reserved_class,
                    "spelling_hex": normalized.hex(),
                },
            )

        if role.role_id in {"D09", "D13"}:
            earlier = [
                fact
                for fact in source_facts
                if fact.role is not None
                and fact.role.event_key < role.event_key
                and fact.domain == "region"
                and fact.spelling == role.spelling
                and fact.role.function_owner == role.function_owner
            ]
            if earlier:
                return _issue(
                    "inventory",
                    "OWN-3",
                    "repeated-region-in-owner",
                    role,
                    {"spelling_hex": role.spelling.hex()},
                    (_origin(earlier[-1]),),
                )

        if role.role_id == "D14":
            paired = next(
                candidate
                for candidate in roles
                if candidate.role_id == "X05"
                and candidate.source_ordinal == role.source_ordinal
                and candidate.node_path == role.node_path
            )
            earlier_binders = [
                fact
                for fact in source_facts
                if fact.role is not None
                and fact.role.role_id == "D14"
                and fact.role.arm_owner == role.arm_owner
                and fact.spelling == role.spelling
                and fact.role.event_key < role.event_key
            ]
            arm_site = _ancestor_site(context, role, "arm")
            arm_entry = [
                fact
                for fact in source_facts
                if fact.role is not None
                and fact.domain == "lexical-ident"
                and fact.spelling == role.spelling
                and fact.role.role_id != "D14"
                and _partition_applies(fact, role)
                and (
                    fact.visible_source is None
                    or (arm_site.source_ordinal, arm_site.byte_start)
                    >= (fact.visible_source, fact.visible_byte)
                )
                and _scope_contains(context, fact.scope_id, context.scopes[role.scope_id].parent or 0)
            ]
            if paired.spelling == role.spelling or earlier_binders or arm_entry:
                origins = tuple(
                    _origin(fact)
                    for fact in sorted(
                        earlier_binders + arm_entry,
                        key=lambda fact: fact.role.event_key,
                    )
                )
                return _issue(
                    "inventory",
                    "GRAM-10",
                    "match-binder-not-fresh",
                    role,
                    {
                        "binder_spelling_hex": role.spelling.hex(),
                        "paired_field_spelling_hex": paired.spelling.hex(),
                    },
                    origins,
                )

        facts = [fact for fact in source_facts if fact.role is role]
        prelude_conflicts = [
            candidate
            for fact in facts
            for candidate in prelude
            if candidate.domain == fact.domain and candidate.spelling == fact.spelling
        ]
        if prelude_conflicts:
            ordered = sorted(
                prelude_conflicts,
                key=lambda fact: (DOMAIN_ORDER[fact.domain], fact.prelude_ordinal),
            )
            return _issue(
                "inventory",
                "TYPE-6",
                "prelude-collision",
                role,
                {"spelling_hex": role.spelling.hex()},
                tuple(_origin(fact) for fact in ordered),
            )

        earlier_same = [
            prior
            for fact in facts
            for prior in source_facts
            if prior.role is not role
            and prior.role.event_key < role.event_key
            and prior.domain == fact.domain
            and prior.spelling == fact.spelling
            and prior.scope_id == fact.scope_id
        ]
        if earlier_same:
            ordered = sorted(earlier_same, key=lambda fact: fact.role.event_key)
            return _issue(
                "inventory",
                "TYPE-6",
                "same-scope-collision",
                role,
                {"spelling_hex": role.spelling.hex()},
                tuple(_origin(fact) for fact in ordered),
            )

        nested_conflicts = [
            prior
            for fact in facts
            for prior in source_facts
            if prior.role is not role
            and fact.partition != "root"
            and prior.domain == fact.domain
            and prior.spelling == fact.spelling
            and _visible(prior, role, context)
            and prior.scope_id != fact.scope_id
        ]
        if nested_conflicts:
            ordered = sorted(nested_conflicts, key=lambda fact: fact.role.event_key)
            return _issue(
                "inventory",
                "TYPE-6",
                "live-shadow-collision",
                role,
                {"spelling_hex": role.spelling.hex()},
                tuple(_origin(fact) for fact in ordered),
            )
    return None
