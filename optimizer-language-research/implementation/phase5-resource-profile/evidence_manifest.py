"""Canonical neutral manifests for independent resource evidence routes.

This module belongs to the workload producer. Neither measurement route may
import it: each route must decode the resulting bytes independently.
"""

from __future__ import annotations

from hashlib import sha256
import json
from pathlib import Path, PurePosixPath
from typing import Mapping, Sequence


SCHEMA = "whitefoot-resource-workload-v1"
FAMILIES = frozenset(("compiler", "codec"))
MAX_U64 = (1 << 64) - 1
MAX_UNITS = 32_768
MAX_PATH_BYTES = 4_096
PARAMETER_NAMES = ("name_decimal_width", "source_records", "unit_count")
TOP_LEVEL_KEYS = {
    "family",
    "generator_revision",
    "parameters",
    "schema",
    "sources",
    "units",
}


class ManifestError(ValueError):
    pass


def _u64(value: object, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ManifestError(f"{field} must be an integer")
    if value < 0 or value > MAX_U64:
        raise ManifestError(f"{field} is outside u64")
    return value


def _digest(value: object, field: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise ManifestError(f"{field} must be one lowercase SHA-256")
    if any(byte not in "0123456789abcdef" for byte in value):
        raise ManifestError(f"{field} must be one lowercase SHA-256")
    return value


def _logical_path(value: object) -> str:
    if not isinstance(value, str) or not value or not value.isascii():
        raise ManifestError("logical_path must be nonempty ASCII")
    if any(ord(character) < 0x21 or ord(character) > 0x7E for character in value):
        raise ManifestError("logical_path must use graphic ASCII")
    path = PurePosixPath(value)
    if (
        path.is_absolute()
        or str(path) != value
        or any(part in ("", ".", "..") for part in path.parts)
    ):
        raise ManifestError("logical_path must be a normalized relative path")
    portable = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-"
    if any(character not in portable and character != "/" for character in value):
        raise ManifestError("logical_path contains a nonportable byte")
    if len(value.encode("ascii")) > MAX_PATH_BYTES:
        raise ManifestError("logical_path exceeds the closed byte bound")
    return value


def generator_revision(generator_path: Path) -> str:
    return sha256(generator_path.read_bytes()).hexdigest()


def build_manifest(
    *,
    family: str,
    units: int,
    revision: str,
    parameters: Mapping[str, int],
    sources: Sequence[tuple[str, bytes]],
) -> dict[str, object]:
    if family not in FAMILIES:
        raise ManifestError(f"unknown workload family: {family}")
    unit_count = _u64(units, "units")
    if not 1 <= unit_count <= MAX_UNITS:
        raise ManifestError("units are outside the closed generator range")
    revision = _digest(revision, "generator_revision")
    if set(parameters) != set(PARAMETER_NAMES):
        raise ManifestError("parameters do not match the closed generator schema")
    expected_parameters = {
        "name_decimal_width": 6,
        "source_records": 1,
        "unit_count": unit_count,
    }
    parameter_records: list[dict[str, object]] = []
    for name in PARAMETER_NAMES:
        value = _u64(parameters[name], name)
        if value != expected_parameters[name]:
            raise ManifestError(f"{name} does not match the closed generator schema")
        parameter_records.append({"name": name, "value": value})
    if len(sources) != 1:
        raise ManifestError("the closed generator schema requires one source")
    source_records: list[dict[str, object]] = []
    seen_paths: set[str] = set()
    for logical_path, source in sources:
        path = _logical_path(logical_path)
        if path in seen_paths:
            raise ManifestError(f"duplicate logical path: {path}")
        seen_paths.add(path)
        if not isinstance(source, bytes):
            raise ManifestError("source payloads must be bytes")
        if not source:
            raise ManifestError("source payload must be nonempty")
        source_records.append(
            {
                "logical_path": path,
                "byte_length": _u64(len(source), "source byte length"),
                "sha256": sha256(source).hexdigest(),
            }
        )
    return {
        "schema": SCHEMA,
        "family": family,
        "units": unit_count,
        "generator_revision": revision,
        "parameters": parameter_records,
        "sources": source_records,
    }


def _validate_encoded_shape(manifest: Mapping[str, object]) -> None:
    if set(manifest) != TOP_LEVEL_KEYS or manifest.get("schema") != SCHEMA:
        raise ManifestError("manifest top level is open, incomplete, or has the wrong schema")
    family = manifest.get("family")
    if family not in FAMILIES:
        raise ManifestError("manifest family is not closed")
    units = _u64(manifest.get("units"), "units")
    if not 1 <= units <= MAX_UNITS:
        raise ManifestError("manifest units are outside the closed generator range")
    _digest(manifest.get("generator_revision"), "generator_revision")
    parameters = manifest.get("parameters")
    if not isinstance(parameters, list) or len(parameters) != len(PARAMETER_NAMES):
        raise ManifestError("manifest parameter vector is malformed")
    expected_values = (6, 1, units)
    for row, name, expected in zip(parameters, PARAMETER_NAMES, expected_values):
        if (
            not isinstance(row, dict)
            or set(row) != {"name", "value"}
            or row.get("name") != name
            or _u64(row.get("value"), name) != expected
        ):
            raise ManifestError("manifest parameter row is malformed")
    sources = manifest.get("sources")
    if not isinstance(sources, list) or len(sources) != 1:
        raise ManifestError("manifest source vector is malformed")
    source = sources[0]
    if not isinstance(source, dict) or set(source) != {
        "byte_length",
        "logical_path",
        "sha256",
    }:
        raise ManifestError("manifest source identity is malformed")
    _logical_path(source["logical_path"])
    if _u64(source["byte_length"], "source byte length") == 0:
        raise ManifestError("manifest source identity is empty")
    _digest(source["sha256"], "source sha256")


def encode_manifest(manifest: Mapping[str, object]) -> bytes:
    _validate_encoded_shape(manifest)
    try:
        return (
            json.dumps(
                manifest,
                ensure_ascii=True,
                allow_nan=False,
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n"
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise ManifestError("manifest is not canonical JSON data") from error
