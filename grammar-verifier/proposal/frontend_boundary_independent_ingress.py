"""Independent descriptor, candidate, and source-bundle ingress."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import struct
from typing import Any

from frontend_boundary_independent_core import IndependentBoundaryError, unhex


HERE = Path(__file__).resolve().parent
CASE_FILE = HERE / "frontend-boundary-cases.json"
SPEC_FILE = HERE / "kernel-spec-successor-candidate.md"
FILE_CAP = 1_048_576
RULE_NAMES = {
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
}


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _file(path: Path) -> bytes:
    try:
        status = path.lstat()
    except OSError as error:
        raise IndependentBoundaryError(f"cannot inspect {path}: {error}") from error
    if not path.is_file() or path.is_symlink() or status.st_size > FILE_CAP:
        raise IndependentBoundaryError(
            f"refusing non-regular, symlinked, or oversized input: {path}"
        )
    try:
        with path.open("rb", buffering=0) as stream:
            raw = stream.read(FILE_CAP + 1)
            if stream.read(1):
                raise IndependentBoundaryError(f"input grew while reading: {path}")
    except OSError as error:
        raise IndependentBoundaryError(f"cannot read {path}: {error}") from error
    if len(raw) != status.st_size or len(raw) > FILE_CAP:
        raise IndependentBoundaryError(f"input size is unstable or excessive: {path}")
    return raw


def _object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    keys = [key for key, _ in pairs]
    if len(keys) != len(set(keys)):
        raise IndependentBoundaryError("descriptor contains a repeated object member")
    return dict(pairs)


def _pretty(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, indent=2, ensure_ascii=True) + "\n").encode("ascii")


def read_cases(path: Path = CASE_FILE) -> tuple[dict[str, Any], bytes]:
    raw = _file(path)
    try:
        value = json.loads(raw.decode("ascii"), object_pairs_hook=_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise IndependentBoundaryError(f"case descriptor is not strict ASCII JSON: {error}") from error
    if not isinstance(value, dict) or _pretty(value) != raw:
        raise IndependentBoundaryError("case descriptor is not canonical JSON")
    if sorted(value) != [
        "authority",
        "bundles",
        "cases",
        "limits",
        "raw_cases",
        "requirements",
        "schema",
    ]:
        raise IndependentBoundaryError("case descriptor root vocabulary changed")
    if value["schema"] != "whitefoot.frontend-boundary-cases.v2":
        raise IndependentBoundaryError("case descriptor version changed")
    if value["authority"] != "proposal evidence only; never language or compiler authority":
        raise IndependentBoundaryError("case descriptor lost its authority boundary")
    if not isinstance(value.get("raw_cases"), list):
        raise IndependentBoundaryError("raw case descriptor collection changed")
    return value, raw


def _rule_sections(raw: bytes) -> dict[str, bytes]:
    try:
        raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise IndependentBoundaryError("candidate bytes are not UTF-8") from error
    starts: list[tuple[int, str]] = []
    offset = 0
    for line in raw.splitlines(keepends=True):
        if line.startswith(b"["):
            close = line.find(b"]")
            if close > 1:
                label = line[1:close]
                if all(byte in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789" for byte in label):
                    decoded = label.decode("ascii", "strict")
                    if "-" in decoded and decoded.rsplit("-", 1)[-1].isdigit():
                        starts.append((offset, decoded))
        offset += len(line)
    result: dict[str, bytes] = {}
    for position, (start, label) in enumerate(starts):
        stop = starts[position + 1][0] if position + 1 < len(starts) else len(raw)
        if label in result:
            raise IndependentBoundaryError(f"candidate repeats numbered rule {label}")
        result[label] = raw[start:stop]
    return result


def specification_contract(path: Path = SPEC_FILE) -> tuple[dict[str, object], bytes]:
    raw = _file(path)
    sections = _rule_sections(raw)
    if not RULE_NAMES <= set(sections):
        raise IndependentBoundaryError("candidate omits a frontend boundary rule")
    semantic_needles = {
        "PROG-1": [
            b"every language name is defined within it or by the prelude",
            b"The only external boundary is the gated FFI wall",
        ],
        "PROG-2": [
            b"ordered nonempty sequence",
            b"one logical path and one exact source-byte sequence",
            b"ordered first by source ordinal and then by source-local item order",
            b"no namespace, scope, import, or lookup key",
        ],
        "DIAG-1": [
            b"closed sum",
            b"`SourceCoordinate` is",
            b"`BundleRootExtent` is the exact",
            b"stage by stage",
            b"Raw lexical scanning is quote-aware and reports the first defect at its cursor.",
            b"The expected-terminal set is",
            b"the following closed attribution rows are tested in order",
            b"A transparent mandatory-name path begins at",
            b"At a stopping decision the total fallback cites the owner",
            b"A [FORM-2] mismatch is",
            b"An input-envelope failure, resource failure, compiler-invariant failure",
        ],
        "FN-1": [b"Function-signature visibility is the [TYPE-6] table."],
        "TYPE-6": [
            b"top-level `fn_decl` signature",
            b"the entire closed compilation unit",
            b"every other declaration governed by name lookup",
            b"immediately after its lexical declaration",
        ],
        "FN-7": [b"fn main() -> unit"],
    }
    for owner, needles in semantic_needles.items():
        section = sections[owner]
        if any(section.count(needle) != 1 for needle in needles):
            raise IndependentBoundaryError(f"candidate contract changed in {owner}")
    rule_rows = {
        name: {"byte_length": len(sections[name]), "sha256": digest(sections[name])}
        for name in sorted(RULE_NAMES)
    }
    return (
        {
            "byte_length": len(raw),
            "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
            "rules": rule_rows,
            "sha256": digest(raw),
        },
        raw,
    )


def _portable_path(value: object) -> bool:
    if not isinstance(value, str) or not value or not value.isascii():
        return False
    legal = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-"
    pieces = value.encode("ascii").split(b"/")
    return all(
        piece
        and piece not in {b".", b".."}
        and all(byte in legal for byte in piece)
        for piece in pieces
    )


def profile(cases: dict[str, Any], override: object) -> tuple[int, int, int, int]:
    source = cases["limits"] if override is None else override
    if not isinstance(source, dict):
        raise IndependentBoundaryError("resource profile is not an object")
    names = ["max_observations", "max_record_bytes", "max_records", "max_total_source_bytes"]
    if sorted(source) != names:
        raise IndependentBoundaryError("resource profile vocabulary changed")
    values = [source[name] for name in names]
    if any(type(value) is not int or value < 0 for value in values):
        raise IndependentBoundaryError("resource profile contains an invalid bound")
    return values[0], values[1], values[2], values[3]


def records(
    cases: dict[str, Any], bundle_name: object, override: object = None
) -> tuple[dict[str, object] | None, list[dict[str, Any]]]:
    bundles = cases.get("bundles")
    if not isinstance(bundles, dict) or not isinstance(bundle_name, str) or bundle_name not in bundles:
        raise IndependentBoundaryError("case names an absent bundle")
    authored = bundles[bundle_name]
    if not isinstance(authored, list):
        raise IndependentBoundaryError("bundle is not a record list")
    if not authored:
        return {"code": "zero-records", "family": "input-envelope-failure"}, []
    decoded: list[dict[str, Any]] = []
    paths: set[str] = set()
    for ordinal, row in enumerate(authored):
        if not isinstance(row, dict) or not {"path", "source_hex"} <= set(row) <= {
            "items",
            "path",
            "source_hex",
        }:
            raise IndependentBoundaryError("source record fields changed")
        logical_path = row["path"]
        if not _portable_path(logical_path):
            return {
                "code": "invalid-logical-path",
                "family": "input-envelope-failure",
                "record_ordinal": ordinal,
            }, []
        assert isinstance(logical_path, str)
        if logical_path in paths:
            return {
                "code": "duplicate-logical-path",
                "family": "input-envelope-failure",
                "record_ordinal": ordinal,
            }, []
        paths.add(logical_path)
        items = row.get("items", [])
        if not isinstance(items, list):
            raise IndependentBoundaryError("source record items are not a list")
        decoded.append(
            {
                "items": items,
                "ordinal": ordinal,
                "path": logical_path,
                "source": unhex(row["source_hex"]),
            }
        )
    _, per_record, count_limit, total_limit = profile(cases, override)
    if len(decoded) > count_limit:
        return {"code": "record-limit", "family": "resource-failure"}, []
    if any(len(row["source"]) > per_record for row in decoded):
        return {"code": "record-byte-limit", "family": "resource-failure"}, []
    if sum(len(row["source"]) for row in decoded) > total_limit:
        return {"code": "total-source-byte-limit", "family": "resource-failure"}, []
    return None, decoded


def envelope(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    failure, _ = records(cases, probe["bundle"], probe.get("profile"))
    return failure if failure is not None else {"family": "frontend-complete"}


def identity(cases: dict[str, Any], probe: dict[str, Any]) -> dict[str, object]:
    left_error, left = records(cases, probe["left"])
    right_error, right = records(cases, probe["right"])
    if left_error is not None or right_error is not None:
        raise IndependentBoundaryError("identity probe contains an invalid envelope")
    return {
        "family": "identity-comparison",
        "left_record_count": len(left),
        "right_record_count": len(right),
        "same": _identity_bytes(left) == _identity_bytes(right),
    }


def _identity_bytes(bound: list[dict[str, Any]]) -> bytes:
    parts = [b"WF-BOUNDARY-PROJECTION\0", struct.pack(">Q", len(bound))]
    for ordinal, record in enumerate(bound):
        path = record["path"].encode("ascii")
        source = record["source"]
        parts.extend(
            (
                struct.pack(">Q", ordinal),
                struct.pack(">Q", len(path)),
                path,
                struct.pack(">Q", len(source)),
                source,
            )
        )
    return b"".join(parts)
