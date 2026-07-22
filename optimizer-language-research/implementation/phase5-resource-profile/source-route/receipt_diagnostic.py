"""Closed nested diagnostic schema for source-route receipts."""

from __future__ import annotations

from dataclasses import dataclass

from model import RouteError
from receipt_values import exact_keys, hex_bytes, node_path, text, text_list, u64


@dataclass(frozen=True)
class DiagnosticFacts:
    source_origins: int
    prelude_origins: int
    paths: int
    path_components: int
    depth: int


def _validate_origin(
    value: object, source_lengths: tuple[int, ...]
) -> tuple[str, int]:
    if not isinstance(value, dict):
        raise RouteError("receipt diagnostic origin is not an object")
    kind = value.get("kind")
    if kind == "prelude":
        origin = exact_keys(
            value,
            {"kind", "prelude_ordinal", "role_or_class"},
            "PRE-1 diagnostic origin",
        )
        u64(origin["prelude_ordinal"], "PRE-1 origin ordinal")
        text(origin["role_or_class"], "PRE-1 origin class")
        return "prelude", 0
    if kind != "source":
        raise RouteError("receipt diagnostic origin kind is not closed")
    origin = exact_keys(
        value,
        {
            "byte_end",
            "byte_start",
            "kind",
            "node_path",
            "role_or_class",
            "role_ordinal",
            "source_ordinal",
            "subtoken_ordinal",
        },
        "source diagnostic origin",
    )
    source_ordinal = u64(origin["source_ordinal"], "origin source ordinal")
    if source_ordinal >= len(source_lengths):
        raise RouteError("receipt diagnostic origin source is outside the bundle")
    start = u64(origin["byte_start"], "origin byte start")
    end = u64(origin["byte_end"], "origin byte end")
    if start > end or end > source_lengths[source_ordinal]:
        raise RouteError("receipt diagnostic origin byte range is outside its source")
    path = node_path(origin["node_path"], "origin node path")
    text(origin["role_or_class"], "origin role or class")
    u64(origin["role_ordinal"], "origin role ordinal")
    u64(origin["subtoken_ordinal"], "origin subtoken ordinal")
    return "source", len(path)


def _validate_payload(stage: str, reason: str, value: object) -> None:
    if stage == "fn8-admission":
        payload = exact_keys(value, {"shape_kind"}, "FN-8 payload")
        if text(payload["shape_kind"], "FN-8 shape") != reason:
            raise RouteError("receipt FN-8 payload disagrees with its reason")
        return
    if reason == "reserved-declaration-name":
        payload = exact_keys(
            value,
            {
                "declaration_role",
                "inventory_ordinal",
                "reserved_class",
                "spelling_hex",
            },
            "reserved-name payload",
        )
        text(payload["declaration_role"], "declaration role")
        u64(payload["inventory_ordinal"], "reservation ordinal")
        text(payload["reserved_class"], "reservation class")
        hex_bytes(payload["spelling_hex"], "reserved spelling")
        return
    if reason in {
        "repeated-region-in-owner",
        "prelude-collision",
        "same-scope-collision",
        "live-shadow-collision",
    }:
        payload = exact_keys(value, {"spelling_hex"}, f"{reason} payload")
        hex_bytes(payload["spelling_hex"], f"{reason} spelling")
        return
    if reason == "match-binder-not-fresh":
        payload = exact_keys(
            value,
            {"binder_spelling_hex", "paired_field_spelling_hex"},
            "match-binder payload",
        )
        hex_bytes(payload["binder_spelling_hex"], "binder spelling")
        hex_bytes(payload["paired_field_spelling_hex"], "paired field spelling")
        return
    if reason == "operation-family-absent":
        payload = exact_keys(
            value,
            {"available_classes", "spelling_hex"},
            "operation-family payload",
        )
        if text_list(payload["available_classes"], "available classes"):
            raise RouteError("receipt absent operation unexpectedly has a class")
        hex_bytes(payload["spelling_hex"], "operation spelling")
        return
    if reason in {"exact-candidate-not-visible", "label-not-enclosing"}:
        payload = exact_keys(
            value,
            {"admissible_classes", "lookup_rank", "spelling_hex"},
            "not-visible payload",
        )
        text_list(
            payload["admissible_classes"],
            "admissible classes",
            allow_empty=False,
        )
        expected_rank = 2 if reason == "label-not-enclosing" else 1
        if u64(payload["lookup_rank"], "lookup rank") != expected_rank:
            raise RouteError("receipt not-visible lookup rank is wrong")
        hex_bytes(payload["spelling_hex"], "not-visible spelling")
        return
    if reason == "admissible-target-absent":
        payload = exact_keys(
            value,
            {
                "admissible_classes",
                "available_classes",
                "lookup_rank",
                "spelling_hex",
            },
            "absent-target payload",
        )
        text_list(
            payload["admissible_classes"],
            "admissible classes",
            allow_empty=False,
        )
        text_list(payload["available_classes"], "available classes")
        if u64(payload["lookup_rank"], "lookup rank") != 3:
            raise RouteError("receipt absent-target lookup rank is wrong")
        hex_bytes(payload["spelling_hex"], "absent-target spelling")
        return
    raise RouteError("receipt diagnostic reason has no closed payload schema")


def validate_diagnostic(
    value: object, source_lengths: tuple[int, ...]
) -> DiagnosticFacts:
    """Validate one selected issue and expose its exact receipt counts."""

    if value is None:
        return DiagnosticFacts(0, 0, 0, 0, 0)
    diagnostic = exact_keys(
        value,
        {
            "byte_end",
            "byte_start",
            "node_path",
            "origins",
            "payload",
            "reason",
            "rule",
            "source_ordinal",
            "stage",
        },
        "selected diagnostic",
    )
    stage = text(diagnostic["stage"], "diagnostic stage")
    text(diagnostic["rule"], "diagnostic rule")
    reason = text(diagnostic["reason"], "diagnostic reason")
    if stage not in {"fn8-admission", "inventory", "lexical-resolution"}:
        raise RouteError("receipt diagnostic stage is not closed")
    source_ordinal = u64(
        diagnostic["source_ordinal"], "diagnostic source ordinal"
    )
    if source_ordinal >= len(source_lengths):
        raise RouteError("receipt diagnostic source is outside the bundle")
    start = u64(diagnostic["byte_start"], "diagnostic byte start")
    end = u64(diagnostic["byte_end"], "diagnostic byte end")
    if start > end or end > source_lengths[source_ordinal]:
        raise RouteError("receipt diagnostic byte range is outside its source")
    primary_path = node_path(diagnostic["node_path"], "diagnostic node path")
    origins = diagnostic["origins"]
    if not isinstance(origins, list):
        raise RouteError("receipt diagnostic origins are not a list")
    source_origins = 0
    prelude_origins = 0
    source_path_components = 0
    source_path_depth = 0
    for origin in origins:
        kind, depth = _validate_origin(origin, source_lengths)
        if kind == "source":
            source_origins += 1
            source_path_components += depth
            source_path_depth = max(source_path_depth, depth)
        else:
            prelude_origins += 1
    _validate_payload(stage, reason, diagnostic["payload"])
    return DiagnosticFacts(
        source_origins,
        prelude_origins,
        1 + source_origins,
        len(primary_path) + source_path_components,
        max(len(primary_path), source_path_depth),
    )
