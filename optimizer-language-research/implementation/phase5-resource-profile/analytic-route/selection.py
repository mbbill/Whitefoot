"""Independent construction-relation admission and resolution selection.

This checks only the two closed generator families. It does not claim to be a
general Whitefoot resolver and cannot grant compiler authority.
"""

from __future__ import annotations

from relation import (
    DOMAIN_CONTRACT,
    DOMAIN_ENUM_CONSTRUCTOR,
    DOMAIN_LABEL,
    DOMAIN_LEXICAL,
    DOMAIN_NOMINAL,
    DOMAIN_OPERATION,
    DOMAIN_STRUCT_CONSTRUCTOR,
    RESERVATION_SPELLINGS,
    LookupEntry,
    PrivateRelation,
)


def _fn8(relation: PrivateRelation) -> tuple[str, ...] | None:
    for block_ordinal, entries in enumerate(relation.requires_shapes):
        if not entries:
            return ("SourceIssue", "FN-8", "missing-final-check", str(block_ordinal))
        for entry_ordinal, shape in enumerate(entries[:-1]):
            if shape != "ordinary-let":
                return (
                    "SourceIssue", "FN-8", "nonfinal-entry", str(block_ordinal),
                    str(entry_ordinal), shape,
                )
        if entries[-1] != "check":
            return (
                "SourceIssue", "FN-8", "missing-final-check", str(block_ordinal)
            )
    return None


def _entry_key(entry: LookupEntry) -> tuple[int, int, str]:
    return (entry.partition, entry.domain, entry.spelling)


def _inventory(relation: PrivateRelation) -> tuple[str, ...] | None:
    nominal_names = {
        entry.spelling
        for entry in relation.lookup_entries
        if entry.partition == 0 and entry.domain == DOMAIN_NOMINAL
    }
    for entry in relation.lookup_entries:
        if entry.domain == DOMAIN_ENUM_CONSTRUCTOR:
            if entry.constructor_owner is None or entry.constructor_owner not in nominal_names:
                return (
                    "CompilerFailure", "enum-constructor-owner", entry.spelling
                )
        elif entry.constructor_owner is not None:
            return ("CompilerFailure", "non-enum-constructor-owner", entry.spelling)

    seen: dict[tuple[int, int, str], LookupEntry] = {}
    for entry in relation.lookup_entries:
        key = _entry_key(entry)
        previous = seen.get(key)
        if previous is not None:
            # PRE-1/source and source/source collisions are source issues.
            # Operation families are already a closed distinct inventory.
            if entry.origin_kind == 2 or previous.origin_kind == 2:
                return (
                    "SourceIssue", "TYPE-6", "collision", entry.spelling,
                    str(entry.domain),
                )
            if entry.origin_kind == previous.origin_kind == 1:
                return ("CompilerFailure", "duplicate-operation", entry.spelling)
        else:
            seen[key] = entry

    reserved = set(RESERVATION_SPELLINGS)
    for role in relation.roles:
        eligible = (
            role.category == "declaration"
            and role.domain == DOMAIN_LEXICAL
            and not role.spelling.startswith("@")
        ) or (
            role.category == "dependent" and role.spelling in ("value", "code")
        )
        if eligible and role.spelling in reserved:
            return (
                "SourceIssue", "FORM-3", "reserved-declaration", role.spelling
            )
    return None


def _visible(entry: LookupEntry, use_event: int) -> bool:
    if entry.origin_kind in (0, 1):
        return True
    if entry.declaration_class == "function" and entry.partition == 0:
        return True
    return entry.source_event < use_event


def _resolution(relation: PrivateRelation) -> tuple[str, ...] | None:
    closed_domains = {
        DOMAIN_LEXICAL,
        DOMAIN_NOMINAL,
        DOMAIN_STRUCT_CONSTRUCTOR,
        DOMAIN_ENUM_CONSTRUCTOR,
        DOMAIN_CONTRACT,
        DOMAIN_LABEL,
        DOMAIN_OPERATION,
    }
    for use in (role for role in relation.roles if role.category == "lexical"):
        if use.domain not in closed_domains:
            return ("CompilerFailure", "unknown-use-domain", use.spelling)
        universe = tuple(
            entry
            for entry in relation.lookup_entries
            if entry.spelling == use.spelling
            and entry.partition in (0, use.partition)
        )
        admissible = tuple(entry for entry in universe if entry.domain == use.domain)
        if not universe:
            return (
                "SourceIssue", "Resolution", "unresolved", use.spelling,
                str(use.event),
            )
        if not admissible:
            return (
                "SourceIssue", "Resolution", "wrong-domain", use.spelling,
                str(use.domain), str(use.event),
            )
        visible = tuple(entry for entry in admissible if _visible(entry, use.event))
        if not visible:
            return (
                "SourceIssue", "Resolution", "not-visible", use.spelling,
                str(use.event),
            )
        if len(visible) != 1:
            return (
                "SourceIssue", "TYPE-6", "ambiguous", use.spelling,
                str(use.event),
            )
    return None


def select_diagnostic(relation: PrivateRelation) -> tuple[str, ...]:
    """Apply the closed FN-8, inventory, then lexical selection order."""
    for check in (_fn8, _inventory, _resolution):
        diagnostic = check(relation)
        if diagnostic is not None:
            return diagnostic
    return ("Complete",)
