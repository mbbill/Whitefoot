"""Primary descriptor, candidate, and source-bundle ingress."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import struct
from typing import Any

from frontend_boundary_primary_core import BoundaryModelError, Record, strict_hex


MODULE_ROOT = Path(__file__).resolve().parent
DESCRIPTOR_PATH = MODULE_ROOT / "frontend-boundary-cases.json"
CANDIDATE_PATH = MODULE_ROOT / "kernel-spec-successor-candidate.md"
MAX_DESCRIPTOR_BYTES = 1 << 20
MAX_CANDIDATE_BYTES = 1 << 20
RULE_HEADING = re.compile(rb"(?m)^\[([A-Z]+-[0-9]+)\]")
CONTRACT_RULES = (
    "DIAG-1",
    "FN-1",
    "FN-7",
    "FORM-1",
    "FORM-2",
    "FORM-3",
    "FORM-4",
    "FORM-5",
    "GRAM-2",
    "GRAM-9",
    "PROG-1",
    "PROG-2",
    "TYPE-6",
)


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_json(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    ).encode("ascii")


def read_regular(path: Path, limit: int) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise BoundaryModelError(f"input is not a regular non-symlink file: {path}")
    try:
        size = path.stat().st_size
    except OSError as error:
        raise BoundaryModelError(f"cannot stat {path}: {error}") from error
    if size > limit:
        raise BoundaryModelError(f"input exceeds byte limit: {path}")
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise BoundaryModelError(f"cannot read {path}: {error}") from error
    if len(raw) != size:
        raise BoundaryModelError(f"input changed while being read: {path}")
    return raw


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise BoundaryModelError(f"duplicate descriptor key: {key}")
        result[key] = value
    return result


def load_descriptor(path: Path = DESCRIPTOR_PATH) -> tuple[dict[str, Any], bytes]:
    raw = read_regular(path, MAX_DESCRIPTOR_BYTES)
    try:
        text = raw.decode("ascii", "strict")
        value = json.loads(text, object_pairs_hook=_reject_duplicate_keys)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise BoundaryModelError(f"descriptor is not strict ASCII JSON: {error}") from error
    if not isinstance(value, dict):
        raise BoundaryModelError("descriptor root is not an object")
    if canonical_json(value) != raw:
        raise BoundaryModelError("descriptor is not canonical sorted two-space JSON")
    if set(value) != {
        "authority",
        "bundles",
        "cases",
        "limits",
        "raw_cases",
        "requirements",
        "schema",
    }:
        raise BoundaryModelError("descriptor root fields changed")
    if value["schema"] != "whitefoot.frontend-boundary-cases.v2":
        raise BoundaryModelError("descriptor schema changed")
    if value["authority"] != "proposal evidence only; never language or compiler authority":
        raise BoundaryModelError("descriptor authority statement changed")
    if (
        not isinstance(value["bundles"], dict)
        or not isinstance(value["cases"], list)
        or not isinstance(value["raw_cases"], list)
    ):
        raise BoundaryModelError("descriptor collections have the wrong shape")
    return value, raw


def _extract_contract(candidate: bytes) -> dict[str, dict[str, object]]:
    try:
        candidate.decode("utf-8", "strict")
    except UnicodeDecodeError as error:
        raise BoundaryModelError("candidate is not UTF-8") from error
    headings = list(RULE_HEADING.finditer(candidate))
    rules: dict[str, bytes] = {}
    for index, heading in enumerate(headings):
        owner = heading.group(1).decode("ascii")
        end = headings[index + 1].start() if index + 1 < len(headings) else len(candidate)
        if owner in rules:
            raise BoundaryModelError(f"candidate has duplicate rule {owner}")
        rules[owner] = candidate[heading.start() : end]
    missing = set(CONTRACT_RULES) - set(rules)
    if missing:
        raise BoundaryModelError(f"candidate is missing contract rules: {sorted(missing)}")
    required_phrases = {
        "PROG-1": (
            b"every language name is defined within it or by the prelude",
            b"The only external boundary is the gated FFI wall",
        ),
        "PROG-2": (
            b"ordered nonempty sequence of logical source records",
            b"No token, trivia item, grammar production below the compilation-unit `program` root, or source span crosses a record boundary.",
            b"ordered first by source ordinal and then by source-local item order",
            b"Top-level declaration order is source ordinal followed by source-local item order.",
        ),
        "DIAG-1": (
            b"SourceBytes(SourceCoordinate)",
            b"SourceNode(NodePath, SourceCoordinate)",
            b"BundleRoot(NodePath, BundleRootExtent)",
            b"The frontend selects defects stage by stage.",
            b"Raw lexical scanning is quote-aware and reports the first defect at its cursor.",
            b"the following closed attribution rows are tested in order",
            b"A transparent mandatory-name path begins at",
            b"At a stopping decision the total fallback cites the owner",
            b"A [FORM-2] mismatch is selected by the first byte offset",
            b"An input-envelope failure, resource failure, compiler-invariant failure",
        ),
        "FN-1": (
            b"Function-signature visibility is the [TYPE-6] table.",
        ),
        "FN-7": (b"Exactly one `fn main() -> unit`",),
        "TYPE-6": (
            b"top-level `fn_decl` signature",
            b"the entire closed compilation unit",
            b"every other declaration governed by name lookup",
            b"immediately after its lexical declaration",
        ),
    }
    for owner, phrases in required_phrases.items():
        for phrase in phrases:
            if rules[owner].count(phrase) != 1:
                raise BoundaryModelError(f"candidate {owner} contract phrase changed")
    return {
        owner: {"byte_length": len(rules[owner]), "sha256": sha256(rules[owner])}
        for owner in CONTRACT_RULES
    }


def candidate_contract(path: Path = CANDIDATE_PATH) -> tuple[dict[str, object], bytes]:
    candidate = read_regular(path, MAX_CANDIDATE_BYTES)
    return (
        {
            "byte_length": len(candidate),
            "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
            "rules": _extract_contract(candidate),
            "sha256": sha256(candidate),
        },
        candidate,
    )


def valid_path(path: object) -> bool:
    if not isinstance(path, str) or not path or not path.isascii():
        return False
    components = path.split("/")
    if any(not part or part in {".", ".."} for part in components):
        return False
    allowed = set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-")
    return all(set(part) <= allowed for part in components)


def limits(descriptor: dict[str, Any], override: object = None) -> dict[str, int]:
    raw = descriptor["limits"] if override is None else override
    if not isinstance(raw, dict) or set(raw) != {
        "max_observations",
        "max_record_bytes",
        "max_records",
        "max_total_source_bytes",
    }:
        raise BoundaryModelError("resource profile fields changed")
    result: dict[str, int] = {}
    for key, value in raw.items():
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise BoundaryModelError(f"resource profile {key} is not a nonnegative integer")
        result[key] = value
    return result


def bind(
    descriptor: dict[str, Any], identifier: object, profile: object = None
) -> tuple[dict[str, object] | None, tuple[Record, ...]]:
    bundles = descriptor["bundles"]
    if not isinstance(identifier, str) or identifier not in bundles:
        raise BoundaryModelError(f"unknown bundle: {identifier!r}")
    rows = bundles[identifier]
    if not isinstance(rows, list):
        raise BoundaryModelError(f"bundle {identifier} is not a list")
    profile_limits = limits(descriptor, profile)
    if not rows:
        return {"code": "zero-records", "family": "input-envelope-failure"}, ()
    parsed: list[Record] = []
    seen: set[str] = set()
    total = 0
    for ordinal, row in enumerate(rows):
        if not isinstance(row, dict) or not {"path", "source_hex"} <= set(row) <= {
            "items",
            "path",
            "source_hex",
        }:
            raise BoundaryModelError(f"bundle {identifier} record fields changed")
        path = row["path"]
        if not valid_path(path):
            return {
                "code": "invalid-logical-path",
                "family": "input-envelope-failure",
                "record_ordinal": ordinal,
            }, ()
        assert isinstance(path, str)
        if path in seen:
            return {
                "code": "duplicate-logical-path",
                "family": "input-envelope-failure",
                "record_ordinal": ordinal,
            }, ()
        seen.add(path)
        source = strict_hex(row["source_hex"], f"bundle {identifier} source")
        items = row.get("items", [])
        if not isinstance(items, list):
            raise BoundaryModelError(f"bundle {identifier} items are not a list")
        total += len(source)
        parsed.append(Record(ordinal, path, source, tuple(items)))
    if len(parsed) > profile_limits["max_records"]:
        return {"code": "record-limit", "family": "resource-failure"}, ()
    if any(len(record.source) > profile_limits["max_record_bytes"] for record in parsed):
        return {"code": "record-byte-limit", "family": "resource-failure"}, ()
    if total > profile_limits["max_total_source_bytes"]:
        return {"code": "total-source-byte-limit", "family": "resource-failure"}, ()
    return None, tuple(parsed)


def envelope_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    failure, _ = bind(descriptor, case["bundle"], case.get("profile"))
    return failure or {"family": "frontend-complete"}


def identity_case(descriptor: dict[str, Any], case: dict[str, Any]) -> dict[str, object]:
    left_failure, left = bind(descriptor, case["left"])
    right_failure, right = bind(descriptor, case["right"])
    if left_failure or right_failure:
        raise BoundaryModelError("identity comparison uses an unbound bundle")
    left_raw = _identity_projection(left)
    right_raw = _identity_projection(right)
    return {
        "family": "identity-comparison",
        "left_record_count": len(left),
        "right_record_count": len(right),
        "same": left_raw == right_raw,
    }


def _identity_projection(records: tuple[Record, ...]) -> bytes:
    parts = [b"WF-BOUNDARY-PROJECTION\0", struct.pack(">Q", len(records))]
    for record in records:
        path = record.path.encode("ascii")
        parts.extend(
            (
                struct.pack(">Q", record.ordinal),
                struct.pack(">Q", len(path)),
                path,
                struct.pack(">Q", len(record.source)),
                record.source,
            )
        )
    return b"".join(parts)
