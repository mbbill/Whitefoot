"""Command-line entry point for the independent source-to-role route."""

from __future__ import annotations

import argparse
from hashlib import sha256
import os
from pathlib import Path
import stat
import tempfile

from model import LogicalSource, RouteError
from receipt import decode_receipt, encode_receipt, measure


MAX_SOURCES = 4_096
MAX_SOURCE_BYTES = 64 << 20
MAX_TOTAL_SOURCE_BYTES = 128 << 20
MAX_MANIFEST_BYTES = 1 << 20
MAX_RECEIPT_BYTES = 8 << 20
READ_CHUNK_BYTES = 64 << 10


def _stable_metadata(metadata: os.stat_result) -> tuple[int, int, int, int, int, int]:
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_mode,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def _read_bounded(path: Path, maximum: int, label: str) -> bytes:
    try:
        before = os.lstat(path)
    except OSError as error:
        raise RouteError(f"cannot inspect {label}: {path}") from error
    if stat.S_ISLNK(before.st_mode) or not stat.S_ISREG(before.st_mode):
        raise RouteError(f"{label} is not a regular nonsymlink file: {path}")
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise RouteError(f"cannot open {label} without following links: {path}") from error
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode):
            raise RouteError(f"{label} is not a regular file: {path}")
        if (opened.st_dev, opened.st_ino) != (before.st_dev, before.st_ino):
            raise RouteError(f"{label} changed during no-follow open: {path}")
        if opened.st_size < 0 or opened.st_size > maximum:
            raise RouteError(f"{label} exceeds the route operational input cap")
        output = bytearray()
        while True:
            remaining = maximum + 1 - len(output)
            if remaining <= 0:
                raise RouteError(f"{label} grew beyond the route operational input cap")
            chunk = os.read(descriptor, min(READ_CHUNK_BYTES, remaining))
            if not chunk:
                break
            output.extend(chunk)
        if len(output) > maximum:
            raise RouteError(f"{label} exceeds the route operational input cap")
        after = os.fstat(descriptor)
        if _stable_metadata(opened) != _stable_metadata(after) or len(output) != opened.st_size:
            raise RouteError(f"{label} changed during its bounded read: {path}")
        return bytes(output)
    finally:
        os.close(descriptor)


def _check_output(path: Path) -> None:
    if not path.name or not path.parent.is_dir():
        raise RouteError("output parent must be an existing directory")
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise RouteError("existing output must be a regular nonsymlink file")


def _check_output_alias(path: Path, input_paths: tuple[Path, ...]) -> None:
    resolved_output = os.path.realpath(os.path.abspath(path))
    if any(
        resolved_output == os.path.realpath(os.path.abspath(input_path))
        for input_path in input_paths
    ):
        raise RouteError("receipt output must not alias an input")
    try:
        output_metadata = path.lstat()
    except FileNotFoundError:
        return
    output_identity = (output_metadata.st_dev, output_metadata.st_ino)
    for input_path in input_paths:
        try:
            input_metadata = input_path.lstat()
        except FileNotFoundError:
            continue
        if output_identity == (input_metadata.st_dev, input_metadata.st_ino):
            raise RouteError("receipt output must not alias an input")


def _write_atomic(
    path: Path,
    value: bytes,
    maximum: int,
    input_paths: tuple[Path, ...],
) -> None:
    if len(value) > maximum:
        raise RouteError("canonical receipt exceeds the route operational output cap")
    _check_output(path)
    _check_output_alias(path, input_paths)
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb", prefix=f".{path.name}.", dir=path.parent, delete=False
        ) as temporary:
            temporary_name = temporary.name
            temporary.write(value)
            temporary.flush()
            os.fsync(temporary.fileno())
        _check_output(path)
        _check_output_alias(path, input_paths)
        os.replace(temporary_name, path)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def run(
    logical_paths: tuple[str, ...],
    source_paths: tuple[Path, ...],
    manifest_path: Path,
    output_path: Path,
) -> bytes:
    if not source_paths or len(logical_paths) != len(source_paths):
        raise RouteError("logical-path and source-file counts differ or are empty")
    if len(source_paths) > MAX_SOURCES:
        raise RouteError("source record count exceeds the route operational cap")
    input_paths = (manifest_path, *source_paths)
    _check_output_alias(output_path, input_paths)
    raw_sources = []
    total = 0
    for path in source_paths:
        raw = _read_bounded(path, MAX_SOURCE_BYTES, "source file")
        total += len(raw)
        if total > MAX_TOTAL_SOURCE_BYTES:
            raise RouteError("source files exceed the total operational input cap")
        raw_sources.append(raw)
    sources = tuple(
        LogicalSource(path, raw)
        for path, raw in zip(logical_paths, raw_sources)
    )
    manifest = _read_bounded(manifest_path, MAX_MANIFEST_BYTES, "manifest")
    receipt = measure(sources, manifest)
    if receipt["workload"] is None:
        raise RouteError("CLI measurement did not retain its mandatory manifest identity")
    encoded = encode_receipt(receipt)
    if encode_receipt(decode_receipt(encoded)) != encoded:
        raise RouteError("source receipt failed its canonical decode-reencode check")
    _write_atomic(output_path, encoded, MAX_RECEIPT_BYTES, input_paths)
    return encoded


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--logical-path", action="append", required=True)
    parser.add_argument("--source-file", action="append", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    encoded = run(
        tuple(arguments.logical_path),
        tuple(arguments.source_file),
        arguments.manifest,
        arguments.output,
    )
    print(f"bytes={len(encoded)} sha256={sha256(encoded).hexdigest()}")


if __name__ == "__main__":
    main()
