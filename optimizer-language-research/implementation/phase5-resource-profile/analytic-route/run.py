"""Bounded process entry point for the independent analytic evidence route."""

from __future__ import annotations

import argparse
from hashlib import sha256
import json
import os
from pathlib import Path
import stat
import tempfile

from manifest import ManifestError, decode as decode_manifest
from measure import FIELD_NAMES, MeasurementError, require_complete
from receipt import (
    PROFILE_SEMANTICS_SHA256,
    PROPOSAL_SHA256,
    SPECIFICATION_SHA256,
    STORAGE_SHA256,
    WORK_SHA256,
    CODE_FILES,
    ReceiptError,
    build,
    decode as decode_receipt,
)


# These are observer-process safeguards, not ResourceProfile maxima or demand
# evidence. Changing them cannot select or loosen a language/compiler profile.
MAX_MANIFEST_BYTES = 1 << 20
MAX_SOURCE_FILES = 64
MAX_SOURCE_FILE_BYTES = 64 << 20
MAX_TOTAL_SOURCE_BYTES = 128 << 20
MAX_RECEIPT_BYTES = 8 << 20
MAX_SUMMARY_BYTES = 8 << 20
READ_CHUNK_BYTES = 64 << 10


class RunError(ValueError):
    """The bounded evidence invocation is malformed or cannot be completed."""


def _read_regular(path: Path, maximum: int, label: str) -> bytes:
    try:
        before = os.lstat(path)
    except OSError as error:
        raise RunError(f"cannot inspect {label}: {path}") from error
    if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
        raise RunError(f"{label} is not a regular nonsymlink file: {path}")
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise RunError(f"cannot open {label} as a nonsymlink file: {path}") from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise RunError(f"{label} is not a regular file: {path}")
        if (metadata.st_dev, metadata.st_ino) != (before.st_dev, before.st_ino):
            raise RunError(f"{label} changed during no-follow open: {path}")
        if metadata.st_size < 0 or metadata.st_size > maximum:
            raise RunError(f"{label} exceeds its operational byte cap: {path}")
        output = bytearray()
        while True:
            remaining = maximum + 1 - len(output)
            if remaining <= 0:
                raise RunError(f"{label} grew beyond its operational byte cap: {path}")
            chunk = os.read(descriptor, min(READ_CHUNK_BYTES, remaining))
            if not chunk:
                break
            output.extend(chunk)
        if len(output) > maximum:
            raise RunError(f"{label} exceeds its operational byte cap: {path}")
        return bytes(output)
    finally:
        os.close(descriptor)


def _check_output(path: Path) -> None:
    if not path.name or not path.parent.is_dir():
        raise RunError("output parent must be an existing directory")
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise RunError("existing output must be a regular nonsymlink file")


def _write_atomic(path: Path, value: bytes) -> None:
    _check_output(path)
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb", prefix=f".{path.name}.", dir=path.parent, delete=False
        ) as temporary:
            temporary_name = temporary.name
            temporary.write(value)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_name, path)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def _summary(encoded: bytes) -> bytes:
    """Project comparison JSON only from a strict receipt round trip."""
    receipt = decode_receipt(encoded)
    manifest = decode_manifest(receipt.manifest_bytes)
    fields = []
    for name, value in zip(FIELD_NAMES, receipt.measurement.actuals):
        fields.append(
            {
                "name": name,
                "state": "available" if value is not None else "unavailable",
                "value": value,
            }
        )
    value = {
        "analytic_code_files": list(CODE_FILES),
        "analytic_code_sha256": receipt.code_digest.hex(),
        "bundle_sha256": receipt.bundle_digest.hex(),
        "derived": [
            {"name": name, "value": amount}
            for name, amount in receipt.measurement.derived
        ],
        "expected_diagnostic": list(receipt.measurement.expected_diagnostic),
        "family": manifest.family,
        "fields": fields,
        "manifest_sha256": receipt.manifest_digest.hex(),
        "profile_semantics_sha256": PROFILE_SEMANTICS_SHA256.hex(),
        "proposal_sha256": PROPOSAL_SHA256.hex(),
        "receipt_sha256": sha256(encoded).hexdigest(),
        "schema": "whitefoot-resource-analytic-summary-v1",
        "source_sha256": [digest.hex() for digest in receipt.source_digests],
        "specification_sha256": SPECIFICATION_SHA256.hex(),
        "status": receipt.status,
        "storage_sha256": STORAGE_SHA256.hex(),
        "trace_gaps": [
            {"field": field, "reason": reason}
            for field, reason in receipt.measurement.trace_gaps
        ],
        "units": manifest.units,
        "generator_revision": manifest.generator_revision,
        "sources": [
            {
                "byte_length": source.byte_length,
                "logical_path": source.logical_path,
                "sha256": source.sha256,
            }
            for source in manifest.sources
        ],
        "work_sha256": WORK_SHA256.hex(),
    }
    return (
        json.dumps(
            value,
            ensure_ascii=True,
            allow_nan=False,
            separators=(",", ":"),
            sort_keys=True,
        )
        + "\n"
    ).encode("ascii")


def run(
    manifest_path: Path,
    source_paths: tuple[Path, ...],
    output_path: Path,
    summary_output_path: Path | None = None,
    selection_mode: bool = False,
) -> bytes:
    if not source_paths or len(source_paths) > MAX_SOURCE_FILES:
        raise RunError("source-file count is outside the operational cap")
    inputs = {os.path.abspath(manifest_path)}
    inputs.update(os.path.abspath(path) for path in source_paths)
    if os.path.abspath(output_path) in inputs:
        raise RunError("receipt output must not alias an input")
    if summary_output_path is not None and os.path.abspath(summary_output_path) in inputs:
        raise RunError("summary output must not alias an input")
    manifest = _read_regular(manifest_path, MAX_MANIFEST_BYTES, "manifest")
    sources = []
    total = 0
    for path in source_paths:
        source = _read_regular(path, MAX_SOURCE_FILE_BYTES, "source file")
        total += len(source)
        if total > MAX_TOTAL_SOURCE_BYTES:
            raise RunError("source files exceed the total operational byte cap")
        sources.append(source)
    encoded = build(manifest, tuple(sources))
    if len(encoded) > MAX_RECEIPT_BYTES:
        raise RunError("canonical receipt exceeds the operational byte cap")
    decoded = decode_receipt(encoded)
    if selection_mode:
        require_complete(decoded.measurement)
    summary = None
    if summary_output_path is not None:
        if os.path.abspath(summary_output_path) == os.path.abspath(output_path):
            raise RunError("receipt and summary outputs must be distinct")
        summary = _summary(encoded)
        if len(summary) > MAX_SUMMARY_BYTES:
            raise RunError("canonical summary exceeds the operational byte cap")
        _check_output(summary_output_path)
    _write_atomic(output_path, encoded)
    if summary is not None and summary_output_path is not None:
        _write_atomic(summary_output_path, summary)
    return encoded


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument(
        "--source-file", type=Path, action="append", required=True
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--summary-output", type=Path)
    parser.add_argument(
        "--selection-mode",
        action="store_true",
        help="require a complete no-gap receipt suitable for profile selection",
    )
    arguments = parser.parse_args()
    try:
        encoded = run(
            arguments.manifest,
            tuple(arguments.source_file),
            arguments.output,
            arguments.summary_output,
            arguments.selection_mode,
        )
    except (RunError, ManifestError, MeasurementError, ReceiptError, OSError) as error:
        parser.exit(1, f"analytic route failed: {error}\n")
    print(f"bytes={len(encoded)} sha256={sha256(encoded).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
