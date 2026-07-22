#!/usr/bin/env python3
"""Verify the exact identity locks for the canonical v0.9 static catalog."""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path
from typing import Sequence

import semantics
import semantics_io


ROOT = Path(__file__).resolve().parents[2]
EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256 = (
    "3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae"
)
ROOT_LOCK_COMPONENTS = ("tests", "spec-catalogs", "v0.9", "static-catalog.sha256")
COMPILER_LOCK_COMPONENTS = ("compiler", "static-semantic-catalog-v0.9.sha256")
LOCK_BYTES = 65


class CatalogIdentityError(ValueError):
    """The canonical catalog or one of its exact identity locks drifted."""


def catalog_sha256(catalog: dict) -> str:
    """Hash the bounded canonical bytes of one validated static catalog."""
    return hashlib.sha256(semantics.canonical_bytes(catalog)).hexdigest()


def parse_lock(raw: bytes, label: str) -> str:
    """Parse exactly 64 lowercase hexadecimal digits followed by one LF."""
    if len(raw) != LOCK_BYTES or not raw.endswith(b"\n"):
        raise CatalogIdentityError(
            f"{label} must be 64 lowercase hexadecimal digits followed by LF"
        )
    digest = raw[:-1]
    if any(byte not in b"0123456789abcdef" for byte in digest):
        raise CatalogIdentityError(f"{label} must contain lowercase hexadecimal")
    return digest.decode("ascii")


def validate_identities(catalog_digest: str, root_lock: bytes, compiler_lock: bytes) -> None:
    """Require the derived, expected, root, and compiler identities to agree."""
    if catalog_digest != EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256:
        raise CatalogIdentityError(
            "canonical static-catalog identity differs from the reviewed v0.9 identity"
        )
    root_digest = parse_lock(root_lock, "root static-catalog lock")
    if root_digest != catalog_digest:
        raise CatalogIdentityError("root static-catalog lock is stale")
    compiler_digest = parse_lock(compiler_lock, "compiler static-catalog lock")
    if compiler_digest != root_digest:
        raise CatalogIdentityError("compiler static-catalog lock differs from root lock")


def read_lock(repository_root: Path, components: tuple[str, ...], label: str) -> bytes:
    """Read one bounded regular-file lock without following symlinks."""
    try:
        return semantics_io.read_fixed_file(
            repository_root,
            components,
            LOCK_BYTES,
            label,
        )
    except semantics_io.CatalogIOError as error:
        raise CatalogIdentityError(str(error)) from error


def check(repository_root: Path = ROOT) -> str:
    """Rebuild the exact catalog and verify both checked-in identity locks."""
    try:
        catalog = semantics.build_from_files(repository_root)
    except (OSError, semantics.SemanticCatalogError) as error:
        raise CatalogIdentityError(str(error)) from error
    digest = catalog_sha256(catalog)
    root_lock = read_lock(repository_root, ROOT_LOCK_COMPONENTS, "root static-catalog lock")
    compiler_lock = read_lock(
        repository_root,
        COMPILER_LOCK_COMPONENTS,
        "compiler static-catalog lock",
    )
    validate_identities(digest, root_lock, compiler_lock)
    return digest


def main(argv: Sequence[str] | None = None) -> int:
    """Run the read-only identity check."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check",), nargs="?", default="check")
    parser.parse_args(argv)
    print(check())
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CatalogIdentityError as error:
        print(f"catalog identity: {error}", file=sys.stderr)
        raise SystemExit(1)
