"""Closed data model for the independent source-to-role evidence route."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class LogicalSource:
    """One ordered logical source record supplied to the route."""

    logical_path: str
    source: bytes


@dataclass(frozen=True)
class ParsedSource:
    """One exact canonical source and its independent parser products."""

    ordinal: int
    logical_path: str
    source: bytes
    tokens: tuple[Any, ...]
    forest: Any


@dataclass(frozen=True)
class NodeSite:
    """One production occurrence with its canonical source identity."""

    source_ordinal: int | None
    path: tuple[int, ...]
    node: Any | None
    kind: str
    byte_start: int | None
    byte_end: int | None
    parent_path: tuple[int, ...] | None


@dataclass(frozen=True)
class Role:
    """One occurrence from the owner-approved closed grammar-role matrix."""

    role_id: str
    role_class: str
    owner_kind: str
    source_ordinal: int
    node_path: tuple[int, ...]
    role_ordinal: int
    subtoken_ordinal: int
    carrier_token: int
    byte_start: int
    byte_end: int
    spelling: bytes
    scope_id: int
    partition_chain: tuple[str, ...]
    function_owner: str | None
    arm_owner: str | None

    @property
    def event_key(self) -> tuple[object, ...]:
        """Return the exact source event-key comparison projection."""

        return (
            self.source_ordinal,
            self.byte_start,
            self.byte_end,
            self.node_path,
            self.role_ordinal,
            self.subtoken_ordinal,
        )


@dataclass(frozen=True)
class Scope:
    """One scope from the approved scope-construction matrix."""

    scope_id: int
    kind: str
    node_path: tuple[int, ...]
    parent: int | None
    depth: int
    source_ordinal: int | None
    byte_start: int | None
    byte_end: int | None
    partition_chain: tuple[str, ...]
    function_owner: str | None
    arm_owner: str | None


@dataclass(frozen=True)
class DiagnosticOrigin:
    """One self-contained source or PRE-1 diagnostic origin."""

    kind: str
    role_or_class: str
    source_ordinal: int | None = None
    node_path: tuple[int, ...] = ()
    byte_start: int | None = None
    byte_end: int | None = None
    role_ordinal: int | None = None
    subtoken_ordinal: int | None = None
    prelude_ordinal: int | None = None

    def as_json(self) -> dict[str, object]:
        """Project the origin to the closed receipt representation."""

        if self.kind == "prelude":
            return {
                "kind": "prelude",
                "prelude_ordinal": self.prelude_ordinal,
                "role_or_class": self.role_or_class,
            }
        return {
            "byte_end": self.byte_end,
            "byte_start": self.byte_start,
            "kind": "source",
            "node_path": list(self.node_path),
            "role_or_class": self.role_or_class,
            "role_ordinal": self.role_ordinal,
            "source_ordinal": self.source_ordinal,
            "subtoken_ordinal": self.subtoken_ordinal,
        }


@dataclass(frozen=True)
class SelectedDiagnostic:
    """The route's selected semantic diagnostic, or no issue on success."""

    stage: str
    rule: str
    reason: str
    source_ordinal: int
    node_path: tuple[int, ...]
    byte_start: int
    byte_end: int
    payload: dict[str, object]
    origins: tuple[DiagnosticOrigin, ...] = ()

    def as_json(self) -> dict[str, object]:
        """Project the selected issue to canonical JSON values."""

        return {
            "byte_end": self.byte_end,
            "byte_start": self.byte_start,
            "node_path": list(self.node_path),
            "origins": [origin.as_json() for origin in self.origins],
            "payload": self.payload,
            "reason": self.reason,
            "rule": self.rule,
            "source_ordinal": self.source_ordinal,
            "stage": self.stage,
        }


class RouteError(RuntimeError):
    """The route cannot publish a closed, identity-bound receipt."""
