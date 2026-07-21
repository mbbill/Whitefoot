#!/usr/bin/env python3
"""Hostile tests for the non-authorizing compiler capability overlay."""

from __future__ import annotations

import copy
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import capability_overlay as overlay
import facet_discrepancies
import facet_discrepancy_inputs as discrepancy_inputs
import semantic_catalog
import semantic_catalog_io


CATALOG = semantic_catalog.build_from_files()
CATALOG_BYTES = semantic_catalog.canonical_bytes(CATALOG)
CATALOG_SHA256 = overlay.sha256(CATALOG_BYTES)
EXPECTED_CATALOG_SHA256 = (
    "3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae"
)
EXPECTED_SPECIFICATION_SHA256 = (
    "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
)
EXPECTED_DECOMPOSITION_SHA256 = (
    "81cc67795feb9dfb9458df7987da44663b8d5ea034921a1c56322e2771e4310c"
)
HISTORICAL_V0_8_CATALOG_SHA256 = (
    "2fa586a8a1d9a49f344d64ad2b5f450a2ae2e8362bc187c70267097b9b427e1d"
)
OPEN_DISCREPANCY_IDS = (
    "discrepancy:v0.9/affine-deref-storage-lifecycle",
    "discrepancy:v0.9/diag3-retained-proof-ref",
    "discrepancy:v0.9/eff1-row-canonicality",
    "discrepancy:v0.9/eff2-local-region-effects",
    "discrepancy:v0.9/fn3-contract-member-semantics",
    "discrepancy:v0.9/fn7-main-return-spelling",
    "discrepancy:v0.9/op1-dotless-reservation",
)
SIDECAR_RAW = discrepancy_inputs.read_regular(
    discrepancy_inputs.ROOT,
    facet_discrepancies.SIDECAR_PATH,
    "test discrepancy sidecar",
    discrepancy_inputs.MAX_SIDECAR_BYTES,
)
facet_discrepancies.parse_and_validate_sidecar(SIDECAR_RAW, CATALOG_BYTES)
SIDECAR = facet_discrepancies.parse_canonical_sidecar(SIDECAR_RAW)

TARGET_FACET = "facet:GATE-1/single-audit-trail"
BLOCKED_FACET = "facet:DIAG-3/check-report-schema"


def fragment(
    *,
    handlers: list[dict] | None = None,
    evidence: list[dict] | None = None,
) -> dict:
    return {
        "catalog_sha256": CATALOG_SHA256,
        "evidence": [] if evidence is None else evidence,
        "format": overlay.FORMAT,
        "handlers": [] if handlers is None else handlers,
        "implementation_id": overlay.IMPLEMENTATION_ID,
    }


def encoded(value: dict) -> tuple[tuple[str, bytes], ...]:
    return (("test.json", overlay.canonical_bytes(value, "test fragment")),)


def validate(
    value: dict,
    *,
    sidecar: dict | None = None,
) -> overlay._ValidatedOverlay:
    return overlay._validate_fragments(
        encoded(value),
        CATALOG,
        SIDECAR if sidecar is None else sidecar,
    )


def target_handlers() -> list[dict]:
    return [
        {
            "facet_ids": [TARGET_FACET],
            "id": "handler:audit/governance",
            "lane": "governance",
        },
        {
            "facet_ids": [TARGET_FACET],
            "id": "handler:audit/report",
            "lane": "report",
        },
    ]


class LiveFoundationTests(unittest.TestCase):
    def test_live_overlay_is_honestly_empty_and_exactly_bound(self) -> None:
        self.assertEqual(CATALOG_SHA256, EXPECTED_CATALOG_SHA256)
        self.assertEqual(
            CATALOG["specification"],
            {
                "path": "spec/kernel-spec-v0.9.md",
                "sha256": EXPECTED_SPECIFICATION_SHA256,
                "version": "0.9",
            },
        )
        self.assertEqual(
            CATALOG["decomposition_sha256"], EXPECTED_DECOMPOSITION_SHA256
        )
        report = overlay.audit_repository()
        self.assertEqual(len(report.facets), 679)
        self.assertEqual(report.closed_facet_ids, ())
        self.assertEqual(len(report.blocked_facet_ids), 16)
        self.assertEqual(report.open_discrepancy_ids, OPEN_DISCREPANCY_IDS)
        self.assertEqual(report.unresolved_receipt_ids, ())

    def test_live_fragment_has_no_verdict_or_claim(self) -> None:
        records = semantic_catalog_io.read_fragment_directory(
            overlay.ROOT,
            overlay.FRAGMENT_DIRECTORY,
            label="test capability overlay",
            max_count=overlay.MAX_FRAGMENT_COUNT,
            max_file_bytes=overlay.MAX_FRAGMENT_BYTES,
            max_total_bytes=overlay.MAX_FRAGMENT_TOTAL_BYTES,
        )
        self.assertEqual([name for name, _ in records], ["foundation.json"])
        document = overlay.parse_fragment_bytes(records[0][1], "live fragment")
        self.assertEqual(document, fragment())
        self.assertTrue(
            {
                "complete",
                "fallback",
                "pending",
                "status",
                "supported",
                "waiver",
            }.isdisjoint(document)
        )

    def test_current_blockers_are_derived_from_revalidated_sidecar(self) -> None:
        state = overlay.audit_repository().explain(BLOCKED_FACET)
        self.assertFalse(state.is_closed)
        self.assertEqual(
            state.blocking_discrepancy_ids,
            ("discrepancy:v0.9/diag3-retained-proof-ref",),
        )

    def test_v0_8_fragment_remains_immutable_historical_evidence(self) -> None:
        records = semantic_catalog_io.read_fragment_directory(
            overlay.ROOT,
            ("capabilities", "whitefoot-rust", "v0.8"),
            label="historical v0.8 capability overlay",
            max_count=overlay.MAX_FRAGMENT_COUNT,
            max_file_bytes=overlay.MAX_FRAGMENT_BYTES,
            max_total_bytes=overlay.MAX_FRAGMENT_TOTAL_BYTES,
        )
        self.assertEqual([name for name, _ in records], ["foundation.json"])
        document = overlay.parse_fragment_bytes(records[0][1], "historical fragment")
        expected = fragment()
        expected["catalog_sha256"] = HISTORICAL_V0_8_CATALOG_SHA256
        self.assertEqual(document, expected)


class FailClosedEvidenceTests(unittest.TestCase):
    def test_absent_handler_reports_every_required_lane_missing(self) -> None:
        validated = validate(fragment())
        state = overlay._derive_report(CATALOG, SIDECAR, validated).explain(
            TARGET_FACET
        )
        self.assertFalse(state.is_closed)
        self.assertEqual(state.missing_handler_lanes, ("report", "governance"))
        self.assertEqual(state.unexercised_lanes, ())

    def test_handlers_remain_unexercised_without_a_replay_provider(self) -> None:
        validated = validate(fragment(handlers=target_handlers()))
        state = overlay._derive_report(CATALOG, SIDECAR, validated).explain(TARGET_FACET)
        self.assertFalse(state.is_closed)
        self.assertEqual(state.missing_handler_lanes, ())
        self.assertEqual(state.unexercised_lanes, ("report", "governance"))
        self.assertEqual(state.missing_evidence_classes, ("determinism", "static-audit"))

    def test_receipt_reference_is_unresolved_and_grants_nothing(self) -> None:
        reference = {
            "id": "receipt:static-audit/example",
            "receipt_sha256": overlay.sha256(b"opaque receipt"),
        }
        validated = validate(fragment(evidence=[reference]))
        report = overlay._derive_report(CATALOG, SIDECAR, validated)
        self.assertEqual(report.closed_facet_ids, ())
        self.assertEqual(report.unresolved_receipt_ids, (reference["id"],))

    def test_constructed_report_is_not_a_release_input(self) -> None:
        self.assertFalse(hasattr(overlay, "require_release_complete"))
        self.assertEqual(overlay.CapabilityReport((), (), ()).closed_facet_ids, ())

    def test_fragment_order_does_not_change_validation(self) -> None:
        handlers = target_handlers()
        left = fragment(handlers=handlers[:1])
        right = fragment(handlers=handlers[1:])
        fragments = (
            ("left.json", overlay.canonical_bytes(left)),
            ("right.json", overlay.canonical_bytes(right)),
        )
        forward = overlay._validate_fragments(fragments, CATALOG, SIDECAR)
        reverse = overlay._validate_fragments(tuple(reversed(fragments)), CATALOG, SIDECAR)
        self.assertEqual(forward, reverse)


class ClosedSchemaTests(unittest.TestCase):
    def test_verdict_waiver_fallback_and_expected_fields_are_forbidden(self) -> None:
        for field, value in (
            ("complete", True),
            ("expected_verdict", "accept"),
            ("fallback", "reject"),
            ("not_applicable", True),
            ("status", "supported"),
            ("waiver", True),
        ):
            changed = fragment()
            changed[field] = value
            with self.subTest(field=field), self.assertRaisesRegex(
                overlay.CapabilityOverlayError, "fields differ"
            ):
                validate(changed)

    def test_stale_catalog_and_unknown_identity_fail(self) -> None:
        changed = fragment()
        changed["catalog_sha256"] = "0" * 64
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "stale catalog"):
            validate(changed)
        changed = fragment()
        changed["implementation_id"] = "other-compiler"
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "unknown implementation"):
            validate(changed)

    def test_noncanonical_and_duplicate_json_fail(self) -> None:
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "canonical"):
            overlay.parse_fragment_bytes(b'{"catalog_sha256":"x"}\n', "noncanonical")
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "duplicate"):
            overlay.parse_fragment_bytes(
                b'{"format":"a","format":"b"}\n', "duplicate"
            )

    def test_unknown_facet_lane_and_unrequired_lane_fail(self) -> None:
        cases = (
            ("facet:GATE-1/not-real", "governance", "unknown facet"),
            (TARGET_FACET, "not-a-lane", "unknown lane"),
            (TARGET_FACET, "frontend", "unrequired lane"),
        )
        for facet_id, lane, message in cases:
            value = fragment(
                handlers=[
                    {
                        "facet_ids": [facet_id],
                        "id": "handler:test/one",
                        "lane": lane,
                    }
                ]
            )
            with self.subTest(message=message), self.assertRaisesRegex(
                overlay.CapabilityOverlayError, message
            ):
                validate(value)

    def test_blocked_claim_and_new_discrepancy_fail(self) -> None:
        blocked = fragment(
            handlers=[
                {
                    "facet_ids": [BLOCKED_FACET],
                    "id": "handler:report/checks",
                    "lane": "report",
                }
            ]
        )
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "discrepancy-blocked"):
            validate(blocked)

        changed_sidecar = copy.deepcopy(SIDECAR)
        changed_sidecar["records"].append(
            {
                "affected_facet_ids": [TARGET_FACET],
                "id": "discrepancy:v0.9/zz-test-boundary",
            }
        )
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "discrepancy-blocked"):
            validate(fragment(handlers=target_handlers()), sidecar=changed_sidecar)

    def test_duplicate_handler_id_and_facet_lane_owner_fail(self) -> None:
        handler = target_handlers()[0]
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "duplicate.*ID"):
            validate(fragment(handlers=[handler, copy.deepcopy(handler)]))
        second = copy.deepcopy(handler)
        second["id"] = "handler:audit/other"
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "multiple handlers"):
            validate(fragment(handlers=[handler, second]))

    def test_sorted_unique_and_count_bounds_fail(self) -> None:
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "handler IDs"):
            validate(fragment(handlers=list(reversed(target_handlers()))))
        with mock.patch.object(overlay, "MAX_HANDLER_COUNT", 1):
            with self.assertRaisesRegex(overlay.CapabilityOverlayError, "handler count"):
                validate(fragment(handlers=target_handlers()))

    def test_receipt_references_are_closed_bounded_and_unaliased(self) -> None:
        first = {
            "id": "receipt:audit/one",
            "receipt_sha256": overlay.sha256(b"one"),
        }
        second = {
            "id": "receipt:audit/two",
            "receipt_sha256": overlay.sha256(b"two"),
        }
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "evidence IDs"):
            validate(fragment(evidence=[second, first]))
        aliased = copy.deepcopy(second)
        aliased["receipt_sha256"] = first["receipt_sha256"]
        with self.assertRaisesRegex(overlay.CapabilityOverlayError, "aliased"):
            validate(fragment(evidence=[first, aliased]))
        with mock.patch.object(overlay, "MAX_EVIDENCE_REFERENCE_COUNT", 1):
            with self.assertRaisesRegex(overlay.CapabilityOverlayError, "reference count"):
                validate(fragment(evidence=[first, second]))


class DescriptorAndResourceTests(unittest.TestCase):
    def test_symlinked_overlay_directory_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            parent = root / "capabilities" / "whitefoot-rust"
            parent.mkdir(parents=True)
            (parent / "v0.9").symlink_to(
                overlay.ROOT.joinpath(*overlay.FRAGMENT_DIRECTORY),
                target_is_directory=True,
            )
            with self.assertRaisesRegex(
                overlay.CapabilityOverlayError, "non-symlink directory"
            ):
                overlay._repository_fragments(root)

    def test_fifo_fragment_is_rejected_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root.joinpath(*overlay.FRAGMENT_DIRECTORY)
            target.mkdir(parents=True)
            os.mkfifo(target / "blocked.json")
            with self.assertRaisesRegex(overlay.CapabilityOverlayError, "not a regular file"):
                overlay._repository_fragments(root)

    def test_directory_entry_cap_fails_before_unbounded_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root.joinpath(*overlay.FRAGMENT_DIRECTORY)
            target.mkdir(parents=True)
            (target / "one.txt").write_text("one", encoding="ascii")
            (target / "two.txt").write_text("two", encoding="ascii")
            with mock.patch.object(overlay, "MAX_DIRECTORY_ENTRIES", 1):
                with self.assertRaisesRegex(overlay.CapabilityOverlayError, "entries exceed"):
                    overlay._repository_fragments(root)


if __name__ == "__main__":
    unittest.main()
