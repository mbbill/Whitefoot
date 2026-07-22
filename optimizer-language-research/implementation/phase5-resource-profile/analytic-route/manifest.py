"""Closed neutral manifest and source-identity verification.

Source bytes are accepted only by ``verify_source_bytes``. That function uses
only ``len`` and SHA-256; the analytic model never receives the bytes.
"""

from dataclasses import dataclass
from hashlib import sha256
import json
from pathlib import PurePosixPath
from typing import Any, Iterable


SCHEMA = "whitefoot-resource-workload-v1"
FAMILY_COMPILER = "compiler"
FAMILY_CODEC = "codec"
GENERATOR_REVISION = "4ecae9410fb82b62c2e8da595d944d29b1de8ae9f9c57983c1eaa393a1bc07d3"
GENERATOR_REVISIONS = {
    FAMILY_COMPILER: GENERATOR_REVISION,
    FAMILY_CODEC: GENERATOR_REVISION,
}

PARAMETER_NAMES = ("name_decimal_width", "source_records", "unit_count")
MANIFEST_KEYS = (
    "family",
    "generator_revision",
    "parameters",
    "schema",
    "sources",
    "units",
)
PARAMETER_KEYS = ("name", "value")
SOURCE_KEYS = ("byte_length", "logical_path", "sha256")
U64_MAX = (1 << 64) - 1
MAX_UNITS = 32_768
MAX_DIMENSION = 1_000_000
MAX_PATH_BYTES = 4096


class ManifestError(ValueError):
    """The neutral manifest or supplied source identities are invalid."""


@dataclass(frozen=True)
class SourceIdentity:
    logical_path: str
    byte_length: int
    sha256: str


@dataclass(frozen=True)
class WorkloadManifest:
    family: str
    units: int
    generator_revision: str
    parameters: tuple[tuple[str, int], ...]
    sources: tuple[SourceIdentity, ...]

    def parameter(self, name: str) -> int:
        for candidate, value in self.parameters:
            if candidate == name:
                return value
        raise ManifestError(f"unknown private parameter lookup: {name}")


def _pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ManifestError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _exact_keys(value: object, expected: tuple[str, ...], label: str) -> dict[str, Any]:
    if type(value) is not dict:
        raise ManifestError(f"{label} must be an object")
    mapping = value
    if tuple(mapping) != expected:
        raise ManifestError(f"{label} keys or key order are not canonical")
    return mapping


def _integer(value: object, label: str, minimum: int, maximum: int) -> int:
    if type(value) is not int or value < minimum or value > maximum:
        raise ManifestError(f"{label} is outside its closed integer range")
    return value


def _digest(value: object, label: str) -> str:
    if type(value) is not str or len(value) != 64:
        raise ManifestError(f"{label} must be a lowercase SHA-256")
    if any(character not in "0123456789abcdef" for character in value):
        raise ManifestError(f"{label} must be a lowercase SHA-256")
    return value


def _path(value: object) -> str:
    if type(value) is not str or not value:
        raise ManifestError("logical_path must be nonempty text")
    if "\x00" in value:
        raise ManifestError("logical_path contains NUL")
    try:
        encoded = value.encode("ascii")
    except UnicodeEncodeError as error:
        raise ManifestError("logical_path is not ASCII") from error
    if any(byte < 0x21 or byte > 0x7E for byte in encoded):
        raise ManifestError("logical_path is not graphic ASCII")
    allowed = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-/"
    if any(byte not in allowed for byte in encoded):
        raise ManifestError("logical_path contains a byte outside the closed alphabet")
    parsed = PurePosixPath(value)
    if parsed.is_absolute() or str(parsed) != value:
        raise ManifestError("logical_path is not a normalized relative POSIX path")
    if any(part in ("", ".", "..") for part in value.split("/")):
        raise ManifestError("logical_path contains a forbidden segment")
    if len(encoded) > MAX_PATH_BYTES:
        raise ManifestError("logical_path exceeds the closed byte bound")
    return value


def validate(manifest: WorkloadManifest) -> WorkloadManifest:
    if manifest.family not in GENERATOR_REVISIONS:
        raise ManifestError("construction family is not closed")
    if manifest.generator_revision != GENERATOR_REVISIONS[manifest.family]:
        raise ManifestError("generator revision does not match the family")
    _integer(manifest.units, "units", 1, MAX_UNITS)
    if tuple(name for name, _ in manifest.parameters) != PARAMETER_NAMES:
        raise ManifestError("parameter names or order are not canonical")
    values = dict(manifest.parameters)
    for name in PARAMETER_NAMES:
        _integer(values[name], name, 1, MAX_DIMENSION)
    if values["name_decimal_width"] != 6:
        raise ManifestError("the selected generator requires six decimal digits")
    if values["source_records"] != 1:
        raise ManifestError("the selected generator emits exactly one source record")
    if len(manifest.sources) != values["source_records"]:
        raise ManifestError("source count does not match source_records")
    if values["unit_count"] != manifest.units:
        raise ManifestError("unit_count must equal the top-level units value")
    seen: set[bytes] = set()
    for source in manifest.sources:
        path = _path(source.logical_path).encode("utf-8")
        _integer(source.byte_length, "source byte_length", 1, U64_MAX)
        _digest(source.sha256, "source sha256")
        if path in seen:
            raise ManifestError("logical paths must be unique")
        seen.add(path)
    return manifest


def encode(manifest: WorkloadManifest) -> bytes:
    validate(manifest)
    value = {
        "family": manifest.family,
        "generator_revision": manifest.generator_revision,
        "parameters": [
            {"name": name, "value": amount} for name, amount in manifest.parameters
        ],
        "schema": SCHEMA,
        "sources": [
            {
                "byte_length": source.byte_length,
                "logical_path": source.logical_path,
                "sha256": source.sha256,
            }
            for source in manifest.sources
        ],
        "units": manifest.units,
    }
    return (json.dumps(
        value,
        ensure_ascii=True,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ) + "\n").encode("ascii")


def decode(encoded: bytes) -> WorkloadManifest:
    try:
        text = encoded.decode("utf-8")
        raw = json.loads(text, object_pairs_hook=_pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ManifestError("manifest is not canonical UTF-8 JSON") from error
    root = _exact_keys(raw, MANIFEST_KEYS, "manifest")
    if root["schema"] != SCHEMA:
        raise ManifestError("manifest schema is wrong")
    family = root["family"]
    if type(family) is not str or family not in GENERATOR_REVISIONS:
        raise ManifestError("construction family is not closed")
    parameters_raw = root["parameters"]
    if type(parameters_raw) is not list:
        raise ManifestError("parameters must be an ordered array")
    parameter_pairs = []
    for value in parameters_raw:
        parameter = _exact_keys(value, PARAMETER_KEYS, "parameter")
        name = parameter["name"]
        if type(name) is not str:
            raise ManifestError("parameter name must be text")
        parameter_pairs.append(
            (name, _integer(parameter["value"], name, 1, MAX_DIMENSION))
        )
    if tuple(name for name, _ in parameter_pairs) != PARAMETER_NAMES:
        raise ManifestError("parameter names or order are not canonical")
    sources_raw = root["sources"]
    if type(sources_raw) is not list:
        raise ManifestError("sources must be an ordered array")
    sources = []
    for value in sources_raw:
        source = _exact_keys(value, SOURCE_KEYS, "source identity")
        sources.append(
            SourceIdentity(
                _path(source["logical_path"]),
                _integer(source["byte_length"], "source byte_length", 1, U64_MAX),
                _digest(source["sha256"], "source sha256"),
            )
        )
    manifest = WorkloadManifest(
        family=family,
        units=_integer(root["units"], "units", 1, MAX_UNITS),
        generator_revision=_digest(root["generator_revision"], "generator_revision"),
        parameters=tuple(parameter_pairs),
        sources=tuple(sources),
    )
    validate(manifest)
    if encode(manifest) != encoded:
        raise ManifestError("manifest JSON representation is not canonical")
    return manifest


def verify_source_bytes(
    manifest: WorkloadManifest, sources: Iterable[bytes]
) -> tuple[bytes, ...]:
    """Verify only byte length and SHA-256, returning the ordered digests."""
    supplied = tuple(sources)
    if len(supplied) != len(manifest.sources):
        raise ManifestError("supplied source count does not match the manifest")
    digests = []
    for identity, source in zip(manifest.sources, supplied):
        if type(source) is not bytes:
            raise ManifestError("a supplied source is not an exact byte string")
        if len(source) != identity.byte_length:
            raise ManifestError("supplied source byte length does not match")
        digest = sha256(source).digest()
        if digest.hex() != identity.sha256:
            raise ManifestError("supplied source SHA-256 does not match")
        digests.append(digest)
    return tuple(digests)
