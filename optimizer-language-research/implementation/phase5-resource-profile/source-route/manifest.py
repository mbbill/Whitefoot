"""Independent decoder for the neutral workload producer manifest."""

from __future__ import annotations

from hashlib import sha256
import json

from identities import digest
from model import LogicalSource, RouteError
from parser_adapter import validate_logical_path


SCHEMA = "whitefoot-resource-workload-v1"
GENERATOR_REVISION = "4ecae9410fb82b62c2e8da595d944d29b1de8ae9f9c57983c1eaa393a1bc07d3"
PARAMETERS = ("name_decimal_width", "source_records", "unit_count")


def _u64(value: object, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise RouteError(f"{label} is not an integer")
    if value < 0 or value > (1 << 64) - 1:
        raise RouteError(f"{label} is outside u64")
    return value


def _sha(value: object, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise RouteError(f"{label} is not one lowercase SHA-256")
    return value


def decode_manifest(raw: bytes, sources: tuple[LogicalSource, ...]) -> dict[str, object]:
    """Decode canonical JSON and verify its complete source/parameter binding."""

    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteError("workload manifest is not JSON") from error
    if not isinstance(value, dict) or set(value) != {
        "schema",
        "family",
        "units",
        "generator_revision",
        "parameters",
        "sources",
    }:
        raise RouteError("workload manifest has an open or incomplete top level")
    try:
        canonical = (
            json.dumps(
                value,
                ensure_ascii=True,
                allow_nan=False,
                separators=(",", ":"),
                sort_keys=True,
            )
            + "\n"
        ).encode("ascii")
    except (TypeError, ValueError) as error:
        raise RouteError("workload manifest contains a noncanonical JSON value") from error
    if canonical != raw:
        raise RouteError("workload manifest JSON is not canonical")
    if value["schema"] != SCHEMA or value["family"] not in {"compiler", "codec"}:
        raise RouteError("workload manifest schema or family is not recognized")
    units = _u64(value["units"], "units")
    if units < 1 or units > 32_768:
        raise RouteError("workload units are outside the closed generator range")
    if _sha(value["generator_revision"], "generator_revision") != GENERATOR_REVISION:
        raise RouteError("workload generator revision is not the pinned producer")
    parameters = value["parameters"]
    if not isinstance(parameters, list) or len(parameters) != len(PARAMETERS):
        raise RouteError("workload parameter vector has the wrong size")
    expected_values = (6, 1, units)
    for record, name, expected in zip(parameters, PARAMETERS, expected_values):
        if not isinstance(record, dict) or set(record) != {"name", "value"}:
            raise RouteError("workload parameter record is malformed")
        if record["name"] != name or _u64(record["value"], name) != expected:
            raise RouteError("workload parameter order or value is wrong")
    descriptors = value["sources"]
    if (
        not isinstance(descriptors, list)
        or len(descriptors) != len(sources)
        or len(descriptors) != expected_values[1]
    ):
        raise RouteError("manifest/source record count mismatch")
    for descriptor, source in zip(descriptors, sources):
        if not isinstance(descriptor, dict) or set(descriptor) != {
            "logical_path",
            "byte_length",
            "sha256",
        }:
            raise RouteError("workload source descriptor is malformed")
        if not isinstance(descriptor["logical_path"], str):
            raise RouteError("manifest logical_path is not text")
        validate_logical_path(descriptor["logical_path"])
        if (
            descriptor["logical_path"] != source.logical_path
            or _u64(descriptor["byte_length"], "byte_length") != len(source.source)
            or _sha(descriptor["sha256"], "source sha256") != digest(source.source)
        ):
            raise RouteError("workload manifest does not bind the supplied source bytes")
    return {
        "family": value["family"],
        "manifest_sha256": sha256(raw).hexdigest(),
        "units": units,
    }
