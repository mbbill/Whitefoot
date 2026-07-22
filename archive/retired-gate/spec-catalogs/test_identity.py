#!/usr/bin/env python3
"""Hostile tests for the exact static-catalog identity locks."""

from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import identity
import semantics


EXPECTED = "3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae"
SPECIFICATION = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
DECOMPOSITION = "81cc67795feb9dfb9458df7987da44663b8d5ea034921a1c56322e2771e4310c"
HISTORICAL_V0_8_CATALOG = (
    "2fa586a8a1d9a49f344d64ad2b5f450a2ae2e8362bc187c70267097b9b427e1d"
)


class CatalogIdentityTests(unittest.TestCase):
    def test_live_catalog_and_both_locks_have_the_exact_identity(self) -> None:
        root_before = identity.read_lock(
            identity.ROOT,
            identity.ROOT_LOCK_COMPONENTS,
            "root static-catalog lock",
        )
        compiler_before = identity.read_lock(
            identity.ROOT,
            identity.COMPILER_LOCK_COMPONENTS,
            "compiler static-catalog lock",
        )
        self.assertEqual(identity.check(), EXPECTED)
        self.assertEqual(
            identity.EXPECTED_STATIC_SEMANTIC_CATALOG_SHA256,
            EXPECTED,
        )
        self.assertEqual(
            identity.read_lock(
                identity.ROOT,
                identity.ROOT_LOCK_COMPONENTS,
                "root static-catalog lock",
            ),
            root_before,
        )
        self.assertEqual(
            identity.read_lock(
                identity.ROOT,
                identity.COMPILER_LOCK_COMPONENTS,
                "compiler static-catalog lock",
            ),
            compiler_before,
        )

    def test_lock_spelling_is_exact_and_fail_closed(self) -> None:
        valid = (EXPECTED + "\n").encode("ascii")
        malformed = (
            EXPECTED.encode("ascii"),
            (EXPECTED + "\r\n").encode("ascii"),
            (EXPECTED.upper() + "\n").encode("ascii"),
            (EXPECTED + "\n\n").encode("ascii"),
            ("g" + EXPECTED[1:] + "\n").encode("ascii"),
        )
        self.assertEqual(identity.parse_lock(valid, "probe"), EXPECTED)
        for raw in malformed:
            with self.subTest(raw=raw):
                with self.assertRaises(identity.CatalogIdentityError):
                    identity.parse_lock(raw, "probe")

    def test_stale_and_cross_lock_substitutions_are_distinct(self) -> None:
        valid = (EXPECTED + "\n").encode("ascii")
        stale = (("0" * 64) + "\n").encode("ascii")
        with self.assertRaisesRegex(identity.CatalogIdentityError, "root.*stale"):
            identity.validate_identities(EXPECTED, stale, valid)
        with self.assertRaisesRegex(identity.CatalogIdentityError, "compiler.*differs"):
            identity.validate_identities(EXPECTED, valid, stale)
        with self.assertRaisesRegex(identity.CatalogIdentityError, "reviewed"):
            identity.validate_identities("0" * 64, valid, valid)

    def test_catalog_identity_is_not_the_decomposition_identity(self) -> None:
        catalog = semantics.build_from_files()
        self.assertEqual(
            catalog["specification"],
            {
                "path": "spec/kernel-spec-v0.9.md",
                "sha256": SPECIFICATION,
                "version": "0.9",
            },
        )
        self.assertEqual(catalog["decomposition_sha256"], DECOMPOSITION)
        self.assertEqual(identity.catalog_sha256(catalog), EXPECTED)
        self.assertNotEqual(EXPECTED, DECOMPOSITION)

        changed = copy.deepcopy(catalog)
        changed["kind"] += "-changed"
        self.assertNotEqual(identity.catalog_sha256(changed), EXPECTED)

    def test_outer_catalog_identity_does_not_use_the_generator_hash_helper(self) -> None:
        catalog = semantics.build_from_files()
        with mock.patch.object(
            semantics,
            "sha256",
            return_value="0" * 64,
        ):
            self.assertEqual(identity.catalog_sha256(catalog), EXPECTED)

    def test_lock_reader_rejects_symlink_substitution(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock = root.joinpath(*identity.ROOT_LOCK_COMPONENTS)
            lock.parent.mkdir(parents=True)
            outside = root / "outside"
            outside.write_text(EXPECTED + "\n", encoding="ascii")
            lock.symlink_to(outside)
            with self.assertRaises(identity.CatalogIdentityError):
                identity.read_lock(
                    root,
                    identity.ROOT_LOCK_COMPONENTS,
                    "probe lock",
                )

    def test_lock_reader_rejects_oversized_regular_file(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock = root.joinpath(*identity.ROOT_LOCK_COMPONENTS)
            lock.parent.mkdir(parents=True)
            lock.write_bytes((EXPECTED + "\n\n").encode("ascii"))
            with self.assertRaises(identity.CatalogIdentityError):
                identity.read_lock(
                    root,
                    identity.ROOT_LOCK_COMPONENTS,
                    "probe lock",
                )

    def test_v0_8_catalog_locks_remain_immutable_historical_evidence(self) -> None:
        expected = HISTORICAL_V0_8_CATALOG + "\n"
        self.assertEqual(
            (identity.ROOT / "tests/spec-catalogs/v0.8/static-catalog.sha256").read_text(
                encoding="ascii"
            ),
            expected,
        )
        self.assertEqual(
            (
                identity.ROOT
                / "compiler/static-semantic-catalog-v0.8.sha256"
            ).read_text(encoding="ascii"),
            expected,
        )
        self.assertNotEqual(EXPECTED, HISTORICAL_V0_8_CATALOG)


if __name__ == "__main__":
    unittest.main()
